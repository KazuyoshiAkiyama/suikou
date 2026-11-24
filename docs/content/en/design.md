---
title: "Design"
weight: 10
doctype: "design"
---

suikou is a layered linter and harness that removes AI style from technical documentation
and makes the result easier to maintain. It covers Japanese and English.

This page follows the rules it defines. Every list is introduced by a complete sentence,
the prose never states how many items a list contains, and no unmarked metaphor appears.

## Why this design is needed

Technical documentation written by generative AI has characteristics that go beyond
individual words. It relies on metaphor and indirect phrasing, it ends sentences the same
way again and again, it connects sentences awkwardly, and it includes so much detail that
the document becomes hard to maintain. Existing tools handle either a word list or a
spelling rule and nothing more, so none of them can detect these characteristics. A spell
checker, for example, cannot notice that a document relies on metaphor.

## Scope

The tool covers technical documentation and technical writing.
What makes prose good depends on the context, so rules cannot be defined until the context
is defined.

The tool covers Japanese and English.
Each style rule has measurements for each language, while the structural rules apply to
both languages.

## Non-goals

Fiction, advertising, and social media posts are out of scope, because their style rules
vary with the situation.

The tool does not rewrite documents. It reports positions and gives guidance derived from
the whole document, and the writer decides what to change. In this repository's own
documents, mechanical substitution has broken the meaning of a sentence more than once.

The tool does not check whether the content is factually correct. Style and structure can
be measured against a corpus, but the truth of a claim cannot.

## The most important constraint

The tool does not report one layer at a time and have the model fix each report in turn.
To keep token use low, a single run reports every layer, and a single round of edits
resolves the report.

## The measurements behind the design

Every rule is based on a measurement or on a published standard.
No rule is based on intuition.
This policy exists because intuitions about style usually turn out to be wrong once they
are measured.
Research on translationese and research on AI-written text reached this finding
independently.

### The human corpus

The Japanese corpus contains both native writing and translations.
The native writing consists of two books by kaityo256 and the future-architect coding
standards. The translations are the Japanese Kubernetes site, the Japanese Vue site, and
the ja_JP translations in the Linux kernel.

The English corpus contains the Linux kernel documentation, gcc, the Rust Book, Django,
Google eng-practices, the AWS S3 user guide, and the AWS CloudFormation user guide.

### The AI corpus

The AI corpus contains 232 documents from four models: Claude, Gemini, ChatGPT, and Grok.
The conditions combined two languages, three prompts (an explanation, a procedure, and a
design rationale), three Japanese registers, and the presence or absence of an instruction
to suppress formatting. Each condition was run in a new chat, with memory and custom
instructions disabled.

### The metrics that were kept

| Metric | Human | AI | Language |
|---|---|---|---|
| renyo over te-form | 1.79–2.00 | 3.62–15.94 | ja |
| MATTR100 | 0.559–0.581 | 0.629–0.665 | ja |
| MATTR100 | 0.58–0.69 | 0.72–0.82 | en |
| Prose ratio | 0.816–0.847 | 0.480–0.711 | ja |
| Prose ratio | 0.66–0.97 | 0.31–0.56 | en |
| Kango ratio | 0.326–0.397 | 0.455–0.519 | ja |
| Kanji ratio | 0.220–0.307 | 0.343–0.394 | ja |
| Demonstratives per 1k | 5.685–6.469 | 1.380–3.388 | ja |
| Formal nouns per 1k | 11.095–11.163 | 4.990–6.953 | ja |
| List ratio | 0.159–0.172 | 0.322–0.433 | ja |
| Simile markers per 1k | 0.84–2.54 | 0.00–0.60 | en |

The ratio of renyo to te-form separates human and AI text more clearly than any other
metric. The te-form is typical of speech and combines clauses into one.
The renyo form is typical of writing and places clauses one after another.
AI output favors the renyo form.
This is the same phenomenon as the excess of participial clauses found in English, and
Minami's hierarchy of subordinate clauses connects the two languages.

Demonstratives and formal nouns require care, because AI uses **fewer** of them than human
writers do. A finding that something is missing cannot point to a position.

### The metrics that were rejected

The coefficient of variation of sentence length could not be distinguished from that of
translated Japanese. The loanword ratio, the pattern of a sahen noun followed by `を` and `する`, and comma
density all overlapped the human range.

Participial clauses and nominalization in English also overlapped the human range.
The effect size of 5.3 reported in earlier research did not reappear in this task.
The likely cause is a difference in experimental design.
The earlier research gave the model a human-written passage and asked it to continue,
whereas this work asked the model to write a document.
**An effect size found in earlier research must not be applied to a different task.**

### The formatting experiment

When the model was instructed to write only prose, without lists, bold, or headings, the
formatting disappeared entirely.
Bold, lists, headings, and the em dash all dropped to zero.

Vocabulary and sentence structure, however, did not change.
When Claude's polite-register output was compared with polite-register output under the
formatting instruction, MATTR went from 0.642 to 0.650 and renyo went from 5.31 to 4.83.
Both differences fall within the margin of error.

This result determines how the work is divided among the layers.
An instruction can remove the formatting layer and nothing more.
The vocabulary layer and the sentence-structure layer can be controlled only by a linter.

### Differences between models

Formatting depends largely on the habits of each model.
The em dash occurred 10.32 times per thousand words in Claude's output and not at all in
Gemini's. Bold occurred 95.4 times per thousand words in Gemini's output and not at all in
Grok's Japanese output.

MATTR, the renyo ratio, formal nouns, and demonstratives all shifted in the same direction
across all four models.
**Rules with a threshold are applied only to metrics that behave this consistently.**

### Differences between registers

To see whether the plain and polite registers affect the metrics, explanations written by
Claude and by Grok were compared.

| Metric | Claude plain/polite | Grok plain/polite |
|---|---|---|
| MATTR | 0.652 / 0.657 | 0.624 / 0.620 |
| Renyo | 5.12 / 5.31 | 5.33 / 3.27 |
| Prose ratio | 0.618 / 0.371 | 0.915 / 0.922 |
| Formal nouns | 7.68 / 3.03 | 4.37 / 7.24 |

The direction of the difference does not agree between models.
The register itself has no consistent effect, and the difference is outweighed by the
difference between models.
**Separate thresholds for each register are not needed.**
Register detection is used only to decide whether a rule that depends on the register
applies.

### Differences between kinds of document

The kind of document requested has a large effect. The effect is far larger than that of
the register and is comparable to the gap between AI and human writing.

| Metric | Explanation | Procedure | Rationale | Human native |
|---|---|---|---|---|
| Renyo per 1k | 3.83 | 7.69 | 4.90 | 2.71 |
| Kango ratio | 0.435 | 0.498 | 0.523 | 0.397 |
| Formal nouns per 1k | 4.63 | 8.11 | 4.00 | 11.16 |
| Demonstratives per 1k | 3.25 | 0.22 | 1.27 | 6.47 |
| Prose ratio | 0.526 | 0.345 | 0.507 | 0.847 |

Even in the least favorable case, renyo, demonstratives, MATTR, and the prose ratio stay
outside the human range for every kind of document.

Formal nouns, however, reach 8.11 in procedures, which leaves almost no margin below the
threshold of 8.5.
The kango ratio reaches 0.435 in explanations, which is close to the 0.397 of native human
writing.
These two metrics need either a separate threshold for each kind of document or a lower
severity.

### Effects of generation order

Generating eight documents from a single prompt could have caused the style to drift, so
the values were centered within each condition and correlated with the order of
generation.
Every structural metric showed a correlation close to zero, so the style did not drift.
Only the kango ratio showed a large correlation, at +0.405.
In this design, the order and the topic change together, so the correlation is interpreted
as an effect of the topic, not of the order.
The sensitivity of the kango ratio to the topic agrees with the analysis above.

### Whether maintainability instructions are followed

A condition that included the rules in the prompt was added, and compliance was measured.
Two levels of instruction were used: a short instruction of two sentences and a detailed
instruction that listed each rule.

| | Condition | M1 rate | M5 mixed | M6 per 1k | M7 per 1k |
|---|---|---|---|---|---|
| EN Grok | none | 0.63 | 0.05 | 0.59 | 0.20 |
| EN Grok | short | 0.00 | 0.00 | 0.44 | 0.00 |
| JA Grok | none | 0.06 | 0.14 | 0.44 | 0.15 |
| JA Grok | short | 0.00 | 0.00 | 0.37 | 0.00 |
| JA Grok | detailed | 0.00 | 0.00 | 0.00 | 0.00 |
| EN Claude | none | — | 0.04 | 0.72 | 0.00 |
| EN Claude | short | 0.00 | 0.00 | 0.32 | 0.00 |
| EN Claude | detailed | 0.00 | 0.00 | 0.00 | 0.00 |

The structural rules, that is, the list lead-in, parallel items, and the trailing
enumeration, were followed completely with the short instruction.
The vocabulary rule on time-dependent words was only halved by the short instruction and
reached zero only with the detailed one.

Stating the number of items, numbering headings, and numbering list items by hand rarely
occur even without any instruction.
These rules do not need to be included in the layer that runs before generation.

Without an instruction, Claude never wrote a lead-in for a list.
There were 24 list blocks, and the line above each one was always a heading or another list
item. This is a different failure from a malformed lead-in, namely a missing lead-in, and
it is consistent with a prose ratio of 0.150.

The instruction also had a side effect. The maintainability instruction alone improved the
document metrics.

| Metric | Claude, none | Short | Human native |
|---|---|---|---|
| Prose ratio | 0.150 | 0.702 | 0.847 |
| List ratio | 0.633 | 0.273 | 0.172 |
| Formal nouns per 1k | 2.75 | 8.11 | 11.16 |

Requiring a lead-in added prose to the output, and as a result the prose ratio and the list
ratio moved toward the human range.
Renyo and demonstratives, however, did not improve.

The short instruction also makes the output shorter.
Grok's Japanese output dropped from 13629 characters to 8010, and its English output from
5053 words to 2260. The detailed instruction does not shorten the output.

## What the tool detects

The findings fall into two groups.
Some findings can be judged true or false within a sentence or a line, and they have a
position. Others can be computed only over the whole document, and they have none.
Most findings in the second group report that something is missing, and something missing
has no location to point to.

### The D group, metrics over the whole document

| ID | Target | Language | Threshold | Preventable | Severity | Steadiness |
|---|---|---|---|---|---|---|
| D1 | MATTR is high | ja, en | ja >0.60 / en >0.71 | Partly | warning | High |
| D2 | Prose ratio is low | ja, en | ja <0.75 / en <0.65 | Yes | warning | High |
| D3 | Renyo over te-form | ja | >3.0 | No | warning | High |
| D4 | Renyo density | ja | >4.0 per 1k | No | warning | High |
| D5 | Formal nouns are scarce | ja | <8.5 per 1k | Yes | info | Low |
| D6 | Demonstratives are scarce | ja | <4.5 per 1k | No | warning | High |
| D7 | Kango ratio is high | ja | >0.42 | No | info | Low |
| D8 | List ratio is high | ja | >0.25 | Yes | warning | Medium |
| D9 | Simile markers are scarce | en | <0.8 per 1k | No | warning | Medium |

Each threshold is set between the upper end of the human range and the lower end of the AI
range.

Steadiness describes how little a metric is affected by the kind of document.
D5 and D7 depend heavily on the kind of document and on the topic, and in the least
favorable case they nearly reach the human range.
Both remain at info until human baselines are available for each kind of document.
When those baselines are available, both will be raised to warning.

### The M group, maintainability

| ID | Target | Shape | Standard |
|---|---|---|---|
| M1 | The lead-in is grammatically joined to the items | Binary | Google developer documentation style guide |
| M2 | The prose states how many items there are | Binary | The same guide |
| M3 | A heading has a section number | Binary | The same guide |
| M4 | A list item has a number typed by hand | Binary | The same guide |
| M5 | List items do not share the same form | Binary | The same guide |
| M6 | A word depends on a point in time | Binary | Google timeless documentation |
| M7 | An enumeration ends with "etc." or similar | Binary | The same guide |

The M group is based on maintainability, not on AI style.
The problem it addresses existed before generative AI, so the rules stay valid as models
change.

The M group is the only layer that must never be calibrated to the corpus average.
The AWS S3 user guide is the only corpus that writes list lead-ins correctly.
The Linux kernel has 105 incomplete lead-ins against 16 complete ones.
Calibrating to the human average would make this layer meaningless.

### The N group, standard but absent from AI in practice

Noun-ending sentences, the sahen pattern, three identical sentence endings in a row, and
the metaphor word list belong to this group.
All of them are useful when editing text written by a person, so they are kept, but with a
lower severity.

A word list of metaphors loses its value as soon as it is published.
The words delve, intricate, and realm were singled out early in 2024, and their frequency
fell immediately afterward.
A word list must never be the only means of detection.

### The A group, findings that need a model

This group includes distinguishing a live metaphor from a dead one, judging whether a
metaphor comes with an explanation, judging whether excessive detail harms
maintainability, and checking that terms are used consistently.

The linter only lists suspicious passages. The decision is left to the harness.

## Architecture

### How the work is divided

The work is divided according to who reads the output.

Output for an editor and a human reader comes from textlint directly, and textlint handles
the rules that report a position. Positions matter in that setting, and incremental runs
are fast.

Output for an agent or a CI job comes from the front-end tool `suikou`, which handles the
document metrics, the combined report, and the control of token use.

```
suikou
├─ textlint --format json      Positional rules, M1 to M7 plus three presets
├─ Document analyzer           D1 to D9
├─ Merge and build guidance
└─ Output: Markdown / JSON / MCP
```

The positional rules are distributed as textlint rules, so an editor such as NeoVim can use
them on their own. `suikou` calls the same textlint, so no rule is implemented twice.

### The implementation language

Every metric that passed validation can be implemented in pure Rust, because
lindera-unidic returns every UniDic feature, including the word origin and the inflected
form. D3 uses the inflected form, and D7 uses the word origin.

Python was needed only for participial clauses, nominalization, modifier stacking, and
dependency distance, and all of those metrics were rejected.

The implementation is therefore split into two tiers.

**The validated tier** is written in pure Rust and covers the D group, the M group, and the N group.
It compiles into a single binary, so it can withstand repeated launches from hooks.

**The exploratory tier** is written in Python and covers corpus calibration, the search for
new metrics, and the recalculation of thresholds. It is never part of the production path.

A metric moves from the exploratory tier to the validated tier only after it has proven
useful.

Nominalization and modifier stacking still have value as metrics that show differences
between models. If either is used in production, a resident sidecar process handles it:
Python keeps running and exchanges newline-delimited JSON over standard input and output,
so the model is loaded only once.
This option must be enabled explicitly and is off by default.

### Performance requirements

The tool is expected to run from Claude Code hooks on every Write and every Edit.
In that use, startup cost dominates, so the UniDic dictionary must be read without copying,
or a resident mode must be provided.

In a release build, loading the embedded dictionary took 0.7 milliseconds in our measurement.
This value does not justify building a resident mode to speed up dictionary loading.

## The CLI

| Command | Role |
|---|---|
| `suikou brief [--profile P] [--lang L]` | Print the constraints to include in the prompt |
| `suikou check <path>` | Print positional findings and document metrics together |
| `suikou baseline <glob>` | Calibrate thresholds from a corpus and write a profile |
| `suikou terms <glob>` | Extract a glossary and an allowlist from a set of documents |
| `suikou selftest` | Check that the installed binary is what it claims to be |
| `suikou mcp` | Start as an MCP server |
| `suikou plot <path> --doctype T` | Write the planned structure into `.suikou/plans/` before writing |

The main options of `check` are listed below.

- `--format md|json|text` selects the output format; the default is md.
- `--lang auto|ja|en` selects the language; the default is auto.
- `--profile oss|service|reference|tutorial|<path>` selects the set of thresholds.
- `--budget <tokens>` sets an upper limit on the size of the report.
- `--quiet` returns only the exit code.

### What brief outputs

brief outputs constraints only for the formatting layer and the maintainability layer.
It does not output vocabulary or sentence-structure metrics such as MATTR and the renyo
ratio.

The formatting experiment showed that an instruction removes formatting completely but
leaves vocabulary and structure unchanged.
Adding a constraint that has no effect to a prompt wastes tokens and also risks
overcorrection.

How long a maintainability instruction must be depends on the kind of rule.
The structural rules are stated briefly, and only the time-dependent words are listed one
by one. A rule that is not violated even without an instruction is left out of brief.

| Rule | Treatment in brief |
|---|---|
| M1, the list lead-in | One sentence |
| M5, parallel items | One sentence |
| M7, the trailing enumeration | One sentence |
| M6, time-dependent words | List the forbidden words |
| M2, the item count | Left out |
| M3, numbered headings | Left out |
| M4, hand-written numbers | Left out |

A short instruction makes the output shorter, so the profile stores the level of detail as
a setting, and the caller chooses the level that fits the task.

## Output formats

### JSON

```json
{
  "document": [
    {
      "metric": "ja.renyo_te_ratio",
      "value": 15.94,
      "threshold": 3.0,
      "direction": "above",
      "guidance": "..."
    }
  ],
  "local": [
    {
      "ruleId": "maint/list-lead-in",
      "severity": "error",
      "message": "...",
      "positions": [{ "line": 12, "column": 1, "text": "..." }],
      "truncated": 0
    }
  ]
}
```

### Markdown

```
## Guidance

(Built from the metrics that broke their threshold. Three to five lines. No raw numbers.)

## What to fix

### The lead-in binds to the items — maint/list-lead-in (6)

- L12 "Use the reload command to:"
- L45 ...
```

### How token use is kept low

Keeping token use low is the most important constraint, so the output follows the rules
below.

- A rule's explanation is printed once for that rule, not once for each line.
- No more than twenty positions are printed for a rule, and the remainder is reported as a count.
- A metric that stays within its threshold is not printed.
- When `--budget` is set, the report is shortened in order of severity.

### Turning a metric into guidance

A list of numbers does not tell a model what to do, so each metric is converted into an
instruction. For example, a high kango ratio becomes an instruction to rewrite kango as
native verbs.

| Metric | Instruction produced |
|---|---|
| D1, MATTR is high | Refer to the same thing by the same name. Stop varying the wording |
| D2, prose ratio is low | Turn the bullet lists back into prose |
| D3 and D4, renyo is heavy | Split the clauses or change them to the te-form |
| D5, formal nouns are scarce | Rewrite nominalized kango in native phrasing |
| D6, demonstratives are scarce | Link paragraphs with demonstratives instead of repeating nouns |
| D7, kango ratio is high | Use the established loanword; do not string kango nouns together |
| D8, lists are heavy | Turn the lists back into prose |
| D9, simile markers are scarce | Mark a comparison explicitly as a simile or an example |

### Severity

| Severity | Applies to | Reasoning |
|---|---|---|
| error | The maintainability layer, M1 to M7 | The violation is clear and not open to debate |
| warning | A D metric beyond its threshold | The threshold has a range and depends on context |
| info | Model-dependent metrics, the word list, exploratory metrics | Advisory only, and some of it goes out of date |

Maintainability is weighted more heavily than AI style, because AI style is a matter of
thresholds, while maintainability is a matter of violating a published standard.

The conditions under which a hook rejects an edit can be configured.

## The rules

### Detecting the register

The plain and polite registers end sentences in different ways, so some rules have to be
switched depending on the register.

The register is detected for each block, not once for the whole input.
A single document may correctly use the polite register in its prose and the plain register
in its list items, so one switch for the whole input would be too coarse.

Thresholds are not separated by register.
A comparison of the two registers found that the direction of the difference did not agree
between models and that there was no consistent effect.
Register detection is used only to decide whether a rule that depends on the register
applies.

The register is detected from the final morpheme of each sentence.
The setting `style: auto | desumasu | dearu` overrides the detection.

The rules that depend on the register are listed below.

- A sentence ending in a noun is forbidden in prose and allowed in a list item.
- The word list of verbose sentence endings is different for each register.

### The list lead-in, M1

In Japanese, the correct forms are a complete sentence ending with a full stop, and a noun
followed by a colon.
A lead-in that ends with a renyo form, a particle, or a conjunctive particle is
grammatically joined to the items, so it breaks when an item is added or removed.

In English, the check is whether the lead-in contains the following, as follows, these,
below, or steps.
A strict check would look for a finite verb, but even this word-based check alone
distinguishes the AWS S3 user guide, at 30 against 145, from the Linux kernel, at 105
against 16.

### The metaphor layer

A term established in a field is not treated as a metaphor.
Cache, tree, and handshake are terms, and the rules do not report them.

The problem is an unfamiliar metaphor used without an explanation. For example, a sentence
saying that a queue behaves like a conveyor belt marks the comparison, whereas a sentence
calling the queue a conveyor belt does not.
Among the 385 words collected by Sakasegawa, the patterns built around entrance, foundation,
core, and boundary belong to this category, as do the patterns built around carve, crush,
step in, and dissolve. The 291 words from Kobak also belong to it, as do tapestry and amidst
from Reinhart.

A simile or a worked example is allowed on one condition.
The condition comes from the TYPO3 guide: a plain explanation must appear next to the
figure of speech.

The allowlist is not shipped with the tool. `suikou terms` builds it from the documents in
use. Each project uses different terms, and building the list locally also addresses the
problem of word lists going out of date.

## The harness

### Keeping the thinking separate from the deliverable

The safeguard against excessive detail is a convention for the workspace.

```
docs/guide.md              The deliverable, which the linter covers
.suikou/notes/guide.md     The thinking, which the linter leaves alone
```

The skill instructs the model as follows.
Options that were considered, options that were rejected, values the model is unsure of,
reasoning, and the sources of measurements all go into the notes.
Only decided content goes into the deliverable.

This separation has more than one effect.
Giving the urge to show reasoning a proper place keeps excessive detail out of the
deliverable.
Keeping the reasoning next to the document also lets a later reader find out why a passage
was written the way it was.

The core of this design is turning a question about what to write into a question about
where to put it.

### The workflow

1. Put the output of `suikou brief` into the prompt.
2. Generate the document, producing a deliverable and a set of notes.
3. Run `suikou check` once and read the combined report.
4. Make every edit in a single pass.
5. Confirm the result with `suikou check --quiet`.

No layer is linted and fixed on its own.
The positional findings show where to edit, and the guidance built from the document
metrics shows how to edit.

### The glossary

`suikou terms` extracts the terminology of a field from the documents in use and writes it
to `.suikou/terms.toml`.

A term in that file is treated as established and is excluded from the metaphor check.
Only metaphorical words that are not in the file are reported.
The same mechanism also checks that terms are used consistently.

## Repository layout

```
suikou/
├─ crates/suikou-core      The analyzer, on lindera-unidic and pulldown-cmark
├─ crates/suikou-cli       The CLI and the MCP server
├─ packages/               npm, holding the positional textlint rules
├─ skills/suikou/          SKILL.md
├─ docs/                   The Hugo documentation site
├─ research/               The exploratory tier in Python
└─ corpus/                 The measurements and the fetch script
```

Everything is kept in one repository so that the output format of the CLI and what the
skill expects cannot fall out of step.
Both change in the same commit, and golden file tests fix them in place.

The canonical copy of the skill is kept in this repository, and a dotfiles setup imports
it. If the actual copy lived in dotfiles, it could drift away from the version of the CLI.

### How the packages are divided

The npm packages are divided by purpose.

- `textlint-rule-preset-tech-maintainability` contains M1 to M7 for both languages, and
  because it has value independent of AI style, it can be used on its own.
- `textlint-rule-preset-ja-ai-structure` contains the Japanese rules that work at the
  sentence level.

## Relationship to other open source projects

| Package | Treatment | Reason |
|---|---|---|
| textlint-rule-preset-ja-technical-writing | Use alongside | Covers spelling and readability, and does not overlap with suikou |
| @textlint-ja/textlint-rule-preset-ai-writing | Use in part | Reuse its rules on exaggeration, mechanical bold, and list formatting |
| p1ass/textlint-rule-preset-ai-words-ja | Use alongside | Serves as the word-list layer; expected to go out of date, so never the only means of detection |
| D1 to D9, M1 to M7 | Implemented here | No equivalent exists |

`no-ai-colon-continuation` from the ai-writing preset is close to M1.
The former forbids a predicate before a colon, while the latter checks whether the lead-in
is grammatically joined to the items.
Their purposes differ, so both can be used together, but they may report the same line
twice. The formatter merges duplicate findings on the same line.

The other presets were checked against these layers and found not to overlap.
`textlint-rule-preset-ja-technical-writing` limits sentences to 100 characters, and AI
prose averages 39.2 characters per sentence, so the limit is never reached.
Its limit on commas is three, and AI prose averages 0.95 commas per sentence.

## The development plan

### The order of implementation

The maintainability layer is implemented first.
Its rules are binary, have no threshold, cite a named standard, and have value independent
of AI style.
This layer alone yields a useful tool.
A later experiment showed that instructions covering this layer are followed almost
completely, and that following them also improves the document metrics, so this layer
offers the best return on the effort.

The stable Japanese document metrics come next: D1, D2, D3, D4, and D6.
The ratio of renyo to te-form separates human and AI text most clearly, and the inflected
form provided by lindera-unidic is enough to compute it.

D5 and D7 are implemented next, at info severity.
Their thresholds will be reviewed once human baselines are available for each kind of
document.

The English side and MCP come last. A resident mode will not be built; D-25 records the
reason.

`jametrics.py`, `enmetrics.py`, and `mcheck.py` in the exploratory tier are the reference
implementation for every metric.
Golden tests confirm that the Rust implementation returns the same values as these scripts.

### Evaluation

The false positive rate is measured on the human corpus.
The targets are the Linux kernel, gcc, the Rust Book, Django, AWS, the Japanese Kubernetes
site, and the Japanese Vue site, and the number of findings reported as error is counted.

The M group is expected to produce many findings even on the human corpus.
Incomplete list lead-ins are common in the Linux kernel.
These are not false positives but real opportunities for improvement.
If the number of findings turns out to be too large to be useful, the default severity may
need to be adjusted.

The D group should produce no findings on the human corpus. Any finding it produces there
is treated as a false positive.

### Expanding the human corpus

The native Japanese corpus is dominated by expository books.
Since the kind of document has a large effect on the metrics, native Japanese procedures
need to be collected.
Whether D5 and D7 can be raised to warning depends on that collection.

## Open questions

The questions below will be decided after the code has been written and evaluated.

- Decide whether the skill calls `suikou brief` every time or embeds its output in SKILL.md.
- Decide how the level of detail of brief is represented as a profile setting.
- Decide how finely thresholds are divided by kind of document.
- Decide how to add the Japanese AWS and Google documentation to the corpus.
- Expand the native Japanese corpus.

The level of detail of brief requires careful judgment, because a short instruction makes
the output shorter. Deciding how to divide thresholds also involves
deciding how the kind of document is detected. The Japanese AWS and Google documentation
is not published on GitHub, so another way to obtain it is needed. The Japanese corpus has
two sources in this environment, and both are expository books.

### Questions already settled

Measurement has settled the questions below, so none of them needs to be revisited.

- Thresholds are not divided by register.
- Data collected from a single prompt that generated several documents is usable.
- Structural maintainability rules are followed with a short instruction.
- Document metrics are not placed inside textlint.
- A resident mode is not built, because startup was measured in milliseconds.
- Rust is the implementation language.

Unlike the structural rules, the vocabulary rules require a detailed instruction.
The front-end tool is responsible for the document metrics.
Python is limited to calibration and exploration.

## Key references

### Quantitative work on AI prose

- Kobak et al., *Science Advances* 2025.
- Reinhart et al., *PNAS* 122(8) 2025.
- Markey et al., *Written Communication* 41(4) 2024.
- Geng and Trotta, *Findings of ACL* 2025.
- Zhang et al., *ACL* 2025.
- Freeburg, arXiv:2603.27006 2026.
- Sakasegawa, 2026.

Kobak and colleagues analyzed 15.1 million PubMed abstracts. They found that 66 percent of
the words that appeared in excess in 2024 were verbs and 14 percent were adjectives, the
opposite of the COVID period, when 79 percent of the excess words were nouns. Reinhart and
colleagues compared texts using Biber's 66 features and showed that instruction tuning
causes the divergence from human writing, while base models stay closer to it. Markey and
colleagues described the style of ChatGPT as dense and disconnected.

Geng and Trotta showed that a word becomes less frequent immediately after it has been
singled out. Zhang and colleagues showed that the preference for lists and bold is built
into the reward model. Freeburg showed that formatting can be suppressed and that the em
dash persists in some models. Sakasegawa compared 70,000 Qiita articles from before and
after generative AI, which makes it the only quantitative study of this kind in Japanese.

### Published standards

- The Google developer documentation style guide.
- The Microsoft Writing Style Guide.
- The TYPO3 Community Language and Writing Guide.
- The JTCA Japanese Style Guide, third edition.

From the Google guide, the sections on lists, headings, timeless documentation, and
inclusive documentation are used. From the Microsoft guide, the section on global
communications is used. The TYPO3 guide provides the basis for allowing a figure of speech
under a condition.

### Linguistics

- The hierarchy of subordinate clauses described by Minami.
- The four-way classification of kango bases described by Nomura.
- Ishiguro, *Bunsho wa Setsuzokushi de Kimaru*.
- Honda, *Nihongo no Sakubun Gijutsu*.
- Meldrum 2009.

Takubo and Yoshimoto refined Minami's hierarchy into four levels.
In Nomura's classification, class C is the class that makes the pattern with `を` and
`する` possible. Ishiguro describes four kinds and ten classes of conjunctions.
Honda describes principles for the order of modifiers and for the placement of commas.
In Meldrum's study, the pronoun was the only one of the four proposed marks of
translationese that was confirmed.
