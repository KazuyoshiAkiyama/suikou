"""英語の原文を軸にして、漢語とカタカナ語の対応表を作る。

同義であることを、意味の知識ではなく英語の原文から決める。
同じ英語の語を含む段落で、プロの日本語訳が当てている語を集めれば、
「version には バージョン と 版 が当たる」という対応が機械的に取れる。

どちらを選ぶかは、分野をまたいだ合計で優勢な側とする。
分野ごとに逆転する語はあるが（MDN は「既定」を使う）、平均で優勢なら従う。

使い方: python pivot_pairs.py <対応づけた頁のディレクトリ> <プロ日本語のディレクトリ> <出力 toml>
"""
import collections, datetime, pathlib, re, sys
import fugashi
from chunks import aligned, kind

MIN_PIVOT = 5        # 英語の軸の語が現れる段落の数の下限
MIN_TOTAL = 10       # 対の合計の出現数の下限
MIN_SHARE = 2.0      # 優勢とみなす比。カタカナ側が漢語側の何倍以上か
MIN_DICE = 0.25      # 軸の語との結び付きの強さ。同じ英語の語の訳であることを求める
MIN_HITS = 3         # 軸の語の段落に、その語が現れる数の下限
TAGGER = fugashi.Tagger()
INLINE = re.compile(r"`[^`]*`|\{\{[^}]*\}\}|\[([^\]]*)\]\([^)]*\)|<[^>]+>")
EN_WORD = re.compile(r"[A-Za-z][a-z]{2,}")
KATAKANA = re.compile(r"^[ァ-ヶー]+$")


def norm(lemma):
    """外来語の語彙素は由来語を伴う（バージョン-version）。前半だけを使う。"""
    return lemma.split("-")[0]


def ja_words(text):
    text = INLINE.sub(lambda m: m.group(1) or " ", text)
    out = []
    for w in TAGGER(text):
        f = w.feature
        if f.pos1 != "名詞" or f.pos2 == "数詞":
            continue
        g = getattr(f, "goshu", "")
        if g in ("漢", "外"):
            out.append((norm(f.lemma or w.surface), g))
    return out


def corpus_counts(paths):
    c = collections.Counter()
    for p in paths:
        for ch in aligned(pathlib.Path(p).read_text(errors="ignore")):
            if kind(ch) == "code":
                continue
            c.update(w for w, _ in ja_words(ch))
    return c


def main():
    aligned_dir, pro_dir, out = (pathlib.Path(a) for a in sys.argv[1:4])
    # 英語の語ごとに、対応する日本語の段落に出た名詞を集める
    assoc = collections.defaultdict(collections.Counter)
    pivot_df = collections.Counter()
    base_df = collections.Counter()
    blocks = [0]
    goshu_of = {}
    for en in sorted(aligned_dir.glob("*.en.md")):
        ja = en.with_name(en.name.replace(".en.md", ".ja.md"))
        if not ja.exists():
            continue
        E, J = aligned(en.read_text()), aligned(ja.read_text())
        if len(E) != len(J):
            continue
        for e, j in zip(E, J):
            if kind(e) == "code":
                continue
            ew = set(w.lower() for w in EN_WORD.findall(INLINE.sub(" ", e)))
            jw = ja_words(j)
            for w, g in jw:
                goshu_of[w] = g
            for x in ew:
                pivot_df[x] += 1
                assoc[x].update({w for w, _ in jw})
            base_df.update({w for w, _ in jw})
            blocks[0] += 1

    # 分野をまたいだ頻度。優劣はこちらで決める。
    groups = collections.defaultdict(list)
    for p in sorted(pro_dir.glob("*.md")):
        groups[p.name.split("__")[0]].append(p)
    per = {g: corpus_counts(ps) for g, ps in groups.items() if g in ("k8s", "mdn", "vue")}
    total = collections.Counter()
    for c in per.values():
        total.update(c)

    rows = []
    for pivot, cand in assoc.items():
        if pivot_df[pivot] < MIN_PIVOT:
            continue
        def dice(w):
            """軸の語とその語が、互いにどれだけ一緒にしか現れないかを測る。
            片方が頻出するだけでは高くならないため、同じ語の訳を選び出せる。"""
            denom = base_df[w] + pivot_df[pivot]
            return 2 * cand[w] / denom if denom else 0.0

        kata = [w for w in cand if goshu_of.get(w) == "外" and KATAKANA.match(w)
                and cand[w] >= MIN_HITS and dice(w) >= MIN_DICE]
        kan = [w for w in cand if goshu_of.get(w) == "漢"
               and cand[w] >= MIN_HITS and dice(w) >= MIN_DICE]
        if not kata or not kan:
            continue
        # 軸の語といちばん強く結びつく側どうしを対にする
        k = max(kata, key=dice)
        c = max(kan, key=dice)
        tk, tc = total[k], total[c]
        if tk + tc < MIN_TOTAL or tc == 0 or tk / tc < MIN_SHARE:
            continue
        rows.append((pivot, c, k, tc, tk, {g: (per[g][c], per[g][k]) for g in per}))
    rows.sort(key=lambda r: -(r[4] + r[3]))

    seen, lines = set(), []
    for pivot, kan, kata, tc, tk, per_g in rows:
        if kan in seen:
            continue
        seen.add(kan)
        detail = " ".join(f"{g}={a}/{b}" for g, (a, b) in sorted(per_g.items()))
        lines.append(f'"{kan}" = {{ prefer = "{kata}", pivot = "{pivot}", kango = {tc}, katakana = {tk} }}  # {detail}')
    head = [
        "# 漢語よりカタカナ語が優勢な対応表。英語の原文を軸にして機械的に作った。",
        "# 作り直す手順は research/lexshift/pivot_pairs.py にある。",
        "",
        "[meta]",
        f'measured = "{datetime.date.today()}"',
        f"min_pivot = {MIN_PIVOT}",
        f"min_total = {MIN_TOTAL}",
        f"min_share = {MIN_SHARE}",
        f"min_dice = {MIN_DICE}",
        'sources = ["kubernetes/website", "mdn/translated-content(ja)", "vuejs-translations/docs-ja"]',
        "",
        "[pairs]",
    ]
    out.write_text("\n".join(head + lines) + "\n")
    print(f"対応表に載せた語: {len(lines)}")
    for l in lines[:20]:
        print("  " + l)


if __name__ == "__main__":
    main()
