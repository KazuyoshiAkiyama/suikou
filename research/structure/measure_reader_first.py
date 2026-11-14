"""README が読み手の必要にどれだけ早く応えるかを測る。

備忘録が混ざった README は、語ではなく分量の配分で分かる。
書き手の関心（仕組み、経緯、измерение）が前に出て、
読み手の必要（何であるか、どう使うか）が後ろに回る。

測るもの。
  先頭の段落の長さ … 何であるかを短く言えているか
  最初のコード塊の位置 … 動かせる例にいつ届くか。文書全体に対する割合で測る
  入れ方の命令の位置 … `npm install` のような命令が現れる位置
  見出しまでの距離 … 最初の見出しが来るまでの文字数

使い方: python measure_reader_first.py [corpus/cache/readmes]
"""

import re
import sys
import pathlib
import statistics

INSTALL_CMD = re.compile(
    r"(?m)^\s*\$?\s*("
    r"npm\s+(i|install)|yarn\s+add|pnpm\s+add|pip\s+install|uv\s+(add|pip)|"
    r"cargo\s+(add|install)|go\s+(get|install)|gem\s+install|brew\s+install|"
    r"apt(-get)?\s+install|docker\s+(run|pull)|git\s+clone|make\s+install|curl\s+-"
    r")"
)


def strip_badges(src: str) -> str:
    """標題の直下に並ぶ画像と連結。読み手への内容ではない。"""
    src = re.sub(r"(?m)^\s*(\[!\[[^\]]*\]\([^)]*\)\]\([^)]*\)\s*)+$", "", src)
    src = re.sub(r"(?m)^\s*(!\[[^\]]*\]\([^)]*\)\s*)+$", "", src)
    src = re.sub(r"(?s)<p align=\"center\">.*?</p>", "", src)
    src = re.sub(r"(?s)<div align=\"center\">.*?</div>", "", src)
    return src


def lead_paragraph(src: str) -> str | None:
    """標題の後、最初の見出しより前にある最初の段落。"""
    m = re.search(r"(?m)^#\s+.+$", src)
    rest = src[m.end():] if m else src
    rest = strip_badges(rest)
    cut = re.search(r"(?m)^#{1,6}\s+", rest)
    if cut:
        rest = rest[: cut.start()]
    for para in re.split(r"\n\s*\n", rest):
        t = para.strip()
        t = re.sub(r"(?s)<[^>]+>", "", t)
        if len(t) >= 20 and not t.startswith(("|", ">", "-", "*", "```")):
            return t
    return None


def measure(p: pathlib.Path) -> dict | None:
    src = p.read_text(encoding="utf-8", errors="replace")
    if len(src) < 400:
        return None
    lead = lead_paragraph(src)
    code = re.search(r"(?m)^```", src)
    inst = INSTALL_CMD.search(src)
    head = re.search(r"(?m)^#{2,6}\s+", src)
    return {
        "lead": len(lead) if lead else None,
        "code": code.start() / len(src) if code else None,
        "install": inst.start() / len(src) if inst else None,
        "head": head.start() / len(src) if head else None,
        "chars": len(src),
    }


def summarise(name: str, vals: list[float], pct: bool) -> str:
    if not vals:
        return f"{name:<20} 無し"
    med = statistics.median(vals)
    q = statistics.quantiles(vals, n=4) if len(vals) > 3 else [med, med, med]
    f = (lambda v: f"{v:.0%}") if pct else (lambda v: f"{v:.0f}")
    return f"{name:<20}{f(q[0]):>9}{f(med):>9}{f(q[2]):>9}{len(vals):>7}"


def main() -> None:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/readmes")
    rows = [m for m in (measure(p) for p in sorted(root.glob("*.md"))) if m]
    n = len(rows)
    print(f"README {n} 件\n")
    print(f"{'測るもの':<20}{'下位1/4':>9}{'中央':>9}{'上位1/4':>9}{'件数':>7}")
    print(summarise("先頭の段落の字数", [r["lead"] for r in rows if r["lead"]], False))
    print(summarise("最初のコード塊", [r["code"] for r in rows if r["code"] is not None], True))
    print(summarise("入れ方の命令", [r["install"] for r in rows if r["install"] is not None], True))
    print(summarise("最初の見出し", [r["head"] for r in rows if r["head"] is not None], True))
    print(summarise("全体の字数", [r["chars"] for r in rows], False))

    have_lead = sum(1 for r in rows if r["lead"])
    print(f"\n先頭に段落を持つ: {have_lead}/{n} ({have_lead / n:.0%})")
    short = [r["lead"] for r in rows if r["lead"]]
    print(f"先頭の段落が300字以内: {sum(1 for v in short if v <= 300)}/{len(short)} "
          f"({sum(1 for v in short if v <= 300) / len(short):.0%})")
    early = [r["code"] for r in rows if r["code"] is not None]
    print(f"コード塊が前半3割の内: {sum(1 for v in early if v <= 0.3)}/{len(early)} "
          f"({sum(1 for v in early if v <= 0.3) / len(early):.0%})")


if __name__ == "__main__":
    main()


# 先頭の段落に現れる書き出しの型。
# 著名 OSS での出現率を測り、ほぼ現れない型を規則の候補とする。
LEAD_PATTERNS: dict[str, str] = {
    "文書への言及": r"^\s*(this|the following)\s+(document|file|readme|page|section|repository|repo)\b",
    "一般論から始める": r"^\s*(in recent years|as (software|systems|applications|projects)|"
                        r"traditionally|nowadays|in today's|modern (software|development)|"
                        r"with the (rise|growth|increase))",
    "書き手の経緯": r"^\s*(i (built|made|wrote|created|started)|we (built|made|started|decided)|"
                     r"this (started|began) as|originally)",
    "目的の宣言": r"^\s*(the (goal|purpose|aim|objective) of this|this (aims|intends|seeks) to)",
    "名前から始める": r"^\s*(\*\*)?[A-Z][\w.\-]*(\*\*)?\s+(is|provides|lets|allows|makes|helps|"
                      r"enables|offers|gives|brings|turns|implements|supports)\b",
    "命令で始める": r"^\s*(a|an|the)?\s*\w*\s*(library|framework|tool|CLI|engine|runtime|"
                    r"compiler|parser|server|client|database)\b",
}


def lead_report(root: pathlib.Path) -> None:
    import re as _re
    leads = []
    for p in sorted(root.glob("*.md")):
        src = p.read_text(encoding="utf-8", errors="replace")
        if len(src) < 400:
            continue
        lp = lead_paragraph(src)
        if lp:
            leads.append(_re.sub(r"\s+", " ", lp))
    print(f"\n先頭の段落を持つ README {len(leads)} 件での書き出しの型")
    print(f"{'型':<18}{'件数':>7}{'率':>7}")
    for name, pat in LEAD_PATTERNS.items():
        c = sum(1 for t in leads if _re.search(pat, t, _re.I))
        print(f"{name:<18}{c:>7}{c / len(leads):>7.0%}")
    print("\n先頭の段落の例（最初の5件）")
    for t in leads[:5]:
        print(f"  {t[:110]}")
