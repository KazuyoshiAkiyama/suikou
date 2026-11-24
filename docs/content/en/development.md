---
title: "Development"
weight: 40
doctype: "reference"
---

This development reference defines the constraints that govern work in this repository, covering builds, version pinning, tokenizer isolation, and golden tests.

## Repository layout

| Path | Description |
|---|---|
| `crates/suikou-core` | Core text analysis library |
| `crates/suikou-cli` | Command-line interface and MCP server |
| `packages/` | Position-reporting textlint rules |
| `skills/suikou` | Agent skill definition |
| `research/` | Python scripts for exploration and calibration |
| `corpus/` | Corpus retrieval script and measurement data |

## Build and test workflow

Execute the workspace checks below during development. Use the tokenizer-free test suite for routine verification, and enable the dictionary feature whenever morphological analysis is involved.

```sh
cargo test --all                           # No tokenizer. Fast. Start here
cargo test --all --features lindera-unidic # With the dictionary. Slower
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

Default features exclude lindera so core logic tests can run without downloading a dictionary. That separation keeps feedback cycles short, allowing rule edits to finish verification within seconds.

Release binaries enable the dictionary feature explicitly:

```sh
cargo build --release -p suikou-cli --features lindera-unidic
```

Dictionary-free binaries support English exclusively. Requesting Japanese analysis from such a build raises an immediate error rather than returning an empty result. Run `suikou selftest` to inspect which build variant is active.

## Version pinning

All dependencies are strictly pinned because an update can silently alter output values without breaking the build. Every dependency uses an exact `=` requirement, `Cargo.lock` is checked into version control, and each version is confirmed on crates.io before being declared.

The lindera crate requires the strictest oversight because both its API and the ordering of UniDic feature fields change across releases. When field indices shift, word origin and inflected form silently read invalid values. Because the kango ratio and renyo count depend on those two fields, misaligned indices corrupt the resulting findings.

Every lindera upgrade must pass `LinderaMorphology::verify_schema`. This verification feeds a known sentence to the tokenizer and confirms that part of speech, word origin, and inflected form appear at their expected indices. On mismatch, the check prints the constant requiring adjustment and exits immediately instead of emitting wrong metrics quietly.

## Tokenizer isolation

Because tokenizer behavior varies across releases, all tokenizer access is isolated behind a dedicated interface. Version-specific code resides exclusively inside `LinderaMorphology` in `crates/suikou-core/src/tokenizer.rs`, while all other modules depend solely on the `Morphology` trait. Unit tests inject `FakeMorphology` so analysis logic runs with no dictionary installed.

New metrics must not bypass this interface. Calling the tokenizer directly prevents tests from executing without a dictionary.

Release-build benchmarks put embedded dictionary initialization at 0.7 milliseconds, with subsequent analysis of one sentence taking 9.6 microseconds.

```sh
cargo run --release --features lindera-unidic --example dict_load
```

## Metric input scope

Vocabulary metrics (MATTR, kanji ratio, kango ratio, demonstratives, and formal nouns) take `Document::body()` as input, whereas sentence-structure metrics (sentence length, commas, renyo, and te-form) take `Document::prose()`.

Restricting vocabulary metrics to prose is prohibited. Some models format almost all output as bullet lists, where a prose-only pass omits more than eighty percent of the document.

## Preprocessing order

Step ordering inside `preprocess` in `markdown.rs` is fixed because delaying backslash-escape removal disrupts sentence segmentation. Documents in the AWS doc_source corpus use backslash-escaped syntax, and retaining those escapes multiplies mean sentence length by three or more.

Regression coverage for this constraint lives in `tests/golden/input/en_escaped.md`.

Every stripping pass preserves newline counts so reported diagnostics match line numbers in the source file. Accurate measurements lose utility if reported against the wrong line.

## Measuring rule firing rates

Adding a rule or modifying a threshold requires measuring its firing frequency across the reference corpus.

```sh
cargo run --release --example structure_rate -- corpus/cache/pro_en en
cargo run --release --example structure_rate -- corpus/cache/pro_ja ja
```

Standard CLI output from `suikou check` caps reported positions per rule and therefore hides true match totals. The example binary invokes rules directly, and its measurements are logged in `TASKS.md`.

Use `corpus/fetch.sh` to download the reference corpus, which licensing restrictions keep out of the repository.

Checking required-section rules depends on a corpus whose documents declare explicit types.

```sh
python research/structure/fetch_typed.py corpus/cache/typed 120
```

This script fetches Kubernetes documentation sorted by `content_type:`. After mapping those metadata values to doctypes inside the `[front_matter]` table of `.suikou/structure.toml`, invoke `check` normally. That configuration is stored in `research/structure/structure.toml`.

Assess report size before and after any rule edit by building the previous revision in a `git worktree`, executing both builds over the identical corpus, and summing total output characters.

Validate the `design` type against Rust RFCs.

```sh
python research/structure/fetch_rfcs.py corpus/cache/rfcs 200
```

Testing a document type against its own source corpus serves as the most demanding verification available.

Validate the `decision` type against published ADRs.

```sh
python research/structure/fetch_adrs.py corpus/cache/adrs 200 12
```

That configuration is stored in `research/structure/structure-adr.toml`.

## Golden tests

Golden tests keep Rust analysis results aligned with the reference implementation. The Python codebase in `research/` serves as that reference, and golden fixtures preserve its expected outputs.

Rule counts are verified by `crates/suikou-core/tests/golden_rules.rs` and document metrics by `golden_metrics.rs`, both checking against `tests/golden/expected/`. English test cases run without a dictionary, while Japanese test cases require `--features lindera-unidic`.

Because lindera and fugashi are distinct tokenizers, numeric checks allow defined tolerances. Permitted margins are an absolute error of 0.02 for ratios, 0.5 for densities per thousand characters, and an exact integer match for the M rules. Word and sentence counts allow a relative error of one percent because those totals vary with tokenizer definitions.

Discrepancies exceeding those tolerances are resolved individually rather than deferring to Python by default, with each rationale documented on the decisions page.

## Evidence requirements for new rules

**This is the strictest constraint in the repository.**

Unverified instincts about prose style rarely survive empirical testing. Many hypotheses proposed during this project's design failed measurement, matching the pattern observed in translationese research.

Proposing a new rule requires supporting evidence from one of the categories below.

- Empirical measurements across a human corpus and AI output whose ranges do not overlap.
- A cited clause from a published style guide.

Rejected metrics and their rejection reasons are logged under `[rejected]` in `corpus/baselines.toml`. **No metric listed there may be reintroduced.** That list includes the coefficient of variation of sentence length, the loanword ratio, and English participial clauses and nominalization. Despite frequent claims that these features signal AI-generated writing, corpus measurements show that every one of them overlaps the human writing range.

## Report size and token budget

Minimizing token usage imposes strict formatting limits so that every report remains small enough to read in one pass. Output generation enforces the constraints below.

- Each rule's explanation appears once per rule and is never repeated per line.
- Each rule prints at most `Report::MAX_POSITIONS_PER_RULE` positions.
- Metrics within their threshold are omitted entirely.
- Document metrics are expressed as instructions rather than numbers.

This design prevents iterative, layer-by-layer exchanges of a document between the linter and the model. Anyone modifying report formatting must verify that this workflow remains preserved and that the output still fits in one pass.
