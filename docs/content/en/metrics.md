---
title: "Metric definitions"
weight: 20
---

This page holds the definitions that the Rust implementation has to satisfy.
The Python under `research/` is the reference implementation, and `tests/golden/` pins
its output.

lindera and fugashi are different tokenizers, so the two implementations are not asked to
agree exactly. The comparison allows a margin. When a metric falls outside that margin,
the question of which implementation is right gets settled case by case.

## Preprocessing

The order changes the result. The steps run in the order below.

1. Strip the YAML front matter.
2. Strip fenced code blocks.
3. **Undo backslash escapes**, turning `\.` back into `.`.
4. Strip HTML tags.
5. Sort the lines into blocks.
6. Strip inline markup, meaning links, code spans, and emphasis.

Skipping the third step breaks sentence splitting.
The AWS doc_source corpus is written in that escaped form, and leaving the escapes in
place inflates the mean sentence length by a factor of three or more.
`tests/golden/input/en_escaped.md` pins this behavior.
The sentence count is 6 before the escapes come out and 11 afterward.

Every step keeps the line count intact. A step that removed lines outright would shift
the line numbers of everything below it, and a finding that points at the wrong line is a
finding nobody can act on.

### How blocks are sorted

| Kind | Test |
|---|---|
| Heading | The line opens with one to six `#` followed by a space |
| List item | Leading space, then `-` `*` `+`, or a number with `.` or `)`, then a space |
| Table | The line opens with `\|` |
| Quote | The line opens with `>` |
| Prose | Any other line that is not empty |

### What each metric covers

| Kind of metric | Covers |
|---|---|
| Vocabulary, meaning MATTR, kanji ratio, kango ratio, demonstratives, formal nouns | Prose and the body of list items |
| Sentence structure, meaning length, commas, endings, renyo, te-form | Prose only |
| Format, meaning prose ratio, list ratio, bold | Every line |

A vocabulary metric must never be measured over prose alone.
Some models put almost everything into bullet lists, and for those models prose alone
throws away more than eighty percent of the text. For example, one model in the corpus
left only fifteen percent of its output in prose.

## Sentence splitting

Japanese splits right after `。` `！` `？`.
A fragment shorter than five characters is dropped, and so is a fragment with no Japanese
character in it.

English follows the spaCy splitter.
A sentence with fewer than four tokens, once punctuation is removed, is dropped.

## Japanese metrics

The tokenizer is UniDic. IPADIC returns neither the word origin nor the inflected form,
so IPADIC cannot serve here.

### ja.mattr100

Take a moving window one hundred tokens wide.
Divide the number of distinct tokens in each window by the width of the window, then
average across every window.
When the token count falls below the width of the window, divide the number of distinct
tokens in the whole text by the token count.
A token is a surface form.

Humans land between 0.559 and 0.581, and AI lands between 0.629 and 0.665.
The threshold is 0.60.

### ja.renyo_per_1k

Count each place where a verb carries an inflected form beginning with 連用形 and the
token right after it is a comma.
Divide by the character count of the prose, spaces excluded, and multiply by one thousand.

### ja.te_per_1k

Count each place where the surface form is `て` or `で`, the part of speech is a
conjunctive particle, and the token right after it is a comma.
Normalize per thousand characters the same way.

### ja.renyo_te_ratio

Divide `renyo_per_1k` by `te_per_1k`.
When the divisor falls below 0.05, the ratio is not reported at all.

Native humans land at 2.00, translated Japanese lands at 1.79, and AI lands between 3.62
and 15.94. The threshold is 3.0.
This metric separates the two groups more sharply than any other.
The te-form leans toward speech and folds clauses into one.
The renyo form leans toward writing and lines clauses up side by side.
AI leans toward the second. The same thing shows up in English as an excess of participial
clauses.

### ja.keishiki_meishi_per_1k

Count each token whose part of speech is a noun and whose surface form is one of
`こと` `もの` `ため` `よう` `点` `場合`. Normalize per thousand characters.

Humans land at 11.1, and AI lands between 4.99 and 6.95.
A value below the threshold of 8.5 is reported.
**This finding is about something missing**, so it carries no position.

The metric is sensitive to the kind of document requested. In procedures, AI climbs to
8.11 and comes close to the threshold. Until human baselines exist for each kind of
document, the severity stays at info.

### ja.demonstrative_per_1k

Count each token whose surface form is one of
`これ` `それ` `この` `その` `これら` `それら` `こちら` `そちら` `ここ` `そこ`.
Normalize per thousand characters.

Humans land between 5.69 and 6.47, and AI lands between 1.38 and 3.39.
A value below the threshold of 4.5 is reported.
This finding is also about something missing.
It relates to zero anaphora in Japanese.

### ja.kango_ratio

Among content words, meaning nouns, verbs, adjectives, adverbs, and adjectival nouns,
take the share whose word origin is `漢`.
The divisor is the count of those whose origin is `和` `漢` `外` or `混`.

Humans land between 0.326 and 0.397, and AI lands between 0.455 and 0.519.
The threshold is 0.42.
The metric is sensitive to the kind of document and to the subject, so the severity stays
at info.

### ja.kanji_ratio

Divide the number of kanji, meaning U+4E00 through U+9FFF, by the number of Japanese
characters, meaning hiragana, katakana, and kanji.

### ja.prose_ratio and ja.list_ratio

The prose ratio divides the character count of the prose by the character count of the
prose plus the body of the list items.
The list ratio divides the number of list item lines by the number of lines that are not
empty.

Both counts exclude spaces.

## English metrics

Only three metrics survived validation.
Participial clauses and nominalization overlapped the human range.
The effect size of 5.3 reported in earlier work did not carry over to the task of writing
a document.

### en.mattr100

The definition matches the Japanese one.
A token is a lowercased surface form, and symbols and spaces are excluded.

Humans land between 0.58 and 0.69, and AI lands between 0.72 and 0.82.
The threshold is 0.71.
Every AI condition sits above every human corpus, which makes this the steadiest metric
of the set.

### en.prose_ratio

The definition matches the Japanese one. Humans land between 0.66 and 0.97, and AI lands
between 0.31 and 0.56. The threshold is 0.65.

### en.simile_marker_per_1k

Count the phrases below and normalize per thousand words. Case is ignored.

`like a` / `like the` / `as if` / `as though` / `akin to` / `similar to` /
`think of it as` / `imagine` / `analogous to` / `for example` / `for instance`

Humans land between 0.84 and 2.54, and AI lands between 0.00 and 0.60.
A value below the threshold of 0.8 is reported.

AI reaches for figurative vocabulary and then leaves the figure unmarked.
This metric matches the policy of avoiding a bare metaphor and reaching instead for a
simile or for an example. A sentence that says a cache behaves like a queue marks the
comparison, and a sentence that calls the cache a queue does not.

## The M rules

### M1, the lead-in to a list

The lead-in is the nearest line above the list that is not empty.
When that line is a heading or another list item, no judgment is made.

The Japanese test runs as follows.

| Result | Condition |
|---|---|
| ok | The line ends in `。` `！` `？` |
| ok | The line ends in `:` or `：` and the token before it is a noun, a suffix, or a pronoun |
| ng | The line ends in `:` or `：` and the token before it is a predicate |
| ng | The last token is a particle |
| ng | The last token is a verb or an auxiliary in the 連用形 |
| other | None of the above |

For English, a line that ends in `:` counts as ok when it contains
`the following` `as follows` `these` `below` `steps` or `following`, and counts as ng
otherwise. A line ending in `.` `!` or `?` counts as ok.

A strict test would look for a finite verb.
Even so, this test alone separates the AWS S3 user guide, at 30 incomplete against 145
complete, from the Linux kernel, at 105 against 16.

The violation rate is `ng / (ok + ng)`.

### M2, stating the number of items in prose

Within prose, headings excluded, count the shapes below.

For Japanese, a numeral followed by `つ` `個` `点` `種` or `類`, followed in turn by one of
`理由` `方法` `ポイント` `点` `要素` `ステップ` `手順` `方針` `特徴` `観点` `側面`
`利点` `欠点` `課題` `原則` `要因` `種類`.
Shapes such as `以下の3つ` and `次の2点` count as well, and so do `3つある` and `3つあります`.

For English, `two` through `seven`, or `2` through `7`, followed by one of
`reasons` `ways` `steps` `things` `types` `categories` `benefits` `points`
`factors` `approaches` `options` `principles` `rules` `components` `parts`.

Writing the number of items into the prose breaks the prose the moment an item is added
or removed.

How far this rule should reach is not settled.
It also catches plain explanatory prose that states a number with no list behind it.
`tests/golden/input/ja_prose.md` is an example of that case.

### M3, numbered headings

Count each heading line where the `#` is followed by a number and then `.` `、` `)` `章`
or `節`.

### M4, hand-written numbers

Count each list item whose marker is `-` `*` or `+` and whose body opens with a number
followed by `.` or `)`.

### M5, parallel items

Within one list, sort the items by the shape of their ending.

Japanese sorts into three groups, meaning items ending in a full stop, items ending in a
noun, and items ending in a predicate.
English sorts into two, meaning items ending in a period and items that do not.

A list holding two or more groups counts as mixed.
The mixed rate divides the number of mixed lists by the number of lists holding two or
more items.

### M6, words that depend on a moment in time

Count each occurrence of the words below. Two on one line count as two.

For Japanese, `現在` `現時点` `最新の` `新しい` `今後` `将来的に` `まもなく` `既存の` `目下`
`現行の`.

For English, `currently` `presently` `eventually` `soon` `latest` `newest` `newer`
`as of this writing` `at present` `in the future` `in the near future` `for now`.

The English vocabulary is held to the strict set.
Adding `new` `now` `future` and `existing` raises the number of false catches.
With those words included, Google eng-practices scored the highest violation rate of any
corpus at 6.13. With the strict set, the same corpus scored 0.58, the lowest of any
corpus.

This is the one rule that a short instruction fails to enforce.
Putting it into the layer that runs before generation means listing the forbidden words
one by one.

### M7, the tail of an enumeration

For Japanese, a line ending in `など` or `等`.
For English, a line ending in `etc.` `and so on` or `and more`.

Both prose and list items are covered.

## The register rule

### register/rare-wago, a native word out of register

The rule reports a native content word used far more often here than in professional
Japanese translation.
It covers content words whose origin is native, and it skips numerals and single
katakana characters.
Field terminology leans on Sino-Japanese words and loanwords, so covering native words
alone keeps that terminology out of the findings.

A word has to appear three times or more.
A word used once cannot be told apart from a turn of phrase.

Two statistics decide a finding.
The log-likelihood ratio from Dunning (1993) has to reach 15.13, which is p < 0.0001 at
one degree of freedom, and the Log Ratio from Hardie (2014) has to reach 7.0, which is
128 times the reference rate. Significance alone is not enough, because using an ordinary
word slightly more often reaches significance on a corpus this size.

The reference lives in `crates/suikou-core/data/register_ja.toml`.
It is a frequency table built from 652 pages of Kubernetes, MDN, and Vue in Japanese,
holding roughly 290,000 content words.
A tutorial stays out, because its register differs, and a word has to appear in two
fields or more to be listed.
`research/lexshift/build_reference.py` rebuilds it.

Words listed in `.suikou/terms.toml` and `.suikou/register-allow.toml` are skipped.
The reference is built from web documentation, which is thin on the vocabulary of
measurement and arithmetic.

### register/prefer-katakana, a Sino-Japanese word where the loanword wins

Some concepts are written either as a Sino-Japanese word or as a katakana loanword.
Where professional Japanese translation prefers the loanword, the table carries the pair
and the rule reports the Sino-Japanese side. A table decides this rather than a
statistic, so every occurrence is reported.

The table lives in `crates/suikou-core/data/kata_pairs_ja.toml`.
Candidates were listed by hand, and measurement across fields decided which ones to keep:
the loanword has to outnumber the Sino-Japanese word at least two to one, the pair has to
reach twenty occurrences, and the Sino-Japanese side has to stay at or below sixty.

That last condition avoids words with more than one sense.
Professionals use 対象, 対応, and 状態 often, but not always in the sense the loanword
carries. Restricting the table to words professionals barely use keeps each pair
trustworthy as a substitution.

A word preceded by a noun is skipped as part of a compound, since rewriting the 版 inside
英語版 or 第3版 would be wrong.

`.suikou/register-allow.toml` exempts a word a project uses in another sense.

## Golden tests

The expected values for each file under `tests/golden/input/` sit in
`tests/golden/expected/`.

| File | Purpose |
|---|---|
| `ja_maintainability.md` | Fire M1 through M7 in Japanese |
| `en_maintainability.md` | Fire the same rules in English |
| `ja_prose.md` | Check the Japanese document metrics on prose-heavy text |
| `en_prose.md` | Check the English ones, which need 445 words or more |
| `en_escaped.md` | Check that undoing escapes changes the sentence split |

The expected values come from inputs written for the purpose.
Regression against a real corpus uses whatever `corpus/fetch.sh` fetches.
Licensing keeps the corpus itself out of the repository.
