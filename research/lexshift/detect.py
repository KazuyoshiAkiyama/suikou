"""参照コーパスに照らして、文体に合わない語を見つける試作。

判定は次の条件による。対の一覧を持たずに検出できる。

- 語種が和である内容語に限る。LLM が漢語を和語に置き換える向きに偏るためであり、
  分野の用語（漢語やカタカナ語が多い）を誤って拾わないためでもある。
- 参照コーパスのどの分野でも百万語あたり MIN_REF 未満しか出ない。
- 調べる文書では MIN_DOC 回以上出る。一度きりの語は言い回しの綾と区別できない。
"""
import collections, json, pathlib, sys
import fugashi
from chunks import aligned, kind
from analyze import units, INLINE
import re

MIN_REF = 5.0     # 参照コーパスでの百万語あたりの下限
MIN_DOC = 3       # 調べる文書での出現の下限
TAGGER = fugashi.Tagger()
CONTENT = {"名詞", "動詞", "形容詞", "形状詞", "副詞"}


def wago_units(text):
    """和語の内容語だけを返す。語種は UniDic の goshu を見る。"""
    text = INLINE.sub(lambda m: m.group(1) or " ", text)
    out = []
    for w in TAGGER(text):
        f = w.feature
        if (f.pos1 in CONTENT) and getattr(f, "goshu", "") == "和":
            base = f.orthBase or w.surface
            if re.search(r"[ぁ-んァ-ヶ一-龯]", base):
                out.append(base)
    return out


def corpus_counts(paths):
    c = collections.Counter(); n = 0
    for p in paths:
        for ch in aligned(p.read_text(errors="ignore")):
            if kind(ch) == "code":
                continue
            u = units(ch)[0]; n += len(u)
            c.update(wago_units(ch))
    return c, n


def main():
    ref_dir, targets = pathlib.Path(sys.argv[1]), sys.argv[2:]
    groups = collections.defaultdict(list)
    for p in sorted(ref_dir.glob("*.md")):
        groups[p.name.split("__")[0]].append(p)
    per = {g: corpus_counts(ps) for g, ps in groups.items()}

    def rare(word):
        return all(c[word] / n * 1e6 < MIN_REF for c, n in per.values())

    for t in targets:
        doc = pathlib.Path(t)
        c, _ = corpus_counts([doc])
        hits = [(w, k) for w, k in c.most_common() if k >= MIN_DOC and rare(w)]
        print(f"\n## {doc.name}（{len(hits)} 語）")
        for w, k in hits[:25]:
            ref = "、".join(f"{g} {cc[w]/nn*1e6:.1f}" for g, (cc, nn) in per.items())
            print(f"  {w}（{k} 回） 参照: {ref}")


if __name__ == "__main__":
    main()
