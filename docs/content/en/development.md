---
title: "Development"
weight: 40
doctype: "reference"
---

This page is a reference for building the project, pinning versions, handling the
tokenizer, and running the golden tests. Every topic on this page is a rule that anyone
working in this repository must follow.

## Repository layout

| Path | Content |
|---|---|
| `crates/suikou-core` | The core of the analysis |
| `crates/suikou-cli` | The CLI and the MCP server |
| `packages/` | The textlint rules that report a position |
| `skills/suikou` | The skill for agents |
| `research/` | Python code for calibration and exploration |
| `corpus/` | The measured values and the script that downloads the corpus |

## How to build and test

Use the commands below to check the core. You will run the first command most often, and
the second command when the dictionary is involved.

```sh
cargo test --all                           # No tokenizer. Fast. Start here
cargo test --all --features lindera-unidic # With the dictionary. Slower
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

lindera is not part of the default features, so the core logic can be tested without
downloading a dictionary. This keeps the cycle of making and checking a change short; a
change to the rules, for example, can be checked in a few seconds.

A release build has to specify that feature explicitly.

```sh
cargo build --release -p suikou-cli --features lindera-unidic
```

A build that lacks the dictionary supports English only.
If that build is asked to process Japanese, it fails immediately instead of returning an
empty analysis. `suikou selftest` tells you which build you have.

## Why and how versions are pinned

Versions are pinned for a specific reason, which this section explains. A version change
can corrupt a value even when the build still succeeds.

Every dependency is pinned with `=`, and `Cargo.lock` is committed to the repository.
Before a version is written into the manifest, it is confirmed to exist on crates.io.

lindera is handled more strictly than any other dependency.
Between versions, both the lindera API and the order of the UniDic features can change.
If that order changes, the word origin and the inflected form are read incorrectly without
any error. The kango ratio and the renyo count depend on those two fields, so if the
program keeps running on incorrect values, the findings themselves become incorrect.

After upgrading lindera, `LinderaMorphology::verify_schema` must pass.
This check runs a sentence with known analysis through the tokenizer and confirms that the
part of speech, the word origin, and the inflected form are read from the expected
positions. If they do not match, the check exits and reports which constant needs to be
corrected. Exiting with an error is better than silently returning an incorrect value.

## Where the tokenizer is handled

The behavior of the tokenizer changes from version to version, so the tokenizer is isolated
behind a boundary.

All version-dependent code is in `LinderaMorphology`, in
`crates/suikou-core/src/tokenizer.rs`.
Every other module accesses the tokenizer only through the `Morphology` trait.
Tests use `FakeMorphology`, so the logic can be tested even when no dictionary is
installed.

A new metric must stay on its side of that boundary.
If it crosses the boundary, the tests can no longer run without a dictionary.

In a release build, loading the embedded dictionary took 0.7 milliseconds in our
measurement. Once the dictionary was loaded, analyzing one sentence took 9.6 microseconds.

```sh
cargo run --release --features lindera-unidic --example dict_load
```

## Which part of the document each metric measures

Vocabulary metrics run over `Document::body()`. These are MATTR, the kanji ratio, the kango
ratio, demonstratives, and formal nouns.
Sentence-structure metrics run over `Document::prose()`. These are sentence length,
commas, renyo, and the te-form.

A vocabulary metric must never be limited to prose.
Some models put nearly all of their output into bullet lists, and for those models a
prose-only measurement discards more than eighty percent of the document.

## Why the preprocessing order must not change

The steps of `preprocess` in `markdown.rs` must run in a fixed order.
If the step that removes backslash escapes is moved later, sentence splitting stops working.
The AWS doc_source corpus uses this escaped form, and when the escapes are not removed the
mean sentence length grows by a factor of three or more.

`tests/golden/input/en_escaped.md` locks in this behavior.

Each removed span is replaced by the same number of newlines, so every finding refers to
the correct line in the file the user has open. A finding with a correct value but the
wrong position is of no use to anyone.

## How to measure the firing rate of a rule

When you add a rule or change a threshold, measure the firing rate on the reference corpus.

```sh
cargo run --release --example structure_rate -- corpus/cache/pro_en en
cargo run --release --example structure_rate -- corpus/cache/pro_ja ja
```

`suikou check` limits how many positions it prints for each rule, so the count cannot be
taken from the CLI output. The example calls the rules directly. Record the results in
`TASKS.md`.

`corpus/fetch.sh` downloads the corpus. The corpus is not stored in the repository for
licensing reasons.

To measure the required-section rules, you need a corpus whose documents declare a type.

```sh
python research/structure/fetch_typed.py corpus/cache/typed 120
```

This script collects Kubernetes pages grouped by their `content_type:`. Map those values
to suikou's doctypes under `[front_matter]` in `.suikou/structure.toml`, and then run
`check` in the usual way. The mapping is in `research/structure/structure.toml`.

When a rule changes, compare the length of the report before and after the change. Build
the earlier revision in a `git worktree`, run both builds on the same corpus, and add up
the characters in each report.

The `design` type is measured on the Rust RFCs.

```sh
python research/structure/fetch_rfcs.py corpus/cache/rfcs 200
```

Because this applies the type to the documents it was derived from, it is the strictest
test available.

The `decision` type is measured on published ADRs.

```sh
python research/structure/fetch_adrs.py corpus/cache/adrs 200 12
```

The configuration for this measurement is in `research/structure/structure-adr.toml`.

## What the golden tests are for

The golden tests keep the values consistent with the reference implementation.
The reference implementation is the Python code under `research/`, and the golden tests
fix the values that it returns.

`crates/suikou-core/tests/golden_rules.rs` checks the rule counts, and
`golden_metrics.rs` checks the document metrics, against the values in `tests/golden/expected/`.
The English tests run without a dictionary, and the Japanese tests require
`--features lindera-unidic`.

lindera and fugashi are different tokenizers, so the comparison allows a tolerance.
A ratio may differ by an absolute 0.02 and a density per thousand characters by 0.5, while
the M rules must match exactly as integers.
Word and sentence counts may differ by a relative one percent, because these counts depend
on how each tokenizer defines a word and a sentence.

When a metric falls outside the tolerance, the correct implementation is decided case by
case, and the reasoning is recorded on the decisions page.
The Python side is not assumed to be correct.

## Why rules are never added on intuition

**No constraint in this repository carries more weight than this one.**

Intuitions about style usually turn out to be wrong once they are measured.
Many of the hypotheses formed while designing this project were disproved by measurement.
Research on translationese reached the same finding about its own hypotheses.

To add a rule, provide one of the following kinds of evidence.

- Measurements of a human corpus and of AI output, showing that their ranges do not overlap.
- A clause from a published style guide, with a citation.

`corpus/baselines.toml` records each rejected metric and the reason for its rejection
under `[rejected]`.
**No metric on that list may be brought back.**
The list includes the coefficient of variation of sentence length, the loanword ratio, and
participial clauses and nominalization in English.
Each of these is widely cited as a marker of AI-written text, yet each one overlaps the
human range when it is measured.

## How token use is kept low

To keep token use low, the report is subject to the constraints listed below. They keep
the report short enough to be read in a single pass.

The report format must meet these requirements.

- A rule's explanation is printed once for that rule, not once for each line.
- No more than `Report::MAX_POSITIONS_PER_RULE` positions are printed for a rule.
- A metric that stays within its threshold is not printed.
- A document metric is printed as an instruction, not as a number.

These constraints exist so that the linter and the model do not have to send a document
back and forth once for every layer.
Before changing the report format, confirm that the change preserves this purpose and that
the report can still be read in a single pass.
