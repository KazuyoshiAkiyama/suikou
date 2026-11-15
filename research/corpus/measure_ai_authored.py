"""生成した文書に備忘が現れたかを測る。

AI 執筆コーパスの収集方法が成立したかを判定するために使う。
手順は `research/corpus/ai_authored.md` にある。

既に規則にしてある形（`structure/work-note` の4つ）が現れるかを数える。
現れなければ、生成の方法が条件を再現していない。
現れれば、規則が覆っていない形を探す段に進める。

比較の相手は人間が書いた文書とする。
`corpus/cache/readmes` と `corpus/cache/pro_ja` を使う。

使い方: python research/corpus/measure_ai_authored.py <生成した文書のディレクトリ>
"""

import pathlib
import re
import sys

# `structure/work-note` が検出する4つの形。実装と同じ正規表現である。
WORK_JA = {
    "試した案": r"(試し|検討し|実験し)(た|たが|てみた)(が|結果|ところ|。)|うまく(いかな|動かな)かった",
    "判明したこと": r"(判明した|分かった|気付いた|発覚した)(。|ため|ので)",
    "作業の段階": r"(第|フェーズ|段階|ステップ)\s*\d+\s*(は|が)?(完了|終了|済み|進行中)"
                  r"|(実装|移行|対応)(は|が)(完了|進行中|途中)(した|である|。)",
    "以前はこうだった": r"(もともと|元々|当初|以前)は[^。]{0,40}(だった|していた)|(旧|古い)(版|バージョン)では",
}
WORK_EN = {
    "試した案": r"(?i)\b(we|i) (tried|attempted|experimented with|considered)\b"
                r"|\bafter (trying|testing|experimenting)\b",
    "判明したこと": r"(?i)\b(we|i) (found|noticed|realized|discovered|learned) that\b"
                    r"|\bit turn(s|ed) out\b",
    "作業の段階": r"(?i)\b(phase|step|stage|milestone) \d+ (is|was|has been) "
                  r"(complete|completed|done|finished)\b"
                  r"|\b(implementation|migration|refactor(ing)?|rewrite) is "
                  r"(complete|done|ongoing|in progress)\b",
    "以前はこうだった": r"(?i)\boriginally (this|it|the|we|there)\b|\bused to (be|use|have|work|live)\b"
                        r"|\bin (earlier|previous) versions,? (this|it|we)\b",
}

# 規則が覆っていない候補。現れるかを見るだけで、採否はここでは決めない。
EXTRA_JA = {
    "注意書き": r"(注意|留意|なお)[：:]|注意点として|ただし現状",
    "未実装の告知": r"(未実装|未対応|今のところ|現状では)",
    "自分への指示": r"(要検討|TODO|FIXME|後で|いずれ)",
    "作業の量": r"(行の|件の|個の)(変更|修正|追加)を(行った|加えた)",
}
EXTRA_EN = {
    "注意書き": r"(?i)\b(note|caveat|gotcha)s?:|\bkeep in mind\b|\bbe aware\b",
    "未実装の告知": r"(?i)\b(not implemented|unimplemented|for now|at this stage)\b",
    "自分への指示": r"(?i)\b(TODO|FIXME|revisit|later on)\b",
    "作業の量": r"(?i)\b(added|changed|modified) \d+ (lines?|files?|tests?)\b",
}


def body(src: str) -> str:
    src = re.sub(r"(?s)```.*?```", "", src)
    src = re.sub(r"(?s)\A---\n.*?\n---\n", "", src)
    return re.sub(r"`[^`]*`", "", src)


def is_ja(name: str) -> bool:
    return ".ja." in name or name.endswith("_ja.md")


def count(files: list[pathlib.Path], pats: dict[str, str]) -> tuple[dict[str, int], int]:
    hits = {k: 0 for k in pats}
    total = 0
    for p in files:
        t = body(p.read_text(encoding="utf-8", errors="replace"))
        total += len(t)
        for k, v in pats.items():
            hits[k] += len(re.findall(v, t))
    return hits, total


def report(label: str, files: list[pathlib.Path], ja: bool) -> None:
    if not files:
        print(f"{label}: 文書が無い")
        return
    work, chars = count(files, WORK_JA if ja else WORK_EN)
    extra, _ = count(files, EXTRA_JA if ja else EXTRA_EN)
    print(f"\n{label}  {len(files)} 件 {chars:,} 字")
    print("  規則にしてある形（work-note）")
    for k, v in work.items():
        print(f"    {k:<16}{v:>4} 件  {v * 10000 / chars:>6.2f}/万")
    print("  規則が覆っていない候補")
    for k, v in extra.items():
        print(f"    {k:<16}{v:>4} 件  {v * 10000 / chars:>6.2f}/万")


def main() -> None:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "")
    gen = sorted(p for p in root.rglob("*.md") if p.is_file())
    report("生成（日本語）", [p for p in gen if is_ja(p.name)], True)
    report("生成（英語）", [p for p in gen if not is_ja(p.name)], False)
    report(
        "人間（英語、著名 OSS の README）",
        sorted(pathlib.Path("corpus/cache/readmes").glob("*.md"))[:60],
        False,
    )
    report(
        "人間（日本語、参照コーパス）",
        sorted(pathlib.Path("corpus/cache/pro_ja").glob("*.md"))[:60],
        True,
    )


if __name__ == "__main__":
    main()
