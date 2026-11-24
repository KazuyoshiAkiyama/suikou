---
title: "Design"
weight: 10
doctype: "design"
---

This system is a layered linter and harness that removes AI-generated style patterns from Japanese and English technical documentation and improves maintainability.

This page follows the rules it defines. Every list follows a complete sentence, item counts are never declared ahead of a list, and no bare metaphors appear.

## Motivation

Generative AI introduces habits into technical documentation that extend beyond individual word choices. Generated prose frequently relies on metaphor and indirect phrasing, repeats identical sentence endings, connects clauses awkwardly, and adds excessive detail that complicates maintenance.

Existing tools check only word lists or notation rules and miss these broader patterns. A spell checker, for instance, cannot detect a document that relies heavily on metaphor.

## Scope

This tool targets technical documentation and technical writing. Because what qualifies as good prose depends on context, reliable rules require a clearly bounded domain.

The supported languages are Japanese and English. Style rules rely on per-language empirical measurements, whereas structural rules apply to both languages.

## Non-goals

Fiction, advertising copy, and social media posts are outside the scope because appropriate style rules vary by genre.

Automatic rewriting is also outside the scope. The tool reports issue locations and document-level guidance while leaving edits to the author, because mechanical substitution corrupted sentence meaning multiple times in this repository's own documentation.

Factual verification is outside the scope as well. Style and structure can be measured against a corpus, whereas the truth of a technical claim cannot.

## Primary constraint

The tool does not evaluate one layer at a time or ask a model to fix separate reports across multiple rounds. To limit token consumption, a single run outputs findings from every layer so that one editing pass can resolve the entire report.

## Empirical basis of the design

Every rule derives from corpus measurements or a published standard rather than intuition. Intuitive assumptions about prose style fail most of the time once tested empirically, a conclusion reached independently in translationese research and studies of AI-generated text.

### Human reference corpus

The Japanese reference corpus combines original writing and translations. Native samples come from two books by kaityo256 and the future-architect coding standards, while translated samples come from the Japanese Kubernetes site, the Japanese Vue site, and the ja_JP directory of the Linux kernel.

The English reference corpus comprises the Linux kernel documentation, gcc, the Rust Book, Django, Google eng-practices, the AWS S3 user guide, and the AWS CloudFormation user guide.

### AI sample corpus

The AI corpus contains 232 documents produced by four models: Claude, Gemini, ChatGPT, and Grok. Experimental conditions crossed two languages, three prompt types covering explanation, procedure, and design rationale, three Japanese registers, and the presence or absence of an instruction suppressing formatting.

Every condition began in a fresh chat session with memory and custom instructions turned off.

### Validated metrics

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

Among all evaluated features, the ratio of renyo to te-form separates human and AI writing most clearly. The te-form aligns more closely with spoken Japanese and integrates clauses into a unified statement, whereas the renyo form belongs to written style and coordinates clauses side by side; AI strongly favors the renyo form. This pattern corresponds to the excess of participial clauses noted in English, and Minami's subordinate-clause hierarchy links the two languages.

Demonstratives and formal nouns require careful handling because AI generates **fewer** of them than human authors do. A finding about absent elements cannot be attached to a specific source position.

### Rejected metrics

The coefficient of variation of sentence length could not be distinguished from translated Japanese. Loanword ratio, the construction pairing a sahen noun with `を` and `する`, and comma density all overlapped with the human range.

In English, participial clauses and nominalization overlapped with the human range as well. The 5.3 effect size reported in prior work did not replicate under this setup, likely due to experimental design: earlier studies supplied human text and requested a continuation, whereas this evaluation requested complete documents written from scratch. **Effect sizes from earlier studies must not be transferred across different tasks.**

### Formatting suppression experiment

When models were instructed to write strictly in prose without lists, bold styling, or headings, formatting artifacts disappeared completely. Bold markup, lists, headings, and em dashes all dropped to zero.

Vocabulary and sentence structure remained unchanged. Comparing Claude's polite Japanese register with and without formatting suppression, MATTR moved from 0.642 to 0.650 and renyo moved from 5.31 to 4.83, both within the margin of error.

This result determines how responsibilities divide across layers. Prompt instructions can eliminate the formatting layer alone, whereas vocabulary and sentence structure must be enforced by a linter.

### Variation across models

Formatting behavior is dominated by model-specific habits. Em dashes occurred at 10.32 per thousand words for Claude versus zero for Gemini, while bold styling occurred at 95.4 per thousand words for Gemini versus zero for Grok in Japanese.

By contrast, MATTR, the renyo ratio, formal nouns, and demonstratives shifted in the same direction across all four models. **Only metrics showing this cross-model consistency belong in threshold-based rules.**

### Variation across registers

Plain and polite Japanese registers were compared on the explanation prompt with Claude and Grok to test whether register alters metric values.

| Metric | Claude plain/polite | Grok plain/polite |
|---|---|---|
| MATTR | 0.652 / 0.657 | 0.624 / 0.620 |
| Renyo | 5.12 / 5.31 | 5.33 / 3.27 |
| Prose ratio | 0.618 / 0.371 | 0.915 / 0.922 |
| Formal nouns | 7.68 / 3.03 | 4.37 / 7.24 |

The direction of change disagrees between the models, showing no systematic register effect beyond inter-model variation. **Thresholds do not need separate values per register.** Register detection is used solely to determine whether register-dependent rules apply.

### Variation across document types

The requested document type has a substantial effect on metric values, far exceeding the effect of register and approaching the size of the difference between AI and human writing.

| Metric | Explanation | Procedure | Rationale | Human native |
|---|---|---|---|---|
| Renyo per 1k | 3.83 | 7.69 | 4.90 | 2.71 |
| Kango ratio | 0.435 | 0.498 | 0.523 | 0.397 |
| Formal nouns per 1k | 4.63 | 8.11 | 4.00 | 11.16 |
| Demonstratives per 1k | 3.25 | 0.22 | 1.27 | 6.47 |
| Prose ratio | 0.526 | 0.345 | 0.507 | 0.847 |

Even in the worst-case document type, renyo, demonstratives, MATTR, and prose ratio remain distinct from the human range.

Formal nouns and kango ratio are more sensitive to document type. Formal nouns reach 8.11 in procedures, leaving almost no margin below the 8.5 threshold, while kango ratio reaches 0.435 in explanations, close to the 0.397 native human value. Both metrics require either per-type thresholds or a lower severity level.

### Effect of generation order

Because generating eight pieces from a single prompt could introduce drift, values were centered within each condition and correlated against generation rank. Every structural metric correlated near zero, confirming that style did not drift.

Only kango ratio showed a notable correlation at +0.405. Because generation rank and subject matter co-varied in this design, that correlation reflects topic effects rather than ordering drift, consistent with the document-type sensitivity noted above.

### Compliance with maintainability instructions

Maintainability rules were added to prompts to measure model compliance. Testing compared a short instruction of two sentences against a detailed instruction enumerating each rule.

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

Structural rules—list lead-ins, parallel list items, and avoiding trailing enumerations—reached full compliance under the short instruction. In contrast, violations of the lexical rule against time-dependent words only halved under the short instruction and dropped to zero only under the detailed instruction.

Declaring item counts, numbering headings, and hand-numbering list items rarely occurred even without prompt guidance. Those three rules do not belong in the pre-generation prompt layer.

Without instructions, Claude omitted list lead-ins entirely. Across 24 list blocks, every preceding line was a heading or another list item; this represents an absent lead-in rather than a malformed one and matches its 0.150 prose ratio.

Maintainability instructions also improved document-level metrics as a secondary effect.

| Metric | Claude, none | Short | Human native |
|---|---|---|---|
| Prose ratio | 0.150 | 0.702 | 0.847 |
| List ratio | 0.633 | 0.273 | 0.172 |
| Formal nouns per 1k | 2.75 | 8.11 | 11.16 |

Requiring list lead-ins induced connecting prose, moving both prose ratio and list ratio toward human baselines. Renyo and demonstratives, however, did not improve.

The short instruction also reduced output length. Grok output fell from 13629 characters to 8010 in Japanese and from 5053 words to 2260 in English, whereas the detailed instruction did not cause this reduction.

## Detection categories

Tool findings divide into binary checks evaluated at the sentence or line level, which report exact positions, and document-wide metrics computed across the full text, which carry no positions. Most document-wide findings identify an absence of expected elements and therefore have no single line to point to.

### D group: document-wide metrics

| ID | Target | Language | Threshold | Preventable | Severity | Stability |
|---|---|---|---|---|---|---|
| D1 | High MATTR | ja, en | ja >0.60 / en >0.71 | Partly | warning | High |
| D2 | Low prose ratio | ja, en | ja <0.75 / en <0.65 | Yes | warning | High |
| D3 | Renyo over te-form | ja | >3.0 | No | warning | High |
| D4 | Renyo density | ja | >4.0 per 1k | No | warning | High |
| D5 | Scarce formal nouns | ja | <8.5 per 1k | Yes | info | Low |
| D6 | Scarce demonstratives | ja | <4.5 per 1k | No | warning | High |
| D7 | High kango ratio | ja | >0.42 | No | info | Low |
| D8 | High list ratio | ja | >0.25 | Yes | warning | Medium |
| D9 | Scarce simile markers | en | <0.8 per 1k | No | warning | Medium |

Every threshold is placed between the upper bound of the human range and the lower bound of the AI range.

Stability reflects how reliably a metric remains separated across different document types. Because D5 and D7 vary with genre and subject and nearly intersect the human range in worst cases, both remain at info severity until per-type human baselines are established. Once those baselines exist, both promote to warning.

### M group: maintainability rules

| ID | Target | Shape | Standard |
|---|---|---|---|
| M1 | Lead-in binds grammatically to list items | Binary | Google developer documentation style guide |
| M2 | Prose declares the count of list items | Binary | Google developer documentation style guide |
| M3 | Heading includes a section number | Binary | Google developer documentation style guide |
| M4 | List item includes a hand-written number | Binary | Google developer documentation style guide |
| M5 | List items lack parallel structure | Binary | Google developer documentation style guide |
| M6 | Word depends on a point in time | Binary | Google timeless documentation |
| M7 | Trailing enumeration | Binary | Google timeless documentation |

M-group rules target document maintainability rather than AI-specific style. Because these issues predate generative AI, the rules do not become obsolete as models change.

This layer must never be calibrated to corpus averages. Among the reference corpora, only the AWS S3 user guide writes list lead-ins properly, whereas the Linux kernel contains 105 incomplete lead-ins against 16 complete ones. Matching the human average would render this layer ineffective.

### N group: conventional rules rarely triggered by AI

This group comprises nominal sentence endings, the sahen verb construction, three identical sentence endings in a row, and the metaphor word list. Although AI rarely triggers them in practice, they remain useful when editing human documentation and are retained at a lower severity.

A metaphor word list loses effectiveness immediately upon publication. After delve, intricate, and realm were widely noted in early 2024, their frequency dropped right away. A fixed word list must never serve as the sole detection mechanism.

### A group: model-assisted judgments

Checks requiring semantic evaluation belong here: distinguishing live metaphors from dead ones, verifying whether a metaphor includes an explanation, judging whether excessive detail impairs maintainability, and checking terminology.

The linter flags candidate passages and stops there. All semantic judgment belongs to the harness.

## Architecture

### Separation of responsibilities

Components divide according to who consumes the output.

Human writers and editors invoke textlint directly for positional rules, where exact locations matter and incremental runs remain fast.

Agents and CI jobs invoke the `suikou` front end, which computes document-wide metrics, builds the unified report, and manages token budgets.

```
suikou
├─ textlint --format json      Positional rules, M1 to M7 plus three presets
├─ Document analyzer           D1 to D9
├─ Merge and build guidance
└─ Output: Markdown / JSON / MCP
```

Implementing positional checks as textlint rules allows editors such as NeoVim to use them standalone. Because `suikou` calls that same textlint rule set, no logic is implemented twice.

### Implementation languages

Every validated metric can be implemented in pure Rust because lindera-unidic returns all UniDic features, including inflected form for D3 and word origin for D7.

Python was needed only for participial clauses, nominalization, modifier stacking, and dependency distance, every one of which was rejected.

Implementation therefore divides into a validated tier and an exploratory tier.

**The validated tier** is pure Rust and implements the D, M, and N groups. Compiling to a single binary handles repeated invocations from hooks.

**The exploratory tier** is Python and handles corpus calibration, candidate metric discovery, and threshold recalculation. It never enters the production path.

A metric moves from the exploratory tier to the validated tier only after validation confirms its reliability.

Nominalization and modifier stacking retain value as model-dependent metrics. If either enters production, a resident Python sidecar communicating via newline-delimited JSON on standard input and output will load the model once; this path is opt-in and off by default.

### Performance

The intended workflow triggers the tool from Claude Code hooks on every Write and Edit operation. Because startup latency dominates that usage, zero-copy reading of the UniDic dictionary and a resident daemon mode were both considered.

In a release build, loading the embedded dictionary takes 0.7 milliseconds. That measurement does not justify building a resident mode solely for dictionary loading.

## CLI reference

| Command | Role |
|---|---|
| `suikou brief [--profile P] [--lang L]` | Print prompt constraints for generation |
| `suikou check <path>` | Output positional findings and document metrics together |
| `suikou baseline <glob>` | Calibrate thresholds from a corpus and write a profile |
| `suikou terms <glob>` | Extract a glossary and allowlist from a set of documents |
| `suikou selftest` | Verify the identity and integrity of the installed binary |
| `suikou mcp` | Run as an MCP server |
| `suikou plot <path> --doctype T` | Write document structure into `.suikou/plans/` before drafting |

The `check` command accepts the main options listed below.

- `--format md|json|text` selects the output format, with md as the default.
- `--lang auto|ja|en` sets the target language, with auto as the default.
- `--profile oss|service|reference|tutorial|<path>` selects the threshold set.
- `--budget <tokens>` caps the report size in tokens.
- `--quiet` suppresses output and returns only the exit code.

### Output of brief

Only the formatting layer and the maintainability layer are emitted by brief. Vocabulary and sentence-structure metrics such as MATTR and the renyo ratio are excluded.

The formatting experiment showed that prompt instructions eliminate formatting artifacts completely while leaving vocabulary and syntax unchanged. Placing ineffective constraints in a prompt wastes tokens and risks over-correction.

Maintainability instructions require different lengths depending on rule type. Structural rules are stated concisely, whereas time-dependent words must be listed explicitly. Rules that never trigger without prompting are excluded from brief.

| Rule | Treatment in brief |
|---|---|
| M1, list lead-in | One sentence |
| M5, parallel items | One sentence |
| M7, trailing enumeration | One sentence |
| M6, time-dependent words | List the forbidden words |
| M2, item count | Omitted |
| M3, numbered headings | Omitted |
| M4, hand-written numbers | Omitted |

Because a short instruction reduces output length, the profile exposes detail level as a setting so callers can select what fits the task.
## Output format

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

### Controlling token consumption

Minimizing token usage is the primary formatting constraint, so reports follow these output rules.

- Each rule explanation appears once per rule rather than repeating on every matching line.
- At most twenty locations appear per rule; any remaining occurrences appear only as a count.
- Metrics within their thresholds are omitted completely.
- Specifying `--budget` truncates the report according to severity order.

### Mapping metrics to guidance

Raw numerical values do not tell a model how to revise text, so every metric maps to an actionable editing instruction. For example, a high kango ratio produces an instruction to rewrite kango compounds using native Japanese verbs.

| Metric | Resulting instruction |
|---|---|
| D1, high MATTR | Refer to each concept with one consistent term instead of varying wording |
| D2, low prose ratio | Convert bullet lists back into continuous prose |
| D3 and D4, frequent renyo | Split long clauses or replace them with the te-form |
| D5, few formal nouns | Replace nominalized kango with native phrasing |
| D6, few demonstratives | Connect paragraphs using demonstratives rather than repeating nouns |
| D7, high kango ratio | Use established loanwords instead of chaining kango nouns |
| D8, excessive lists | Convert bullet lists back into continuous prose |
| D9, few simile markers | Mark comparisons explicitly as similes or examples |

### Severity levels

| Severity | Scope | Rationale |
|---|---|---|
| error | Maintainability layer (M1 to M7) | Clear violation of an objective standard |
| warning | D metric exceeding its threshold | Thresholds include margin and vary with context |
| info | Model-dependent metrics, word lists, exploratory metrics | Advisory; some entries lose relevance over time |

Maintainability takes precedence over AI style because maintainability violations break an explicit written standard, whereas AI style findings depend on statistical thresholds.

Which severity levels cause a hook to reject an edit is configurable.

## Rule design

### Register detection

Plain and polite Japanese registers use different sentence endings, so certain rules must adapt to the active register.

Register detection runs per text block rather than once for an entire file. A single document can legitimately combine polite body prose with plain list items, making document-wide switching too coarse.

Metric thresholds do not vary by register. Comparisons between both registers revealed neither directional agreement across models nor any systematic difference. Register detection exists solely to determine whether register-specific rules apply.

Detection examines the final morpheme of each sentence. Setting `style: auto | desumasu | dearu` overrides automatic detection.

The following rules depend on the detected register.

- Noun-ending sentences are disallowed in prose and permitted inside list items.
- The word list for redundant sentence endings differs between plain and polite registers.

### List lead-ins (M1)

In Japanese, valid lead-ins take the form of either a complete sentence ending in a period or a noun followed by a colon. A lead-in ending in a renyo form, a particle, or a conjunctive particle attaches grammatically to the items beneath it, so inserting or removing an item breaks the sentence structure.

In English, the rule checks whether the lead-in contains expressions such as "the following", "as follows", "these", "below", or "steps". Although a strict syntactic rule would check for a finite verb, this lexical check alone separates the AWS S3 user guide (30 against 145) from the Linux kernel documentation (105 against 16).

### Metaphor checks

Standard technical terminology established within a domain is not treated as metaphor. Terms such as cache, tree, and handshake are domain vocabulary and are ignored by the rules.

The rules target unfamiliar metaphors introduced without explanation. Writing that a queue behaves like a conveyor belt explicitly marks a comparison, whereas calling the queue a conveyor belt does not. Among the 385 words compiled by Sakasegawa, figurative uses involving entrance, foundation, core, and boundary fall into this category, along with patterns involving carve, crush, step in, and dissolve. The 291 words identified by Kobak, as well as tapestry and amidst from Reinhart, also belong here.

Similes and worked examples are permitted under a single condition derived from the TYPO3 guide: a plain explanation must appear alongside the figure of speech.

No static allowlist ships with the tool. Instead, `suikou terms` generates the allowlist from the target project's own documents. Because terminology varies across projects, building the list locally prevents a fixed word list from becoming outdated.

## Authoring harness

### Separating draft notes from deliverables

A workspace convention keeps excessive detail out of finished documents.

```
docs/guide.md              The deliverable, which the linter covers
.suikou/notes/guide.md     The thinking, which the linter leaves alone
```

The skill directs the model to record considered options, rejected alternatives, uncertain values, supporting rationale, and measurement sources in the notes file. Only finalized content belongs in the deliverable.

This separation works in multiple ways. Giving the model a dedicated location for explanatory reasoning reduces its tendency to insert extraneous detail into the deliverable. Preserving the rationale beside the finished file also lets subsequent readers trace why a passage was written in a particular way.

Converting a writing restriction into a file-placement rule is the central principle of this design.

### Execution workflow

The authoring sequence proceeds as follows.

1. Include the output of `suikou brief` in the prompt.
2. Generate both a deliverable document and an accompanying notes file.
3. Run `suikou check` once and review the combined report.
4. Apply all required edits in a single pass.
5. Verify the result with `suikou check --quiet`.

No rule layer is checked and repaired in isolation. Positional diagnostics identify where edits are required, while guidance derived from document metrics specifies how to revise the text.

### Project glossary

Running `suikou terms` extracts domain terminology from the local documents and writes the results to `.suikou/terms.toml`.

Terms recorded in that file count as established vocabulary and are excluded from metaphor checks. Only figurative terms outside the glossary trigger reports. The same mechanism checks terminology consistency across documents.

## Repository structure

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

Storing all components in one repository prevents the CLI output schema and the skill's expectations from diverging. Both change together in a single commit, enforced by golden-file tests.

This repository holds the canonical copy of the skill, which a dotfiles setup imports. Maintaining the authoritative copy inside dotfiles would allow it to fall out of sync with the CLI version.

### Package boundaries

The npm packages are divided by responsibility.

- `textlint-rule-preset-tech-maintainability` contains M1 to M7 for both languages and stands alone because it provides value independent of AI style checking.
- `textlint-rule-preset-ja-ai-structure` contains the Japanese sentence-level rules.

## Relationship to existing open-source tools

| Package | Integration strategy | Rationale |
|---|---|---|
| textlint-rule-preset-ja-technical-writing | Use alongside | Covers notation and readability with no overlap |
| @textlint-ja/textlint-rule-preset-ai-writing | Use selectively | Reuses rules for overstatement, mechanical bold formatting, and list formatting |
| p1ass/textlint-rule-preset-ai-words-ja | Use alongside | Provides a word list that loses effectiveness over time and never serves as the sole check |
| D1 to D9, M1 to M7 | Implemented here | No equivalent implementation exists |

The `no-ai-colon-continuation` rule from the ai-writing preset is close in scope to M1. Whereas the former forbids placing a predicate before a colon, the latter checks whether the lead-in attaches grammatically to the list items. Because their goals differ, both rules coexist, though they may flag the same line simultaneously. The formatter merges duplicate findings reported on the same line.

Existing presets were tested against these layers and show no overlap. Although `textlint-rule-preset-ja-technical-writing` limits sentences to 100 characters, AI-generated prose averages 39.2 characters and does not trigger the rule. It also restricts sentences to three commas, whereas AI-generated prose averages 0.95 commas per sentence.

## Implementation roadmap

### Implementation sequence

Work starts with the maintainability layer. Its rules are binary, require no thresholds, reference explicit standards, and remain useful outside AI style checking. Completing this layer alone produces a practical tool. A subsequent experiment demonstrated that instructions covering this layer achieve near-complete compliance and simultaneously improve document-level metrics, offering the highest return on implementation effort.

Stable Japanese document metrics come next, covering D1, D2, D3, D4, and D6. Among these, the ratio of renyo to te-form separates texts most clearly and can be calculated directly from inflected forms provided by lindera-unidic.

D5 and D7 follow at info severity. Their thresholds will be reconsidered once human baselines exist for each document type.

English support and the MCP server come last. Resident mode is omitted for the reasons recorded in D-25.

Within the exploratory tier, `jametrics.py`, `enmetrics.py`, and `mcheck.py` act as reference implementations for all metrics. Golden tests verify that the Rust implementation outputs identical values.

### Evaluation criteria

False-positive rates are evaluated against the human corpus. Target corpora include the Linux kernel, gcc, the Rust Book, Django, AWS, the Japanese Kubernetes site, and the Japanese Vue site, measuring the number of findings reported as error.

The M group is expected to fire frequently even on human-authored documentation. Incomplete list lead-ins are prevalent in the Linux kernel documentation. Such findings represent legitimate opportunities for improvement rather than false positives. If the report volume proves too high in practice, default severities may require adjustment.

Conversely, the D group should remain silent on the human corpus. Any finding it reports on human-authored text counts as a false positive.

### Expanding the human corpus

The native Japanese corpus leans toward expository books. Because document type substantially alters metric values, native Japanese procedural documentation must be added to the corpus. Promoting D5 and D7 to warning severity depends on collecting those baselines.

## Remaining design questions

The following decisions are deferred until implementation and evaluation are complete.

- Decide whether the skill executes `suikou brief` on each run or embeds its output inside SKILL.md.
- Decide how the brief's level of detail is represented as a profile setting.
- Decide how finely thresholds should be partitioned by document type.
- Decide how to ingest the Japanese AWS and Google documentation into the corpus.
- Expand the native Japanese corpus.

Choosing the brief's detail level requires a trade-off because shorter instructions reduce output volume. Partitioning thresholds also requires deciding how to detect each document's type. Because the Japanese AWS and Google documentation is not hosted on GitHub, adding it requires an alternative acquisition method. The native Japanese corpus draws on two sources, both of which are expository books.

### Settled decisions

Empirical measurements settled the following points, so none of them require reconsideration.

- Thresholds do not split by register.
- Data collected from a single batched prompt is usable.
- Structural maintainability rules hold under concise instructions.
- Document metrics do not belong inside textlint.
- Resident mode is not built because startup time was measured in milliseconds.
- Rust is the implementation language.

Unlike structural rules, vocabulary rules require detailed instructions. The front-end CLI owns document metrics, while Python is restricted to calibration and exploratory search.

## References

### Quantitative studies of AI prose

The design draws on the following quantitative studies of AI-generated text.

- Kobak et al., *Science Advances* 2025.
- Reinhart et al., *PNAS* 122(8) 2025.
- Markey et al., *Written Communication* 41(4) 2024.
- Geng and Trotta, *Findings of ACL* 2025.
- Zhang et al., *ACL* 2025.
- Freeburg, arXiv:2603.27006 2026.
- Sakasegawa, 2026.

Kobak et al. analyzed 15.1 million PubMed abstracts and found that 66 percent of excess words in 2024 were verbs and 14 percent were adjectives, reversing the COVID-period pattern in which 79 percent were nouns. Reinhart et al. compared texts across Biber's 66 features and demonstrated that instruction tuning drives the stylistic divergence, leaving base models closer to human writing. Markey et al. characterized the ChatGPT register as dense and disconnected.

Geng and Trotta showed that a word's usage frequency drops immediately after it is publicly called out. Zhang et al. demonstrated that preferences for lists and bold text are embedded in the reward model. Freeburg showed that formatting habits can be suppressed while the em dash persists in some models. Sakasegawa compared 70,000 Qiita articles before and after generative AI, representing the only quantitative study of its kind in Japanese.

### Style guides and standards

The maintainability rules draw on the following published style guides.

- The Google developer documentation style guide.
- The Microsoft Writing Style Guide.
- The TYPO3 Community Language and Writing Guide.
- The JTCA Japanese Style Guide, third edition.

The Google guide informs the rules on lists, headings, timeless documentation, and inclusive documentation. The Microsoft guide informs the guidance on global communications. The TYPO3 guide provides the conditional rule permitting figures of speech.

### Linguistic literature

The Japanese linguistic analysis builds on the following works.

- Minami's hierarchy of subordinate clauses.
- Nomura's four-way classification of kango bases.
- Ishiguro, *Bunsho wa Setsuzokushi de Kimaru*.
- Honda, *Nihongo no Sakubun Gijutsu*.
- Meldrum 2009.

Takubo and Yoshimoto refined Minami's hierarchy into four levels. In Nomura's classification, class C enables the `を` and `する` construction. Ishiguro categorizes conjunctions into four kinds and ten classes. Honda defines principles for modifier ordering and comma placement. In Meldrum's study, pronouns were the only one of the four purported marks of translationese that held up.
