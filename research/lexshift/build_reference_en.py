"""プロの英語の文書から、文体の参照になる頻度表を作る。

英語には語種の区別がないため、和語に絞るような構造の絞り込みは使えない。
機能語は原文と訳文で頻度が揃うため、対数尤度比の側が自然に除く。

トークンの定義は `crates/suikou-core/src/metrics/en.rs` の `tokenize_words` に合わせる。
小文字にした表層形とし、前後の記号を落とす。Rust 側に英語の見出し語化が無いため、
表層形で持つ方が両側で同じ数え方になる。

使い方: python build_reference_en.py <プロ英文のディレクトリ> <出力の toml>
"""
import collections, datetime, pathlib, re, sys
from chunks import aligned, kind

MIN_COUNT = 3
MIN_DOMAINS = 2
INLINE = re.compile(r"`[^`]*`|\{\{[^}]*\}\}|\[([^\]]*)\]\([^)]*\)|<[^>]+>")


def words(text):
    text = INLINE.sub(lambda m: m.group(1) or " ", text)
    out = []
    for w in text.split():
        w = w.strip("".join(c for c in w if not c.isalnum())) if not w.isalnum() else w
        w = w.strip(".,:;!?()[]{}\"'`*_|<>#").lower()
        if w and any(c.isalpha() for c in w):
            out.append(w)
    return out


def main():
    d, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    groups = collections.defaultdict(list)
    for p in sorted(d.glob("*.md")):
        groups[p.name.split("__")[0]].append(p)
    per = {}
    for g, ps in groups.items():
        c = collections.Counter(); n = 0
        for p in ps:
            for ch in aligned(p.read_text(errors="ignore")):
                if kind(ch) == "code":
                    continue
                ws = words(ch); n += len(ws); c.update(ws)
        per[g] = (c, n, len(ps))
    total, domains = collections.Counter(), collections.Counter()
    for c, _, _ in per.values():
        total.update(c); domains.update(set(c))
    keep = sorted((w, k) for w, k in total.items() if k >= MIN_COUNT and domains[w] >= MIN_DOMAINS)
    ref_words = sum(n for _, n, _ in per.values())
    lines = [
        "# プロの英語の文書から作った語の頻度表。小文字にした表層形で持つ。",
        "# 調べる文書での頻度をこの表と比べ、偏って多い語を文体の指摘に出す。",
        "# 作り直す手順は research/lexshift/build_reference_en.py にある。",
        "",
        "[meta]",
        f'measured = "{datetime.date.today()}"',
        f"min_domains = {MIN_DOMAINS}",
        f"min_count = {MIN_COUNT}",
        "content_words = " + str(ref_words),
        "pages = " + str(sum(p for _, _, p in per.values())),
        'sources = ["kubernetes/website(en)", "mdn/content", "vuejs/docs"]',
        "",
        "[words]",
    ]
    lines += [f'"{w}" = {k}' for w, k in keep if '"' not in w and "\\" not in w]
    out.write_text("\n".join(lines) + "\n")
    for g, (c, n, p) in per.items():
        print(f"{g}: {p} 頁、語 {n}、異なり {len(c)}")
    print(f"頻度表の語: {len(keep)}、参照の語 {ref_words} / 出力 {out} ({out.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
