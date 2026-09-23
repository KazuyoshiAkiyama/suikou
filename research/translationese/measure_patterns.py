"""翻訳調の候補パターンを、対になったコーパスで測る。

候補の列挙は人が行い、採否は実測で決める。D-26 と同じ手順である。

`corpus/cache/k8s/` には同じ文書の人間訳（`.ja.md`）と AI 訳（`.haiku.md`、
`.sonnet.md`）が対で入っている。同じ原文に対する訳を比べるため、
分野や題材の違いが差に紛れ込まない。

出力は1万文字あたりの出現数である。文書の長さが揃わないため、率で比べる。

使い方: python measure_patterns.py [corpus/cache/k8s]
"""

import re
import sys
import pathlib
import statistics

# 候補。coji/natural-japanese (MIT) の翻訳調パターン集から採った。
# 採否はここでは決めない。測った結果で決める。
CANDIDATES: dict[str, str] = {
    "することができる": r"することができ(る|ます|た|ない)",
    "することが可能": r"することが可能",
    "という点で": r"という点で",
    "という観点": r"という観点(から|で)",
    "することによって": r"することによって",
    "を持つこと": r"を持つ(こと|存在)",
    "にとって重要": r"にとって(重要|不可欠)",
    "と言えるだろう": r"と言えるだろう",
    "に他ならない": r"に(他|ほか)ならない",
    "間違いない": r"であることは間違いない",
    # 無生物主語。翻訳研究が挙げる型を、機械で拾える形に絞る。
    "無生物主語＋示す": r"(こと|事実|結果|データ|図|表|調査)は、?[^。]{0,20}(示して|示す|意味する|物語る)",
}


def body_text(src: str) -> str:
    """フロントマターとコードを落とした本文。粗くてよい。率で比べるためである。"""
    src = re.sub(r"(?s)\A---\n.*?\n---\n", "", src)
    src = re.sub(r"(?s)```.*?```", "", src)
    return src


def rates(paths: list[pathlib.Path]) -> tuple[dict[str, float], int]:
    total = 0
    hits = dict.fromkeys(CANDIDATES, 0)
    for p in paths:
        text = body_text(p.read_text(encoding="utf-8", errors="replace"))
        total += len(text)
        for name, pat in CANDIDATES.items():
            hits[name] += len(re.findall(pat, text))
    if total == 0:
        return {k: 0.0 for k in CANDIDATES}, 0
    return {k: v * 10000 / total for k, v in hits.items()}, total


def main() -> None:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/k8s")
    ai = sorted(root.glob("*.haiku.md")) + sorted(root.glob("*.sonnet.md"))
    # 対になっている文書だけを人間側に採る。題材を揃えるためである。
    stems = {p.name.rsplit(".", 2)[0] for p in ai}
    human = sorted(p for p in root.glob("*.ja.md") if p.name.rsplit(".", 2)[0] in stems)

    h_rate, h_chars = rates(human)
    a_rate, a_chars = rates(ai)
    print(f"人間訳 {len(human)} 文書（{h_chars:,} 字） / AI訳 {len(ai)} 文書（{a_chars:,} 字）")
    print()
    print(f"{'パターン':<20} {'人間':>8} {'AI':>8} {'比':>8}")
    for name in CANDIDATES:
        h, a = h_rate[name], a_rate[name]
        ratio = "—" if h == 0 else f"{a / h:.1f}"
        if h == 0 and a > 0:
            ratio = "∞"
        print(f"{name:<20} {h:>8.2f} {a:>8.2f} {ratio:>8}")


if __name__ == "__main__":
    main()


# 追加の候補。明文の作法書に基づくもの。
# 二重否定は「公用文作成の考え方」が分かりにくい書き方として挙げている。
EXTRA: dict[str, str] = {
    "二重否定": r"(ない|なく|ず)[^。]{0,8}(ない|ません|ぬ)(。|、|$)",
    "の連鎖3つ": r"[^。、\s]{1,8}の[^。、\s]{1,8}の[^。、\s]{1,8}の",
}
