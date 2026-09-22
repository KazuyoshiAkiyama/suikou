"""英語の原文と、プロの日本語訳からの訳し戻しを比べる。

内容は同じで、語の選び方だけが違う。その差を統計で取り出す。
使い方: python analyze_en.py <キャッシュのディレクトリ> <訳し戻しの接尾辞（例: sonnet-en）>
"""
import collections, math, pathlib, sys
from chunks import aligned, kind
from build_reference_en import words

G2_CRIT = 15.13
MIN_PAGES = 3
MIN_SUPPORT = 2


def g2(a, b, A, B):
    e1 = A * (a + b) / (A + B); e2 = B * (a + b) / (A + B)
    s = 0.0
    if a: s += a * math.log(a / e1)
    if b: s += b * math.log(b / e2)
    return 2 * s


def main():
    d, tag = pathlib.Path(sys.argv[1]), sys.argv[2]
    rows = []
    for en in sorted(d.glob("*.en.md")):
        llm = en.with_name(en.name.replace(".en.md", f".{tag}.md"))
        if not llm.exists():
            continue
        E, L = aligned(en.read_text()), aligned(llm.read_text())
        if len(E) != len(L):
            continue
        for e, l in zip(E, L):
            if kind(e) == "code" or kind(e) != kind(l):
                continue
            rows.append((en.name, e, l))
    pro_cnt, llm_cnt = collections.Counter(), collections.Counter()
    llm_pages = collections.defaultdict(set)
    pro_df = collections.Counter(); sets = []
    for page, e, l in rows:
        ew, lw = words(e), words(l)
        pro_cnt.update(ew); llm_cnt.update(lw)
        for w in set(lw):
            llm_pages[w].add(page)
        pro_df.update(set(ew)); sets.append((set(ew), set(lw)))
    P, L = sum(pro_cnt.values()), sum(llm_cnt.values()); N = len(sets)
    print(f"# 訳し戻し（{tag}）と英語の原文の比較\n")
    print(f"対応づけた頁 {len({r[0] for r in rows})}、段落の組 {N}、語 LLM {L} / 原文 {P}\n")
    print("| LLM が使う語 | LLM | 原文 | G² | LogRatio | 原文が代わりに使う語 |")
    print("|---|---|---|---|---|---|")
    res = []
    for w in set(pro_cnt) | set(llm_cnt):
        a, b = llm_cnt[w], pro_cnt[w]
        if len(llm_pages[w]) < MIN_PAGES:
            continue
        g = g2(a, b, L, P)
        lr = math.log2(((a + 0.5) / L) / ((b + 0.5) / P))
        if g >= G2_CRIT and lr > 0:
            res.append((w, a, b, g, lr))
    for w, a, b, g, lr in sorted(res, key=lambda x: -x[3])[:30]:
        hit = [(ew, lw) for ew, lw in sets if w in lw and w not in ew]
        sub = collections.Counter()
        for ew, lw in hit:
            sub.update(ew - lw)
        n = len(hit)
        cand = [(x, c, (c / n) / (pro_df[x] / N)) for x, c in sub.items()
                if c >= MIN_SUPPORT and pro_df[x]]
        cand = sorted([c for c in cand if c[2] >= 2.0], key=lambda x: -x[1] * math.log(x[2]))[:3]
        s = ", ".join(f"{x}({c})" for x, c, _ in cand) or "—"
        print(f"| {w} | {a} | {b} | {g:.1f} | {lr:+.2f} | {s} |")


if __name__ == "__main__":
    main()
