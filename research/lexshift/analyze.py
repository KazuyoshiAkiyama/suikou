"""同じ英語の原文から作ったプロ訳と LLM 訳を比べ、LLM が偏って使う語と、
プロがその位置で代わりに使う語を、段落の対応から機械的に取り出す。

使い方: python analyze.py <キャッシュのディレクトリ> <LLM訳の接尾辞（例: sonnet）>

判定は次の手順による。人の判断で語を選ぶ段階は含まない。

1. 過剰使用の検出。語ごとに LLM 訳とプロ訳の頻度を比べ、対数尤度比 G²
   （Dunning 1993。Rayson and Garside 2000 の二コーパス比較の形）と
   Log Ratio（Hardie 2014）を出す。G² の閾値は慣例の 15.13（p < 0.0001）とする。
   一つの頁だけに偏った語を拾わないよう、LLM 訳の中で MIN_PAGES 頁以上に出ることを条件にする。
2. 置き換え先の発見。原文の同じ段落を訳した LLM 訳とプロ訳の組のうち、
   LLM 訳にその語があってプロ訳にない組を集める。その組のプロ訳にだけ現れる語を数え、
   全段落での現れやすさに対する持ち上がり度（lift）で順位を付ける。
3. 原文の語の特定。同じ組の英語の段落の語を、同じ lift で順位付けする。
"""
import json, math, re, sys, pathlib, collections
import fugashi
from chunks import aligned, kind

MIN_PAGES = 3          # 1頁に偏った語を除くための下限
G2_CRIT = 15.13        # p < 0.0001（自由度1）の慣例の臨界値
MIN_SUPPORT = 2        # 置き換え先として挙げるのに要る段落の組の数
CONTENT = {"名詞", "動詞", "形容詞", "形状詞", "副詞", "連体詞", "代名詞"}
TAGGER = fugashi.Tagger()
INLINE = re.compile(r"`[^`]*`|\{\{[^}]*\}\}|\[([^\]]*)\]\([^)]*\)|<[^>]+>")
EN_WORD = re.compile(r"[A-Za-z][a-z]+")


def units(text):
    """内容語の単位を返す。接尾辞は直前の名詞につなげる（効果+的 → 効果的）。
    書き分けを区別したいため、語彙素ではなく書字形基本形で数える。"""
    text = INLINE.sub(lambda m: m.group(1) or " ", text)
    out, pos = [], []
    for w in TAGGER(text):
        f = w.feature
        p1 = f.pos1 or ""
        base = f.orthBase or w.surface
        if not re.search(r"[ぁ-んァ-ヶ一-龯]", base):
            continue
        if p1 == "接尾辞" and out and pos[-1] == "名詞":
            out[-1] += base
            continue
        if p1 in CONTENT:
            out.append(base); pos.append(p1)
    return out, pos


def g2(a, b, A, B):
    e1 = A * (a + b) / (A + B); e2 = B * (a + b) / (A + B)
    s = 0.0
    if a: s += a * math.log(a / e1)
    if b: s += b * math.log(b / e2)
    return 2 * s


def log_ratio(a, b, A, B):
    return math.log2(((a + 0.5) / A) / ((b + 0.5) / B))


def main():
    d = pathlib.Path(sys.argv[1]); tag = sys.argv[2]
    rows = []                           # (頁, 英語の段落, プロ訳の段落, LLM訳の段落)
    for en_path in sorted(d.glob("*.en.md")):
        llm_path = en_path.with_name(en_path.name.replace(".en.md", f".{tag}.md"))
        ja_path = en_path.with_name(en_path.name.replace(".en.md", ".ja.md"))
        if not llm_path.exists():
            continue
        E, J, L = (aligned(p.read_text()) for p in (en_path, ja_path, llm_path))
        if not (len(E) == len(J) == len(L)):
            continue
        for e, j, l in zip(E, J, L):
            if kind(e) == "code" or [kind(e), kind(j), kind(l)].count(kind(e)) != 3:
                continue
            rows.append((en_path.name, e, j, l))
    pages = len({r[0] for r in rows})

    pro_cnt, llm_cnt = collections.Counter(), collections.Counter()
    llm_pages = collections.defaultdict(set); posof = {}
    pro_df, en_df = collections.Counter(), collections.Counter()
    sets = []
    for page, e, j, l in rows:
        ju, jp = units(j); lu, lp = units(l)
        pro_cnt.update(ju); llm_cnt.update(lu)
        for u, p in zip(ju + lu, jp + lp):
            posof.setdefault(u, p)
        for u in set(lu):
            llm_pages[u].add(page)
        ews = set(w.lower() for w in EN_WORD.findall(e))
        pro_df.update(set(ju)); en_df.update(ews)
        sets.append((set(ju), set(lu), ews))
    P, Lt = sum(pro_cnt.values()), sum(llm_cnt.values()); N = len(sets)

    def keyness(over):
        res = []
        for u in set(pro_cnt) | set(llm_cnt):
            a, b = llm_cnt[u], pro_cnt[u]
            if over and len(llm_pages[u]) < MIN_PAGES:
                continue
            g = g2(a, b, Lt, P); lr = log_ratio(a, b, Lt, P)
            if g >= G2_CRIT and ((lr > 0) == over):
                res.append((u, a, b, g, lr))
        return sorted(res, key=lambda x: -x[3])

    def substitutes(u):
        hit = [(ju, lu, ews) for ju, lu, ews in sets if u in lu and u not in ju]
        n = len(hit)
        sub, src = collections.Counter(), collections.Counter()
        for ju, lu, ews in hit:
            sub.update(ju - lu); src.update(ews)
        def top(cnt, df, k=3):
            sc = [(w, c, (c / n) / (df[w] / N)) for w, c in cnt.items() if c >= MIN_SUPPORT and df[w]]
            sc = [x for x in sc if x[2] >= 2.0]
            return sorted(sc, key=lambda x: (-x[1] * math.log(x[2]), x[0]))[:k]
        return n, top(sub, pro_df), top(src, en_df)

    print(f"# LLM 訳（{tag}）とプロ訳の語の比較\n")
    print(f"対応づけた頁 {pages}、段落の組 {N}、内容語 LLM {Lt} / プロ {P}\n")
    print("## LLM が偏って使う語\n")
    print("| 語 | 品詞 | LLM | プロ | G² | LogRatio | 段落の組 | プロが代わりに使う語 | 原文の語 |")
    print("|---|---|---|---|---|---|---|---|---|")
    for u, a, b, g, lr in keyness(True)[:40]:
        n, subs, srcs = substitutes(u)
        s1 = "、".join(f"{w}({c}, {posof.get(w,'')})" for w, c, _ in subs) or "—"
        s2 = ", ".join(f"{w}({c})" for w, c, _ in srcs) or "—"
        print(f"| {u} | {posof.get(u,'')} | {a} | {b} | {g:.1f} | {lr:+.2f} | {n} | {s1} | {s2} |")
    print("\n## LLM が避ける語（プロが偏って使う語）\n")
    print("| 語 | 品詞 | LLM | プロ | G² | LogRatio |")
    print("|---|---|---|---|---|---|")
    for u, a, b, g, lr in keyness(False)[:30]:
        print(f"| {u} | {posof.get(u,'')} | {a} | {b} | {g:.1f} | {lr:+.2f} |")


if __name__ == "__main__":
    main()
