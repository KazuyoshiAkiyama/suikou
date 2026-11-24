---
title: "Tuning"
weight: 5
doctype: "howto"
---

Tuning suikou means measuring your own documents and setting each threshold to the value
that their characteristics call for. It does not mean editing thresholds to suit your taste.

## What to check before tuning

Run `suikou check` once before you change any setting. The layer that produces findings
determines which file you edit.

```sh
suikou check docs/*.md --format text
```

Base your judgment on the number of findings. The exit code is 1 only when there is an
error, so if you look only at the exit code you will miss every warning.

```sh
suikou check docs/*.md --format text | grep -c ':[0-9]*:[0-9]*:'
```

## What suikou examines

suikou has three layers, and each layer is tuned in a different place.

| Layer | What it examines | What it reports | Examples |
|---|---|---|---|
| Local rules | The form of an expression within a line | The position and how to fix it | `maint/list-lead-in`, `register/rare-wago` |
| Document metrics | A value computed over the whole document | An instruction for rewriting | `ja.kango_ratio`, `en.mattr100` |
| Structure rules | Whether sections are present, and their order | The sections that are missing | `structure/missing-section` |

Because a local rule reports a position, you can see at once what to change. A document
metric is never reported as a number; suikou converts it into an instruction first. A
structure rule is applied only to a document that declares a type.

For Japanese, a morphological analyzer determines each word's origin and inflected form.
For English, the analysis works on whole words. This difference between the languages is
handled entirely inside `tokenizer.rs`.

[Metrics]({{< relref "metrics.md" >}}) gives the exact definition of every metric and rule.
[Design]({{< relref "design.md" >}}) explains how the work is divided among the layers.

## How the thresholds were set

Each threshold was derived by measuring documents written by professionals, and it was set
at the point where human prose and machine prose separate.

Consider the ratio of suspended conjugation at the end of a clause. The value was 2.00 for
native human writing, 1.79 for human translation, and 3.62 to 15.94 for model output.
Based on these values, the threshold is 3.0.

Changing a threshold by intuition shifts the point that separates human and machine prose.
Measure your own corpus before you change a threshold.

[Metrics]({{< relref "metrics.md" >}}) reports the values each metric showed on each corpus.
The metrics that were measured and then rejected are listed under `[rejected]` in
`corpus/baselines.toml`.

## Which settings can be tuned, and where

Use the table below to find where to make the change you want.

| Change you want | Where to make it |
|---|---|
| Change a threshold for all documents | `[thresholds]` in `.suikou/profile.toml` |
| Change a threshold for one doctype | `[doctype.<type>]` in `.suikou/profile.toml` |
| Stop a metric from being reported | `disable` in `.suikou/profile.toml` |
| Stop a local rule from being reported for one doctype | `rules` under `[doctype.<type>]` |
| Fix a name that your project has agreed on | `.suikou/defined.toml` |
| Add a spelling variant, or cancel a built-in one | `.suikou/glossary.toml` |
| Permit a term specific to your field | `.suikou/terms.toml` |
| Assign a type to a document | `[paths]` in `.suikou/structure.toml` |

Only the fields you set replace the built-in values; you do not need to copy the entire
profile.

An adjustment for a single doctype is written in the following form.

```toml
[doctype.readme]
reason = "An entry point, where lists carry most of it"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

Record the reason for the adjustment in `reason`. A test verifies that every built-in
adjustment includes a reason. The reason is kept so that a later reader does not have to
go through the same deliberation again.

[Metrics]({{< relref "metrics.md" >}}) describes the file format and the order in which
settings are applied, in the part about adjusting a threshold. [Writing]({{< relref "writing.md" >}})
explains why the glossary has three separate stores.

## How to tune effectively

### Calibrate the thresholds on your own corpus

`suikou baseline` computes thresholds from the corpus you pass to it.

```sh
suikou baseline docs/ --lang en > .suikou/profile.toml
```

When a metric cannot be calibrated, it is omitted from the output and the reason is written
to standard error. A metric cannot be calibrated when there are too few samples, or when its
bound falls outside the metric's range so that it would never fire. No value is filled in
with a zero or a placeholder, so you can tell when a threshold has no basis.

### Use the firing rate to judge a threshold

When a metric fires on 70% of your documents, the cause is a threshold that does not fit,
not a characteristic of the documents.

The table below shows the result of applying the default thresholds to 240 READMEs from
well-known open source projects.

| Metric | README | Reference corpus |
|---|---|---|
| `en.mattr100` | 70% | 15% |
| `en.prose_ratio` | 32% | 11% |

This difference was the basis for setting separate thresholds for each doctype. You can
take the same measurement on your own corpus.

```sh
for f in docs/*.md; do suikou check "$f" --format text; done | grep -oE 'en\.[a-z_]+' | sort | uniq -c
```

### Identify the cause before choosing a remedy

A metric that fires too often has one of three causes, and each cause needs a different
remedy.

| Cause | How to check | Remedy |
|---|---|---|
| The metric has no resolution at this document length | Look at the firing rate for each word-count band | Set a lower limit for the metric, whatever the doctype |
| The distribution varies by doctype | Compare the firing rate across doctypes | Set a separate threshold for each doctype |
| The distribution is too wide to draw a boundary | Read the report from `suikou baseline` | Do not apply the metric to that doctype |

If you suspect the doctype first, the same defect remains in every other type. The density
of simile markers seemed to be a doctype problem but turned out to depend on document
length; D-43 in [Decisions]({{< relref "decisions.md" >}}) records the details.

### Make one change at a time

Measure the firing rate after every change. If you change several settings at once, you
cannot tell which change had the effect.

### Treat info findings as reference only

The requirement is that there are no errors and no warnings. A reference value such as the
Sino-Japanese ratio is reported at the info level, and it is not something to reduce to zero.

## Tuning to avoid

### Do not try to minimize a metric

An attempt to bring the Sino-Japanese ratio down to zero by substituting native verbs made
the prose worse. To lower that ratio, use established loanwords and avoid placing several
Sino-Japanese nouns in a row. D-27 in [Decisions]({{< relref "decisions.md" >}}) records
the incident.

### Do not tune only to silence findings

Putting every metric under `disable` makes the findings disappear, and it also hides the
problems in the documents that those findings pointed to.

When a metric fires often, identify the cause first. If you exclude the metric without
checking, the same defect remains in every other document.

### Do not replace words by intuition

Before you replace a flagged word, count how often the candidate occurs in the reference
corpus. `読み手` does not occur even once in four million characters of professional
Japanese, while `読者` occurs 13 times, and only counting reveals that difference.

[Writing]({{< relref "writing.md" >}}) describes the procedure for choosing a word, in the
part about the order in which the checks run.

## How to confirm the result

After tuning, run suikou on the same corpus again.

```sh
for f in docs/*.md; do suikou check "$f" --format text; done | grep -c ':[0-9]*:[0-9]*:'
```

If the firing rate is around 10%, the thresholds fit your documents. If a metric still
fires at 70%, its cause was never identified, and no threshold change will resolve it.

If the count drops to zero, check whether too much has been excluded by counting the lines
under `disable` and `rules` in `.suikou/profile.toml`.

This document covers only the tuning procedure. The metrics are defined in
[Metrics]({{< relref "metrics.md" >}}), the reasoning behind the design is in
[Design]({{< relref "design.md" >}}), and the history of each decision is in
[Decisions]({{< relref "decisions.md" >}}).
