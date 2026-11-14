"""README に備忘録が混ざる度合いを測る。

AI に文書を書かせると、書き手が自分のために残す覚書が README に入る。
進行の状態、やってみて分かったこと、まだ直していない箇所などである。
読み手はそれを必要としない。Diátaxis が言う、一つの文書が複数の必要に応える状態である。

候補の列挙は人が行い、採否は実測で決める。D-26 と同じ手順である。

比べる2群は次のとおり。
  著名 OSS … GitHub の星の数で選んだ README。読み手が多く、直され続けている
  このリポジトリ … すべて AI が書いた文書。CLAUDE.md がそう宣言している

出力は1万文字あたりの出現数である。文書の長さが揃わないため、率で比べる。

使い方: python measure_memo.py
"""

import re
import pathlib
from collections import Counter

# 候補。読み手の必要ではなく書き手の覚書を示す形に絞る。
CANDIDATES: dict[str, str] = {
    # 進行の状態。時点に依存し、直されないまま古くなる。
    "currently/for now": r"\b(currently|for now|at the moment|at present|as of now)\b",
    "work in progress": r"\b(work in progress|WIP|under construction|not yet (implemented|supported|done))\b",
    "TODO/FIXME": r"\b(TODO|FIXME|XXX|HACK)\b",
    "coming soon": r"\b(coming soon|planned|will be added|future work|in the future)\b",
    # 書き手の経緯。読み手の操作には関わらない。
    "一人称の経緯": r"\b(I (added|wrote|changed|decided|tried|found|noticed)|we (decided|realized|found that))\b",
    "以前はこうだった": r"\b(previously|originally|used to be|in earlier versions|has been (refactored|rewritten|moved))\b",
    "note that": r"\bnote that\b",
    # 文書そのものへの言及。読み手には不要な自己言及である。
    "この文書への言及": r"\bthis (document|file|readme|section) (describes|explains|covers|contains)\b",
}

# 見出しに置かれた覚書。節として立っている場合は別に数える。
HEADING_MEMO = r"(?mi)^#{1,4}\s+.*\b(status|roadmap|changelog|known issues|limitations|notes|todo|history|what's new)\b"


def body(src: str) -> str:
    src = re.sub(r"(?s)\A---\n.*?\n---\n", "", src)
    src = re.sub(r"(?s)```.*?```", "", src)
    src = re.sub(r"(?s)<!--.*?-->", "", src)
    src = re.sub(r"`[^`]*`", "", src)
    return src


def rates(paths: list[pathlib.Path]) -> tuple[dict[str, float], dict[str, int], int, int]:
    total = 0
    hits = Counter()
    docs = Counter()
    head = 0
    for p in paths:
        raw = p.read_text(encoding="utf-8", errors="replace")
        text = body(raw)
        total += len(text)
        if re.search(HEADING_MEMO, raw):
            head += 1
        for name, pat in CANDIDATES.items():
            c = len(re.findall(pat, text, re.I))
            hits[name] += c
            if c:
                docs[name] += 1
    if total == 0:
        return {}, {}, 0, 0
    return {k: hits[k] * 10000 / total for k in CANDIDATES}, dict(docs), total, head


def main() -> None:
    oss = sorted(pathlib.Path("corpus/cache/readmes").glob("*.md"))
    mine = [
        p for p in pathlib.Path(".").glob("*.md")
    ] + sorted(pathlib.Path("docs/content/en").glob("*.md"))
    mine = [p for p in mine if p.name not in ("TASKS.md",)]

    o_rate, o_docs, o_chars, o_head = rates(oss)
    m_rate, m_docs, m_chars, m_head = rates(mine)
    print(f"著名 OSS {len(oss)} 件（{o_chars:,} 字） / このリポジトリ {len(mine)} 件（{m_chars:,} 字）")
    print(f"覚書の見出しを持つ割合: OSS {o_head / len(oss):.0%} / 自分 {m_head / len(mine):.0%}\n")
    print(f"{'候補':<22}{'OSS':>8}{'自分':>8}{'比':>7}   OSS で現れる文書数")
    for name in CANDIDATES:
        o, m = o_rate[name], m_rate[name]
        ratio = "∞" if o == 0 and m > 0 else ("—" if o == 0 else f"{m / o:.1f}")
        print(f"{name:<22}{o:>8.2f}{m:>8.2f}{ratio:>7}   {o_docs.get(name, 0)}/{len(oss)}")


if __name__ == "__main__":
    main()
