---
title: "Metric definitions"
weight: 20
doctype: "reference"
---

This reference specifies the metric and rule definitions that the Rust implementation must satisfy. The Python code under `research/` serves as the reference implementation, and `tests/golden/` stores its expected outputs.

Because lindera and fugashi are different tokenizers, the two implementations are not required to produce identical numbers. Comparisons allow a tolerance margin, and any metric falling outside that margin is resolved case by case to decide which implementation is correct.

## Preprocessing

Processing order changes the output, so preprocessing runs the steps below in exact sequence.

1. Remove YAML front matter.
2. Remove fenced code blocks.
3. Remove Hugo shortcodes whose contents are code.
4. **Unescape backslash sequences**, converting `\.` back into `.`.
5. Remove HTML tags.
6. Classify lines into blocks.
7. Remove inline markup (links, code spans, and emphasis).

Fenced code blocks are identified by scanning line by line. Following CommonMark, a fence ends only on a line with the same delimiter character, at least as long as the opening delimiter, and with nothing following it. A longer fence can enclose a shorter one, so terminating on an inner fence would cause the remaining code to be parsed as prose.

The third step targets `highlight`, `mermaid`, and any `tab` shortcode that declares `codelang`, because the markup explicitly identifies their contents as code. Shortcodes such as `note` and `caution` enclose prose, so stripping them would delete body text. An opening tag with no matching closing tag is left untouched, because treating it as unclosed would drop every subsequent line in a document with unbalanced tags.

Omitting the fourth step prevents accurate sentence splitting. Documents in the AWS doc_source corpus use backslash-escaped punctuation, and leaving those escapes intact inflates mean sentence length by a factor of three or more. `tests/golden/input/en_escaped.md` verifies this behavior: the sentence count is 6 before unescaping and 11 afterward.

Every step preserves the total line count. Removing lines outright would shift line numbers below the edit and produce findings that point to the wrong lines, making them impossible to act on.

### Block classification

| Block category | Detection rule |
|---|---|
| Heading | Up to three leading spaces, followed by one to six `#` characters and a space |
| List item | Optional leading spaces, followed by `-`, `*`, `+`, or a number with `.` or `)`, and a space |
| Table | Line starts with `\|` |
| Quote | Line starts with `>` |
| Prose | Any remaining non-empty line |

The three-space limit on headings follows CommonMark. Four or more leading spaces indicate an indented code block, so a `#` inside one is not classified as a heading.

### Scope of each metric category

| Metric category | Input scope |
|---|---|
| Vocabulary (MATTR, kanji ratio, kango ratio, demonstratives, formal nouns) | Prose and list item bodies |
| Sentence structure (length, commas, endings, renyo, te-form) | Prose only |
| Formatting (prose ratio, list ratio, bold) | Every line |

Headings fall outside the vocabulary and sentence-structure metrics. The register rules (`register/`) and the glossary spelling rule (`glossary/variant`) leave headings alone as well. The rules that do read headings are listed under "Heading rules".

The list-ratio denominator counts non-empty lines, and heading lines are among them. A document with many headings therefore shows a lower list ratio.

Vocabulary metrics must never run on prose alone. Some models place almost all content into bullet lists, so analyzing prose alone discards more than eighty percent of the text. For instance, one model in the evaluation corpus wrote only fifteen percent of its output as prose.

## Sentence splitting

Japanese text splits immediately after `。`, `！`, and `？`. Segments shorter than five characters are discarded, along with segments containing no Japanese characters.

English text uses the spaCy sentence splitter. Sentences with fewer than four tokens after punctuation removal are discarded.

## Japanese metrics

Japanese metrics use UniDic for tokenization. IPADIC provides neither word origin nor inflected form, so it cannot be used here.

### ja.mattr100

Use a moving window of one hundred tokens across the text, divide the count of distinct tokens in each window by the window width, and average across all windows. When total tokens fall below the window width, divide the distinct token count of the entire text by the total token count. Surface forms serve as tokens.

Human writing ranges from 0.559 to 0.581, and AI output ranges from 0.629 to 0.665. The threshold is 0.60.

### ja.renyo_per_1k

Count every verb whose inflected form begins with 連用形 and is immediately followed by a comma. Divide by the character count of prose (excluding spaces) and multiply by one thousand.

### ja.te_per_1k

Count every conjunctive particle whose surface form is `て` or `で` and is immediately followed by a comma. Normalize per thousand characters using the same denominator.

### ja.renyo_te_ratio

Divide `renyo_per_1k` by `te_per_1k`. If the denominator falls below 0.05, omit the ratio rather than reporting it.

Native Japanese human text measures 2.00, translated Japanese measures 1.79, and AI output ranges from 3.62 to 15.94. The threshold is 3.0.

Among all metrics, this ratio produces the clearest separation between human and AI writing. The te-form reflects spoken usage and connects clauses into a single unit, whereas the renyo form reflects written usage and coordinates clauses in parallel. AI favors the renyo form, corresponding to its overuse of participial clauses in English.

### ja.keishiki_meishi_per_1k

Count every noun token whose surface form is `こと`, `もの`, `ため`, `よう`, `点`, or `場合`, and normalize per thousand characters.

Human writing measures 11.1, while AI output ranges from 4.99 to 6.95. Values below the 8.5 threshold are reported. **Because this finding indicates an absence**, it carries no location.

This metric varies with the requested document type. In procedural documents, AI values reach 8.11, nearly meeting the threshold. Severity stays at info until human baselines exist for each document type.

### ja.demonstrative_per_1k

Count every token whose surface form is `これ`, `それ`, `この`, `その`, `これら`, `それら`, `こちら`, `そちら`, `ここ`, or `そこ`, and normalize per thousand characters.

Human writing ranges from 5.69 to 6.47, while AI output ranges from 1.38 to 3.39. Values below the 4.5 threshold are reported. This finding likewise indicates an absence and relates to zero anaphora in Japanese.

### ja.kango_ratio

Among content words (nouns, verbs, adjectives, adverbs, and adjectival nouns), compute the proportion whose word origin is `漢`. The denominator is the count of content words with origin `和`, `漢`, `外`, or `混`.

Human writing ranges from 0.326 to 0.397, while AI output ranges from 0.455 to 0.519. The threshold is 0.42. Because document type and subject matter affect this ratio, severity stays at info.

### ja.kanji_ratio

Divide the count of kanji characters (U+4E00 through U+9FFF) by the total count of Japanese characters (hiragana, katakana, and kanji).

### ja.prose_ratio and ja.list_ratio

The prose ratio divides the character count of prose by the combined character count of prose and list item bodies. The list ratio divides the count of list item lines by the total count of non-empty lines.

Both counts exclude spaces.

## English metrics

Only three English metrics passed validation. Participial clauses and nominalization overlapped with the human range, and the 5.3 effect size reported in prior work did not replicate on the document-writing task.

### en.mattr100

The formula matches the Japanese definition. Tokens are lowercased surface forms with symbols and spaces excluded.

Human writing ranges from 0.58 to 0.69, and AI output ranges from 0.72 to 0.82. The threshold is 0.71. Every AI condition exceeds every human corpus, making this the most consistent metric in the set.

### en.prose_ratio

The formula matches the Japanese definition. Human writing ranges from 0.66 to 0.97, and AI output ranges from 0.31 to 0.56. The threshold is 0.65.

### en.simile_marker_per_1k

Count case-insensitive occurrences of the phrases below and normalize per thousand words.

`like a` / `like the` / `as if` / `as though` / `akin to` / `similar to` / `think of it as` / `imagine` / `analogous to` / `for example` / `for instance`

Human writing ranges from 0.84 to 2.54, while AI output ranges from 0.00 to 0.60. Values below the 0.8 threshold are reported.

AI frequently uses figurative terms without marking the comparison. This metric aligns with the convention of avoiding bare metaphors in favor of explicit similes or concrete examples: stating that a cache behaves like a queue marks the comparison, whereas calling the cache a queue does not.

## M rules

### M1: List lead-ins

The lead-in is the closest non-empty line preceding a list. When that line is a heading or another list item, no evaluation is made.

Japanese lead-ins are classified using the criteria below.

| Result | Condition |
|---|---|
| ok | Line ends with `。`, `！`, or `？` |
| ok | Line ends with `:` or `：` preceded by a noun, suffix, or pronoun |
| ng | Line ends with `:` or `：` preceded by a predicate |
| ng | Final token is a particle |
| ng | Final token is a verb or auxiliary in 連用形 |
| other | None of the conditions above apply |

In English, a line ending in `:` is classified as ok when it contains `the following`, `as follows`, `these`, `below`, `steps`, or `following`, and as ng otherwise. A line ending in `.`, `!`, or `?` is classified as ok.

A stricter check would verify the presence of a finite verb. Even without that check, this rule distinguishes the AWS S3 user guide (30 incomplete versus 145 complete) from the Linux kernel documentation (105 incomplete versus 16 complete).

The violation rate is `ng / (ok + ng)`.

### M2: Item counts in prose

In prose outside headings, match the patterns below.

In Japanese, match a numeral followed by `つ`, `個`, `点`, `種`, or `類`, and then one of `理由`, `方法`, `ポイント`, `点`, `要素`, `ステップ`, `手順`, `方針`, `特徴`, `観点`, `側面`, `利点`, `欠点`, `課題`, `原則`, `要因`, or `種類`. Forms such as `以下の3つ`, `次の2点`, `3つある`, and `3つあります` also match.

In English, match `two` through `seven`, or `2` through `7`, followed by one of `reasons`, `ways`, `steps`, `things`, `types`, `categories`, `benefits`, `points`, `factors`, `approaches`, `options`, `principles`, `rules`, `components`, or `parts`.

Stating an item count in prose becomes inaccurate the moment an item is added or removed.

**Findings are reported only when the count refers to upcoming items.** The table below lists the qualifying signals.

| Signal | Rationale |
|---|---|
| Sentence contains 以下, 次の, 下記, `following`, `below`, or `these` | Explicitly announces upcoming items |
| Followed by a list, heading, or table | Separate structure editable independently of the sentence |
| Followed by prose containing an enumerating word | Likewise editable independently of the sentence |

Counts resolved within the same sentence are ignored, as in `無効化の方法は大きく分けて二つある。時間で切る方法と、変更を検知して切る方法である。` Adding a third method requires editing that passage anyway, so the count cannot become inconsistent with the items.

This condition was derived by classifying 168 matches across 798 reference documents. Out of 81 matches with no forward-reference marker followed by prose, only 3 contained an enumerating word. Applying the filter reduced matching documents from 112 to 67, as recorded in D-45.

The English enumerating words (`first`, `second`, and `one is`) were adapted from the Japanese rule rather than measured on an English corpus.

### M3: Numbered headings

Count every heading line where `#` is followed by a number and then `.`, `、`, `)`, `章`, or `節`. The scope is every heading except the title; the body is never read. The judgment lives in `headings.rs`.

Document titles are excluded. Architecture decision records use titles such as `# 3. Title`, where the number identifies the record rather than a section sequence that changes when sections move. Because 155 of 190 published ADRs triggered false positives, the leading level-one heading is exempt.

### M4: Manual list numbering

Count every list item using `-`, `*`, or `+` whose body starts with a number followed by `.` or `)`.

### M5: Parallel list endings

Group all items within a single list by the form of their ending.

Japanese separates items into three groups: those ending with a full stop, those ending with a noun, and those ending with a predicate. English separates items into two groups: those ending with a period and those that do not.

A list containing items from two or more groups is classified as mixed. The mixed rate divides the count of mixed lists by the total count of lists containing two or more items.

### M6: Time-dependent expressions

Count every occurrence of the expressions below. Two occurrences on the same line count as two findings. The scope is headings including the title, plus prose, list items, and tables. The heading judgment lives in `headings.rs`, and its positions join the body positions in a single finding.

In Japanese, match `現在`, `現時点`, `最新の`, `新しい`, `今後`, `将来的に`, `まもなく`, `既存の`, `目下`, and `現行の`, plus `最近`, `近年`, `直近`, `先日`, `従来`, `当面`, and `順次`.

In English, match `currently`, `presently`, `eventually`, `soon`, `latest`, `newest`, `newer`, `as of this writing`, `at present`, `in the future`, `in the near future`, and `for now`, plus `recently`, `lately`, `these days`, `nowadays`, `so far`, `for the time being`, and `up to now`.

Both groups exhibit the same issue: each expression takes the time of writing as its reference point. Google's guidelines on timeless documentation single out `recently`, `soon`, and `new`, and the remaining items complete that set.

### Unanchored relative time spans

Durations measured relative to the time of writing cannot be listed as fixed words because they contain numbers, as in `直近3ヶ月`, `過去2年`, `in the past three months`, `last week`, and `next month`. Pattern matching detects them instead.

### Skipping sentences with an absolute anchor

Referring to a point in time is not an error; the error is phrasing whose meaning changes depending on when it is read. When the same sentence includes an absolute anchor, the rule skips that sentence.

| Anchor type | Example |
|---|---|
| Calendar year | `Measured in March 2026` |
| Version number | `verified against lindera 6.0.0` |
| Numbered document | `RFC 7322 states` |

`Verified against lindera 6.0.0` states a past fact that remains accurate over time, whereas `The current
feature order is this` changes truth value depending on when it is read. What distinguishes them is the anchor, not the grammatical tense.

Anchors apply only within the same sentence; a year in an adjacent sentence of the same paragraph does not exempt the sentence being checked. Only a `v`-prefixed version or a three-part version number qualifies as a version anchor, preventing expressions such as `2.5 times` from matching.

This mechanism operationalizes the policy permitting deliberate references to a point in time. Rather than leaving the decision to subjective judgment, the rule reduces it to checking for the presence of an anchor.

### Narrow scope of the English term list

The English term list is limited to the strict set above. Adding `new`, `now`, `future`, and `existing` increases false positives sharply: with those words included, Google eng-practices recorded the highest violation rate of any corpus at 6.13, whereas with the strict set it recorded the lowest rate of any corpus at 0.58.

This is the single rule that a brief instruction fails to enforce. Applying it in the pre-generation layer requires listing each prohibited term explicitly.

### M7: Trailing enumeration markers

In Japanese, match lines ending with `など` or `等`. In English, match lines ending with `etc.`, `and so on`, or `and more`.

Both prose and list items are evaluated.

## Heading rules

The rules that read heading text live in `crates/suikou-core/src/headings.rs`. When each rule narrows its own block scope, nothing short of reading the implementation tells you which rules reach headings.

| Rule | Headings it reads |
|---|---|
| `structure/heading-form` | Chapter titles only |
| `maint/numbered-heading` | Every heading except the title |
| `maint/time-dependent` | Every heading, title included |
| `glossary/defined-name` | Every heading, judged together with the body |

One rule id yields one finding. For a rule that reads both the body and the headings, the heading side returns positions only and the body side folds them into a single finding. Two findings would print the same rule explanation twice in the Markdown output and would apply `MAX_POSITIONS_PER_RULE` once per producer.

### structure/heading-form: Chapter titles ending in a predicate

This rule emits a warning when the last morpheme of a chapter title is a verb, an auxiliary verb, or an adjective.

A chapter title is whatever `structure::sections` returns: the leading level-one heading is the document title rather than a section, and only the shallowest remaining level counts as a chapter. The level is not pinned to two, so a document that starts at `##` and one that starts at `###` share a single definition.

The tail is trimmed before the judgment. Punctuation goes, and a closing bracket takes everything back to its opening bracket with it, so that `設定を変更する（任意）` still reads as a predicate ending. A closing bracket with no opener trims nothing. A heading with no Japanese character is skipped, because applying a Japanese tokenizer to an identifier heading says nothing about its form. For instance, `ja.renyo_per_1k` ends in a symbol token that carries no grammatical form at all.

The rule runs on Japanese documents only. English draws no line between a noun phrase and a predicate ending, and the English path never loads a tokenizer.

Chapter titles end in a predicate in 3.34% of cases across 798 documents written by native Japanese authors, 1.78% across 123 published ADRs, 11.16% across 200 procedure documents, and 6.37% across 150 AI outputs. Human and AI distributions do not separate, so the figure cannot serve as a style signal; the rule rests instead on deviation from the convention that dominates the reference corpus. Below the chapter level the rate runs from 15% to 26%, which is why only chapter titles are in scope.

The rule is off for the `howto` and `log` doctypes. A chapter of a procedure document is a single operation and a chapter of a work log is a task, so a predicate ending is correct in both. For example, `テーブルを作成する` names the step the reader performs. The procedure corpus fires at 24.5%, far above the 10% that counts as a fit. The `decision` doctype keeps the rule: a published ADR carries its decision statement in the level-one title, which `structure::sections` drops, leaving noun-phrase section names to be judged. That corpus fires at 4.9%, inside the target.

Documents that declare no doctype are in scope. Per-doctype exclusions engage only once a doctype resolves, so an undeclared procedure document draws findings at the 24.5% rate. Such a document either declares its doctype or turns the rule off in `.suikou/profile.toml`.

See D-52 in [Decision record]({{< relref "decisions.md" >}}) for the reasoning.

## Register rules

### register/rare-wago: Overused native Japanese words

This rule flags native Japanese content words that occur much more frequently in the target text than in professional Japanese technical translation. It inspects only content words of native origin and skips numerals and single katakana characters. Because domain terminology relies primarily on Sino-Japanese words and loanwords, restricting the check to native words keeps technical terms out of the results.

A word must occur three or more times to trigger a finding, because a single occurrence cannot be distinguished from an isolated phrasing choice.

Two statistical criteria determine whether a finding is reported. Dunning's (1993) log-likelihood ratio must reach 15.13 (p < 0.0001 at one degree of freedom), and Hardie's (2014) Log Ratio must reach 7.0 (128 times the reference rate). Statistical significance alone is insufficient, because using a common word slightly more often can still reach significance at this corpus size.

The reference frequency table is stored in `crates/suikou-core/data/register_ja.toml`. It is built from 652 Japanese documentation pages from Kubernetes, MDN, and Vue, totaling roughly 290,000 content words. Tutorials are excluded due to their different register, and a word must occur in at least two domains to be included. Running `research/lexshift/build_reference.py` rebuilds the table.

Words listed in `.suikou/terms.toml` and `.suikou/register-allow.toml` are skipped, because the web-documentation reference corpus contains few terms from measurement and arithmetic.

### register/prefer-katakana: Sino-Japanese words replaced by loanwords

Some concepts can be written either as a Sino-Japanese word or as a katakana loanword. When professional Japanese translation prefers the loanword, the lookup table stores the pair and the rule reports the Sino-Japanese form. Because detection uses a lookup table rather than a statistical threshold, every occurrence is reported.

The table is stored in `crates/suikou-core/data/kata_pairs_ja.toml`. Candidate pairs were compiled manually and filtered using cross-domain counts: the loanword must outnumber the Sino-Japanese word by at least two to one, the pair must reach twenty occurrences, and the Sino-Japanese form must stay at or below sixty occurrences.

The upper limit filters out words with multiple senses. Professional translators use 対象, 対応, and 状態 frequently, but not always in the sense expressed by the loanword. Limiting the table to Sino-Japanese words that professionals rarely use ensures that each pair remains reliable as a substitution.

A match preceded by a noun is skipped as part of a compound, avoiding incorrect rewrites of 版 in terms such as 英語版 or 第3版.

Projects using a listed word in a different sense can exempt it in `.suikou/register-allow.toml`.
## Structural rules

Unlike style rules, structural rules do not depend on the language of a document.
Section sets, section ordering, and paragraph boundaries belong to document organization rather than language-specific vocabulary, leaving only localized section names and sentence segmentation to vary per language.

Corpus measurements support this separation.
Paragraphs of seven or more sentences occur at a rate of 0.27 percent in professional English and 0.59 percent in professional Japanese, placing both languages in a comparable range.

The structural design builds on established documentation standards.
RFC 7322 serves as the model for defining required sections in a recommended order and for allowing a section to contain nothing but subsections.
Per-doctype section definitions draw from the Rust RFC template, the Kubernetes Enhancement Proposal, Michael Nygard's Architecture Decision Record, and Diátaxis.
Treating each paragraph as a single unit of thought follows the Google developer documentation style guide and Strunk's The Elements of Style.

### Doctypes

Doctype definitions reside in `crates/suikou-core/data/structure.toml`.
Each entry specifies required sections, recommended section order, accepted heading synonyms, the question each section answers, and the content expected inside it.

A section may also specify a `scale` value indicating approximate length.
Both `plot` and `brief` display this guidance, whereas `check` never reports violations for it: across human-written documents, quartiles range up to 2.5 or 3 times the median, preventing a clean separation.
Only `readme` includes this attribute, as documented in D-46.

| Doctype | Purpose |
|---|---|
| `design` | Design proposal covering what is built, why it is needed, and its scope |
| `enhancement` | Phased proposal covering the test plan, graduation criteria, and rollback strategy |
| `decision` | Decision record covering context, decision, and consequences |
| `howto` | Task procedure starting with prerequisites and ending with verification |
| `reference` | Reference documentation with no required sections |
| `log` | Record log of decisions, tasks, or changes |
| `readme` | Project README whose opening paragraph states what the subject is |
| `overview` | Entry-point document whose opening states what the subject is |

Running `suikou doctype` prints a decision tree of at most two questions for selecting a doctype.
Because CI jobs and agents invoke it directly, the command prints the tree without prompting interactively.
The tree is defined under `[[select]]` in `structure.toml`, and a test verifies that every doctype is reachable through it.

Doctype declarations are resolved in precedence order, starting with the source closest to the document.

| Source | Syntax or configuration |
|---|---|
| Document front matter | `doctype: "design"` |
| Mapped front-matter field from another tool | `[front_matter]` and `[front_matter.map]` |
| `[paths]` in `.suikou/structure.toml` | `"README.md" = "readme"` |
| Plot header | `<!-- suikou:doctype design -->` |

When two path prefixes match, the longer prefix takes precedence.

External documentation systems often declare document types under another front-matter field, such as Hugo's `content_type`.
Configuring that field name and mapping its values assigns doctypes without editing any source files.

```toml
[front_matter]
field = "content_type"

[front_matter.map]
task = "howto"
reference = "reference"
```

Any front-matter value absent from the mapping leaves the doctype undeclared rather than inferring a type.

Because a plot is created with `--doctype`, it records the doctype directly, so a document backed by a plot requires no separate declaration.
Storing that annotation as an HTML comment keeps it invisible when the plot is rendered as Markdown.

Documents with no matching declaration from any of those sources are exempt from doctype checks, preventing documents written before the rule existed from failing once the rule is enabled.

An unrecognized doctype name triggers an immediate error.
Ignoring unknown names silently would disable validation whenever a doctype name is misspelled.

### structure/missing-section

This rule reports an error when no heading matches a required section of the declared doctype.

Matching checks top-level headings against each section's synonym list.
An initial level-one heading is treated as the document title rather than a section and excluded.
Because doctypes govern only the top-level outline, subheadings nested inside a section do not satisfy required sections.

A section whose role is to state the subject can be satisfied by the lead paragraph rather than an explicit heading when the doctype sets `lead_ok`.
In RFC 7322, the abstract appears between the title and the table of contents without a heading, and READMEs follow the same pattern where adding a heading reduces readability.

Previously, this exemption applied automatically to whichever required section came first.
As a result, any opening paragraph satisfied the first required section regardless of its role, causing all 120 Kubernetes concept pages to pass as `howto` documents even though an opening paragraph cannot supply prerequisites or context.

Heading matching checks both English and Japanese synonym lists so documents written in one language can use headings from the other.

Error reports include more than the missing section name; they also show the question the section answers and the content expected inside it.

Required sections vary across doctypes.
For `howto`, only prerequisites are required, while steps and verification are optional.
Running the check across 120 Kubernetes task pages flagged steps as absent on all 120 pages and verification as absent on 113, because the procedural steps form the body of a how-to guide rather than a single dedicated section.
D-31 records this rationale.

Removing those sections from error detection keeps them in writing guidance.
Both `plot` and `brief` still ask how to perform the task and how the reader confirms it succeeded, reflecting the distinction between what automated checks can verify and what helps authors before drafting.

### structure/section-order

This rule emits a warning when detected sections depart from the recommended order defined by the doctype.
Section order guides the reader through the document rather than acting as a strict constraint.

Checking 190 published ADRs verified that the recommended sequence matches established practice.
Of those, 189 contained enough sections to evaluate ordering, and the rule triggered on none of them.

### structure/long-paragraph

This rule emits a warning when a paragraph contains more than `MAX_SENTENCES_PER_PARAGRAPH` sentences.
The Google developer documentation style guide sets the upper bound at five or six sentences, and the constant adopts six.

Blank lines delimit paragraphs.
Japanese sentences are segmented with `split_sentences_ja`, whereas English splits only when a sentence terminator is followed by whitespace so decimal values such as `11.095` are not split into two sentences, which would inflate the count.

### structure/empty-section

This rule emits a warning when a leaf section has no body content.

Container sections that hold subsections need no body text of their own, following RFC 7322.
A section whose only content is a code block counts as non-empty.

### structure/plot-mismatch

This rule emits a warning when document headings diverge from an approved plot under `.suikou/plans/`.

Because plot files are local working drafts kept out of version control, CI pipelines cannot read them.
This check therefore operates in local development environments and agent loops.

### structure/lead-opener

This rule emits a warning when the lead paragraph opens with a flagged phrasing pattern.

The lead paragraph is the first prose block following the document title and preceding the first section heading.
Documents without a lead paragraph are not checked.

Flagged opening patterns are maintained separately per language.

| Pattern | Japanese | English |
|---|---|---|
| Broad generalizations | `近年`, `従来`, `昨今` | `In recent years`, `Traditionally` |
| Origin narrative | `筆者は`, `当初は` | `I built`, `We decided`, `Originally` |
| Statement of document purpose | `本文書の目的は` | `The goal of this`, `This aims to` |

Adoption was grounded in corpus measurements.
Across 831 English lead paragraphs — 199 notable open-source READMEs, 20 Rust RFCs, 10 ADRs, and 602 English reference documents — each flagged pattern occurred zero times.
These patterns were likewise absent across 653 lead paragraphs in the Japanese reference corpus.

Self-referential openings (`This document describes`, `本文書は`) are not flagged.
They appear in 37 of 602 English reference documents (6%), where API documentation uses them effectively.

Evaluated across 2,080 real documents, the rule produced zero warnings, as documented in D-39.

The rule does not detect writer's notes placed in the body of a document.
Requiring the lead paragraph to begin with the subject keeps such notes out of the opening.

### Writing guidance

Structural rules also provide guidance that produces no lint findings.
Before any prose is written, `suikou plot` and `suikou brief --doctype` present the section outline and composition principles.

Reporting violations shows what to remove rather than what to write.
Each section therefore includes the question it answers and what belongs inside it, accompanied by rules for structuring paragraphs and placing conclusions first.
All of this guidance is maintained in a single location, `structure::writing_rules`.

## Adjusting thresholds

### Global overrides

Only fields explicitly configured in `.suikou/profile.toml` override built-in defaults.
Replacing the entire profile would require duplicating every setting just to modify a single threshold.

```toml
[thresholds."ja.kango_ratio"]
value = 0.55

# Metrics to stop reporting
disable = ["en.simile_marker_per_1k"]
```

Defining a threshold for a metric that has no built-in default requires both `value` and `direction`.
Because either field alone cannot evaluate a pass or fail condition, supplying only one produces an error rather than being ignored silently.

### Per-doctype overrides

A document's purpose changes how metrics apply to it.
For instance, a README serves as an entry point and is less formal than reference documentation.

```toml
[doctype.readme]
reason = "Agreed with the user: an entry point is mostly lists"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

The `reason` field records why an override exists so later readers do not have to repeat the same deliberation.
Every built-in adjustment includes a rationale, and a test verifies that none are missing.

The `rules` field suppresses local rules for that doctype.
Whereas numeric metrics can be loosened through threshold adjustments, local rules are either reported or suppressed.

Configuration layers apply in order: the built-in profile, built-in doctype adjustments, project-wide overrides, and project-level doctype adjustments.
Project configuration always takes highest precedence so teams can override any built-in adjustment that does not fit their measurements.

Doctype sections cannot introduce new metrics; only global overrides can add them.
Allowing metrics to be defined per doctype would make it hard to track which criteria apply to which documents.

### Built-in doctype adjustments

Built-in adjustments are defined for `readme`, `design`, and `decision`, derived from measuring notable open-source corpora: 240 English READMEs, 150 Japanese READMEs, 200 Rust RFCs, and 190 published ADRs.

Under default thresholds, English READMEs trigger warnings at the rates shown below.

| Metric | README | Reference | Ratio |
|---|---|---|---|
| `en.mattr100` | 70% | 15% | 4.7 |
| `en.prose_ratio` | 32% | 11% | 2.9 |
| `en.simile_marker_per_1k` | 74% | 45% | 1.6 |

When a metric triggers on 70% of a genre, it indicates a mismatched threshold rather than a defect in the documents.

| Metric | README (adjusted) | Reference (unchanged) |
|---|---|---|
| `en.mattr100` | 6% | 15% |
| `en.prose_ratio` | 7% | 11% |
| `en.simile_marker_per_1k` | 0% | 45% |

Across 150 Japanese READMEs, adjusted warning rates fall between 0% and 2%.

Formality differences do not shift in only one direction.
Japanese READMEs have a higher kango ratio than the reference corpus—0.62 versus 0.52—because enumerating features and components concentrates Sino-Japanese compounds.
D-41 records this measurement.

## Context excerpts in diagnostics

Each diagnostic location displays the matched text first, followed by a narrow surrounding excerpt.

```
- L5:78 Currently｜…e limiting to the public API. Currently, a
```

Placing the match first lets the reader see immediately which word requires editing.
Extracting a fixed prefix from the start of the line would omit matches occurring later in the line and print identical excerpts when multiple matches share a line.

The context window extends fourteen characters to each side in Japanese and twenty characters to each side in English, reflecting the greater information density of Japanese characters.

## Golden tests

Expected outputs for each fixture in `tests/golden/input/` are stored under `tests/golden/expected/`.

| File | Purpose |
|---|---|
| `ja_maintainability.md` | Trigger M1 through M7 in Japanese |
| `en_maintainability.md` | Trigger the same maintainability rules in English |
| `ja_prose.md` | Check Japanese document metrics on prose-heavy text |
| `en_prose.md` | Check English document metrics, which require 445 words or more |
| `en_escaped.md` | Check that unescaping changes sentence segmentation |

These expected outputs come from inputs authored specifically for the test suite.
Regression testing against real-world documents runs on the corpus downloaded by `corpus/fetch.sh`, because licensing prevents storing the corpus in the repository.

## Glossary rules

### glossary/defined-name: Names other than the defined one

This rule emits an error when a name registered in `.suikou/defined.toml` is called something else.

Each entry carries the defined name in `name`, the alternative wordings that refer to the same thing in `instead_of`, and the reason behind the choice in `note`. A `note` is printed alongside the finding.

The scope is prose, list items, and headings. Keeping one name for one thing is a question of identifiers rather than of register, akin to a symbol in source code: a reader who meets a different word in a heading has no way to tell that the subject is the same. Body and heading positions join in a single finding.

With no configuration file present the rule reports nothing.

### glossary/variant

This rule emits a warning when a spelling diverges from the canonical form defined in the glossary.

It checks orthographic variants, normalizing `サーバ` to `サーバー`, as well as multi-character calques, normalizing `根因` to `根本原因`.

Vocabulary selection belongs to `register/prefer-katakana`, preventing two rules from flagging the same location and forcing the reader to inspect duplicate diagnostics.

One-character calques are excluded.
Substring matching would match `題` inside `問題`, a distinction that requires morphological analysis, which `prefer-katakana` performs.

The rule inspects prose and list bodies while skipping headings and code.
Because it requires no dictionary, it runs even in builds compiled without one.

### Orthography conventions

The third edition of the JTCA guideline on loanword orthography writes English endings -er, -or, -ar, and -y with a long-vowel mark.

Measured across 4.38 million characters of the reference corpus, real usage aligns with the guideline for -er, -or, and -ar.

| Word | With long-vowel mark | Without long-vowel mark |
|---|---|---|
| サーバー | 591 | 130 |
| ユーザー | 1830 | 328 |
| パラメーター | 158 | 6 |
| クラスター | 1279 | 5 |

The -y ending shows the opposite trend, so each word is settled by corpus measurement rather than derived mechanically from the guideline.

| Word | With long-vowel mark | Without long-vowel mark |
|---|---|---|
| プロパティ | 3 | 870 |
| メモリ | 75 | 402 |
| ディレクトリ | 9 | 227 |
| コンテナ | 95 | 1783 |

Although `コンテナ` derives from an -er ending, usage in this field has settled on the spelling without the long-vowel mark.
