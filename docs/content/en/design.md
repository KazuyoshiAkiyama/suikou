---
title: "Design"
weight: 10
doctype: "design"
---

A layered linter and a harness that strip AI style out of technical documentation and
leave the result easier to maintain. The languages covered are Japanese and English.

This page follows the rules it sets out. Every list opens with a complete sentence, the
prose never states how many items a list holds, and no bare metaphor appears.

## Motivation

Technical documentation written by generative AI carries habits that reach past the level
of the single word. It leans on metaphor and on indirect phrasing, it repeats the same
sentence endings, it joins sentences awkwardly, and it spells out so much detail that the
document becomes hard to maintain. Existing tools cover a word list or a notation rule
and nothing else, so none of them reach these habits. A spell checker, for example,
never sees that a document leans on metaphor.

## Scope

The scope is technical documentation and technical writing.
What counts as good prose depends on the setting, so no rule can be fixed until the
setting is fixed.

The languages covered are Japanese and English.
The style rules carry a measurement per language; the structural rules apply to both.

## Non-goals

Fiction, advertising, and social posts fall outside the scope, because the style rules
change with the setting.

The tool does not rewrite a document. It reports positions and it states a direction
drawn from the whole document, and the writer decides what to change. Mechanical
substitution has broken the meaning of a sentence more than once in this repository's own
documents.

The tool does not judge whether the content is factually correct. Style and structure can
be measured against a corpus; the truth of a claim cannot.

## The heaviest constraint

The tool does not report one layer at a time and ask the model to fix each report in turn.
To hold down token use, one run reports every layer, and one edit clears the report.

## The measurements behind the design

Every rule rests on a measurement or on a written standard.
Not one rule came from a hunch.
That policy exists because a hunch about style turns out to be wrong most of the time
once someone measures it.
Research on translationese and research on AI prose found the same thing separately.

### The human corpus

For Japanese, the corpus holds both native writing and translation.
The native writing comes from two books by kaityo256 and from the future-architect coding
standards. The translations come from the Japanese Kubernetes site, the Japanese Vue
site, and the ja_JP tree of the Linux kernel.

For English, the corpus holds the Linux kernel documentation, gcc, the Rust Book, Django,
Google eng-practices, the AWS S3 user guide, and the AWS CloudFormation user guide.

### The AI corpus

The AI corpus holds 232 pieces from four models, meaning Claude, Gemini, ChatGPT, and
Grok. The conditions crossed two languages, three prompts covering explanation,
procedure, and design rationale, three Japanese registers, and the presence or absence of
an instruction to suppress formatting.
Each condition started a fresh chat with memory and custom instructions turned off.

### The metrics that survived

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

The ratio of renyo to te-form separates the groups more sharply than anything else.
The te-form leans toward speech and folds clauses into one.
The renyo form leans toward writing and lines clauses up side by side.
AI leans toward the second.
That is the same thing as the excess of participial clauses found in English, and the
hierarchy of subordinate clauses described by Minami links the two languages.

Demonstratives and formal nouns need care, because AI uses **fewer** of them than a human
does. A finding about something missing cannot point at a position.

### The metrics that were rejected

The coefficient of variation of sentence length could not be told apart from translated
Japanese. The loanword ratio, the pattern of a sahen noun with `を` and `する`, and comma
density all overlapped the human range.

Participial clauses and nominalization in English overlapped the human range as well.
The effect size of 5.3 reported in earlier work did not carry over to this task.
The cause appears to be the design of the experiment.
The earlier work handed a human passage over and asked for a continuation, while this
work asked for a document to be written.
**An effect size from earlier work must not be carried across tasks.**

### The formatting experiment

Under the instruction to write in prose alone, without lists, bold, or headings, the
formatting disappeared completely.
Bold, lists, headings, and the em dash all fell to zero.

Vocabulary and sentence structure, on the other hand, did not move.
Comparing the polite register from Claude against the same register with formatting
suppressed, MATTR went from 0.642 to 0.650 and renyo went from 5.31 to 4.83.
Both differences sit inside the margin of error.

That result fixes how the layers divide the work.
An instruction can remove the formatting layer and nothing else.
The vocabulary layer and the sentence-structure layer can only be held down by a linter.

### Differences between models

Formatting is dominated by the habits of the individual model.
The em dash ran at 10.32 per thousand words for Claude and at zero for Gemini.
Bold ran at 95.4 per thousand words for Gemini and at zero for Grok in Japanese.

MATTR, the renyo ratio, formal nouns, and demonstratives all moved the same direction
across all four models.
**A rule that carries a threshold belongs only on a metric that behaves this way.**

### Differences between registers

Whether the plain and polite registers move the metrics was tested on the explanation
prompt with Claude and with Grok.

| Metric | Claude plain/polite | Grok plain/polite |
|---|---|---|
| MATTR | 0.652 / 0.657 | 0.624 / 0.620 |
| Renyo | 5.12 / 5.31 | 5.33 / 3.27 |
| Prose ratio | 0.618 / 0.371 | 0.915 / 0.922 |
| Formal nouns | 7.68 / 3.03 | 4.37 / 7.24 |

The direction does not agree across models.
No systematic effect from the register itself appears, and the difference is absorbed by
the difference between models.
**Thresholds do not need to split by register.**
Register detection is used only to decide whether a register-dependent rule applies.

### Differences between kinds of document

The kind of document requested matters a great deal. It matters far more than the
register, and it rivals the gap between AI and human writing.

| Metric | Explanation | Procedure | Rationale | Human native |
|---|---|---|---|---|
| Renyo per 1k | 3.83 | 7.69 | 4.90 | 2.71 |
| Kango ratio | 0.435 | 0.498 | 0.523 | 0.397 |
| Formal nouns per 1k | 4.63 | 8.11 | 4.00 | 11.16 |
| Demonstratives per 1k | 3.25 | 0.22 | 1.27 | 6.47 |
| Prose ratio | 0.526 | 0.345 | 0.507 | 0.847 |

Measured against the worst case, renyo, demonstratives, MATTR, and the prose ratio stay
clear of the human range for every kind of document.

Formal nouns, on the other hand, reach 8.11 in procedures, which leaves almost no room
against the threshold of 8.5.
The kango ratio reaches 0.435 in explanations, which sits close to the 0.397 of native
human writing.
These two need either a threshold per kind of document or a lower severity.

### Effects of generation order

Generating eight pieces from one prompt could have introduced drift, so the values were
centered within each condition and correlated against rank.
Every structural metric came out near zero, so no drift in style occurred.
The kango ratio alone correlated at +0.405, which is large.
In this design the rank and the subject move together, so that correlation reads as an
effect of subject rather than of order.
Kango sensitivity to subject agrees with the analysis above.

### Whether maintainability instructions are followed

A condition carrying the rules in the prompt was added, and compliance was measured.
Two levels were used, meaning a short instruction of two sentences and a detailed
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

The structural rules, meaning the list lead-in, parallel items, and the trailing
enumeration, were followed completely under the short instruction.
The vocabulary rule, meaning words that depend on a moment in time, only halved under the
short instruction and reached zero only under the detailed one.

Stating the number of items, numbering headings, and hand-numbering list items barely
occur even with no instruction at all.
None of those three need a place in the layer that runs before generation.

Claude never wrote a list lead-in with no instruction given.
There are 24 list blocks, and the line above each one is always a heading or another list
item. That is a different failure from a malformed lead-in, namely the absence of one,
and it agrees with a prose ratio of 0.150.

The instruction carries a second effect. The maintainability instruction alone improved
the document metrics.

| Metric | Claude, none | Short | Human native |
|---|---|---|---|
| Prose ratio | 0.150 | 0.702 | 0.847 |
| List ratio | 0.633 | 0.273 | 0.172 |
| Formal nouns per 1k | 2.75 | 8.11 | 11.16 |

Asking for a lead-in produced prose, and the prose ratio and the list ratio moved toward
the human range as a result.
Renyo and demonstratives, on the other hand, did not improve.

The short instruction also shrinks the output.
Grok in Japanese fell from 13629 characters to 8010, and Grok in English fell from 5053
words to 2260. The detailed instruction does not cause that shrinkage.

## What the tool finds

What the tool finds falls into two groups.
Some findings are true or false at the level of a sentence or a line and carry a position.
Others can only be computed across the whole document and carry none.
Most of the second group report something missing, and something missing has nowhere to
point.

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

Each threshold sits between the top of the human range and the bottom of the AI range.

Steadiness describes how well a metric holds up as the kind of document changes.
D5 and D7 are sensitive to the kind of document and to the subject, and in the worst case
they nearly touch the human range.
Until human baselines exist for each kind of document, both stay at info.
Once those baselines exist, both move up to warning.

### The M group, maintainability

| ID | Target | Shape | Standard |
|---|---|---|---|
| M1 | The lead-in binds grammatically to the items | Binary | Google developer documentation style guide |
| M2 | The prose states how many items there are | Binary | The same guide |
| M3 | A heading carries a section number | Binary | The same guide |
| M4 | A list item carries a hand-written number | Binary | The same guide |
| M5 | List items do not share a shape | Binary | The same guide |
| M6 | A word depends on a moment in time | Binary | Google timeless documentation |
| M7 | An enumeration trails off | Binary | The same guide |

The M group rests on maintainability rather than on AI style.
The problem predates generative AI, so it does not age as models change.

The M group is the one layer that must never be matched to the corpus average.
The AWS S3 user guide is the only corpus that writes list lead-ins correctly.
The Linux kernel carries 105 incomplete lead-ins against 16 complete ones.
Matching the human average would empty this layer of meaning.

### The N group, standard but absent from AI in practice

Noun-stopped sentences, the sahen pattern, three identical sentence endings in a row, and
the metaphor word list belong here.
All of them help when a human piece is being edited, so they stay, but at a lower
severity.

A metaphor word list ages the moment it is published.
The words delve, intricate, and realm were called out early in 2024, and their frequency
dropped right after.
A word list must never be the only line of defense.

### The A group, findings that need a model

Telling a live metaphor from a dead one, judging whether a metaphor carries an
explanation, judging whether excessive detail hurts maintainability, and checking
terminology all belong here.

The linter lists the suspect passages and stops there. The judgment belongs to the
harness.

## Architecture

### How the work divides

The work divides by who reads the output.

An editor and a human read textlint directly, and textlint covers the rules that carry a
position. Position matters there, and an incremental run is fast.

An agent and a CI job read the front-end tool `suikou`, which covers the document
metrics, the combined report, and the control of token use.

```
suikou
├─ textlint --format json      Positional rules, M1 to M7 plus three presets
├─ Document analyzer           D1 to D9
├─ Merge and build guidance
└─ Output: Markdown / JSON / MCP
```

The positional rules ship as textlint rules, so an editor such as NeoVim can use them
alone. `suikou` calls that same textlint, so nothing is implemented twice.

### The implementation language

Every metric that survived validation can be written in pure Rust, because
lindera-unidic returns every UniDic feature, including the word origin and the inflected
form. D3 reads the inflected form and D7 reads the word origin.

Python was needed for participial clauses, nominalization, modifier stacking, and
dependency distance, and every one of those was rejected.

The work therefore splits into two tiers.

**The validated tier** is pure Rust and covers the D group, the M group, and the N group.
It compiles to one binary, so it stands up to repeated launches from hooks.

**The exploratory tier** is Python and covers corpus calibration, the search for new
metrics, and the recalculation of thresholds. It never enters the production path.

A metric moves from the exploratory tier to the validated tier only after it proves out.

Nominalization and modifier stacking still hold value as model-dependent metrics.
Should either one enter production, a resident sidecar handles it: Python stays running
behind newline-delimited JSON on standard input and output, so the model loads once.
That path is opt-in and stays off by default.

### Performance

The expected use runs the tool from Claude Code hooks on every Write and every Edit.
Launch cost dominates that use, so the UniDic dictionary is read without copying, or a
resident mode is offered.

Measured in a release build, loading the embedded dictionary takes 0.7 milliseconds.
That figure does not support building a resident mode for the sake of dictionary loading.

## The CLI

| Command | Role |
|---|---|
| `suikou brief [--profile P] [--lang L]` | Print the constraints to put in the prompt |
| `suikou check <path>` | Print positional findings and document metrics together |
| `suikou baseline <glob>` | Calibrate thresholds from a corpus and write a profile |
| `suikou terms <glob>` | Extract a glossary and an allowlist from a set of documents |
| `suikou selftest` | Check that the installed binary is what it claims to be |
| `suikou mcp` | Start as an MCP server |
| `suikou plot <path> --doctype T` | Write the structure into `.suikou/plans/` before writing |

The main options on `check` are listed below.

- `--format md|json|text` picks the output shape, and md is the default.
- `--lang auto|ja|en` picks the language, and auto is the default.
- `--profile oss|service|reference|tutorial|<path>` picks the set of thresholds.
- `--budget <tokens>` caps the size of the report.
- `--quiet` returns only the exit code.

### What brief emits

Only the formatting layer and the maintainability layer are emitted.
Vocabulary and sentence-structure metrics such as MATTR and the renyo ratio are not.

The formatting experiment showed that a prompt removes formatting completely and leaves
vocabulary and structure untouched.
Putting a constraint that does not work into a prompt wastes tokens, and it risks
over-correction as well.

A maintainability instruction needs a different length for each kind of rule.
The structural rules are stated briefly, and only the time-dependent words are listed out.
A rule that never fires without an instruction stays out of brief.

| Rule | Treatment in brief |
|---|---|
| M1, the list lead-in | One sentence |
| M5, parallel items | One sentence |
| M7, the trailing enumeration | One sentence |
| M6, time-dependent words | List the forbidden words |
| M2, the item count | Left out |
| M3, numbered headings | Left out |
| M4, hand-written numbers | Left out |

A short instruction shrinks the output, so the profile carries the level of detail as a
setting and the caller picks what suits the job.

## The output shape

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

### Holding down token use

Holding down token use is the heaviest constraint, so the output follows the rules below.

- The explanation for a rule is printed once per rule and never repeated per line.
- At most twenty positions are printed per rule, and the rest is reported as a count.
- A metric inside its threshold is not printed at all.
- With `--budget` set, the report is trimmed in order of severity.

### Turning a metric into guidance

A list of numbers does not tell a model what to do, so each metric turns into an
instruction. For instance, a high kango ratio turns into an instruction to open kango
into native verbs.

| Metric | Instruction produced |
|---|---|
| D1, MATTR is high | Call the same thing by the same name. Stop varying the wording |
| D2, prose ratio is low | Fold the bullet lists back into prose |
| D3 and D4, renyo is heavy | Split the clauses or switch to the te-form |
| D5, formal nouns are scarce | Unpack nominalized kango into native phrasing |
| D6, demonstratives are scarce | Link paragraphs with demonstratives instead of repeating nouns |
| D7, kango ratio is high | Reach for the established loanword; stop stacking kango nouns |
| D8, lists are heavy | Fold the lists back into prose |
| D9, simile markers are scarce | Mark a comparison as a simile or as an example |

### Severity

| Severity | Applies to | Reasoning |
|---|---|---|
| error | The maintainability layer, M1 to M7 | The breach is plain and not open to argument |
| warning | A D metric past its threshold | The threshold has width and depends on context |
| info | Model-dependent metrics, the word list, exploratory metrics | Advisory, and some of it ages |

Maintainability outranks AI style, because the first is a matter of thresholds while the
second is a breach of a written standard.

What causes a hook to reject an edit is configurable.

## The rules

### Detecting the register

The plain and polite registers end sentences differently, so some rules have to switch.

The switch happens per block rather than at the entry point.
One document may legitimately carry polite prose and plain list items, so a single switch
at the entry point would be too coarse.

Thresholds do not split by register.
Comparing the two registers showed no agreement in direction across models and no
systematic effect.
Register detection is used only to decide whether a register-dependent rule applies.

Detection reads the final morpheme of the sentence.
A setting of `style: auto | desumasu | dearu` overrides it.

The register-dependent rules are listed below.

- A noun-stopped sentence is forbidden in prose and allowed in a list item.
- The word list for padded sentence endings differs between the two registers.

### The list lead-in, M1

In Japanese the correct shapes are a complete sentence ending in a full stop, and a noun
followed by a colon.
A lead-in ending in a renyo form, a particle, or a conjunctive particle binds
grammatically to the items, so adding or removing an item breaks it.

In English the test asks whether the lead-in contains the following, as follows, these,
below, or steps.
A strict test would look for a finite verb, and even so this word test alone separates
the AWS S3 user guide, at 30 against 145, from the Linux kernel, at 105 against 16.

### The metaphor layer

A term established in a field is not a metaphor.
Cache, tree, and handshake are terms, and the rules leave them alone.

The problem is an unfamiliar metaphor used with no explanation beside it. A sentence that
says a queue behaves like a conveyor belt marks the comparison, and a sentence that calls
the queue a conveyor belt does not.
Among the 385 words collected by Sakasegawa, the patterns around entrance, foundation,
core, and boundary belong here, and so do the patterns around carve, crush, step in, and
dissolve. The 291 words from Kobak belong here, and so do tapestry and amidst from
Reinhart.

A simile and a worked example are allowed on one condition.
That condition comes from the TYPO3 guide, namely that a plain explanation sits beside
the figure.

The allowlist does not ship with the tool. `suikou terms` builds it from the documents at
hand. Terms differ between projects, and building the list locally answers the problem of
a word list that ages.

## The harness

### Separating thinking from the deliverable

The guard against excessive detail takes the form of a workspace convention.

```
docs/guide.md              The deliverable, which the linter covers
.suikou/notes/guide.md     The thinking, which the linter leaves alone
```

The skill instructs the model as follows.
Options considered, options rejected, values held without confidence, reasoning, and the
source of a measurement all go into the notes.
Only settled content goes into the deliverable.

The effect runs in more than one direction.
Giving the urge to show reasoning a legitimate destination pulls excessive detail out of
the deliverable.
Keeping the reasoning beside the document also lets a later reader trace why a passage
reads the way it does.

Turning a question about writing into a question about placement is the core of this
design.

### The workflow

1. Put the output of `suikou brief` into the prompt. 2. Generate, producing a deliverable
and a set of notes. 3.

Run `suikou check` once and read the combined report. 4. Apply every edit in one pass. 5.
Confirm with `suikou check --quiet`.

No layer gets linted and fixed on its own.
The positional findings say where to edit, and the guidance built from the document
metrics says how to edit.

### The glossary

`suikou terms` extracts field terminology from the documents at hand and writes
`.suikou/terms.toml`.

A term in that file counts as established and leaves the metaphor check.
Only the metaphorical vocabulary outside the file is reported.
The same machinery checks terminology for consistency.

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

Everything sits in one repository so that the output shape of the CLI and the
expectations of the skill cannot drift apart.
They move in one commit, and golden file tests pin them.

The canonical copy of the skill lives here, and a dotfiles setup pulls it in.
Keeping the real copy in dotfiles would let it drift away from the version of the CLI.

### How the packages divide

The npm side divides by purpose.

- `textlint-rule-preset-tech-maintainability` holds M1 to M7 for both languages, and it
  carries value apart from AI style, so it stands alone.
- `textlint-rule-preset-ja-ai-structure` holds the Japanese sentence-level rules.

## Relation to other open source

| Package | Treatment | Reason |
|---|---|---|
| textlint-rule-preset-ja-technical-writing | Use alongside | Covers notation and readability, and does not overlap |
| @textlint-ja/textlint-rule-preset-ai-writing | Use parts | Reuse the overstatement, mechanical bold, and list format rules |
| p1ass/textlint-rule-preset-ai-words-ja | Use alongside | A word list, treated as something that ages, never the only defense |
| D1 to D9, M1 to M7 | Written here | Nothing equivalent exists |

`no-ai-colon-continuation` from the ai-writing preset sits close to M1.
The first forbids a predicate before the colon, and the second asks whether the lead-in
binds to the items.
The purposes differ, so the two coexist, but they can report the same line twice.
The formatter merges duplicates on one line.

The other presets were checked against these layers and do not overlap.
`textlint-rule-preset-ja-technical-writing` caps sentence length at 100 characters, and AI
prose averages 39.2 characters, so nothing trips.
It caps commas at three, and AI prose averages 0.95 per sentence.

## The plan

### The order of work

The maintainability layer comes first.
Its rules are binary, they carry no threshold, their standard is named, and they hold
value apart from AI style.
That layer alone produces something useful.
A later experiment showed that an instruction covering this layer is followed almost
completely, and that following it improves the document metrics as well, which makes it
the best return on the effort.

The steady Japanese document metrics come next, meaning D1, D2, D3, D4, and D6.
The ratio of renyo to te-form separates most sharply, and the inflected form from
lindera-unidic is enough to compute it.

D5 and D7 come next at info severity.
Their thresholds get revisited once human baselines exist per kind of document.

The English side, MCP, and resident mode come last.

`jametrics.py`, `enmetrics.py`, and `mcheck.py` in the exploratory tier are the reference
implementation for every metric.
Golden tests confirm that the Rust implementation returns the same values.

### Evaluation

The false positive rate gets measured against the human corpus.
The Linux kernel, gcc, the Rust Book, Django, AWS, the Japanese Kubernetes site, and the
Japanese Vue site are the targets, and the count reported as error is what gets counted.

The M group should fire heavily even on the human corpus.
Incomplete list lead-ins dominate the Linux kernel.
Those are not false positives. They are real room for improvement.
Should the count prove too large to work with, the default severity may need adjusting.

The D group should stay silent on the human corpus. Anything it reports there is a false
positive.

### Thickening the human corpus

The Japanese native corpus leans toward expository books.
Given how much the kind of document moves the metrics, native Japanese procedures have to
be collected.
Whether D5 and D7 can move up to warning depends on that.

## Open questions

These get settled after the code is written and evaluated.

- Decide whether the skill calls `suikou brief` every time or embeds the output in SKILL.md.
- Decide how the level of detail in brief is expressed as a profile setting.
- Decide how finely thresholds split by kind of document.
- Decide how to add the Japanese AWS and Google documentation to the corpus.
- Thicken the Japanese native corpus.

The level of detail in brief needs a judgment call, because a short instruction shrinks
the output. Splitting thresholds includes
deciding how the kind of document is detected. The Japanese AWS and Google documentation
is not on GitHub, so adding it needs another route. The Japanese corpus draws on two
sources today, and both are expository books.

### Questions already settled

Measurement settled the items below, so none of them needs revisiting.

- Thresholds do not split by register.
- Data collected from one batched prompt is usable.
- Structural maintainability rules hold under a short instruction.
- Document metrics do not belong inside textlint.
- Resident mode is not built. Startup was measured in milliseconds.
- Rust is the implementation language.

Vocabulary rules, unlike the structural ones, need a detailed instruction.
The front-end tool owns the document metrics.
Python is limited to calibration and search.

## Key references

### Quantitative work on AI prose

- Kobak et al., *Science Advances* 2025.
- Reinhart et al., *PNAS* 122(8) 2025.
- Markey et al., *Written Communication* 41(4) 2024.
- Geng and Trotta, *Findings of ACL* 2025.
- Zhang et al., *ACL* 2025.
- Freeburg, arXiv:2603.27006 2026.
- Sakasegawa, 2026.

Kobak and colleagues examined 15.1 million PubMed abstracts and found that 66 percent of the
excess words of 2024 were verbs and 14 percent adjectives, which inverts the pattern of the
COVID period, where 79 percent were nouns. Reinhart and colleagues compared texts across the
66 features of Biber and showed that instruction tuning drives the divergence, leaving base
models closer to human writing. Markey and colleagues described the ChatGPT register as
dense and disconnected.

Geng and Trotta showed that a word drops in frequency right after it is called out. Zhang
and colleagues showed that the preference for lists and bold is baked into the reward model.
Freeburg showed that formatting can be suppressed and that the em dash survives in some
models. Sakasegawa compared 70,000 Qiita articles before and after generative AI, which is
the only quantitative study of its kind in Japanese.

### Written standards

- The Google developer documentation style guide.
- The Microsoft Writing Style Guide.
- The TYPO3 Community Language and Writing Guide.
- The JTCA Japanese Style Guide, third edition.

The Google guide contributes its sections on lists, headings, timeless documentation, and
inclusive documentation. The Microsoft guide contributes its section on global
communications. The TYPO3 guide is the source for allowing a figure of speech on
condition.

### Linguistics

- The hierarchy of subordinate clauses described by Minami.
- The four-way classification of kango bases described by Nomura.
- Ishiguro, *Bunsho wa Setsuzokushi de Kimaru*.
- Honda, *Nihongo no Sakubun Gijutsu*.
- Meldrum 2009.

Takubo and Yoshimoto refined the hierarchy from Minami into four levels.
In the classification from Nomura, class C is what makes the pattern with `を` and
`する` possible. Ishiguro sets out four kinds and ten classes of conjunction.
Honda sets out the principles of modifier order and of comma placement.
In Meldrum, the pronoun is the only one of the four marks of translationese that held up.
