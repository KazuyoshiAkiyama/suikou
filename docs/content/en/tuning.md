---
title: "Tuning"
weight: 5
doctype: "howto"
---

Tuning suikou aligns each threshold with the measured properties of your own documentation rather than personal preference.

## Before you start

Run `suikou check` on your documentation prior to modifying any configuration. Identifying which inspection layer emits a finding determines which configuration file you need to edit.

```sh
suikou check docs/*.md --format text
```

Track the total count of reported findings instead of relying on the exit status. The command returns exit code 1 solely when errors occur, so inspecting the exit code alone overlooks every warning.

```sh
suikou check docs/*.md --format text | grep -c ':[0-9]*:[0-9]*:'
```

## What suikou inspects

Analysis is divided across separate inspection layers, each configured in its own location.

| Layer | Scope | Output | Examples |
|---|---|---|---|
| Local rules | Patterns within a single line | Exact location and correction | `maint/list-lead-in`, `register/rare-wago` |
| Document metrics | Aggregate values across the full document | Rewrite instruction | `ja.kango_ratio`, `en.mattr100` |
| Structure rules | Presence and ordering of sections | Missing section | `structure/missing-section` |

Local rules report the exact position, making the required edit direct. Document metrics never report raw numbers; instead, they translate measurements into rewrite instructions. Structure rules apply exclusively to documents with a declared doctype.

Japanese text is processed by a morphological analyzer that identifies word etymology and inflected forms. English text is tokenized into words. Both processing paths are encapsulated inside `tokenizer.rs`.

Complete specifications for all metrics and rules appear in [Metrics]({{< relref "metrics.md" >}}). How responsibilities are divided across layers is documented in [Design]({{< relref "design.md" >}}).

## How default thresholds are derived

Default thresholds were established by measuring professionally written documents and placing each cutoff where human prose and machine-generated text diverge.

Consider the clause-final renyo ratio. Native human writing measures 2.00, human translation measures 1.79, and model output ranges from 3.62 to 15.94. The cutoff is placed at 3.0.

Changing a threshold based on intuition eliminates that separation. Measure your own corpus before adjusting any value.

Per-corpus measurements for every metric are listed in [Metrics]({{< relref "metrics.md" >}}). Metrics that were evaluated and discarded appear under `[rejected]` in `corpus/baselines.toml`.

## Configurable settings

Find the adjustment you want to make in the table to locate its configuration file and key.

| Adjustment | Configuration location |
|---|---|
| Override a threshold globally | `[thresholds]` in `.suikou/profile.toml` |
| Override a threshold for one doctype | `[doctype.<type>]` in `.suikou/profile.toml` |
| Suppress a metric globally | `disable` in `.suikou/profile.toml` |
| Suppress a local rule for one doctype | `rules` under `[doctype.<type>]` |
| Standardize an established project name | `.suikou/defined.toml` |
| Add a spelling variant or override a built-in one | `.suikou/glossary.toml` |
| Allow a domain-specific term | `.suikou/terms.toml` |
| Assign a doctype to a document | `[paths]` in `.suikou/structure.toml` |

Overrides merge selectively with built-in defaults, so only modified fields need to be declared rather than duplicating an entire profile.

A doctype-specific override takes the following form.

```toml
[doctype.readme]
reason = "An entry point, where lists carry most of it"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

Record the rationale for each override in `reason`. Automated tests verify that every built-in override includes this field, so subsequent maintainers do not have to repeat the original analysis.

File syntax and override precedence are described in the section on threshold adjustment in [Metrics]({{< relref "metrics.md" >}}). Why glossary data is split across three separate stores is explained in [Writing]({{< relref "writing.md" >}}).

## Tuning workflow

### Generate a baseline from your corpus

`suikou baseline` computes candidate thresholds directly from a provided corpus.

```sh
suikou baseline docs/ --lang en > .suikou/profile.toml
```

Any metric that cannot be calibrated is omitted from the output, with the cause printed to standard error: insufficient sample count, or a computed bound outside the metric's valid range that would never trigger. No zero defaults or placeholder entries are inserted, keeping ungrounded thresholds visible.

### Evaluate metric firing rates

When a metric triggers on 70% of your files, that rate points to a mismatched threshold rather than a defect across your documents.

Applying default thresholds to 240 README files from prominent open-source projects produces the following firing rates.

| Metric | README corpus | Reference corpus |
|---|---|---|
| `en.mattr100` | 70% | 15% |
| `en.prose_ratio` | 32% | 11% |

This divergence provided the empirical basis for splitting thresholds by doctype. You can run the same frequency check across your own documentation.

```sh
for f in docs/*.md; do suikou check "$f" --format text; done | grep -oE 'en\.[a-z_]+' | sort | uniq -c
```

### Diagnose the root cause before applying a fix

Excessive firing stems from distinct underlying causes, each requiring a specific fix.

| Root cause | Diagnostic check | Remedy |
|---|---|---|
| Insufficient resolution at this document length | Inspect firing rates across word-count bands | Gate the metric regardless of doctype |
| Distribution differs across doctypes | Compare firing rates between doctypes | Override the threshold per doctype |
| Distribution is too wide to bound reliably | Review the diagnostics from `suikou baseline` | Exclude the metric for that doctype |

Assuming a doctype mismatch without checking first leaves the same underlying defect across every other doctype. The density of simile markers initially appeared to vary by doctype before analysis showed it depended on document length; D-43 in [Decisions]({{< relref "decisions.md" >}}) documents that investigation.

### Adjust one setting per iteration

Re-measure firing rates after every individual edit. Modifying several parameters simultaneously makes it impossible to isolate which change produced the observed effect.

### Treat info diagnostics as reference values

A passing run requires no errors and no warnings. Reference indicators such as the kango ratio are emitted at info severity and are not meant to be lowered to zero.

## Practices to avoid

### Do not minimize metrics as a goal

Lowering the kango ratio to zero by substituting native verbs degraded prose quality. To bring that ratio down, rely on established loanwords and avoid consecutive kango nouns. D-27 in [Decisions]({{< relref "decisions.md" >}}) records that outcome.

### Do not disable checks just to clear warnings

Listing every check under `disable` clears the findings while concealing genuine problems in the documentation.

Whenever a metric triggers frequently, diagnose its cause first. Suppressing it without investigation leaves the same defect unresolved across all other documents.

### Do not substitute words without corpus evidence

Check candidate word frequencies in the reference corpus before replacing a flagged term. Across four million characters of professional Japanese text, `読み手` never appears once, whereas `読者` appears 13 times, a disparity revealed only by counting.

The word-selection workflow is detailed in the section on check order in [Writing]({{< relref "writing.md" >}}).

## Verify the results

Re-run the check against the same corpus after completing your adjustments.

```sh
for f in docs/*.md; do suikou check "$f" --format text; done | grep -c ':[0-9]*:[0-9]*:'
```

A firing rate near 10% indicates that the thresholds fit your documentation. A metric remaining near 70% signals that its underlying cause was never isolated, and further threshold tuning will not resolve it.

If the total finding count drops to zero, verify that exclusions are not overly broad by counting the entries under `disable` and `rules` in `.suikou/profile.toml`.

This guide covers only the tuning procedure. Metric definitions are in [Metrics]({{< relref "metrics.md" >}}), architectural rationale is in [Design]({{< relref "design.md" >}}), and individual design records are in [Decisions]({{< relref "decisions.md" >}}).
