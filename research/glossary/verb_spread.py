"""動詞の曖昧さを、訳の分かれ方で測る。

1つの日本語の動詞が複数の別の動作を兼ねているかは、取る名詞の種類では測れない。
`含む` は多くの名詞を取るが意味は1つである。

そこで、その動詞を含む文を英語に訳させ、現れる英語の動詞の種類を数える。
種類が多い動詞は、読み手が文脈から意味を埋めることになる。
置き換え先に別の広い動詞を選んでも、曖昧さは減らない。

使い方:
    plamo-translate server &
    python research/glossary/verb_spread.py
"""

import glob
import pathlib
import re
import subprocess
import sys
from collections import Counter

CLI = "plamo-translate"
VERBS = ["置く", "設定する", "配置する", "作る", "作成する", "行う", "示す", "定める"]
PER_VERB = 8

# 英語の動詞らしい語をこの一覧から拾う。訳文から主動詞を取り出すために使う。
STOP = {
    "is", "are", "was", "were", "be", "been", "being", "has", "have", "had",
    "do", "does", "did", "can", "will", "would", "should", "may", "might",
    "the", "a", "an", "to", "of", "in", "on", "for", "and", "or", "that",
    "this", "it", "you", "we", "they", "not", "with", "by", "as", "if",
}


def body(s: str) -> str:
    s = re.sub(r"(?s)\A---\n.*?\n---\n", "", s)
    s = re.sub(r"(?s)```.*?```", "", s)
    return re.sub(r"`[^`]*`", "", s)


def sentences(verb: str) -> list[str]:
    out = []
    for p in sorted(glob.glob("corpus/cache/pro_ja/*.md")):
        text = body(pathlib.Path(p).read_text(encoding="utf-8", errors="replace"))
        for s in re.split(r"(?<=。)", text.replace("\n", "")):
            s = s.strip()
            if verb in s and 20 <= len(s) <= 120:
                out.append(s)
                if len(out) >= PER_VERB:
                    return out
    return out


def translate(text: str) -> str:
    r = subprocess.run(
        [CLI, "--from", "Japanese", "--to", "English", "--input", text, "--no-stream"],
        capture_output=True,
        text=True,
        timeout=240,
    )
    lines = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    return lines[-1] if lines else ""


def verbs_in(en: str) -> set[str]:
    words = re.findall(r"[a-z]+", en.lower())
    return {w for w in words if w not in STOP and len(w) > 3}


def main() -> None:
    print(f"{'動詞':<10}{'文':>4}{'英語の語の種類':>14}{'1文あたり':>10}")
    detail = {}
    for v in VERBS:
        ss = sentences(v)
        if not ss:
            print(f"{v:<10}{0:>4}  コーパスに例が無い")
            continue
        seen: Counter[str] = Counter()
        for s in ss:
            for w in verbs_in(translate(s)):
                seen[w] += 1
        # 2文以上に現れた語は話題の語である。1文だけの語がその動詞の訳の揺れを表す。
        once = [w for w, c in seen.items() if c == 1]
        print(f"{v:<10}{len(ss):>4}{len(seen):>14}{len(once) / len(ss):>10.1f}")
        detail[v] = seen.most_common(8)
    print("\nよく現れた英語の語:")
    for v, top in detail.items():
        print(f"  {v:<10} " + " ".join(f"{w}×{c}" for w, c in top))


if __name__ == "__main__":
    main()
