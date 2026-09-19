#!/usr/bin/env python3
"""M1〜M7（メンテナンス性層）の検出器。日英対応。
S2実験（指示の遵守率）の測定に使う。
"""
import re, sys, json, glob, os
import fugashi

TAGGER = fugashi.Tagger()

LIST_RE = re.compile(r'^(\s*)([-*+]|\d+[.)])\s+(.*)$')
HEAD_RE = re.compile(r'^#{1,6}\s')

M2_JA = re.compile(r'[0-9０-９一二三四五六七八九十]+\s*[つ個点種類]の\s*'
                   r'(?:理由|方法|ポイント|点|要素|ステップ|手順|方針|特徴|観点|側面|利点|欠点|課題|原則|要因|種類)'
                   r'|(?:以下|次)の\s*[0-9０-９一二三四五六七八九十]+\s*[つ個点]'
                   r'|[0-9０-９一二三四五六七八九十]+\s*[つ個]あ(?:る|ります)')
M2_EN = re.compile(r'\b(two|three|four|five|six|seven|2|3|4|5|6|7)\s+'
                   r'(reasons|ways|steps|things|types|categories|benefits|points|factors|'
                   r'approaches|options|principles|rules|components|parts|key\s+\w+|main\s+\w+)\b', re.I)
M3_RE = re.compile(r'^#{1,6}\s*(?:第?\s*[0-9０-９]+\s*[.．、)）章節]|[0-9０-９]+\s*[-–]\s)')
M4_RE = re.compile(r'^\s*[-*+]\s+[0-9０-９]+\s*[.)．）、]\s')
M6_JA = re.compile(r'現在|現時点|最新の|新しい|今後|将来的に|まもなく|既存の|目下|現行の')
M6_EN = re.compile(r'\b(currently|presently|eventually|soon|latest|newest|newer|'
                   r'as of this writing|at present|in the (?:near )?future|for now)\b', re.I)
M7_JA = re.compile(r'(?:など|等)[。、）\)]?\s*$')
M7_EN = re.compile(r'\b(etc\.|and so on|and more)\s*[.)]?\s*$', re.I)

EN_LEADIN_OK = re.compile(r'\b(the following|as follows|these|below|steps|following)\b', re.I)


def strip(text):
    text = re.sub(r'^---\n.*?\n---\n', '', text, flags=re.S)
    text = re.sub(r'```.*?```', '', text, flags=re.S)
    text = re.sub(r'\\([.\-+*_#`\[\]()<>|])', r'\1', text)
    return text


def ja_lead_kind(line):
    """箇条書き導入文の分類: ok_sentence / ok_taigen_colon / ng_predicate_colon / ng_bound / other"""
    s = line.strip()
    s = re.sub(r'\*\*|\*|`', '', s)
    if re.search(r'[。！？]$', s):
        return 'ok_sentence'
    if re.search(r'[:：]$', s):
        core = s[:-1].strip()
        ws = list(TAGGER(core))
        if not ws:
            return 'other'
        p = ws[-1].feature.pos1
        return 'ok_taigen_colon' if p in ('名詞', '接尾辞', '代名詞') else 'ng_predicate_colon'
    ws = list(TAGGER(s))
    if not ws:
        return 'other'
    last = ws[-1]
    cf = getattr(last.feature, 'cForm', '') or ''
    if last.feature.pos1 == '助詞':
        return 'ng_bound'
    if last.feature.pos1 in ('動詞', '助動詞') and cf.startswith('連用形'):
        return 'ng_bound'
    return 'other'


def en_lead_kind(line):
    s = line.strip()
    s = re.sub(r'\*\*|\*|`', '', s)
    if s.endswith(':'):
        return 'ok_sentence' if EN_LEADIN_OK.search(s) else 'ng_colon_bare'
    if re.search(r'[.!?]$', s):
        return 'ok_sentence'
    return 'other'


def item_ending_class(text, lang):
    t = text.strip()
    t = re.sub(r'\*\*|\*|`', '', t).strip()
    if lang == 'en':
        return 'period' if re.search(r'[.!?]$', t) else 'none'
    if re.search(r'[。！？]$', t):
        return 'kuten'
    ws = list(TAGGER(t))
    if not ws:
        return 'other'
    p = ws[-1].feature.pos1
    if p in ('名詞', '接尾辞', '代名詞', '形状詞'):
        return 'taigen'
    return 'yougen'


def analyze(name, text, lang):
    text = strip(text)
    lines = text.split('\n')
    n_body = len([l for l in lines if l.strip()])
    res = dict(name=name, lang=lang, lines=n_body,
               m1_ok=0, m1_ng=0, m2=0, m3=0, m4=0, m5_blocks=0, m5_mixed=0, m6=0, m7=0)

    # リストブロックの抽出
    i = 0
    while i < len(lines):
        if LIST_RE.match(lines[i]):
            start = i
            items = []
            while i < len(lines) and (LIST_RE.match(lines[i]) or not lines[i].strip()):
                m = LIST_RE.match(lines[i])
                if m:
                    items.append(m.group(3))
                    if M4_RE.match(lines[i]):
                        res['m4'] += 1
                i += 1
            # M1: 直前の非空行
            j = start - 1
            while j >= 0 and not lines[j].strip():
                j -= 1
            if j >= 0 and not HEAD_RE.match(lines[j].strip()) and not LIST_RE.match(lines[j]):
                kind = (ja_lead_kind if lang == 'ja' else en_lead_kind)(lines[j])
                if kind.startswith('ok'):
                    res['m1_ok'] += 1
                elif kind.startswith('ng'):
                    res['m1_ng'] += 1
            # M5: 項目末尾の形の混在
            if len(items) >= 2:
                res['m5_blocks'] += 1
                if len({item_ending_class(x, lang) for x in items}) > 1:
                    res['m5_mixed'] += 1
            # M7: 項目が「など」で終わる
            for x in items:
                if (M7_JA if lang == 'ja' else M7_EN).search(x.strip()):
                    res['m7'] += 1
        else:
            s = lines[i].strip()
            if HEAD_RE.match(s) and M3_RE.match(s):
                res['m3'] += 1
            if not HEAD_RE.match(s):
                res['m2'] += len((M2_JA if lang == 'ja' else M2_EN).findall(s))
                res['m7'] += len((M7_JA if lang == 'ja' else M7_EN).findall(s))
            res['m6'] += len((M6_JA if lang == 'ja' else M6_EN).findall(s))
            i += 1
    return res


if __name__ == '__main__':
    out = []
    for pat in sys.argv[1:]:
        for p in sorted(glob.glob(pat, recursive=True)):
            lang = 'ja' if ('日本語' in p or '/ja' in p) else 'en'
            t = open(p, encoding='utf-8', errors='ignore').read()
            out.append(analyze(p, t, lang))
    print(json.dumps(out, ensure_ascii=False))
