#!/usr/bin/env python3
"""英語テクニカルライティングのAI臭さ／メンテナンス性指標を測る。
Reinhart(2025)のBiber系素性、Kobak(2025)の超過語、Google style guideの
メンテナンス性規定を検出可能な形に落とし込んだもの。
"""
import re, sys, json, glob, os, statistics as st
import spacy

NLP = spacy.load('en_core_web_sm', disable=['ner', 'lemmatizer'])
NLP.max_length = 400000

# Kobak et al. 2025 の rare set の一部 + common set 10語
KOBAK = set("""across additionally comprehensive crucial enhancing exhibited insights notably
particularly within delve delves delving intricate intricacies meticulous meticulously pivotal
underscore underscores underscored underscoring showcase showcases showcased showcasing realm
realms noteworthy notable nuanced encompass encompasses encompassing leverage leverages
leveraging harness harnesses harnessing streamline streamlined streamlines bolster bolstered
bolstering garnered elucidate elucidates elucidating unveil unveils unveiled unveiling
groundbreaking transformative seamless seamlessly unparalleled multifaceted foundational
interplay comprehending facilitating necessitate necessitates align aligning aligns alongside
amidst akin conversely consequently emphasize emphasizes emphasizing ensuring evolving
exceptional holistic imperative invaluable paving pinpoint poised predominantly pronounced
refine refines remarkable robust scrutinize surpass surpasses surpassed surpassing swift
swiftly thorough ultimately uncharted underexplored unlocking versatility warranting yielding
tapestry camaraderie unspoken palpable solace fleeting unravel cacophony""".split())

# Google style guide "Timeless documentation" の回避語
# 厳格版: 文脈によらずメンテナンス性を損なう語のみ
TIMELESS = set("""currently presently eventually soon latest newest newer""".split())
TIMELESS_PHRASE = re.compile(r"\b(as of this writing|at present|at the time of writing|"
                             r"does not yet|doesn't yet|in the (?:near )?future|for now|"
                             r"is now available|recently (?:added|introduced|released))\b", re.I)

NOMIN = re.compile(r'.+(tion|tions|ment|ments|ance|ances|ence|ences|ity|ities|ness|nesses|ism|isms)$', re.I)
COUNT_DECL = re.compile(r'\b(two|three|four|five|six|2|3|4|5|6)\s+(reasons|ways|steps|things|key\s+\w+|main\s+\w+|types|categories|benefits|points|factors|approaches|options|principles|rules|components|parts)\b', re.I)
SIMILE = re.compile(r'\b(like a|like the|as if|as though|akin to|similar to|think of it as|imagine|analogous to|for example|for instance)\b', re.I)


def strip_markup(text, kind):
    if kind == 'texi':
        text = re.sub(r'@(example|verbatim|smallexample|group|multitable|menu).*?@end \1', ' ', text, flags=re.S)
        text = re.sub(r'@[a-zA-Z]+\{([^}]*)\}', r'\1', text)
        text = re.sub(r'^@\w+.*$', '', text, flags=re.M)
    elif kind == 'rst':
        text = re.sub(r'^::\s*$.*?(?=^\S)', ' ', text, flags=re.S | re.M)
        text = re.sub(r'^\s{4,}\S.*$', '', text, flags=re.M)
        text = re.sub(r'``([^`]*)``', r'\1', text)
        text = re.sub(r'^[=~^\-*#"\']{3,}$', '', text, flags=re.M)
    else:  # md
        text = re.sub(r'^---\n.*?\n---\n', '', text, flags=re.S)
        text = re.sub(r'```.*?```', ' ', text, flags=re.S)
        text = re.sub(r'\\([.\-+*_#`\[\]()<>|])', r'\1', text)  # AWS等のエスケープ解除
        text = re.sub(r'^-{3,}$', '', text, flags=re.M)
    raw = text
    body, heads = [], []
    for line in text.split('\n'):
        s = line.strip()
        if not s:
            continue
        if re.match(r'^#{1,6}\s', s):
            heads.append(re.sub(r'^#+\s*', '', s)); continue
        if re.match(r'^(\s*)([-*+]|\d+[.)])\s', line) or s.startswith('|') or s.startswith('>'):
            continue
        body.append(s)
    prose = '\n'.join(body)
    prose = re.sub(r'!?\[([^\]]*)\]\([^)]*\)', r'\1', prose)
    prose = re.sub(r'`[^`]*`', ' ', prose)
    prose = re.sub(r'\*\*([^*]*)\*\*', r'\1', prose)
    prose = re.sub(r'<[^>]+>', ' ', prose)
    prose = re.sub(r'https?://\S+', ' ', prose)
    return prose, heads, raw


def mattr(toks, w=100):
    if len(toks) < w:
        return len(set(toks)) / max(len(toks), 1)
    return sum(len(set(toks[i:i+w]))/w for i in range(len(toks)-w+1)) / (len(toks)-w+1)


def analyze(name, text, kind):
    prose, heads, raw = strip_markup(text, kind)
    if len(prose) < 1500:
        return None
    doc = NLP(prose[:NLP.max_length])
    words = [t for t in doc if not t.is_punct and not t.is_space]
    nw = len(words)
    if nw < 400:
        return None
    sents = [s for s in doc.sents if len([t for t in s if not t.is_punct]) >= 4]
    if len(sents) < 20:
        return None
    slen = [len([t for t in s if not t.is_punct]) for s in sents]

    partic = sum(1 for t in doc if t.tag_ == 'VBG' and t.dep_ in ('advcl', 'acl'))
    nomin = sum(1 for t in words if t.pos_ == 'NOUN' and NOMIN.match(t.text) and len(t.text) > 6)
    csubj = sum(1 for t in doc if t.dep_ == 'csubj')
    phrasal_cc = sum(1 for t in doc if t.dep_ == 'cc' and t.head.pos_ in ('NOUN', 'PROPN', 'ADJ', 'ADV'))
    clausal_cc = sum(1 for t in doc if t.dep_ == 'cc' and t.head.pos_ in ('VERB', 'AUX'))
    passives = [t for t in doc if t.dep_ == 'auxpass']
    agentless = 0
    for t in passives:
        if not any(c.dep_ == 'agent' for c in t.head.children):
            agentless += 1
    depdist = st.mean([abs(t.i - t.head.i) for t in words if t.head is not t]) if nw else 0

    # modifier stack: 3語以上の ADJ/NOUN が名詞の前に連続
    mstack = 0
    run = 0
    has_propn = False
    for t in doc:
        if t.pos_ in ('ADJ', 'NOUN', 'PROPN'):
            run += 1
            has_propn = has_propn or t.pos_ == 'PROPN'
        else:
            if run >= 4 and not has_propn:
                mstack += 1
            run = 0
            has_propn = False

    low = [t.text.lower() for t in words]
    kobak = sum(1 for w in low if w in KOBAK)
    timeless = sum(1 for w in low if w in TIMELESS) + len(TIMELESS_PHRASE.findall(prose))

    emdash = len(re.findall(r'—|(?<=\w)--(?=\w)| -- ', prose))
    bold = len(re.findall(r'\*\*[^*]+\*\*', raw))
    lines = [l for l in raw.split('\n') if l.strip()]
    nlist = len([l for l in lines if re.match(r'^\s*([-*+]|\d+[.)])\s', l)])
    # 不完全な箇条書き導入（コロンで終わり、the following/these/as follows を含まない）
    bad_lead = 0
    good_lead = 0
    for m in re.finditer(r'^([^\n]{6,}):\s*\n+\s*(?:[-*+]|\d+[.)])\s', raw, flags=re.M):
        lead = m.group(1)
        if re.search(r'\b(the following|as follows|these|below|steps)\b', lead, re.I):
            good_lead += 1
        else:
            bad_lead += 1
    counts = len(COUNT_DECL.findall(prose))
    simile = len(SIMILE.findall(prose))
    p1k = lambda n: n / nw * 1000

    return {
        'name': name, 'words': nw, 'sents': len(sents),
        'mean_sent_len': round(st.mean(slen), 1),
        'sent_len_cv': round(st.pstdev(slen)/st.mean(slen), 3),
        'MATTR100': round(mattr(low), 3),
        'participial_1k': round(p1k(partic), 2),
        'nominalization_1k': round(p1k(nomin), 2),
        'csubj_1k': round(p1k(csubj), 3),
        'phrasal_cc_1k': round(p1k(phrasal_cc), 2),
        'clausal_cc_1k': round(p1k(clausal_cc), 2),
        'phrasal_cc_ratio': round(phrasal_cc/max(phrasal_cc+clausal_cc, 1), 3),
        'agentless_passive_1k': round(p1k(agentless), 2),
        'mean_dep_dist': round(depdist, 2),
        'modifier_stack_1k': round(p1k(mstack), 2),
        'kobak_1k': round(p1k(kobak), 2),
        'timeless_violation_1k': round(p1k(timeless), 2),
        'emdash_1k': round(p1k(emdash), 2),
        'bold_1k': round(p1k(bold), 2),
        'list_ratio': round(nlist/max(len(lines), 1), 3),
        'bad_list_lead': bad_lead, 'good_list_lead': good_lead,
        'count_declaration': counts,
        'simile_marker_1k': round(p1k(simile), 2),
    }


if __name__ == '__main__':
    rows = []
    for pat in sys.argv[1:]:
        for p in sorted(glob.glob(pat, recursive=True)):
            kind = 'texi' if p.endswith('.texi') else 'rst' if p.endswith(('.rst','.txt')) else 'md'
            try:
                t = open(p, encoding='utf-8', errors='ignore').read()
                r = analyze(os.path.basename(p), t, kind)
            except Exception as e:
                continue
            if r:
                rows.append(r)
    print(json.dumps(rows, ensure_ascii=False))
