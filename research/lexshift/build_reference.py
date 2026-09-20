"""プロの日本語訳から、文体の参照になる和語の頻度表を作る。

規則の側は、調べる文書での頻度と参照の頻度を対数尤度比で比べる。
あるかないかの二値にしないのは、参照を大きくすると稀な語でも数回は現れ、
二値では「文体から外れた語」と「ただ珍しい語」を見分けられなくなるためである。

使い方: python build_reference.py <プロ訳のディレクトリ> ../../crates/suikou-core/data/register_ja.toml
"""
import collections, pathlib, re, sys, datetime
import fugashi
from chunks import aligned, kind
from analyze import units, INLINE

MIN_COUNT = 3        # 分野をまたいだ合計の下限。一度きりの出現を拾わないため
MIN_DOMAINS = 2      # 現れる分野の数の下限。一つの分野の癖を拾わないため

# 参照に使う分野。仕様や手引きの文書に揃える。
# 入門書（Rust Book、React の学ぶ）は文体が柔らかく、「確かめる」が36回出るなど
# 語の選び方が違う。設計書にあるとおり、文書の種類による差は文体による差より大きい。
REFERENCE_STYLE = {"k8s", "mdn", "vue"}
TAGGER = fugashi.Tagger()
CONTENT = {"名詞", "動詞", "形容詞", "形状詞", "副詞"}
KATAKANA_ONE = re.compile(r"^[ァ-ヶー]$")


def wago(text):
    text = INLINE.sub(lambda m: m.group(1) or " ", text)
    out = []
    for w in TAGGER(text):
        f = w.feature
        if f.pos1 not in CONTENT or getattr(f, "goshu", "") != "和":
            continue
        if f.pos2 == "数詞":
            continue
        # Rust 側の Token は語彙素を持つため、語彙素で揃える。
        base = f.lemma or w.surface
        if KATAKANA_ONE.match(base) or not re.search(r"[ぁ-んァ-ヶ一-龯]", base):
            continue
        out.append(base)
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
                n += len(units(ch)[0]); c.update(wago(ch))
        per[g] = (c, n, len(ps))
    used = {g: v for g, v in per.items() if g in REFERENCE_STYLE}
    total, domains = collections.Counter(), collections.Counter()
    for c, _, _ in used.values():
        total.update(c)
        domains.update(set(c))
    # 頻度表に載せるのは、2分野以上に現れる語とする。
    # 1分野だけの語は、その分野の言い回しであって参照にならない。
    keep = sorted((w, k) for w, k in total.items() if domains[w] >= MIN_DOMAINS)
    reference_words = sum(n for _, n, _ in used.values())
    lines = [
        "# プロの日本語訳から作った、和語の頻度表。語彙素で持つ。",
        "# 調べる文書での頻度をこの表と比べ、偏って多い語を文体の指摘に出す。",
        "# 作り直す手順は research/lexshift/build_reference.py にある。",
        "",
        "[meta]",
        f'measured = "{datetime.date.today()}"',
        f"min_domains = {MIN_DOMAINS}",
        "content_words = " + str(reference_words),
        "pages = " + str(sum(pg for _, _, pg in used.values())),
        'sources = ["kubernetes/website(ja)", "mdn/translated-content(ja)", "vuejs-translations/docs-ja"]',
        'note = "入門書は文体が違うため参照に入れない"',
        "",
        "[words]",
    ]
    lines += [f'"{w}" = {k}' for w, k in keep]
    out.write_text("\n".join(lines) + "\n")
    for g, (c, n, p) in per.items():
        print(f"{g}: {p} 頁、内容語 {n}、和語の異なり {len(c)}")
    print(f"頻度表の語: {len(keep)}、参照の内容語 {reference_words} / 出力 {out} ({out.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
