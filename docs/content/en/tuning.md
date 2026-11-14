---
title: "Tuning"
weight: 5
doctype: "howto"
---

Tuning suikou is not a matter of rewriting thresholds to taste. It is a matter of measuring
your own documents and moving each threshold to where their properties put it.

## Before you begin

Run `suikou check` once before changing anything. Which layer reports findings decides
which file you write.

```sh
suikou check docs/*.md --format text
```

Judge by the number of findings. The exit code is 1 for an error only, so reading it alone
misses every warning.

```sh
suikou check docs/*.md --format text | grep -c ':[0-9]*:[0-9]*:'
```

## What suikou looks at

There are three layers, and each is tuned somewhere different.

| Layer | What it reads | What it reports | Examples |
|---|---|---|---|
| Local rules | A form inside a line | The position and the fix | `maint/list-lead-in`, `register/rare-wago` |
| Document metrics | A figure over the whole document | An instruction to rewrite | `ja.kango_ratio`, `en.mattr100` |
| Structure rules | Which sections exist, in what order | The missing section | `structure/missing-section` |

A local rule gives the position, so what to change is immediate. A document metric is never
reported as a number; it is turned into an instruction. A structure rule applies only to a
document that declares a type.

Japanese is read through a morphological analyzer, which gives the word's origin and its
inflected form. English is read as words. That boundary is closed inside `tokenizer.rs`.

Every metric and rule is defined exactly in [Metrics]({{< relref "metrics.md" >}}). How the
layers divide the work is in [Design]({{< relref "design.md" >}}).

## Where the thresholds came from

Each threshold was derived by measuring documents written by professionals, and placed
where human and machine prose separate.

Take the ratio of suspended conjugation at the end of a clause. Native human writing runs
2.00, human translation 1.79, and model output 3.62 to 15.94. The threshold is 3.0.

Moving a threshold by ear breaks that separation. Measure your own corpus before you move
one.

What each metric showed on which corpus is in [Metrics]({{< relref "metrics.md" >}}). The
metrics that were measured and rejected are listed under `[rejected]` in
`corpus/baselines.toml`.

## What can be tuned

Look up what you want to change.

| What you want to change | Where to write it |
|---|---|
| A threshold everywhere | `[thresholds]` in `.suikou/profile.toml` |
| A threshold for one doctype | `[doctype.<type>]` in `.suikou/profile.toml` |
| Stop reporting a metric | `disable` in `.suikou/profile.toml` |
| Stop reporting a local rule for one doctype | `rules` under `[doctype.<type>]` |
| Pin a name your project has settled | `.suikou/defined.toml` |
| Add a spelling variant, or cancel a built-in one | `.suikou/glossary.toml` |
| Allow a term from your field | `.suikou/terms.toml` |
| Assign a type to a document | `[paths]` in `.suikou/structure.toml` |

Only the fields you write replace the built-in values; there is no need to copy a whole
profile.

A per-doctype adjustment is written like this.

```toml
[doctype.readme]
reason = "An entry point, where lists carry most of it"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

Put the account of the adjustment in `reason`. A test checks that every built-in adjustment
carries one. It is there so that whoever reads it later does not repeat the deliberation.

The format and the order of application are in [Metrics]({{< relref "metrics.md" >}}), under
adjusting a threshold. Why the glossary keeps three separate stores is in
[Development]({{< relref "development.md" >}}).

## How to tune

### Calibrate from your own corpus

`suikou baseline` derives thresholds from the corpus you give it.

```sh
suikou baseline docs/ --lang en > .suikou/profile.toml
```

A metric it cannot calibrate is left out, with the reason on standard error: too few
samples, or a bound that falls outside the metric's range and would never fire. Nothing is
filled in with a zero or a placeholder, so a threshold without grounds is visible.

### Judge by how often a metric fires

A metric firing on 70% of your documents is reporting a mismatched threshold, not a
property of the documents.

Here is what the default thresholds do to 240 READMEs from notable open source.

| Metric | README | Reference corpus |
|---|---|---|
| `en.mattr100` | 70% | 15% |
| `en.prose_ratio` | 32% | 11% |

That gap is what grounded splitting thresholds by doctype. The same measurement works on
your own corpus.

```sh
for f in docs/*.md; do suikou check "$f" --format text; done | grep -oE 'en\.[a-z_]+' | sort | uniq -c
```

### Separate the cause before choosing the remedy

A metric that fires too often has one of three causes, and each takes a different remedy.

| Cause | How to check | Remedy |
|---|---|---|
| No resolution at this document length | Read the rate per band of word count | Gate the metric, regardless of doctype |
| The distribution differs by doctype | Compare the rate across doctypes | Change the threshold per doctype |
| The distribution is too wide to bound | Read what `suikou baseline` reports | Do not apply it to that doctype |

Suspecting the doctype first leaves the same defect in every other type. The simile-marker
density looked like a doctype problem and turned out to be about document length; D-43 in
[Decisions]({{< relref "decisions.md" >}}) records it.

### Change one thing at a time

Measure the firing rate after each change. Moving several at once leaves no way to tell
which one did the work.

### Read info as a reference

The condition to meet is no errors and no warnings. A reference figure such as the
Sino-Japanese ratio is reported at info, and is not something to drive to zero.

## What not to tune

### Do not aim to minimise a metric

Driving the Sino-Japanese ratio to zero by swapping in native verbs degraded the prose. If
you want that ratio lower, use the loanwords that are established and stop stacking
Sino-Japanese nouns together. D-27 in [Decisions]({{< relref "decisions.md" >}}) records
what happened.

### Do not tune for silence

Listing everything under `disable` does make the findings go away, and takes the problems
in the documents out of view with them.

Where a metric fires often, separate the cause first. Excluding it without checking leaves
the same defect in every other document.

### Do not replace a word by ear

Count the candidate in the reference corpus before replacing a flagged word. `読み手` does
not appear once in four million characters of professional Japanese, while `読者` appears 13
times, and nothing reveals that gap except counting.

The procedure for choosing a word is in [Development]({{< relref "development.md" >}}),
under the order the checks run in.

## Verify

Run the same corpus again after tuning.

```sh
for f in docs/*.md; do suikou check "$f" --format text; done | grep -c ':[0-9]*:[0-9]*:'
```

A firing rate around 10% means the thresholds fit your documents. A metric still at 70%
means the cause was never separated, and no threshold will settle it.

If the count reaches zero, check whether too much was excluded: count the lines under
`disable` and `rules` in `.suikou/profile.toml`.

This document covers the tuning procedure only. The metrics are defined in
[Metrics]({{< relref "metrics.md" >}}), the grounds for the design are in
[Design]({{< relref "design.md" >}}), and the account of each judgement is in
[Decisions]({{< relref "decisions.md" >}}).
