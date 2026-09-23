"""段落頭の接続詞と、文末の同形反復を測る。

候補は明文の作法書から採った。採否はここでは決めず、実測で決める。

- 段落頭の接続詞: 石黒圭『文章は接続詞で決まる』が、接続詞は必要な箇所に置くものであり、
  段落の頭に機械的に並べる書き方を戒めている。
- 文末の同形反復: 木下是雄『理科系の作文技術』ほかが、文末の形をそろえ続けることを戒める。

対になった人間訳と AI 訳、それに参照コーパスで比べる。

使い方: python measure_flow.py
"""

import re
import sys
import pathlib

CONJUNCTIONS = [
    "しかし", "また", "そして", "そのため", "さらに", "つまり",
    "一方", "このように", "なぜなら", "したがって", "ただし", "しかしながら",
]

# 文末の形。表層で分ける。実装に入れる場合は活用形で分けることになる。
ENDINGS = [
    ("ます", r"ます$"),
    ("ました", r"ました$"),
    ("です", r"です$"),
    ("である", r"であ(る|り)$"),
    ("だ", r"だ$"),
    ("する", r"(する|します)$"),
    ("ない", r"(ない|ません)$"),
]


def body(src: str) -> str:
    src = re.sub(r"(?s)\A---\n.*?\n---\n", "", src)
    src = re.sub(r"(?s)```.*?```", "", src)
    src = re.sub(r"(?m)^\s*[-*+]\s+.*$", "", src)
    src = re.sub(r"(?m)^\s*#+\s+.*$", "", src)
    src = re.sub(r"(?m)^\s*\|.*$", "", src)
    return src


def paragraphs(text: str) -> list[str]:
    return [p.strip() for p in re.split(r"\n\s*\n", text) if p.strip()]


def sentences(text: str) -> list[str]:
    out = [s.strip() for s in re.split(r"(?<=[。！？])", text.replace("\n", ""))]
    return [s for s in out if len(s) >= 5]


def ending_of(sentence: str) -> str | None:
    core = sentence.rstrip("。！？」）)")
    for name, pat in ENDINGS:
        if re.search(pat, core):
            return name
    return None


def measure(paths: list[pathlib.Path]) -> dict[str, float]:
    paras = 0
    lead = 0
    runs = 0
    sents = 0
    for p in paths:
        text = body(p.read_text(encoding="utf-8", errors="replace"))
        for para in paragraphs(text):
            paras += 1
            if any(para.startswith(c) for c in CONJUNCTIONS):
                lead += 1
            ends = [ending_of(s) for s in sentences(para)]
            sents += len(ends)
            run = 1
            for a, b in zip(ends, ends[1:]):
                run = run + 1 if a is not None and a == b else 1
                if run == 3:
                    runs += 1
    return {
        "段落": paras,
        "頭が接続詞": lead * 100 / paras if paras else 0.0,
        "文": sents,
        "同形3連": runs * 100 / paras if paras else 0.0,
    }


def main() -> None:
    root = pathlib.Path("corpus/cache")
    ai = sorted((root / "k8s").glob("*.haiku.md")) + sorted((root / "k8s").glob("*.sonnet.md"))
    stems = {p.name.rsplit(".", 2)[0] for p in ai}
    human = sorted(p for p in (root / "k8s").glob("*.ja.md") if p.name.rsplit(".", 2)[0] in stems)
    pro = sorted((root / "pro_ja").glob("*.md"))

    sets = [("人間訳（対）", human), ("AI訳（対）", ai), ("参照コーパス", pro)]
    print(f"{'群':<14} {'文書':>5} {'段落':>7} {'頭が接続詞':>10} {'同形3連':>9}")
    for name, paths in sets:
        if not paths:
            print(f"{name:<14} 見つからない")
            continue
        m = measure(paths)
        print(
            f"{name:<14} {len(paths):>5} {m['段落']:>7} "
            f"{m['頭が接続詞']:>9.1f}% {m['同形3連']:>8.1f}%"
        )


if __name__ == "__main__":
    main()
