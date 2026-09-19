#!/usr/bin/env python3
"""日本語テクニカルライティングのAI臭さ指標を測る。
逆瀬川(2026)の指標を再現しつつ、本プロジェクトの候補指標を追加する。
"""
import re, sys, json, statistics as st
from collections import Counter
import fugashi

TAGGER = fugashi.Tagger()

# ---------- 前処理 ----------
def strip_markdown(text):
    """コード・リンク・フロントマターを除去し、(地の文, 箇条書き, 見出し, 生テキスト) を返す"""
    text = re.sub(r'^---\n.*?\n---\n', '', text, flags=re.S)      # front matter
    text = re.sub(r'```.*?```', '', text, flags=re.S)             # fenced code
    text = re.sub(r'~~~.*?~~~', '', text, flags=re.S)
    text = re.sub(r'<[^>]+>', '', text)                            # html
    raw = text
    body, lists, heads = [], [], []
    for line in text.split('\n'):
        s = line.strip()
        if not s:
            continue
        if re.match(r'^#{1,6}\s', s):
            heads.append(re.sub(r'^#+\s*', '', s)); continue
        if re.match(r'^(\s*)([-*+]|\d+[.)])\s', line):
            lists.append(s); continue
        if re.match(r'^\s{4,}\S', line) or s.startswith('|') or s.startswith('>'):
            continue
        body.append(s)
    def clean(s):
        s = re.sub(r'!?\[([^\]]*)\]\([^)]*\)', r'\1', s)   # links/images
        s = re.sub(r'`[^`]*`', '', s)                       # inline code
        s = re.sub(r'\*\*([^*]*)\*\*', r'\1', s)
        s = re.sub(r'\*([^*]*)\*', r'\1', s)
        return s
    return '\n'.join(clean(x) for x in body), lists, heads, raw

def sentences(prose):
    out = []
    for para in prose.split('\n'):
        for s in re.split(r'(?<=[。！？])', para):
            s = s.strip()
            if len(s) >= 5 and re.search(r'[ぁ-んァ-ヶ一-龠]', s):
                out.append(s)
    return out

# ---------- 指標 ----------
KANJI = re.compile(r'[\u4e00-\u9fff]')
JA = re.compile(r'[ぁ-んァ-ヶー\u4e00-\u9fff]')
DEMONSTRATIVE = {'これ','それ','この','その','これら','それら','こちら','そちら','ここ','そこ','こう','そう'}
KEISHIKI_MEISHI = {'こと','もの','ため','よう','点','場合'}

def mattr(tokens, window=100):
    if len(tokens) < window:
        return len(set(tokens)) / max(len(tokens), 1)
    vals = []
    for i in range(len(tokens) - window + 1):
        w = tokens[i:i+window]
        vals.append(len(set(w)) / window)
    return sum(vals) / len(vals)

def clean_inline(s):
    s = re.sub(r'!?\[([^\]]*)\]\([^)]*\)', r'\1', s)
    s = re.sub(r'`[^`]*`', '', s)
    s = re.sub(r'\*\*([^*]*)\*\*', r'\1', s)
    s = re.sub(r'\*([^*]*)\*', r'\1', s)
    s = re.sub(r'^\s*([-*+]|\d+[.)])\s*', '', s)
    return s


def analyze(name, text):
    prose, lists, heads, raw = strip_markdown(text)
    listtext = '\n'.join(clean_inline(x) for x in lists)
    alltext = prose + '\n' + listtext          # 地の文＋箇条書き本文
    sents = sentences(prose)
    if len(sents) < 8:
        return None
    lens = [len(re.sub(r'\s', '', s)) for s in sents]
    prose_chars = sum(lens)

    awords = list(TAGGER(alltext))
    asurf = [w.surface for w in awords]
    acontent = [w for w in awords if w.feature.pos1 in ('名詞','動詞','形容詞','副詞','形状詞')]
    agoshu = Counter(getattr(w.feature, 'goshu', None) for w in acontent)
    an_goshu = sum(v for k, v in agoshu.items() if k in ('和','漢','外','混'))
    all_chars = len(re.sub(r'\s', '', alltext))
    akanji = sum(1 for c in alltext if KANJI.match(c))
    aja = sum(1 for c in alltext if JA.match(c))
    ademo = sum(1 for w in awords if w.surface in DEMONSTRATIVE)

    words = list(TAGGER(prose))
    surf = [w.surface for w in words]
    content = [w for w in words if w.feature.pos1 in ('名詞','動詞','形容詞','副詞','形状詞')]
    goshu = Counter(getattr(w.feature, 'goshu', None) for w in content)
    n_goshu = sum(v for k, v in goshu.items() if k in ('和','漢','外','混'))

    ten = sum(s.count('、') for s in sents)
    kanji = sum(1 for c in prose if KANJI.match(c))
    ja_chars = sum(1 for c in prose if JA.match(c))

    # 文末
    enders, taigen, desumasu = [], 0, 0
    for s in sents:
        core = s.rstrip('。！？')
        ws = list(TAGGER(core))
        if not ws:
            continue
        last = ws[-1]
        tail = ''.join(w.surface for w in ws[-2:])
        enders.append(tail)
        if last.feature.pos1 in ('名詞','形状詞') and getattr(last.feature,'pos2','') != '助動詞語幹':
            taigen += 1
        if re.search(r'(です|ます|ました|ません|でしょう|ください)$', core):
            desumasu += 1

    # 文末3連続重複
    run3 = sum(1 for i in range(len(enders)-2)
               if enders[i] == enders[i+1] == enders[i+2])

    # 連用中止 / テ形接続
    renyo = te = 0
    for i, w in enumerate(words[:-1]):
        nxt = words[i+1]
        if nxt.surface == '、':
            cf = getattr(w.feature, 'cForm', '') or ''
            if w.feature.pos1 == '動詞' and cf.startswith('連用形'):
                renyo += 1
            if w.surface in ('て','で') and w.feature.pos2 == '接続助詞':
                te += 1

    # サ変名詞 + を + する/行う/実施
    sahen_wo = 0
    for i in range(len(words)-2):
        if (words[i].feature.pos1 == '名詞' and words[i+1].surface == 'を'
                and words[i+2].feature.lemma in ('為る','行う','実施')):
            sahen_wo += 1

    demo = sum(1 for w in words if w.surface in DEMONSTRATIVE)
    keishiki = sum(1 for w in words
                   if w.feature.pos1 == '名詞' and w.surface in KEISHIKI_MEISHI)

    # 書式
    body_lines = [l for l in raw.split('\n') if l.strip()]
    n_list = len([l for l in body_lines if re.match(r'^\s*([-*+]|\d+[.)])\s', l)])
    bold = len(re.findall(r'\*\*[^*]+\*\*', raw))
    dash = len(re.findall(r'[—―–]', raw))
    matome = sum(1 for h in heads if 'まとめ' in h)
    # 項目数の明示（メンテナンス性）
    count_decl = len(re.findall(r'[0-9０-９一二三四五六七八九十]+[つ個点]の(?:理由|方法|ポイント|点|要素|ステップ|手順|方針|特徴)'
                                r'|(?:以下|次)の[0-9０-９一二三四五六七八九十]+[つ個点]', prose))
    # コロン終わりの箇条書き導入（不完全導入文）
    colon_lead = len(re.findall(r'[^\n]{4,}[:：]\s*\n\s*[-*+]', raw))

    per1000 = lambda n: n / prose_chars * 1000 if prose_chars else 0
    return {
        'name': name,
        '地の文字数': prose_chars,
        '全本文字数': all_chars,
        '地の文比率': round(prose_chars/all_chars, 3) if all_chars else 0,
        'MATTR100_全': round(mattr(asurf), 3),
        '漢字率_全': round(akanji/aja, 3) if aja else 0,
        '漢語率_全': round(agoshu.get('漢',0)/an_goshu, 3) if an_goshu else 0,
        '外来語率_全': round(agoshu.get('外',0)/an_goshu, 3) if an_goshu else 0,
        '指示詞_全千字': round(ademo/all_chars*1000, 2) if all_chars else 0,
        '文数': len(sents),
        '平均文長': round(st.mean(lens), 1),
        '文長変動係数': round(st.pstdev(lens)/st.mean(lens), 3),
        '読点_文あたり': round(ten/len(sents), 3),
        '読点間文字数': round(prose_chars/(ten+len(sents)), 1),
        'MATTR100': round(mattr(surf), 3),
        '漢字率': round(kanji/ja_chars, 3) if ja_chars else 0,
        '漢語率': round(goshu.get('漢',0)/n_goshu, 3) if n_goshu else 0,
        '外来語率': round(goshu.get('外',0)/n_goshu, 3) if n_goshu else 0,
        'ですます率': round(desumasu/len(sents), 3),
        '体言止め率': round(taigen/len(sents), 3),
        '文末3連続重複': run3,
        '連用中止_千字': round(per1000(renyo), 2),
        'テ形接続_千字': round(per1000(te), 2),
        'サ変ヲスル行ウ_千字': round(per1000(sahen_wo), 2),
        '指示詞_千字': round(per1000(demo), 2),
        '形式名詞_千字': round(per1000(keishiki), 2),
        '箇条書き行数': n_list,
        '箇条書き割合': round(n_list/len(body_lines), 3) if body_lines else 0,
        '太字_千字': round(per1000(bold), 2),
        'ダッシュ_千字': round(per1000(dash), 3),
        '見出しまとめ': matome,
        '項目数明示': count_decl,
        'コロン導入': colon_lead,
    }

if __name__ == '__main__':
    import glob, os
    rows = []
    for pat in sys.argv[1:]:
        for p in sorted(glob.glob(pat, recursive=True)):
            try:
                t = open(p, encoding='utf-8', errors='ignore').read()
            except Exception:
                continue
            r = analyze(os.path.basename(os.path.dirname(p)) + '/' + os.path.basename(p), t)
            if r:
                rows.append(r)
    print(json.dumps(rows, ensure_ascii=False, indent=1))
