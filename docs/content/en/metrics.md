---
title: "Metric definitions"
weight: 20
doctype: "reference"
---

This page is a reference for the definitions that the Rust implementation must satisfy.
The reference implementation is the Python code under `research/`, and `tests/golden/`
fixes its output.

Because lindera and fugashi are different tokenizers, the two implementations are not
required to agree exactly, and the comparison allows a tolerance. When a metric falls
outside that tolerance, the correct implementation is decided case by case.

## Preprocessing

The order of the steps affects the result, so the steps run in the order below.

1. Remove the YAML front matter.
2. Remove fenced code blocks.
3. Remove the Hugo shortcodes whose content is code.
4. **Undo backslash escapes**, so that `\.` becomes `.` again.
5. Remove HTML tags.
6. Classify each line as a type of block.
7. Remove inline markup, that is, links, code spans, and emphasis.

Fenced code blocks are detected by scanning line by line. As CommonMark specifies, a fence
closes only on a line that uses the same marker as the opening fence, is at least as long,
and has nothing after it. A long fence can contain a shorter one, and treating the inner
fence as the close would cause the rest of the code to be read as prose.

The third step applies to `highlight`, `mermaid`, and any `tab` that declares `codelang`.
In each of these cases the document itself declares that the content is code. A shortcode
such as `note` or `caution` encloses prose, so removing it would remove the body text as
well. An opening tag with no matching close is left in place, because treating it as open
would remove every line after it in a document whose tags are unbalanced.

If the fourth step is skipped, sentence splitting stops working.
The AWS doc_source corpus uses this escaped form, and when the escapes are not removed the
mean sentence length grows by a factor of three or more.
`tests/golden/input/en_escaped.md` locks in this behavior.
That file contains 6 sentences before the escapes are removed and 11 afterward.

No step changes the number of lines. If a step removed lines entirely, every line after
the removal would get a different line number, and a finding that refers to the wrong line
cannot be acted on.

### How lines are classified into blocks

| Kind | Rule |
|---|---|
| Heading | Up to three spaces, then one to six `#` characters followed by a space |
| List item | Leading spaces, then `-` `*` `+`, or a number with `.` or `)`, then a space |
| Table | The line begins with `\|` |
| Quote | The line begins with `>` |
| Prose | Any other line that is not empty |

The limit of three spaces before a heading follows CommonMark. A line indented by four or
more spaces is code, and a `#` inside an indented code block must not be read as a heading.

### Which text each metric covers

| Kind of metric | Text covered |
|---|---|
| Vocabulary: MATTR, kanji ratio, kango ratio, demonstratives, formal nouns | Prose and the text of list items |
| Sentence structure: length, commas, endings, renyo, te-form | Prose only |
| Format: prose ratio, list ratio, bold | Every line |

A vocabulary metric must never be limited to prose.
Some models put nearly all of their output into bullet lists, and for those models a
prose-only measurement discards more than eighty percent of the text. In the corpus, for
example, one model left only fifteen percent of its output as prose.

## Sentence splitting

Japanese text is split immediately after `。` `！` `？`.
A fragment of fewer than five characters is discarded, as is a fragment that contains no
Japanese characters.

English text is split by spaCy.
A sentence with fewer than four tokens after punctuation is removed is discarded.

## Japanese metrics

The tokenizer uses UniDic. IPADIC provides neither the word origin nor the inflected form,
so it cannot be used for these metrics.

### ja.mattr100

Use a moving window that is one hundred tokens wide.
For each window, divide the number of distinct tokens by the window width, and then take
the average over all windows.
If the text has fewer tokens than the window width, divide the number of distinct tokens
in the whole text by the number of tokens.
Tokens are surface forms.

Human text ranges from 0.559 to 0.581, and AI text ranges from 0.629 to 0.665.
The threshold is 0.60.

### ja.renyo_per_1k

Count every position where a verb has an inflected form that begins with 連用形 and the
next token is a comma.
Divide the count by the number of characters in the prose, excluding spaces, and multiply
by one thousand.

### ja.te_per_1k

Count every position where the surface form is `て` or `で`, the part of speech is a
conjunctive particle, and the next token is a comma.
Convert the count to a rate per thousand characters in the same way.

### ja.renyo_te_ratio

This ratio is `renyo_per_1k` divided by `te_per_1k`.
If the divisor is below 0.05, the ratio is not reported.

Native human writing is at 2.00, human translation at 1.79, and AI output ranges from 3.62
to 15.94. The threshold is 3.0.

This metric separates human and AI text more clearly than any other. The te-form is typical
of speech and joins clauses into one. The renyo form is typical of writing and places
clauses one after another. AI output favors the renyo form. The same tendency appears in
English as an excess of participial clauses.

### ja.keishiki_meishi_per_1k

Count every token whose part of speech is a noun and whose surface form is one of
`こと` `もの` `ため` `よう` `点` `場合`. Convert the count to a rate per thousand characters.

Human text is at 11.1, and AI text ranges from 4.99 to 6.95.
A value below the threshold of 8.5 is reported.
**This finding indicates that something is missing**, so it has no position.

The value depends heavily on the kind of document requested. In procedures, AI text rises
to 8.11, close to the threshold. The severity remains at info until human baselines are
available for each kind of document.

### ja.demonstrative_per_1k

Count every token whose surface form is one of
`これ` `それ` `この` `その` `これら` `それら` `こちら` `そちら` `ここ` `そこ`.
Convert the count to a rate per thousand characters.

Human text ranges from 5.69 to 6.47, and AI text ranges from 1.38 to 3.39.
A value below the threshold of 4.5 is reported.
This finding also indicates that something is missing.
The metric is related to zero anaphora in Japanese.

### ja.kango_ratio

Among content words, that is, nouns, verbs, adjectives, adverbs, and adjectival nouns,
compute the proportion whose word origin is `漢`.
The divisor is the number of content words whose origin is `和` `漢` `外` or `混`.

Human text ranges from 0.326 to 0.397, and AI text ranges from 0.455 to 0.519.
The threshold is 0.42.
The value depends heavily on the kind of document and on its topic, so the severity remains
at info.

### ja.kanji_ratio

Divide the number of kanji, defined as U+4E00 through U+9FFF, by the number of Japanese
characters, defined as hiragana, katakana, and kanji.

### ja.prose_ratio and ja.list_ratio

The prose ratio is the number of characters in prose divided by the number of characters
in prose and list-item text combined.
The list ratio is the number of list-item lines divided by the number of lines that are not
empty.

Spaces are excluded from both counts.

## English metrics

Only three metrics passed validation.
Participial clauses and nominalization overlapped the human range.
The effect size of 5.3 reported in earlier research did not reappear when the task was to
write a document.

### en.mattr100

The definition is the same as for Japanese.
Tokens are lowercased surface forms, and symbols and spaces are excluded.

Human text ranges from 0.58 to 0.69, and AI text ranges from 0.72 to 0.82.
The threshold is 0.71.
Every AI condition scores higher than every human corpus, which makes this the most stable
metric of the set.

### en.prose_ratio

The definition is the same as for Japanese. Human text ranges from 0.66 to 0.97, and AI text
ranges from 0.31 to 0.56. The threshold is 0.65.

### en.simile_marker_per_1k

Count the phrases below and convert the count to a rate per thousand words. Matching is not
case-sensitive.

`like a` / `like the` / `as if` / `as though` / `akin to` / `similar to` /
`think of it as` / `imagine` / `analogous to` / `for example` / `for instance`

Human text ranges from 0.84 to 2.54, and AI text ranges from 0.00 to 0.60.
A value below the threshold of 0.8 is reported.

AI output uses figurative vocabulary but does not mark the comparison as figurative.
This metric corresponds to the policy of avoiding metaphor and using a simile or an example
instead. For example, a sentence saying that a cache behaves like a queue marks the
comparison, whereas a sentence calling the cache a queue does not.

## The M rules

### M1, the lead-in to a list

The lead-in is the closest non-empty line above the list.
If that line is a heading or another list item, the rule makes no judgment.

The Japanese check works as shown in the table below.

| Result | Condition |
|---|---|
| ok | The line ends with `。` `！` `？` |
| ok | The line ends with `:` or `：`, and the token before it is a noun, a suffix, or a pronoun |
| ng | The line ends with `:` or `：`, and the token before it is a predicate |
| ng | The last token is a particle |
| ng | The last token is a verb or an auxiliary in the 連用形 |
| other | None of the above |

In English, a line that ends with `:` is ok if it contains
`the following` `as follows` `these` `below` `steps` or `following`, and ng if it does
not. A line that ends with `.` `!` or `?` is ok.

A strict check would look for a finite verb.
Even so, this check alone distinguishes the AWS S3 user guide, with 30 incomplete against 145
complete, from the Linux kernel, with 105 against 16.

The violation rate is `ng / (ok + ng)`.

### M2, stating the number of items in prose

In prose, excluding headings, count the patterns below.

In Japanese, count a numeral followed by `つ` `個` `点` `種` or `類`, which is then followed by one of
`理由` `方法` `ポイント` `点` `要素` `ステップ` `手順` `方針` `特徴` `観点` `側面`
`利点` `欠点` `課題` `原則` `要因` `種類`.
Patterns such as `以下の3つ` and `次の2点` are counted too, as are `3つある` and `3つあります`.

In English, count `two` through `seven`, or `2` through `7`, followed by one of
`reasons` `ways` `steps` `things` `types` `categories` `benefits` `points`
`factors` `approaches` `options` `principles` `rules` `components` `parts`.

If the number of items is written in the prose, the prose becomes wrong as soon as an item
is added or removed.

**The rule reports a count only when the count refers forward.** The signals are listed below.

| Signal | Reason |
|---|---|
| The sentence contains 以下, 次の, 下記, `following`, `below` or `these` | It announces what comes next |
| A list, a heading or a table follows | That structure can be edited separately from the sentence |
| Prose containing an enumerating word follows | That prose can also be edited separately |

A count whose items are all given within the same sentence is not reported, as in this
example:
`無効化の方法は大きく分けて二つある。時間で切る方法と、変更を検知して切る方法である。`
Adding a third method would require rewriting that sentence, so the count cannot fall out
of step with the items.

This condition was derived by classifying the 168 firings in 798 reference documents. Of
the 81 firings that had no signal word and were followed by prose, only 3 had an enumerating
word. The condition reduced the number of documents that fire from 112 to 67. D-45 records
the details.

The English enumerating words, `first`, `second` and `one is`, were taken over from the
Japanese side and were not measured on an English corpus.

### M3, numbered headings

Count every heading line in which the `#` is followed by a number and then by `.` `、` `)` `章`
or `節`.

The title is excluded. An ADR title is written as `# 3. Title`, and that number identifies
the record; it is not a section number, and nothing has to be renumbered when a section
moves. Of 190 published ADRs, 155 fired on the title, so the first level-one heading is
exempt.

### M4, hand-written numbers

Count every list item whose marker is `-` `*` or `+` and whose text begins with a number
followed by `.` or `)`.

### M5, parallel items

Within each list, classify the items by how they end.

Japanese items fall into three groups: those that end with a full stop, those that end with
a noun, and those that end with a predicate.
English items fall into two groups: those that end with a period and those that do not.

A list that contains items from two or more groups counts as mixed.
The mixed rate is the number of mixed lists divided by the number of lists that contain two
or more items.

### M6, words that depend on a moment in time

Count every occurrence of the words below. Two occurrences on the same line count as two.

In Japanese, count `現在` `現時点` `最新の` `新しい` `今後` `将来的に` `まもなく` `既存の` `目下`
`現行の`, and also `最近` `近年` `直近` `先日` `従来` `当面` `順次`.

In English, count `currently` `presently` `eventually` `soon` `latest` `newest` `newer`
`as of this writing` `at present` `in the future` `in the near future` `for now`,
and also `recently` `lately` `these days` `nowadays` `so far` `for the time being`
`up to now`.

The words in the second group behave like those in the first: each takes the time of
writing as its reference point. Google's guidance on timeless documentation lists `recently`, `soon`, and
`new`, and the second group completes that set.

### Relative periods with no anchor

A period measured from the present cannot be detected from a word list, because it contains
a number, as in
`直近3ヶ月`, `過去2年`, `in the past three months`, `last week`, `next month`. These are
detected with a pattern instead.

### A sentence with an absolute anchor is passed over

Referring to a point in time is not itself the error. The error is a meaning that changes
depending on when the text is read. If the same sentence contains an absolute anchor, the
rule does not fire.

| Anchor | Example |
|---|---|
| A calendar year | `Measured in March 2026` |
| A version number | `verified against lindera 6.0.0` |
| A numbered document | `RFC 7322 states` |

`Verified against lindera 6.0.0` states a past fact and does not go out of date. `The current
feature order is this` may be true or false depending on when it is read. The difference
between the two lies in the anchor, not in the tense.

Anchors are checked sentence by sentence; a year in a neighboring sentence of the same
paragraph does not anchor this one. Only a form prefixed with `v` or a three-part number
counts as a version, so that `2.5 times` is not mistaken for a version.

This check is how the rule implements the commitment to allow a deliberate reference to a
point in time. Deliberateness cannot be checked by a machine, so the check is reduced to
whether an anchor is present.

### Why the English vocabulary is limited

The English word list is restricted to the strict set.
Adding `new` `now` `future` and `existing` increases the number of false positives.
With those words included, Google eng-practices had the highest violation rate of any
corpus, at 6.13. With the strict set, the same corpus had the lowest rate of any corpus, at
0.58.

This is the only rule that a model does not follow when it is given as a short instruction.
If the rule is placed in the layer that runs before generation, the forbidden words must be
listed one by one.

### M7, the tail of an enumeration

In Japanese, count lines that end with `など` or `等`.
In English, count lines that end with `etc.` `and so on` or `and more`.

The rule covers both prose and list items.

## The register rule

### register/rare-wago, a native word out of register

This rule reports native content words that are used much more often than in professional
Japanese translation.
It covers content words of native origin and excludes numerals and single katakana
characters.
Technical terms in a field are mostly Sino-Japanese words or loanwords, so limiting the rule
to native words keeps those terms out of the findings.

A word must appear three or more times.
With a single occurrence, an out-of-register word cannot be distinguished from an incidental
phrase.

A finding requires two statistics.
The log-likelihood ratio of Dunning (1993) must be at least 15.13, which corresponds to p < 0.0001 at
one degree of freedom, and the Log Ratio of Hardie (2014) must be at least 7.0, which corresponds to
128 times the reference rate. Significance alone is not enough, because in a corpus of this
size even an ordinary word used slightly more often becomes significant.

The reference is stored in `crates/suikou-core/data/register_ja.toml`.
It is a frequency table built from 652 Japanese pages of Kubernetes, MDN, and Vue, which
contain roughly 290,000 content words.
Tutorials are excluded because their register is different, and only words that appear in
two or more fields are included.
`research/lexshift/build_reference.py` rebuilds the table.

Words listed in `.suikou/terms.toml` and `.suikou/register-allow.toml` are excluded.
The reference was built from web documentation, which contains little vocabulary from
measurement and arithmetic.

### register/prefer-katakana, a Sino-Japanese word where the loanword wins

Some concepts can be written either as a Sino-Japanese word or as a katakana loanword.
When professional Japanese translation prefers the loanword, the pair is listed in a table,
and the rule reports the Sino-Japanese form. The decision comes from the table, not from a
statistic, so every occurrence is reported.

The table is stored in `crates/suikou-core/data/kata_pairs_ja.toml`.
Candidates were proposed by hand, and measurements across several fields decided which ones
to keep. The loanword must occur at least twice as often as the Sino-Japanese word, the pair
must occur at least twenty times in total, and the Sino-Japanese word must occur no more
than sixty times.

The last condition excludes words that have more than one meaning.
Professionals use 対象, 対応, and 状態 often, but not always with the meaning of the
loanword. Limiting the table to Sino-Japanese words that professionals rarely use makes each
pair reliable as a substitution.

A word that follows a noun is treated as part of a compound and is not reported, because
replacing the 版 in 英語版 or 第3版 would be wrong.

`.suikou/register-allow.toml` exempts a word that a project uses with a different meaning.

## The structural rules

Unlike the style rules, the structural rules do not depend on the language.
The set of sections, their order, and the paragraph as a unit belong to the logic of a
document, which is independent of the vocabulary of any particular language.
Measurement supports this. Paragraphs of seven or more sentences account for 0.27 percent
of paragraphs in professional English and 0.59 percent in professional Japanese, so both
languages fall within the same order of magnitude. Only the section names and the sentence
splitter are defined separately for each language.

The structural framework is based on the sources below.
RFC 7322 provides the precedent for a set of required sections in a recommended order, and
it also states that a section may contain only subsections.
The sections of each doctype are based on the Rust RFC template, the Kubernetes Enhancement
Proposal, Michael Nygard's Architecture Decision Record, and Diátaxis.
Treating each paragraph as a single unit of thought is based on the Google developer
documentation style guide and on Strunk's The Elements of Style.

### Doctypes

The doctypes are defined in `crates/suikou-core/data/structure.toml`.
Each doctype specifies its required sections, the recommended order, the synonyms accepted
for each heading, the question each section answers, and what each section should contain.

A section can also have a `scale`, which indicates how much to write. `plot` and `brief`
print it, but `check` never reports on it, because in human documents the quartiles spread
to 2.5 or 3 times the median and the value does not separate human from AI text. Only
`readme` has one. D-46 records the details.

| Doctype | Purpose |
|---|---|
| `design` | A proposal or design: what will be built, why it is needed, and what its scope is |
| `enhancement` | A proposal introduced in stages: the test plan, the graduation criteria, and the rollback |
| `decision` | A decision record: the context, the decision, and its consequences |
| `howto` | A procedure: it starts with the prerequisites and ends with the verification |
| `reference` | A reference: no section is required |
| `log` | A collection of records: decisions, tasks, or changes |
| `readme` | A README: the first paragraph states what the project is |
| `overview` | An entry point: it begins by stating what the subject is |

`suikou doctype` presents the questions that determine the doctype, two at most. The tool
does not prompt for answers; it prints the decision tree, because it is called from CI and
from agents. The tree is defined in `[[select]]` in `structure.toml`, and a test confirms
that every type can be reached from it.

The tool looks for a doctype declaration in three places, starting with the one closest to
the document.

| Where | What it looks like |
|---|---|
| The front matter | `doctype: "design"` |
| A field written by another tool | `[front_matter]` and `[front_matter.map]` |
| `[paths]` in `.suikou/structure.toml` | `"README.md" = "readme"` |
| The top of the plot | `<!-- suikou:doctype design -->` |

When two path prefixes match, the longer prefix takes precedence.

Some documents already declare a type under a different name. Hugo's `content_type` is an
example. By naming that field and mapping its values, you can set the doctype without
editing any document.

```toml
[front_matter]
field = "content_type"

[front_matter.map]
task = "howto"
reference = "reference"
```

A value that is not in the map declares no doctype, so that suikou does not assign a type
based on its own guess.

A plot is created with `--doctype`, so it already has a type. A document with a plot
therefore does not need its own declaration. The marker is an HTML comment so that it is not
visible when the plot is rendered as Markdown.

A document that is not declared in any of the three places is not checked by any doctype
rule. This prevents a document written before a rule existed from starting to fail once the
rule is added.

If a declared doctype name is not defined, the tool exits with an error immediately.
Ignoring the name silently would disable the check whenever the name is misspelled.

### structure/missing-section

This rule fires at error when no heading corresponds to a required section of the declared
doctype.

Headings are matched by using the synonym list, and only top-level headings are considered.
The first level-one heading is the title of the document, not a section, so it is excluded.
A doctype defines the top-level outline; a subheading within a section is not itself a
section.

A section that states the subject can be satisfied by the lead paragraph instead of a
heading; the doctype marks such a section with `lead_ok`. The abstract in RFC 7322 appears
between the title and the table of contents with no heading of its own, and a README follows
the same pattern. Requiring a heading in that position makes the document harder to read.

At one time, this exemption applied to whichever required section came first. With that
rule, the lead paragraph could satisfy any first required section, and all 120 Kubernetes
concept pages passed as the `howto` type. Prerequisites and context cannot be satisfied by an
opening paragraph.

Headings are matched against the synonyms of both languages, because a document written in
one language may use headings in the other.

The finding does more than name the missing section. It also gives the question that the
section answers and what the section should contain.

The required sections differ by doctype. For `howto`, only the prerequisites are required;
the steps and the verification are not. When the type was applied to 120 Kubernetes task
pages, steps were reported missing on all 120 and verification on 113, because the steps of
a how-to are its body and not a separate section. D-31 records the reasoning.

Removing these sections from detection does not remove them from the guidance. `plot` and
`brief` still ask how the task is done and how the reader can tell that it worked. What can
be detected is not the same as what is worth asking before writing.

### structure/section-order

This rule fires at warning when the sections found in the document do not follow the order
that the doctype recommends. The order is a path for the reader through the document, not a
rule that must never be broken.

The recommended order was checked against 190 published ADRs to confirm that it matches
practice. Of those, 189 contain enough sections for the order to be judged, and the rule
fires on none of them.

### structure/long-paragraph

This rule fires at warning when a paragraph contains more sentences than
`MAX_SENTENCES_PER_PARAGRAPH`. The Google developer documentation style guide suggests
five or six sentences as an upper limit, and the constant is set to six accordingly.

Paragraphs are separated by blank lines. Japanese sentences are split with
`split_sentences_ja`. English text is split only where a sentence-ending mark is followed by
whitespace, because splitting at every period would divide `11.095` into two sentences and
inflate the count.

### structure/empty-section

This rule fires at warning when a section at the lowest level has no body.

A section that contains subsections does not need its own body; RFC 7322 states this
explicitly. A section whose only content is a code block counts as having a body.

### structure/plot-mismatch

This rule fires at warning after comparing the headings of the body with an approved plot in
`.suikou/plans/`.

A plot is a working document kept out of version control, so CI cannot see it.
The rule therefore works only on a local machine and inside an agent's loop.

### structure/lead-opener

This rule reports at warning how the lead paragraph begins.

The lead paragraph is the first prose after the title and before the first section heading.
A document without a lead paragraph produces no finding.

The forms that are reported are defined separately for each language.

| Form | Japanese | English |
|---|---|---|
| Opens with a general statement | `近年`, `従来`, `昨今` | `In recent years`, `Traditionally` |
| Opens with the author's history | `筆者は`, `当初は` | `I built`, `We decided`, `Originally` |
| Declares the purpose of the document | `本文書の目的は` | `The goal of this`, `This aims to` |

The forms were chosen by measurement. In 831 lead paragraphs, taken from 199 READMEs of
well-known open source projects, 20 Rust RFCs, 10 ADRs, and 602 English reference
documents, none of these forms occurred even once. In Japanese, the forms likewise did not
occur in any of the 653 lead paragraphs in the reference corpus.

Self-reference (`This document describes`, `本文書は`) is not reported. It occurs in 37 of
602 English reference documents (6%), and it is a natural form in API documentation.

When the rule was run on 2,080 real documents, it fired zero times. D-39 records the details.

This rule does not detect the author's notes. Notes written in the middle of a document are
beyond its reach. Requiring the lead paragraph to start with the subject is what moves such
notes toward the end of the document.

### Guidance for writing

The structural rules also produce output that reports no problems.
`suikou plot` and `suikou brief --doctype` present the sections and the writing rules before
any of the body is written.

A list of violations shows what to avoid but does not show what to write. For this reason,
each section is presented with the question it answers and what it should contain, along with
rules for building a paragraph and for stating the conclusion first.
The text of this guidance is kept in a single place, `structure::writing_rules`.

## Adjusting a threshold

### Settings for all documents

Only the fields written in `.suikou/profile.toml` replace the built-in values. If the whole
profile had to be replaced, changing a single number would require copying everything.

```toml
[thresholds."ja.kango_ratio"]
value = 0.55

# Metrics to stop reporting
disable = ["en.simile_marker_per_1k"]
```

To add a metric that has no built-in threshold, specify both `value` and `direction`. One of
them alone is not enough to make a judgment, so the run fails instead of silently ignoring
the setting.

### Settings for one doctype

How a metric should be applied depends on what the document is for. A README is an entry
point and is less formal than a reference document.

```toml
[doctype.readme]
reason = "Agreed with the user: an entry point is mostly lists"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

`reason` records why the adjustment was made, so that a later reader does not have to go
through the same deliberation again. Every built-in adjustment has a reason, and a test
confirms this.

`rules` suppresses a local rule. A metric can be relaxed by changing its threshold, but a
local rule either reports or does not.

Settings are applied in this order: the built-in profile, the built-in doctype adjustment,
the project's own settings, and then the project's own doctype adjustment. The project's
settings are always applied last, because if a built-in adjustment could not be overridden,
there would be no remedy where the measurements do not fit.

A doctype adjustment cannot add a metric; only the settings for all documents can. If
different metrics appeared for different doctypes, it would no longer be possible to tell
which document was judged by which metric.

### The built-in adjustments

The `readme`, `design` and `decision` doctypes each have an adjustment. The adjustments were
derived from measurements of well-known open source projects: 240 READMEs in English and 150
in Japanese, 200 Rust RFCs, and 190 published ADRs.

With the default thresholds, READMEs fire at the rates shown below.

| Metric | README | Reference | Ratio |
|---|---|---|---|
| `en.mattr100` | 70% | 15% | 4.7 |
| `en.prose_ratio` | 32% | 11% | 2.9 |
| `en.simile_marker_per_1k` | 74% | 45% | 1.6 |

When a metric fires on 70% of a genre, the cause is a threshold that does not fit, not a
characteristic of the documents.

| Metric | README, adjusted | Reference, unchanged |
|---|---|---|
| `en.mattr100` | 6% | 15% |
| `en.prose_ratio` | 7% | 11% |
| `en.simile_marker_per_1k` | 0% | 45% |

For 150 Japanese READMEs, the firing rates after adjustment fall between 0% and 2%.

The difference in formality does not always point the same way. Japanese READMEs have a
higher Sino-Japanese ratio than the reference corpus, at 0.62 against 0.52, because listing
the names of features and components produces many compounds. D-41 records the details.

## The excerpt beside a finding

Each position shows the matched text first, followed by a short window of the surrounding
text.

```
- L5:78 Currently｜…e limiting to the public API. Currently, a
```

The match is shown first so that the reader knows which word to change. If a fixed number of
characters were taken from the start of the line, a match further along the line would be
left out, and a line with several matches would show the same excerpt once for each match.

The window extends fourteen characters on each side in Japanese and twenty in English,
because a Japanese character carries more information than an English one.

## Golden tests

The expected values for each file in `tests/golden/input/` are stored in
`tests/golden/expected/`.

| File | Purpose |
|---|---|
| `ja_maintainability.md` | Trigger M1 through M7 on Japanese text |
| `en_maintainability.md` | Trigger the same rules on English text |
| `ja_prose.md` | Check the Japanese document metrics on prose-heavy text |
| `en_prose.md` | Check the English document metrics, which need 445 words or more |
| `en_escaped.md` | Check that undoing escapes changes how sentences are split |

The expected values are generated from inputs written for this purpose.
Regression testing against a real corpus uses the corpus that `corpus/fetch.sh` downloads.
The corpus itself is not stored in the repository for licensing reasons.

## The glossary rule

### glossary/variant

This rule fires at warning when a spelling differs from the one the glossary has settled on.

It covers spelling, such as changing `サーバ` to `サーバー`, and literal translations of more than one
character, such as changing `根因` to `根本原因`.

Word choice itself is handled by `register/prefer-katakana`. If two rules pointed at the same
place, the reader would have to read the same finding twice.

Literal translations of a single character are not covered. Substring matching would catch
the `題` inside `問題`, so morphological analysis would be needed, and `prefer-katakana`
already performs that analysis.

The rule checks prose and list-item text, not headings or code. It does not need a
dictionary, so it also runs in the build without one.

### How spelling is decided

The third edition of the JTCA guideline on loanword spelling writes the English endings
-er, -or, -ar, and -y with the long-vowel mark.

Measurement over 4.38 million characters of the reference corpus confirms the guideline for -er, -or, and -ar.

| Word | With the mark | Without |
|---|---|---|
| サーバー | 591 | 130 |
| ユーザー | 1830 | 328 |
| パラメーター | 158 | 6 |
| クラスター | 1279 | 5 |

For the -y ending the result is the opposite, so the spelling of each word is decided by
measurement instead of being derived from the guideline.

| Word | With the mark | Without |
|---|---|---|
| プロパティ | 3 | 870 |
| メモリ | 75 | 402 |
| ディレクトリ | 9 | 227 |
| コンテナ | 95 | 1783 |

`コンテナ` ends in -er, but this field has settled on the spelling without the mark.
