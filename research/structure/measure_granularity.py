"""節ごとの粒度を測る。テンプレートに書ける値かを確かめるために使う。

既存の型（`structure.toml`）は、どの節を置くかと、その節が答える問いを持つ。
どれだけ書くかは持たない。
書く側はそこを自分で決めることになり、節ごとの分量が揃わない。

節ごとに、長さと中身の構成を測る。
中央値のばらつきが小さければ、テンプレートに書ける。

使い方: python research/structure/measure_granularity.py <コーパス> [見出しの言語]
"""

import pathlib
import re
import statistics
import sys

# 節の同定。`measure_readmes.py` の同義語と同じ分類にする。
#
# 部分一致は使わない。`What's New` が `説明` に、
# 連結を含む見出しが別の節に入る誤りが出る。
SECTIONS: dict[str, list[str]] = {
    "説明": ["about", "overview", "introduction", "what is", "what it is", "description",
             "synopsis"],
    "機能": ["features", "feature", "why", "highlights", "capabilities", "benefits"],
    "入れ方": ["install", "installation", "setup", "getting started", "quick start",
               "quickstart", "building", "build", "download"],
    "使い方": ["usage", "using", "example", "examples", "how to use", "basic usage",
               "quick example"],
    "設定": ["configuration", "config", "options", "settings"],
    "文書": ["documentation", "docs", "guide", "guides", "reference", "manual",
             "learn more"],
    "寄与": ["contributing", "contribute", "development", "developing", "contributors"],
    "許諾": ["license", "licence", "licensing"],
}

# 本文がこれより短い節は、中身が小見出しの側にある。粒度の標本にならない。
MIN_BODY = 40


def clean_heading(h: str) -> str:
    """見出しから連結と画像と飾りを取り除く。"""
    h = re.sub(r"!\[[^\]]*\]\([^)]*\)", "", h)
    h = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", h)
    h = re.sub(r"<[^>]+>", "", h)
    h = re.sub(r"[*_`~#:]", "", h)
    h = re.sub(r"[\U0001F000-\U0001FAFF\u2600-\u27BF]", "", h)
    return h.strip().lower()


def classify(h: str) -> str | None:
    h = clean_heading(h)
    if not h:
        return None
    for name, syns in SECTIONS.items():
        for s in syns:
            # 一致か、語として先頭か末尾に来る場合だけを採る。
            if h == s or h.startswith(s + " ") or h.endswith(" " + s):
                return name
    return None


def is_target_language(src: str) -> bool:
    """日本語か英語の文書かどうか。中国語の README を除くために使う。"""
    kana = sum(1 for c in src if "\u3040" <= c <= "\u30ff")
    han = sum(1 for c in src if "\u4e00" <= c <= "\u9fff")
    latin = sum(1 for c in src if c.isascii() and c.isalpha())
    # 漢字が多く仮名が少ない場合は中国語とみなす。
    if han > 200 and kana < han / 10:
        return False
    return latin > 200 or kana > 200


def blocks_of(body: str) -> dict:
    """節の中身の構成。コード塊、箇条書き、表、地の文の量を数える。"""
    code = len(re.findall(r"(?m)^```", body)) // 2
    without_code = re.sub(r"(?s)```.*?```", "", body)
    lines = [l for l in without_code.split("\n") if l.strip()]
    bullets = sum(1 for l in lines if re.match(r"^\s*(?:[-*+]\s|[0-9]+[.)]\s)", l))
    table = sum(1 for l in lines if l.lstrip().startswith("|"))
    prose = [l for l in lines if l not in ()
             and not re.match(r"^\s*(?:[-*+]\s|[0-9]+[.)]\s|\||#|>)", l)]
    return {
        "字数": len(without_code),
        "コード塊": code,
        "箇条書き行": bullets,
        "表の行": table,
        "地の文の行": len(prose),
        "小見出し": len(re.findall(r"(?m)^#{3,4}\s", body)),
    }


def sections_of(src: str) -> list[tuple[str, str]]:
    src = re.sub(r"(?s)<!--.*?-->", "", src)
    heads = list(re.finditer(r"(?m)^(#{2,3})\s+(.+?)\s*#*$", src))
    out = []
    for i, m in enumerate(heads):
        end = heads[i + 1].start() if i + 1 < len(heads) else len(src)
        out.append((m.group(2), src[m.end():end]))
    return out


def main() -> None:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/readmes")
    per: dict[str, list[dict]] = {k: [] for k in SECTIONS}
    files = sorted(root.glob("*.md"))
    kept = 0
    for p in files:
        src = p.read_text(encoding="utf-8", errors="replace")
        if not is_target_language(src):
            continue
        kept += 1
        for head, body in sections_of(src):
            name = classify(head)
            # 中身が小見出しの側にある節は、粒度の標本にならない。
            if name and len(body.strip()) >= MIN_BODY:
                per[name].append(blocks_of(body))

    print(f"{root.name} {kept} 件（対象の言語でない {len(files) - kept} 件を除外）\n")
    keys = ["字数", "コード塊", "箇条書き行", "地の文の行", "小見出し"]
    print(f"{'節':<8}{'件数':>6}" + "".join(f"{k:>22}" for k in keys))
    print(f"{'':<8}{'':>6}" + "".join(f"{'下位 中央 上位':>22}" for _ in keys))
    for name, rows in per.items():
        if len(rows) < 8:
            print(f"{name:<8}{len(rows):>6}   標本が足りない")
            continue
        line = f"{name:<8}{len(rows):>6}"
        for k in keys:
            v = sorted(r[k] for r in rows)
            q = statistics.quantiles(v, n=4)
            line += f"{q[0]:>7.0f}{statistics.median(v):>7.0f}{q[2]:>8.0f}"
        print(line)


if __name__ == "__main__":
    main()
