"""README の前向きの情報と、著者の文脈の書き留めを区別して測る。

最初の計測は2つを混同していた。
著名な OSS の `currently` や `Roadmap` は読み手に向けた情報である。
今後の計画を示し、採用の判断を助け、支援を募る。読み手に次の行動を与える。

AI エージェントが書くのは向きが逆である。
自分が後で読むための文脈であり、読み手に行動を与えない。
試した案、変更の経緯、未着手の箇所である。

両者を分ける手掛かりを3つ測る。
  区画されているか … 読み手が飛ばせる見出しの下にあるか、説明の地の文に混ざっているか
  主語が何か       … 機能や版か、作業や変更そのものか
  宛先があるか     … 読み手への呼びかけや参照先を伴うか

使い方: python measure_memo2.py [corpus/cache/readmes]
"""

import re
import sys
import pathlib
from collections import Counter

# 前向きの情報と文脈の書き留めに共通して現れる語。
MARKERS = r"\b(currently|for now|at the moment|planned|coming soon|not yet|TODO|FIXME|roadmap|in progress|work in progress|WIP)\b"

# 読み手が飛ばせる区画。見出しがその中身を予告している。
SIGNPOST = re.compile(
    r"(?i)\b(status|roadmap|known issues|limitations|future|plans?|planned|"
    r"upcoming|todo|backlog|what's next|coming|progress|help wanted|"
    r"contributing|support|sponsor|funding|changelog|release notes|history)\b"
)

# 読み手に行動を与える形。呼びかけ、参照先、判断の材料。
ADDRESSED = re.compile(
    r"(?i)(#\d+|issues?/\d+|\bpull request|\bPR\b|contributions? (are )?welcome|"
    r"help wanted|please |if you |you can |see the|feel free|we welcome|"
    r"\bvote\b|\bsponsor|\bdonate|\bupvote)"
)

# 作業そのものを主語にする形。読み手に行動を与えない。
ABOUT_THE_WORK = re.compile(
    r"(?i)\b(we (refactored|rewrote|moved|renamed|changed|replaced|tried|"
    r"decided|realized|found)|was (refactored|rewritten|moved|renamed)|"
    r"i (tried|found|noticed|changed|decided)|after (trying|testing)|"
    r"originally (this|it|the)|used to be|the implementation (was|has been))"
)


def strip_code(src: str) -> str:
    return re.sub(r"(?s)```.*?```", "", src)


def sections(src: str) -> list[tuple[str, str]]:
    """(見出し, その節の地の文) の並び。見出しの前の部分は空の見出しで表す。"""
    src = strip_code(src)
    out = []
    pos = 0
    last = ""
    for m in re.finditer(r"(?m)^#{1,6}\s+(.+?)\s*#*$", src):
        out.append((last, src[pos : m.start()]))
        last = m.group(1)
        pos = m.end()
    out.append((last, src[pos:]))
    return out


def main() -> None:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/readmes")
    files = sorted(root.glob("*.md"))

    inside = 0
    outside = 0
    addressed = 0
    about_work = 0
    plain = 0
    examples_out = []

    for p in files:
        for head, body in sections(p.read_text(encoding="utf-8", errors="replace")):
            signposted = bool(SIGNPOST.search(head))
            for sent in re.split(r"(?<=[.!?])\s+|\n\s*\n|\n\s*[-*+]\s+", body):
                t = re.sub(r"\s+", " ", sent).strip()
                if not t or not re.search(MARKERS, t, re.I):
                    continue
                if signposted:
                    inside += 1
                    continue
                outside += 1
                if ADDRESSED.search(t):
                    addressed += 1
                elif ABOUT_THE_WORK.search(t):
                    about_work += 1
                    if len(examples_out) < 6:
                        examples_out.append((p.name, t[:100]))
                else:
                    plain += 1

    total = inside + outside
    print(f"README {len(files)} 件。前向きの語を含む文 {total} 件\n")
    print(f"{'区画':<34}{'件数':>7}{'率':>7}")
    print(f"{'予告した見出しの下にある':<34}{inside:>7}{inside / total:>7.0%}")
    print(f"{'説明の地の文に混ざっている':<34}{outside:>7}{outside / total:>7.0%}")
    print()
    print("地の文に混ざった分の内訳")
    print(f"{'  読み手に行動を与える':<34}{addressed:>7}{addressed / outside:>7.0%}")
    print(f"{'  作業そのものを述べる':<34}{about_work:>7}{about_work / outside:>7.0%}")
    print(f"{'  どちらでもない':<34}{plain:>7}{plain / outside:>7.0%}")
    if examples_out:
        print("\n作業そのものを述べた例")
        for name, t in examples_out:
            print(f"  {name[:34]:<36}{t}")


if __name__ == "__main__":
    main()
