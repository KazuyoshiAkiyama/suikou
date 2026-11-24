---
title: "Decisions"
weight: 30
doctype: "log"
---

This decision log records every design choice where viable alternatives existed, preventing subsequent readers from repeating completed investigations.

## D-01 Excluding document-wide metrics from textlint

Diagnostics in textlint attach to specific AST node positions, whereas metrics calculated across an entire document have no single node or line to reference. Most document-wide metrics detect scarcity—specifically scarce formal nouns, scarce demonstratives, and scarce simile markers—and an absence has no location in the source text. For example, a shortage of formal nouns characterizes the document as a whole rather than any individual line.

An early prototype registered these metrics as textlint rules attached to the Document node, which introduced an architectural flaw. Implementing them inside textlint confines analysis to JavaScript, where no tokenizer exposes word origin, preventing computation of the kango ratio. Discarding a validated metric for implementation convenience would invert the project's priorities.

Instead, the front-end CLI invokes textlint and merges textlint findings with its own document-level metrics. This front-end tool can also act directly as an MCP server, and a single combined report is easier for an agent to consume.

## D-02 Implementing production analysis in Rust and limiting Python to calibration

All metrics that passed validation can be written in pure Rust because lindera-unidic supplies every UniDic morphological feature, including word origin and inflected form.

Python was required only for participial clauses, nominalization, modifier stacking, and dependency distance, and empirical testing rejected all four. Metrics requiring deeper syntactic parsing proved increasingly sensitive to experimental setup, and their previously reported effect sizes failed to reproduce.

Work is therefore divided into two tiers: Python is reserved for exploratory search and threshold calibration, while Rust implements finalized metrics. A metric moves into Rust only after Python experiments confirm its validity.

Because Claude Code hooks execute the linter on every Write and Edit action, process startup latency dominates execution cost. Loading spaCy takes two to five seconds, ruling Python out of the production runtime.

## D-03 Processing Markdown line by line instead of using a parser

The reference implementation under `research/` scans text line by line, so matching its golden outputs requires using the same approach. Avoiding a Markdown parser library also reduces dependencies and prevents unexpected behavioral changes across library versions.

## D-04 Isolating the tokenizer behind a trait

Both the lindera API and the positional order of UniDic feature fields change across releases. Encapsulating all version-dependent code inside a single struct limits the affected code to a few dozen lines.

Excluding lindera from default Cargo features also enables running unit tests for rules and metrics without downloading a dictionary, accelerating local verification.

## D-05 Verifying UniDic feature order at startup

Pinning a dependency version does not prevent silent misbehavior after an upgrade. If the UniDic feature order shifts, the code reads word origin and inflected form from wrong indices and returns incorrect values without raising an error.

Both the kango ratio and the renyo count depend on those two fields, so silent misreads invalidate the resulting findings. Analyzing a known probe word at startup and terminating on any mismatch is preferable to producing erroneous diagnostics.

## D-06 Sharing one threshold table across plain and polite registers

Experiments across two models tested whether plain and polite registers shift metric values. The direction of change disagreed between models, revealing no consistent pattern; inter-model variance outweighs any register effect.

Register detection remains solely to govern register-specific rules such as noun-stopped sentences. Maintaining a second threshold table is unnecessary.

## D-07 Assigning info severity to formal nouns and kango ratio

Both formal nouns and the kango ratio vary substantially across document genres. Formal nouns approach the threshold in procedural documentation, whereas the kango ratio approaches native human values in explanatory prose and also varies by technical domain.

Because the Japanese human corpus consists mostly of expository books and lacks procedural texts, both metrics remain at info severity until genre-specific baselines exist. Once those baselines are established, both metrics will be raised to warning.

## D-08 Treating external effect sizes as non-transferable across tasks

Prior literature reported English participial clauses at 5.3 times the human rate, suggesting a threshold of nine to ten. Direct measurement instead overlapped with the human distribution, making the metric unusable, and nominalization behaved identically.

Experimental design accounts for this mismatch. The earlier study provided a human passage and requested a continuation in the same register, whereas this project prompted models to draft complete documents.

Published studies remain useful sources of candidate hypotheses, but their effect sizes do not transfer across task types.

## D-09 Setting maintainability standards above the human average

Among the evaluated human corpora, only the AWS S3 user guide consistently writes proper list lead-ins, whereas incomplete lead-ins predominate in the Linux kernel documentation. Matching the human average would nullify the maintainability layer.

This layer does not aim to strip AI writing traits. Its purpose is to produce documentation of higher structural quality than typical human-authored text.

## D-10 Dividing constraints between prompt instructions and linting

Comparing generations with and without a formatting prompt showed that explicit instructions eliminated formatting errors completely while leaving vocabulary and sentence structure unaffected.

A matching experiment with maintainability instructions showed that structural rules held completely under a short prompt, while time-dependent words reached zero only under a detailed prompt. Stating item counts, numbering headings, and manually numbering list items never occurred even without instructions.

Accordingly, `brief` includes only rules that prompting reliably enforces. Including ineffective instructions wastes tokens and risks over-correction.

As a secondary benefit, maintainability instructions alone improved quantitative document metrics. Requiring complete list lead-ins generated additional prose, moving both the prose ratio and the list ratio toward the human range.

## D-11 Generating the term list from project documents instead of shipping one

Distinguishing active figurative metaphors from conventionalized terminology is the hardest problem in the metaphor layer. Terms such as cache and handshake are established technical vocabulary rather than metaphors. Similarly, stating that a buffer behaves like a waiting room explicitly marks the comparison, whereas calling a buffer a waiting room leaves the figurative comparison unmarked.

Furthermore, once specific words gain notoriety as AI indicators, their frequency drops rapidly; terms publicized early in 2024 declined immediately afterward. Any static shipped word list loses accuracy over time.

Building the term list directly from a project's own documents avoids this obsolescence. Beyond accommodating vocabulary differences across projects, local generation provides the structural solution to outdated word lists.

## D-12 Correcting the kango ratio test expectation

The `kango_ratio_counts_only_content_words` test failed on the input 設定を変更すること because the implementation returned 0.5 while the test asserted a value greater than 0.5.

Although `TASKS.md` instructs fixing the implementation when a test fails, this case is an exception. The test comment counted three content words—設定, 変更, and こと—but the `FakeMorphology` specification also registers する as a verb, resulting in four content words. Because the metric specification includes every content word with origin 和, 漢, 外, or 混 in the denominator, する counts as well, making 2/4 the exact value required by the specification. Changing the implementation to pass the test would violate that specification.

Updating the expected value to exactly 0.5 preserves the test's actual purpose of verifying that particles are excluded from the denominator. Including the particle を in the denominator would yield 0.4, keeping both cases clearly distinguishable.

Implementation code is corrected when it diverges from the specification; test expectations are corrected when they diverge instead.

## D-13 Reporting every occurrence for M2 and M6

The reference implementation counts every match on a line, whereas the Rust implementation initially emitted at most one finding per block. On `tests/golden/input/en_maintainability.md`, where a single line contains two listed words, the reference returned 2 while Rust returned 1.

Per-occurrence counting was adopted to satisfy a core project constraint. When a linter reports only one of two violations on a line, editing the document to resolve that single report causes `check --quiet` to fail again on the remaining violation, defeating the goal of avoiding iterative fix-and-recheck cycles.

The previously unused `column` field on `Position` indicates that per-occurrence reporting was the original intent. Recording a column number for each match distinguishes multiple findings on the same line.

Rule M7 anchors to line endings and can trigger at most once per line, so this change does not affect it.

## D-14 Comparing word and sentence counts using relative error

The comparison policy allowed an absolute tolerance of 0.02 for ratios, 0.5 for densities, and exact equality for M rules, none of which suited word counts.

On `en_prose.md`, the reference reported 445 words while Rust reported 444. The difference stems from `trade-off`: spaCy splits it into three tokens and discards punctuation to yield two words, whereas whitespace splitting yields one word. The file contains exactly one hyphenated compound, accounting for the entire discrepancy.

This gap reflects different tokenizer definitions rather than an implementation error, and enforcing exact equality would require incompatible definitions to produce identical counts. A relative error tolerance of one percent detects systematic drift while accommodating minor tokenization differences.

## D-15 Reconciling formal noun scope between specification and reference implementation

The metric specification defines vocabulary metrics—including formal nouns—over prose combined with list-item bodies, and the Rust implementation follows that specification.

By contrast, the reference script `research/jametrics.py` reports demonstratives for both prose alone and prose plus list items, but reports formal nouns over prose alone.

On `tests/golden/input/ja_maintainability.md`, prose-only calculation yields 14.98 while prose plus list items yields 15.77. The 0.79 difference exceeds the 0.5 tolerance allowed for densities.

The Rust implementation must remain unchanged for the reason given in the specification: some models place nearly all content inside bullet lists, so restricting measurement to prose discards more than eighty percent of their text.

The discrepancy belongs to the calibrated threshold instead. Both the human range of 11.095 to 11.163 and the D5 threshold of 8.5 were derived under the prose-only definition. Including list items increases the computed value, causing D5 to trigger less readily than during calibration, which further supports the D-07 decision to keep D5 at info severity.

This discrepancy remains open pending recalibration. Recalibrating requires adding the prose-plus-list calculation to `research/jametrics.py` and re-evaluating the corpus.

## D-16 Shipping a single binary with an embedded dictionary

Evaluating distribution formats began by measuring artifact sizes with and without the dictionary.

| Artifact | Size |
|---|---|
| Binary with embedded dictionary | 191 MB |
| Binary with embedded dictionary (xz-compressed) | 26 MB |
| Binary without dictionary | 1.1 MB |

Running `strip` produces no meaningful size reduction because dictionary data, rather than debug symbols, accounts for nearly the entire file.

Storing the dictionary externally and fetching it on first run is feasible; lindera 6.0.0 offers `load_dictionary_from_path`, which reads directly without copying. Embedding was chosen nonetheless because reliability is critical for a tool invoked from hooks on every Write and Edit, where each added failure mode lowers dependability.

An external dictionary introduces three failure modes: missing, corrupted, or version-mismatched files. For example, an upgrade that swaps the dictionary causes the binary to read features from wrong positions; even when self-checks detect the mismatch, the user cannot proceed. Requiring no network access after installation is better suited to hook execution.

Distribution uses one archive per operating system. Combining all platforms into a single archive reaches 78 MB compressed and forces every user to download binaries for unused platforms.

Release targets are x86_64 Linux on musl, Apple Silicon macOS, and x86_64 Windows. Building Linux binaries against musl eliminates glibc version dependencies, preventing hook failures on older distributions or inside containers. Intel Mac and ARM Linux are excluded, and users on those platforms build from source.

## D-17 Keeping the dictionary Cargo feature disabled by default

Although `suikou-cli` gained the `lindera-unidic` feature, that feature remains disabled by default. Enabling it by default would include the dictionary in `cargo test --all`, contradicting the D-04 requirement that core logic remain testable without a dictionary.

Instead, the release workflow enables the feature explicitly and executes the binary to confirm dictionary availability before packaging, while a dictionary-free build fails immediately inside `morphology::load` whenever Japanese analysis is requested.

Silently returning empty tokenization output is unacceptable. Empty tokenization reduces the kango ratio, renyo count, and formal noun count to zero, and because zero is below the thresholds, an inconsistent mix of triggered and untriggered rules can falsely report a clean document.

Output from `suikou --version` distinguishes builds with and without the dictionary so users can identify their binary when investigating why Japanese text went unanalyzed.

## D-18 Verifying release artifacts by running dictionary analysis

Testing the release workflow locally under `act` exposed a packaging defect: the produced binary was 1.2 MB while `--version` reported an embedded dictionary, which was impossible given the 191 MB dictionary size.

Dead-code elimination caused the omission. Because every subcommand still executed `todo!()`, no reachable path called `morphology::load`, allowing the linker to discard the embedded dictionary. Adding a reachable call site increased the binary to 191.5 MB and produced real morphological analysis.

The smoke test was also inadequate. Because `--version` branches at compile time via `#[cfg]`, it confirms only that the feature flag was enabled, not that the dictionary actually exists in the linked artifact, allowing a dictionary-free binary to pass.

A `selftest` subcommand loads the dictionary and analyzes a test sentence. Verifying actual artifact behavior rather than build settings follows the same principle as verifying feature order at startup.

Giving `morphology::load` an active caller also allowed removing the dead-code suppression in `crates/suikou-cli/src/morphology.rs`. Users additionally gain a direct command to confirm that their binary matches its reported capabilities.

## D-19 Preserving original line numbers across preprocessing

Running `suikou check` against this repository's own documentation revealed that reported line numbers pointed four lines above the actual text, matching the exact four-line length of the stripped front matter.

Preprocessing deleted front matter and fenced code blocks completely, shifting every subsequent line upward and breaking alignment with the original file.

Preprocessing therefore preserves the exact line count by replacing each removed block with the same number of empty lines. Because blank lines are excluded from counting, non-empty line counts and the list ratio remain unchanged.

Linter diagnostics must reference exact line numbers in the user's source file; accurate findings reported at shifted positions cannot be used effectively.

## D-20 Building documentation as an English-default Hugo site

Documentation files moved from the top level of `docs/` into `docs/content/{en,ja}/`, with English set as the default language.

Rather than importing an external theme, minimal layouts are maintained directly in the repository. Keeping layouts internal minimizes dependencies and extends the project's version-pinning policy to documentation.

Maintaining a single canonical source per page also prevents content duplication between the repository and the published site.

## D-21 Controlling brief detail via a CLI flag rather than a profile field

Although the design document outlines storing brief verbosity inside a profile, the implementation omits that option and controls detail exclusively through `--detail concise|balanced|detailed`.

The Profile struct is dedicated to holding metric thresholds. Adding a brief-formatting field would assign the struct two unrelated responsibilities: calibrated threshold values and an output-format selector. Combining both kinds of data would make the scope modified by `suikou baseline` ambiguous, so Profile remains restricted to thresholds.

The CLI flag also expanded to a third level beyond the initial sketch. TASKS.md originally proposed concise and detailed, and the implementation added balanced as an intermediate tier once structural rules proved expressible in single sentences while time-dependent words still required a complete list every time. Concise abbreviates that list-plus-sentences format further, whereas detailed adds the rationale behind each rule.

Profile-based detail configuration remains an open question. Moving detail selection into profiles is deferred until a concrete demand for per-project defaults appears.

## D-22 Computing baseline thresholds with the Tukey outlier fence

Implementing `suikou baseline` required a method for selecting threshold values without AI reference data. Although `docs/content/en/design.md` sets each threshold between the upper bound of the human range and the lower bound of the AI range—such as 0.60 for `ja.mattr100`—baseline calibration inspects only a human corpus and has no visibility into the AI range.

Calibration therefore derives a boundary strictly from the human distribution by identifying what that distribution treats as an outlier.

The boundary uses the Tukey outlier fence: the interquartile range equals the difference between the third and first quartiles, and the fence is placed `FENCE_K` times that range beyond the nearer quartile. The coefficient 1.5 comes from Tukey's standard outlier definition and offers a stronger statistical justification than an arbitrary margin invented for this project. A looser coefficient of 3.0 defines Tukey's "far out" threshold, which this project avoids because the small sample counts typical of baseline runs would place that fence too far away to be meaningful.

Quartiles are computed via the hinge method: split the sorted samples at the median into two halves and take each half's median as Q1 and Q3, excluding the middle element from both halves when the sample count is odd. Although linear-interpolation percentiles also exist, the hinge method is simple enough to verify manually in unit tests.

Any metric with fewer than four samples remains uncalibrated. With three samples or fewer, one half contains only a single value, causing the interquartile range to reflect that lone element rather than distribution spread. Substituting zero or a placeholder would conceal missing evidence from profile readers, so uncalibrated metrics are omitted from the output and the reason is printed to standard error.

The initial implementation contained a defect for below-direction metrics, which are ratios or per-thousand counts and are non-negative by definition. When samples cluster tightly, Q1 - 1.5*IQR can become negative. Calibrating on this project's own Japanese documentation (`docs/content/ja/*.md`, `README.ja.md`, `CLAUDE.md`, `TASKS.md`) produced fences of -4.96 for `ja.demonstrative_per_1k` and -4.98 for `ja.keishiki_meishi_per_1k`. Because a non-negative metric can never satisfy the below check (`value < threshold`) against a negative threshold, the rule never triggers. The output still parses as valid TOML, so `suikou check` reports zero findings—an outcome easily mistaken for a valid calibration.

Clamping negative fences to zero does not fix the defect. Because ratios and densities cannot be negative, a threshold of zero still requires a value strictly less than zero and remains unreachable, replacing one unreachable threshold with another. Instead, calibration treats any below-direction fence at or below zero as a failure, matching the insufficient-sample behavior by omitting the metric from output and logging the reason to standard error.

Above-direction metrics do not receive parallel treatment. Whereas all metrics share zero as a lower bound, their upper bounds differ: ratios have a maximum of 1, while per-thousand counts have no fixed upper limit. Adding per-metric upper bounds would introduce metric-specific logic into otherwise generic calibration code, so boundary rejection remains limited to the below direction.

Baseline calibration preserves `guidance`, `direction`, and `severity` unchanged from the bundled oss profile. Regenerating that metadata on every run would leave baseline output outdated whenever oss.toml changes, so calibration modifies only `value`.

Because licensing excludes the corpus downloaded by `corpus/fetch.sh` from the repository, reproducing `corpus/baselines.toml` and confirming the order of magnitude of the bundled profiles remain unverified. Documents under `tests/golden/input/` cannot substitute because no metric there meets the four-sample minimum. TASKS.md records this verification as open until `suikou baseline` is run against a full human corpus and compared with `corpus/baselines.toml`.

End-to-end behavior has been confirmed for a below-direction threshold that survives calibration. Calibrating against the Japanese documents listed above retained `ja.prose_ratio` with a fence of 0.69, and running `suikou check --profile` with that profile triggered the rule on a document dominated by bullet lists while remaining silent on a prose-only document, both verified against actual command output.
## D-23 Implementation trade-offs in the textlint preset

Implementing `packages/textlint-rule-preset-tech-maintainability` (T7) involved multiple independent design choices. Each decision is documented separately below so that the reasoning for one does not mix with another.

**Standard textlint configuration cannot display `maint/` as the rule prefix.** The task specification requested rule IDs matching the Rust implementation, such as `maint/list-lead-in`. Inspecting textlint's preset loader revealed that the printed rule prefix derives from the configuration key in `.textlintrc` after removing its leading `preset-` segment. Enabling the package in the standard way via `"preset-tech-maintainability": true` therefore outputs all findings with the `tech-maintainability/` prefix. Because a configuration key must begin with `preset-` for textlint to treat it as a preset, and that same `preset-` segment is unconditionally removed before forming the prefix, standard loading provides no way to produce `maint/`.

Each individual rule name following the prefix, such as `list-lead-in`, matches the Rust side as required. Only the prefix adheres to textlint's loader convention; altering it would require renaming the package to something like `textlint-rule-preset-maint`, creating a mismatch with its directory name and causing greater confusion than the prefix difference itself. Keeping the package name aligned with its directory is preferable, and the README explicitly notes that findings print under `tech-maintainability/`. Golden-parity tests avoid this loader limitation altogether by invoking `@textlint/kernel` directly and manually registering every rule under `maint/`.

**Rules match against the raw Markdown source instead of the textlint AST.** textlint parses Markdown into structural nodes like paragraphs and headings, merging a soft-wrapped paragraph across multiple lines into a single Paragraph node. By contrast, `markdown.rs` generates one block per line and keeps soft-wrapped continuation lines distinct. This difference appears on lines 14 and 15 of `en_maintainability.md`, where a single sentence spans two lines: evaluating joined node text would allow a regular expression to match across words that the Rust analyzer never combines.

Matching against textlint's Paragraph node text was weighed against porting the line-oriented block builder to JavaScript and using textlint solely to supply the raw source text. Although the AST approach resembles a conventional textlint rule more closely, finding counts diverge whenever node granularity disagrees with line granularity. The implementation therefore uses the ported block builder: `src/lib/blocks.js` reproduces the block builder and regular expressions from `markdown.rs`, and every rule retrieves the raw file content via `context.getSource(node)` and runs it through this local builder before checking any rule. textlint handles file reading, configuration handling, and finding output without parsing document structure for the rules.

As a trade-off, reported line and column coordinates are calculated on the preprocessed string rather than the original file. Coordinates can diverge from the raw source wherever escape sequences were decoded or inline markup was removed. To compensate, `src/lib/position.js` scans the target line from left to right for the matched text and estimates the corresponding character position in the raw source. This approximation ensures that findings are never discarded rather than guaranteeing exact column numbers; parity checks against `tests/golden/` compare finding counts only and do not evaluate position accuracy.

**IPADIC merges UniDic's four taigen categories into a single part of speech.** For Japanese, M1 and M5 check whether the final token of a line or list item is a taigen, meaning a nominal word capable of ending a sentence independently. suikou-core relies on UniDic and classifies a token as taigen when its top-level part of speech is noun, suffix, pronoun, or adjectival noun stem. kuromojin, the standard morphological analyzer for textlint, uses IPADIC, which lacks that four-way division at the top level.

Direct inspection of kuromojin 3.0.1 confirmed that words UniDic tags as suffix, pronoun, or adjectival noun stem all appear in IPADIC with noun as the top-level part of speech and pronoun, suffix, or adjectival-noun stem as the subcategory. Specifically, "これ" (this) returns noun/pronoun and "静か" (quiet) returns noun/adjectival-noun-stem. Under IPADIC, UniDic's four-way check therefore reduces to testing whether the top-level part of speech is noun, which forms the basis of `isTaigen` in `src/lib/morphology-ja.js`.

Checking whether a verb or auxiliary ends in the ren'youkei required equivalent verification. UniDic appends subtypes such as "連用形-一般", whereas IPADIC reports the bare string "連用形" without a subtype, as verified on "使い" and "し". Consequently, the Rust check `cform.startsWith("連用形")` ports over unchanged.

**Diagnostic messages match the document language, whereas the Rust analyzer outputs Japanese only.** In suikou-core, `finding()` returns Japanese text even when analyzing English files, as verified by running `suikou check` on `en_maintainability.md`.

Users of the textlint preset who write exclusively in English cannot act on Japanese diagnostics. While the detection logic and regular expressions remain identical to the Rust implementation, the preset selects either a Japanese or English message from `src/lib/messages.js` according to the detected document language. Because golden-test parity evaluates finding counts without comparing message strings, this difference does not affect the parity requirement.

**The test suite uses `node:test` directly instead of textlint-tester.** textlint-tester expects Mocha's global `describe` and `it` functions; when those globals are missing, its fallback invokes the test callback without awaiting its result. Because that fallback ignores the Promise returned by each test case, running under `node --test` allowed assertions to report success before the asynchronous lint run finished.

Between adding Mocha as a dependency to retain textlint-tester and building a minimal wrapper over `@textlint/kernel`, the zero-dependency option was chosen. `test/support/lint.js` wraps the kernel concisely, registers all rules under the `maint/` prefix, and awaits results directly, integrating with `node --test`'s native async test functions without an adapter.

**Overlap with `no-ai-colon-continuation` from `@textlint-ja/textlint-rule-preset-ai-writing` remains unsuppressed.** That rule likewise relies on kuromojin to detect whether text preceding a colon ends with a predicate, overlapping with the Japanese check in M1.

The rules serve different purposes: that rule flags a stylistic pattern typical of AI-generated prose, whereas M1 detects list lead-ins that break structurally when items are added or removed. Both rules may still report on the same line, and no suppression mechanism links them. Users who enable both presets are aware of both and can decide which diagnostic applies to their document, whereas automatically hiding one would remove useful information. The README documents this overlap instead.

## D-24 Term extraction segments words by structure rather than part of speech alone

Japanese forms compound nouns from adjacent nouns. Because UniDic assigns bound suffixes such as 素 and 性 a separate part-of-speech tag distinct from noun, collecting nouns alone split 形態素解析 into 形態 and 解析 and truncated 保守性 to 保守. Running extraction against this repository's own documentation revealed these splits.

Including the suffix tag alongside the noun tag resolves those splits, but it also permits counters such as つ to start a standalone compound. UniDic provides no finer tag separating counters from derivational suffixes, making a counter exclusion list impractical. Instead, the extractor enforces a structural constraint: a suffix may extend a sequence only when a noun already precedes it, so a suffix separated by a numeral—such as つ in 3つ—never initiates a term on its own.

UniDic records whitespace only as a token boundary rather than emitting a token for the space itself. Concatenating adjacent Latin-script tokens without whitespace produces words absent from the source text; running extraction on `design.md` merged several occurrences of Linux kernel into Linuxkernel. The assembler therefore inserts a space between adjacent Latin-script tokens while omitting spaces between a Latin-script token and a Japanese token, such as 仕様 attached to API, matching standard Japanese orthography.

To filter out generic words, the extractor reuses `KEISHIKI_MEISHI` from `metrics/ja.rs` rather than maintaining a separate stoplist. If a generic word enters the glossary, the only consequence is that it loses its exclusion from the metaphor check, which carries a minor cost. Omitting a legitimate domain term is more harmful, so the design favors retention.

Testing extraction on this repository's documentation exposed another problem: single-character words occupied most of the top of the word list. Out of 395 extracted words, 54 consisted of one character, and the top twenty entries included 文 (50), 形 (49), 語 (48), 指示 (43), 値 (36), 地 (28), 数 (27), 版 (22), 行 (22), and 層 (20). Here, 地 is the remainder after の split 地の文 into two tokens, while the others are ordinary Japanese words that occur frequently across general writing rather than identifying any specific domain.

Manually listing words to exclude was rejected because curating such a list relies on guesswork. Instead, the filter applies a structural rule based on assembled word length, discarding any candidate that is one character long. Removing single-character candidates is standard in Japanese term extraction because multi-character terms convey specific meaning, whereas single-character words are minimal units with broad senses that rarely identify a domain. A two-character word formed from a single token, such as 文書, passes unaffected. After applying this length filter, the top extracted terms for this repository's documentation became 指示, 規則, 指標, 人間, 文書, 日本語, 指摘, 閾値, 辞書, and 実装, with zero single-character entries.

English extraction operates without a part-of-speech tagger or sentence splitter. It identifies candidates from all-uppercase words, capitalized words appearing mid-sentence, and inline code consisting of a bare identifier. Table rows are skipped during scanning because table cells do not form sentences, preventing sentence-initial detection from triggering and allowing repeated column labels to exceed the occurrence threshold; testing on this repository's English documentation showed table cell values entering the glossary in this way.

Headings beginning with an identifier present a similar edge case. In this document's own heading, D-04, The tokenizer hides behind a trait, treating D-04 as the initial word made the following word The appear mid-sentence. The fix maintains sentence-initial status across consecutive all-uppercase words at the start of a text segment.

Headings containing two adjacent identifier-like words introduce an additional edge case. In this document's heading D-13, M2 and M6 count every occurrence, the absence of punctuation between D-13 and M2 merges them into a single token, D-13 M2, costing M2 that occurrence and potentially reducing its count below the occurrence threshold. Because omitting a valid term is less harmful than fabricating a word absent from the source, this behavior remains a documented limitation. Fixing it would require a rule deciding when to merge adjacent identifier-like words, and no rule has been found that separates such heading prefixes without breaking multi-token product names like AWS S3.

The default minimum occurrence threshold is two. A word appearing once cannot be distinguished from incidental phrasing, and terminology consistency checks require a word to occur more than once. This default follows from those two requirements rather than from empirical measurement.

## D-25 Hand-written JSON-RPC for MCP and omission of a daemon

T9 required decisions on MCP server implementation and daemon support.

**The MCP server relies solely on `serde_json`.** The alternatives were adding a dedicated crate such as `rmcp` or implementing `initialize`, `tools/list`, and `tools/call` manually.

The server handles only those three methods and does not use resources, prompts, or server-initiated notifications. Adding a dedicated crate would introduce a versioned dependency for unused features, conflicting with this repository's policy of pinning exact versions with `=` and rerunning `verify_schema` on every update. Because both crates already depend on `serde_json`, implementing the protocol directly adds zero dependencies. In addition, the stdio transport consists of one JSON-RPC message per line, making a hand-written transport straightforward and low-risk. The server resides in `crates/suikou-cli/src/mcp.rs` and delegates exclusively to `check::analyze` and `brief::run`, keeping CLI and MCP evaluation unified.

**No daemon is implemented.** The original rationale for a daemon was paying the dictionary load cost only once. Measurements in T2 recorded 0.68-0.70 ms in release builds, contradicting that rationale, and TASKS.md required benchmarking process startup itself before finalizing the decision. In the benchmark, a release build (`--features
lindera-unidic`) ran `suikou check` on a short document fifty times to record wall-clock duration per invocation,
alongside fifty runs of `suikou --version` as a baseline. The table below reports the measurements collected on WSL2.

| Command | Mean duration | Max duration |
|---|---|---|
| `suikou --version` | about 1.5-1.6 ms | about 1.9-2.4 ms |
| `suikou check` (dictionary loaded, short document) | about 3.7-3.9 ms | about 4.3-4.7 ms |

Because `suikou --version` never loads the dictionary, its runtime measures process startup alone: reading the roughly 200 MB binary, linking it, and parsing command-line arguments. The extra time in `suikou check` amounts to a couple of milliseconds, covering dictionary loading (0.7 ms) plus analysis of a short document. Both figures are negligible compared with the multi-second duration of an LLM turn, which is the context where hooks execute on Write and Edit actions. Running a daemon would introduce operational complexity—managing sockets or named pipes, detecting file modifications, and serializing concurrent requests—for latency savings too small to justify it.

The `suikou daemon` subcommand remains available and prints these measurements together with this decision rather than performing work. Keeping the subcommand instead of deleting it ensures that anyone who invokes it directly sees why no daemon runs.

## D-26 Detecting off-register native words by reference corpus comparison

When translating English into Japanese, generative models sometimes substitute a native Japanese word for a Sino-Japanese equivalent—rendering 確認する as 確かめる or 必要である as 要る. This phrasing feels overly casual for technical documentation and slows reading speed.

Rather than building a fixed substitution table of word pairs—which would require enumerating stylistic differences across every technical domain—the rule establishes register expectations by comparing word frequencies against a corpus of professional Japanese translations.

Corpus counts support this design. Within Kubernetes alone, 使う occurs 320 times in the Sonnet translation versus 33 times in the professional translation, which would seem to justify rewriting 使う to 使用. Expanding the corpus to include Kubernetes, MDN, and Vue yields 461 occurrences of 使う in professional translations. Other candidate pairs—次 versus 以下, 仕組み versus メカニズム, and 既定 versus デフォルト—failed for the same reason; MDN uses 既定, demonstrating that lexical conventions vary by domain.

Findings are evaluated using Dunning's (1993) log-likelihood ratio to test statistical significance and Hardie's (2014) Log Ratio to measure effect size. Combining these two statistics is necessary because on a corpus of this scale, a slight increase in an ordinary word's frequency can reach statistical significance on its own.

The minimum Log Ratio threshold was selected empirically by comparing twelve held-out professional pages against this repository's Japanese documentation. A threshold of 4 flagged 9 words across the professional pages, a threshold of 6 flagged 1 word, and a threshold of 7 flagged none. Because the design treats any warning on human-written professional text as a false positive, the threshold is set where professional pages produce zero warnings, accepting missed detections as a trade-off.

Constructing the reference corpus enforces additional criteria. Tutorials are excluded because the Rust Book translation contains 確かめる 36 times, which would improperly admit it into the reference; this aligns with measured evidence that document genre affects metrics more strongly than register. Furthermore, a word must occur in at least two domains to qualify, because terms restricted to a single domain reflect domain-specific vocabulary rather than general technical usage.

Project-specific allowances are configured in `.suikou/register-allow.toml`. Since the web-documentation reference corpus contains limited vocabulary for measurement and arithmetic, individual projects supply those missing terms locally—the same pattern used in D-11, where the glossary is derived from the project's own documents.

## D-27 Revising the guidance for the kango ratio

D7 previously advised writers to replace kango with native Japanese verbs. Empirical measurements contradict that advice.

The table below compares content-word origin proportions between professional translations and this repository.

| | Native | Sino-Japanese | Loanword | Mixed |
|---|---|---|---|---|
| Professional translation | 0.384 | 0.329 | 0.280 | 0.007 |
| This repository | 0.470 | 0.463 | 0.050 | 0.017 |

Professional translators do not reduce the kango ratio by substituting native verbs. Instead, they rely on established loanwords, and their share of native words is actually lower than in this repository. The revised guidance therefore directs writers to adopt established loanwords and avoid combining consecutive kango nouns.

The previous advice caused tangible harm. Applying it during a rewrite converted 検証 into 確かめ, 制約 into 縛り, and 抽出 into 取り出し, degrading the document register. The rule introduced in D-26 exists specifically to detect that register degradation.

Reducing a metric to zero is not an objective. D7 uses the info severity level to indicate advisory status, and enforcing it as a pass-fail check was the initial error. Documents in this repository must clear all errors and warnings, whereas info-level diagnostics serve strictly as advice.

## D-28 Failure of the register rule to generalize to English

Attempts to adapt the D-26 method to English failed, so no English register rule ships.

The first experiment compared target documents against an English reference corpus. A frequency table built from 652 pages and 510,000 words of English Kubernetes, MDN, and Vue documentation was evaluated with the same statistical tests. Even with the threshold raised to 9, fourteen held-out professional English pages still produced 20 findings. Top-ranked words on both sides consisted of domain terminology: portworx, cinder, and ebs in the professional sample, and prose, corpus, and metric in this repository. Filtering to words occurring across all three domains still left topic-specific vocabulary such as operations rather than register indicators.

This failure results from differences in lexical structure. In Japanese, filtering for native word origin removes domain terminology almost automatically because technical vocabulary is predominantly Sino-Japanese or borrowed as loanwords. English provides no equivalent etymological division: any text on a given subject uses its topic vocabulary much more frequently than a general corpus does, so frequency counts alone cannot distinguish topic vocabulary from register.

The second experiment controlled for topic by having the model translate professional Japanese translations back into English and comparing the output against the original English across 36 pages and 30,000 words per side. Zero words reached statistical significance: against a critical value of 15.13, the highest values were your at 13.2 and will at 11.1, and the original human text uses both more often than the model output does, opposite to the target pattern. Under the equivalent Japanese setup, 使う occurred 320 times versus 33, yielding a log-likelihood ratio of 320.

This negative outcome across two separate experiments parallels D-08. Just as effect sizes do not transfer across tasks, a detection method validated in one language does not automatically transfer to another language.

These experiments tested lexical choice only. MATTR, the prose ratio, and simile markers evaluate separate properties and remain in place.

Both experiments are reproducible under `research/lexshift/`, where `build_reference_en.py` constructs the English frequency table and `translate_back.sh` generates the back-translations. Retaining the negative results prevents anyone from repeating these experiments.

## D-29 Preferring katakana loanwords over Sino-Japanese equivalents via corpus counts

D-26 rejected a fixed word-pair table because usage conventions vary across domains. That position is revised specifically for pairs consisting of a Sino-Japanese word and a katakana loanword: a single counter-example no longer disqualifies a pair, and the form favored across the combined domains takes precedence.

Several automated methods for discovering synonym pairs were evaluated and discarded. Pivoting through an English term to link co-occurring words within aligned paragraphs identifies collocations rather than synonyms, generating pairs such as 永続 with ボリューム and 番号 with ポート. Lift and the Dice coefficient produced the same failure mode. Lexical substitution involves alternation rather than co-occurrence, and the paragraph level is too coarse a unit to detect it.

Candidate proposal and empirical selection were therefore decoupled: a human lists candidate pairs and corpus measurements decide which ones pass, following the workflow established in D-26. Out of 97 candidates, 53 passed.

To be retained, the katakana loanword must occur at least two to one relative to the Sino-Japanese word, the combined pair must reach twenty occurrences, and the Sino-Japanese form must stay at or below sixty occurrences. Every pair in the table satisfies all three criteria.

The upper limit of sixty excludes polysemous words. Corpus counts show professional translators using 対象, 対応, 状態, and 場合 heavily, often in senses distinct from the corresponding loanword. For example, this repository uses 対象 to refer to the scope a metric covers rather than an object. Limiting the table to Sino-Japanese words that appear infrequently in professional text preserves the reliability of each pair.

Any target word immediately preceded by a noun is skipped as part of a compound, preventing erroneous rewrites of 版 inside 英語版 or 第3版.

Projects can exempt words used in a distinct sense via `.suikou/register-allow.toml`. This repository exempts 記録, 一覧, 比率, 経路, 分岐, and 実体; here, 比率 denotes a ratio such as the kango ratio rather than a rate.
## D-30 Enforce document structure and plan outlines before drafting

Sentence-level checks cannot repair a structurally flawed document. A missing section, a broken logical sequence, or a paragraph carrying three distinct claims cannot be identified by reading one sentence at a time.

Structural rules apply equally to English and Japanese. Required sections, section sequence, and paragraph scope belong to the organization of an argument rather than to the vocabulary of a specific language. Corpus statistics confirm this consistency: paragraphs containing seven or more sentences occur at 0.27 percent in professional English and 0.59 percent in professional Japanese. Unlike D-28, where a register check failed to transfer across languages, structural patterns hold in both.

Every doctype and its mandatory sections derive from published external standards rather than ad-hoc invention. RFC 7322 specifies mandatory sections in a recommended sequence, while individual doctypes draw from the Rust RFC template, the Kubernetes Enhancement Proposal, Michael Nygard's Architecture Decision Record, and Diátaxis. The maximum paragraph length of six sentences comes from the Google developer documentation style guide, preserving the rule against arbitrary, ungrounded checks.

Only omitting a required section produces an error; out-of-order sections, overly long paragraphs, and empty sections produce warnings. Without a required section, a document fails to answer the primary question readers bring to it, whereas section sequence and paragraph length guide presentation rather than acting as absolute prohibitions.

Heading matching inspects only the highest heading level in the document body. An initial level-one heading functions as the document title rather than a section and is excluded. Inspecting deeper heading levels allows a subheading inside one section to falsely match another section and distort the order check, an error previously encountered in this repository's English design document.

Only the first section can be satisfied by an opening paragraph without an explicit heading. Both RFC 7322 abstracts (placed between the title and the table of contents) and README files follow this convention; inserting a mandatory `## What it is` heading degrades readability compared to plain opening text.

### Outline approval precedes drafting

Running `suikou plot` creates a plan template under `.suikou/plans/` containing each section heading, the question that section answers, and a blank line for its one-line claim. If a section's core claim cannot be stated in one line, the reasoning is unfinished, and the plan surfaces that gap before prose is drafted.

Plan files stay outside version control because they record intermediate planning rather than final deliverables. CI therefore cannot inspect them, and plan verification runs only locally and inside an agent loop. That limitation is intentional: structural defects should be resolved before drafting begins rather than blocked at a pre-release check.

### Guidance alongside violation detection

Previous rules worked solely by flagging violations for removal. Listing violations indicates what to omit, not what content to supply.

To offer constructive direction, each template includes the question a section answers and the material expected inside it, presented before drafting through `suikou plot` and `suikou brief --doctype`. All guidance text lives in a single location, `structure::writing_rules`, so drafting instructions stay synchronized with lint diagnostics.

Default token output does not increase because the template is emitted only when `--doctype` is passed.

## D-31 Empirical testing removed two required sections from the how-to doctype

Each doctype's required sections originated from a published framework rather than invention. Based on Diátaxis, the how-to doctype initially required three sections: prerequisites, steps, and verification. Two of those three requirements failed immediately when tested against professional documentation.

Evaluating `howto` on 120 Kubernetes task pages flagged `Steps` as absent on all 120 pages and `Verify` as absent on 113 pages. Not a single page passed cleanly.

Heading frequency counts explain why. Among H2 headings across those pages, 111 use `{{%/* heading "prerequisites" */%}}` and 84 use `{{%/* heading "whatsnext" */%}}`. The remaining headings are task-specific titles such as `Create a namespace` or `Edit a Secret`, each naming the concrete operation performed.

In a how-to guide, the procedural steps constitute the document body rather than a single labeled section. Diátaxis describes a how-to guide as a sequence of steps, not as a document containing a literal `Steps` heading. Mandating that generic title would erase task-specific actions from the outline, contradicting the principle that headings alone should convey the progression of the document.

As a result, `steps` and `verify` were removed from the required list, while `prerequisites` remains mandatory.

### Re-evaluation exposed two additional defects

During the initial run, all 120 pages passed the `prerequisites` requirement—not because every page contained prerequisites, but because the rule was too permissive to catch omissions.

Lead-paragraph substitution had been determined solely by position, so any document opening with a paragraph automatically satisfied its first mandatory section regardless of meaning. Running `howto` against 120 concept pages reported zero findings, showing that the doctype failed to differentiate document categories.

Eligibility for lead-paragraph substitution is a property of the section rather than its ordinal position. Only sections whose purpose is introducing the subject carry `lead_ok`; prerequisites and context require explicit headings. After this change, 117 of the 120 concept pages report a missing required section, demonstrating valid discrimination.

Synonym matching previously checked only the document's declared language. Because Japanese Kubernetes translations retain the `{{%/* heading "prerequisites" */%}}` shortcode verbatim, matching solely against Japanese synonyms flagged 110 of 111 pages as missing the section. Matching was updated to check synonyms from both languages, since a heading's language may differ from the surrounding prose.

### Violation rates after fixes

| Corpus | Pages | Missing required section |
|---|---|---|
| English task pages | 120 | 9 (7.5%) |
| Japanese task pages | 111 | 6 (5.4%) |
| English concept pages checked as `howto` | 120 | 117 (97.5%) |
| Japanese concept pages checked as `howto` | 120 | 116 (96.7%) |

Both languages fall within the same range. The check flags nearly every mismatched document while reporting almost no violations on matching task pages, confirming that the doctype separates the two categories. Manual review of the flagged task pages shows that they genuinely omit `Before you begin`, and several declare `task` despite being explanatory articles (such as `Dependency on Docker explained`). Those cases represent deviations from Kubernetes's own template rather than linter false positives.

### Removed from validation, retained in guidance

The guiding question and scope notes remain inside the doctype definition. Both `suikou plot` and `suikou brief --doctype howto` continue asking how to perform the procedure and how readers confirm that it succeeded.

Automatable checks and pre-writing prompts are distinct sets. Explaining how to verify an outcome characterizes an effective procedure, yet detecting a heading named `Verify` does not reliably measure whether verification exists. Separating guidance from automated detection is the core principle of this decision.

### External validation limited to the how-to doctype

The required sections for `design`, `decision`, and `overview` have been tested solely against documents in this repository. Because those files were authored specifically to satisfy the doctype definitions, they provide no independent verification.

External benchmarking requires corpora with explicit document-type labels. Kubernetes defines `content_type` with values `task`, `concept`, `reference`, and `tutorial`, and only `task` maps to an existing doctype in this project. Diátaxis explanation and tutorial correspond to the other two categories, and additional doctypes will not be introduced until suitable external corpora are available for testing.

## D-32 The design doctype failed against its source Rust RFC corpus

The required sections of `design` were drawn from the Rust RFC template. Validating that claim required running the doctype against 200 RFCs from the top level of `text/` in `rust-lang/rfcs`, sampled downward from the highest RFC numbers where the template had stabilized. Evaluating a doctype against its source corpus provides the strictest possible test.

The initial run reported `Design` missing on all 200 RFCs, `Non-goals` on 199, and `Scope` on 198. Each of the three failures had a separate cause.

### Section name mismatch

The 200 missing-section reports for `Design` resulted from naming differences rather than an invalid requirement.

In the Rust RFC template, that material appears under `Guide-level explanation` and `Reference-level explanation`, occurring 157 and 148 times respectively across the corpus. Adding `Explanation` to the synonym list covers both headings, while `Implementation` and `Proposal` were added as widely used design-document section titles.

### Requiring sections absent from the template

Neither `Scope` nor `Non-goals` belongs to the Rust RFC template. Both originated in the Kubernetes Enhancement Proposal, meaning two templates had been combined into one requirement set.

Both sections were changed to optional while retaining their guiding questions and scope descriptions. As in D-31, `plot` and `brief` still ask authors to state boundaries and exclusions.

### Two parser defects

Heading indentation previously had no upper bound. Under CommonMark, four or more leading spaces denote an indented code block rather than a heading. A `#` inside an indented code block was misparsed as a level-one heading, making level one the top heading level of the document and hiding its actual sections. Heading indentation is capped at three spaces.

Fenced code blocks were previously stripped with a regular expression. When an outer fence enclosed a shorter inner fence, the expression closed prematurely at the inner fence and treated subsequent code as prose, causing lines like `# [dependencies]` to register as headings. A line-by-line scanner replaced the regular expression: a fence closes only on a line using the same marker character, at least as long as the opening fence, with no trailing content, matching CommonMark.

Both fixes were ported to the textlint preset, and both implementations report identical M-rule counts on real RFCs.

### Failure rate after fixes

After these corrections, 21 of the 200 RFCs (10.5 percent) report a missing section. Eighteen of those are not feature proposals—they are roadmaps, project group charters, team changes, and policy RFCs stored in `text/` that do not follow the feature template. The remaining three use bespoke headings such as `Proposed additions` or `Solution`.

Synonym additions stop at this point. Continuing to add custom headings would drive the failure rate toward zero by overfitting the sample rather than reflecting a standard; a heading qualifies for the synonym list only by appearing in a published template.

## D-33 The decision doctype passed validation and enabled order measurement

The mandatory sections for `decision` derive from Michael Nygard's Architecture Decision Record. Validation ran against 190 ADRs collected from 28 repositories via `research/structure/fetch_adrs.py`, which discovers records from the initial entry generated by `adr-tools` and excludes index files and templates that are not decision records.

| Rule | Out of 190 |
|---|---|
| Missing required section | 20 (10.5%) |
| Out-of-order sections | 0 |
| Long paragraph | 6 (3.2%) |
| Empty leaf section | 2 (1.1%) |

Unlike `howto` and `design`, this doctype held without modification. The difference lies in adopting a single template intact rather than combining multiple sources: `design` had blended the Rust RFC with the Kubernetes Enhancement Proposal, and `howto` had treated prose descriptions in Diátaxis as literal section headings, whereas `decision` follows the ADR structure directly.

### First measurement of section ordering

Checking section order requires at least two recognized doctype sections in the same document. Only 7 of 120 Kubernetes task pages contained two recognized sections, yielding no usable signal. In the ADR sample, 189 of 190 documents contain at least two recognized sections, and zero trigger an ordering warning.

Real-world records consistently follow the order of context, decision, and consequences, providing empirical support for keeping section order as a recommended check.

### Breakdown of the remaining 20 failures

Eleven follow MADR, which names its top-level headings `Context and Problem Statement` and `Decision Outcome` and nests consequences beneath `Decision Outcome`. Because matching checks only top-level headings, nested consequence sections are not detected.

Top-level-only matching remains unchanged. Inspecting deeper heading levels allows a subheading inside one section to falsely match another section and break the ordering check, as observed in D-31. Supporting MADR requires defining a separate doctype rather than searching nested levels.

Among the remaining nine records, several contain only `Status` and `Context`—omitting both the decision and its consequences, which constitutes a real structural gap—while others write `Proposal` instead of `Decision`.

Following the same boundary established in D-32, no additional synonyms are added.

## D-34 Evaluated natural-japanese rules yielded zero adoptions

`coji/natural-japanese` (MIT) is a writing skill for readable workplace Japanese comprising 28 detectors, a list of banned phrases, and a collection of translationese patterns. Evaluating whether any rule could transfer followed this repository's standard empirical process.

Under D-26, a rule qualifies for inclusion only when human and AI measurement distributions do not overlap, or when it cites a specific clause in a published style guide. Candidates may be compiled manually, while empirical measurement decides adoption.

### Measurement methodology

`corpus/cache/k8s/` pairs human and AI Japanese translations of identical source documents so topic differences cannot confound the comparison. The 798-document reference corpus was analyzed alongside them using scripts in `research/translationese/`.

### Results

| Candidate | Human | AI | Reference | Outcome |
|---|---|---|---|---|
| 「することができる」 | 2.43 | 0.85 | — | Reversed direction |
| Ten translationese patterns | near zero | near zero | — | Absent in technical prose |
| Conjunction starting a paragraph | 1.6% | 1.2% | 1.6% | No separation |
| Three consecutive identical sentence endings | 11.8% | 10.5% | 6.0% | No separation |
| Double negative | 0.16 | 0.11 | 0.16 | No separation |
| Three consecutive 「の」 | 3.67 | 3.03 | 2.84 | Higher in human text |
| 48-phrase catalogue | 1 at most | 5 at most | — | Below the G² threshold |

Except for the two percentage metrics representing paragraph share, all figures denote occurrences per 10,000 characters.

None of the candidates passed. Each rejected rule and its rationale are documented under `[rejected]` in `corpus/baselines.toml`.

### Why no metric separated human and AI prose

The banned-phrase catalogue was designed for essays and business prose. Expressions such as 「いかがでしたか」 and 「見ていきましょう」 never occur in technical documentation; out of 48 listed phrases, only 19 appear anywhere in the evaluated corpus. Consistent with D-28, language models produce different stylistic patterns depending on document genre.

Evaluating translationese requires attention to the baseline corpus. Because the human reference set consists of professionally translated English documentation, translationese structures are inherent to it, making it impossible to test whether prose reads as translated in absolute terms. What can be tested is whether AI translations appear more heavily translated than human translations, and the measurements show they do not.

### Adopted literature evidence

While no detector was adopted, the accompanying evidence matrix was incorporated as a reference. `corpus/reports/research/prose-rhythm-evidence-matrix.md` pairs published research with empirical measurements for sentence-length variance, burstiness, autocorrelation, TTR, and perplexity, classifying each as counter-evidential.

That independent study reaches the same conclusion as this repository. For the coefficient of variation of sentence length, it observes a 32.4 percent false-positive rate on human prose at the active threshold. For autocorrelation, it measures an effect opposite to the hypothesis and marks the detector for removal.

Independent confirmation from another author on a separate corpus strengthens these conclusions, so the citation is kept on record alongside the `[rejected]` list.

### Side-by-side comparison on a single document

Comparing rule lists alone does not indicate what each tool flags in practice, so both linters were executed on a model-generated Japanese design document.

| | suikou | natural-japanese |
|---|---|---|
| Findings | 13 across 5 rules, plus 4 document-level directions | 6 across 5 categories |
| Overlapping rules | none | none |

Their outputs were completely disjoint. suikou reported an item count written in prose, numbered headings, non-parallel list items, time-dependent expressions, and a Sino-Japanese word where a loanword predominates—each representing an issue that breaks on subsequent edits or becomes outdated. natural-japanese flagged clichés, an inanimate grammatical subject, translationese, and prose rhythm—issues where wording sounds unnatural.

Both tools flagged the same opening sentence for different reasons: one for a time-dependent word and the other for a cliché.

On a professionally written Japanese translation of a Kubernetes task page, suikou reported two findings and natural-japanese reported one, demonstrating low noise from both tools.

### Finding volume across a matched corpus

Across twelve human translations and twelve AI translations of identical source documents, total findings were distributed as follows.

| | Human | AI | Ratio |
|---|---|---|---|
| suikou | 43 | 62 | 1.44 |
| natural-japanese | 87 | 82 | 0.94 |

Per-rule counts explain the volume distribution. `translationese` reported 13 findings on human text versus 2 on AI text, and `translationese_morph` reported 19 versus 2. Both skew toward human translations, aligning with the earlier result: because the human baseline consists of professional English-to-Japanese translations, translationese features occur frequently within it.

### The single candidate showing the expected direction

Only `repeated_sentence_lead`—which triggers when the first two morphemes of a sentence repeat six times in one document—fired more heavily on AI text. Because suikou lacked an equivalent detector, it was benchmarked using the same methodology.

| Corpus | Documents | Documents triggering | Per 100 sentences |
|---|---|---|---|
| Human translation | 39 | 35% | 14.2 |
| AI translation | 78 | 50% | 16.4 |
| Reference corpus | 60 | 26% | 10.6 |

Although the direction matches the hypothesis, the distributions overlap. The AI-to-human ratio is 1.15, and 26 percent of the reference corpus triggers the rule. Because adoption requires non-overlapping ranges, this candidate fails the criterion.

Its output format also conflicts with repository constraints. Because the detector flags every matching sentence, a single 71-sentence document generated 17 findings, violating the requirement that each rule report a bounded number of locations.

The rule was not adopted, though its measurements remain recorded as the lone candidate showing the expected direction.

## D-35 Time-dependence is determined by explicit anchors rather than tense

Language models over-adapt to their immediate session context. When drafting during a session, a model treats that session's timestamp as the document's permanent present, producing statements like `現在レイテンシが悪化しており` that hold true only on the day written.

Re-grounding M6 on this context-anchoring mechanism rather than on general document staleness explains why this rule alone fails under a brief instruction and requires spelling out every prohibited term explicitly, as established empirically for `brief`.

### Verb tense does not predict staleness

A proposal to prohibit temporal expressions altogether so documents remain timeless was rejected.

| Sentence | Tense | Becomes outdated |
|---|---|---|
| `2026年3月に2.5倍になった` | Past | Never |
| `現在は640ミリ秒である` | Non-past | By tomorrow |

Whether a statement becomes outdated depends on whether an absolute anchor is present.

Targeting verb tense would invalidate established doctypes. As verified across 190 published ADRs in D-33, a decision record's context section relies on past tense. Procedural ordering also uses tense forms without depending on when the text is read.

### Formalizing the exception with checkable anchors

Because subjective exceptions cannot be verified mechanically, exemptions are defined strictly through detectable anchors.

When the same sentence includes a calendar year, a version number, or a numbered document identifier, the rule does not trigger. For instance, `lindera 6.0.0 で現在の素性の順序を確認した` states a past fact that will not become outdated.

Anchor detection operates strictly within each sentence; a year in an adjacent sentence does not exempt the target sentence. Only a `v`-prefixed string or a three-part version number qualifies as a version anchor, preventing ordinary decimal values from triggering the exemption. Treating `2.5 times` as an anchor would suppress every finding in its sentence, a bug observed while implementing this rule.

### Expressions beyond static word lists

Relative durations measured from the present contain numeric quantities, so no fixed word list can enumerate them. Expressions such as `直近3ヶ月`, `過去2年`, and `先週` are matched via patterns instead.

Testing on a real document confirmed this gap. On a model-written design document, M6 caught `現在`, `現時点`, `今後`, and `既存の`, yet missed `直近3ヶ月でリクエスト数が約2.5倍に増加した`.

The added terms complete the set listed in Google's timeless-documentation guidance alongside `recently`, `soon`, and `new`, extending an already cited guideline rather than introducing an untested hypothesis.

### Measurement results

| Corpus | Before | After |
|---|---|---|
| 60 Japanese task pages | 119 findings / 36 documents | 123 / 36 |
| 60 reference documents | 179 / 39 | 185 / 39 |
| 39 AI translations | 85 / 26 | 92 / 27 |

The increase is minor because reference documentation rarely relies on unanchored time references. Across 200 documents, the anchor exemption suppressed one finding: `既存のHyper-V分離サポートは、v1.10の試験的な機能であり`, which documents behavior tied to `v1.10` and therefore remains accurate.

Every document in this repository passes both before and after this change.

### Out-of-scope temporal expressions

Past tense itself, descriptions of ongoing conditions, and expressions of sequence are not targeted. Procedures require explicit ordering and decision records require historical context; removing them strips useful information from readers without providing any benefit.
## D-36 Reducing report output by a tenth while increasing detail

This tool operates under an output budget that prohibits increasing emitted token volume. Each rule's character cost was measured against the diagnostic value it provides.

### Rule detection rates and output cost

Per-document averages were measured across 39 AI translations and 39 human translations of identical source pages.

| Rule | AI | Human | Ratio | Chars per finding |
|---|---|---|---|---|
| `maint/time-dependent` | 2.36 | 2.26 | 1.05 | 59 |
| `register/prefer-katakana` | 1.36 | 0.64 | 2.12 | 84 |
| `register/rare-wago` | 1.33 | 0.41 | 3.25 | 60 |
| `maint/parallel-items` | 0.44 | 0.41 | 1.06 | 69 |
| `structure/long-paragraph` | 0.23 | 0.18 | 1.29 | 88 |
| `maint/item-count` | 0.18 | 0.10 | 1.75 | 93 |
| `maint/list-lead-in` | 0.00 | 0.46 | 0.00 | 92 |

Register rules distinguish AI translations from human ones most effectively. The M rules derive from an established style guide and trigger on human-authored text as well, which aligns with the design choice in D-09 rather than contradicting it.

In terms of total findings, `maint/time-dependent` accounts for the largest share. Its 1.05 ratio offers no signal about authorship, yet the rule remains enabled because it reflects a published guideline and resolving its findings improves the document.

### Excerpts that omitted the matched text

Previously, the snippet printed next to each location consisted of the line's leading 60 characters. Whenever a match occurred past that prefix, the snippet left it out entirely. Lines containing multiple findings repeated that same 60-character prefix for every entry, producing three identical snippets for three separate findings with no indication of which term required editing.

Reports instead lead with the matched string inside a compact context window. Window length varies by language because fourteen Japanese characters span roughly a clause whereas fourteen English characters cover only two words. English excerpts use a width of twenty characters, keeping total report length comparable to the earlier fixed sixty-character prefix.

### Excluding numbered document titles

M3 flags manually numbered headings because inserting or deleting sections forces renumbering. A document title behaves differently: an ADR titled `# 3. Title` uses that number as a permanent identifier. Across 190 published ADRs, 155 triggered the rule on their title. Exempting the leading level-one heading lowers that count to 11.

### Resolving contradictory heading advice

`register/prefer-katakana` favors loanword terminology, yet the section-heading synonym table listed only Sino-Japanese variants. As a result, a decision record using `## コンテキスト` received a missing-context warning—one rule flagging a heading produced to satisfy another. The synonym table includes loanword forms as well to eliminate this conflict.

### Output size comparison

| Corpus | Before | After |
|---|---|---|
| 60 reference documents | 71,129 | 61,447 |
| 60 ADRs | 38,704 | 30,972 |
| 60 RFCs | 58,745 | 58,597 |
| 39 AI translations | 46,979 | 42,272 |

Combined character volume drops from 215,557 to 193,288, a reduction of a tenth. Both the narrower excerpt window and the exemption for numbered titles contribute to this decrease. Because every location entry includes the matched text directly, the shorter report provides more actionable detail.

## D-37 Separating word choice from orthography and exporting the glossary

Choosing vocabulary intuitively often produces awkward native Japanese coinages. As documented in D-27, `検証` was replaced with `確かめ` solely to reduce the kanji ratio. While `register/rare-wago` detects such wording after drafting, writers lacked an explicit criterion for choosing words up front.

Reviewing the formal-writing guidance in [yomiyasu](https://github.com/nanaism/yomiyasu) (MIT License, Copyright (c) 2026 nanaism) yielded a structured way to make that decision through targeted questions. No phrasing is borrowed from that source; only the evaluation framework was adapted, using original wording.

### Decision questions for word choice

Apply these questions in order.

1. Would a practitioner writing in this domain select this term in a specification or procedure?
2. Does the expression translate back into English word for word?

Answering no to the first question and yes to the second identifies a literal translation copied directly from English.

The first question underscores that recurrence alone proves nothing. Because phrasing from one AI-authored document propagates into subsequent ones, finding a term elsewhere in this repository does not demonstrate standard usage. Since a model authored the documents here, that caution applies directly to this repository.

### Basis for separating terms and orthography

Lexical choice and spelling rely on different authorities, so the glossary tracks them separately.

| Category | Determines | Authority |
|---|---|---|
| `term` | Word selection | Corpus frequency and PLaMo Translate output |
| `orthography` | Spelling form | JTCA guideline verified per word against the corpus |

Empirical testing demonstrated why a single mechanism cannot handle both. Translating the same English word across separate sentences, PLaMo Translate output both `コンピュータ` and `コンピューター`. It appended the long-vowel mark for `サーバー`, `ユーザー`, and `クラスター`, yet omitted it for `ブラウザ`, `パラメータ`, and `フォルダ`.

A translation model can identify the right word, but it cannot enforce consistent spelling. Spelling conventions require a published rule.

### Disagreement between the guideline and corpus usage

The third edition of the JTCA loanword orthography guideline prescribes a trailing long-vowel mark for English endings -er, -or, -ar, and -y.

Measured across 4.38 million characters, usage confirms the guideline for -er, -or, and -ar. However, -y trends in the opposite direction: `プロパティ` appears 870 times compared with 3 for `プロパティー`. Similarly, although `コンテナ` ends in -er, technical usage favors omitting the mark by 1783 to 95.

Each entry is therefore decided through corpus measurement rather than inferred mechanically from the guideline. Any word that diverges from the guideline receives the `measured-against-jtca` marker so the exception remains documented.

### Exporting the glossary in portable formats

`crates/suikou-core/data/glossary_ja.toml` is the canonical source file, from which `glossary/suikou-ja.prh.yml` and `glossary/suikou-ja.csv` are generated.

The prh format was selected because prh is widely adopted for checking Japanese orthographic variants and its rule files work directly in textlint without modification. Passing the generated dictionary to textlint detected the spelling variants and automatically normalized them with `textlint --fix`.

### Omitting single-character calques from this rule

`glossary/variant` checks orthographic variants and calques longer than one character.

Substring matching on a single kanji produces widespread false matches, such as capturing `題` inside `問題`. Running a substring count on the reference corpus turned up 613 matches for `題`, almost all belonging to ordinary compounds such as `問題`, `課題`, and `話題` rather than the calque. Identifying single-character cases requires morphological analysis, which `prefer-katakana` already handles.

### Existing inconsistency in this repository

Running the rule uncovered `サーバ` in four documents within this repository, which were updated to `サーバー` to match the glossary.

The check also triggers on 7 of 60 reference documents and 10 of 60 Kubernetes task pages. These hits are valid findings rather than false positives: the reference corpus itself mixes 591 occurrences of `サーバー` with 130 of `サーバ`, and flagging minority variants is the rule's purpose.

## D-38 Counting orthography per word and defaulting to the long vowel on ties

### Context

The spelling-variant policy underwent three revisions.

Initially, the long-vowel mark was omitted across all entries (`サーバ`, `コンピュータ`). A second policy added the mark everywhere based on the third edition (2015) of the JTCA katakana guideline, which specifies a trailing long-vowel mark for `-er`, `-or`, `-ar`, and `-y`.

Neither blanket policy reflected actual usage in the reference corpus.

### Decision

Choose each word's spelling from its majority form in the reference corpus. Apply the long-vowel form only when counts are inconclusive—when the majority fails to reach twice the minority count or total occurrences are fewer than 50.

### Rationale

Each policy was evaluated across 12 words totaling 8,540 occurrences.

| Policy | Usage disagreement | Incorrect words |
|---|---|---|
| Long vowel throughout | 45.9% | 5 |
| No long vowel throughout | 54.1% | 7 |
| Counted per word | 9.5% | 0 |

Either uniform rule conflicts with corpus usage roughly half the time. Because `サーバー` and `コンピュータ` are both majority forms in the reference corpus, any single blanket rule inevitably gets one wrong.

The long-vowel mark serves as the fallback because the JTCA guideline recommends it, leaving no basis to override a published standard when corpus counts are inconclusive.

### Consequences

Orthography and word choice remain separated because they rest on different authorities: word choice depends on how narrowly meaning is established, whereas orthography depends on usage data and published rules. `glossary_ja.toml` stores them under separate `[[orthography]]` and `[[term]]` sections.

Machine translation resolves word choice reliably but not spelling consistency. PLaMo Translate generated both `コンピュータ` and `コンピューター` for the same source word in different sentences. On word choice, it matched human translators across 90.6% of 200 aligned paragraphs, and all three disagreements involved an alternate form of the same word rather than an incorrect term.

Words absent from the corpus cannot be counted, so the fallback guarantees a definite spelling for out-of-corpus entries.

## D-39 Requiring only a lead paragraph for READMEs and adding an opening-form rule

### Context

Model-generated READMEs frequently include internal author notes—work status, testing observations, or statements declaring the document's purpose—that offer no value to readers.

According to Diátaxis, most documentation defects arise when a single page attempts to serve multiple purposes simultaneously. Personal working notes and a README fulfill distinct needs.

Whether the `overview` document type fit READMEs had remained unmeasured and was tracked as an open item in TASKS.md.

### Decision

Add a `readme` document type whose sole required element is the lead paragraph.

Remove `usage` from the required sections of `overview` and broaden its heading synonyms.

Add the `structure/lead-opener` rule to flag opening patterns that published documentation avoids.

### Rationale

A corpus of 240 READMEs was assembled from prominent open-source projects, selected by star count across 8 languages and capped at 3 repositories per organization (spanning 199 organizations) so neither a single ecosystem nor a single organization would bias the sample.

Evaluating those files with `overview` reported a missing required section in 205 cases (85%). This matches the findings in D-31, where `howto` dropped its required steps section, and D-32, where `design` dropped its required scope section.

| Section | Matched by heading |
|---|---|
| Install | 62% |
| Contributing | 53% |
| License | 49% |
| Documentation | 41% |
| Usage | 22% |

Even the most frequent section appears in only 62% of files, so mandating it would fail two READMEs in five. README structure does not reside in heading names.

Instead, README structure is defined by the lead paragraph.

| Metric | Result |
|---|---|
| Contains a lead paragraph | 78% (187/240) |
| Lead within 300 characters (among those with a lead) | 83% (156/187) |
| Median lead length | 121 characters |
| Median first-heading position | 13% into the document |

Requiring only a lead paragraph triggers on 11 of the 240 READMEs (5%). Three of those are written in Chinese, which lies outside this tool's supported scope. Another is RxJava, which places the next release date in a heading before defining what RxJava is.

Question-style headings were added to the synonym list after encountering `## What's Rails?`. Although this pattern can match `What's new`, that collision happens once across the 240 READMEs while matching legitimate question headings 21 times.

The opening-form rule includes only patterns with zero occurrences across 831 English lead paragraphs.

| Pattern | README | Rust RFC | ADR | Reference (en) |
|---|---|---|---|---|
| Starts with generalities | 0/199 | 0/20 | 0/10 | 0/602 |
| Starts with project origin or history | 0/199 | 0/20 | 0/10 | 0/602 |
| Declares the document's purpose | 0/199 | 0/20 | 0/10 | 0/602 |
| Refers to itself | 1/199 | 0/20 | 0/10 | **37/602** |

In Japanese, the first three patterns were likewise absent across 653 lead paragraphs from the reference corpus.

Self-referential openings (`This document describes`, `本文書は`) remain outside the rule. They appear in 6% of English reference documents, where API documentation uses them effectively. Although this is the most recognizably machine-written opening among the four patterns, corpus evidence confirms its acceptance in practice.

Tested against 2,080 published documents—240 READMEs, 200 Rust RFCs, 190 ADRs, 652 English reference documents, and 798 Japanese reference documents—the rule produces zero findings.

### Consequences

README working notes are addressed through opening structure rather than banned vocabulary. Established open-source projects also use terms such as `currently` and `TODO`—45 of the 240 READMEs include `currently`—leaving no basis for prohibiting them.

However, that initial count combined two distinct usages. In prominent open-source repositories, `currently` and `Roadmap` inform readers about upcoming plans, support adoption decisions, and invite contributions rather than serving as internal author notes. Vocabulary alone could not distinguish them because the difference lies in what a sentence gives the reader rather than the words chosen, a distinction addressed in D-40.

Lead paragraphs, in contrast, follow a clear convention: make the subject the grammatical subject and state what it is, without opening on background, intent, or history.

This rule does not detect working notes directly, so notes placed later in a document remain outside its scope. Requiring a subject-first lead moves such notes out of the opening; removing them from body sections falls to the convention that every section must answer the question implied by its heading.

## D-40 Judging sentences by reader value rather than vocabulary

### Context

D-39 determined that vocabulary alone could not catch internal author notes, because 45 of 240 prominent READMEs contain `currently` and 48 include a Status or Roadmap heading.

That measurement conflated two distinct usages.

In prominent open-source repositories, `currently` and `Roadmap` address readers directly: they communicate plans, aid adoption decisions, invite contributions, and give readers a concrete next step.

Agent-authored notes serve the opposite function by recording context for the agent's own subsequent sessions without offering anything useful to readers.

Because identical words serve different purposes, filtering on vocabulary alone cannot separate them.

### Decision

Evaluate sentences based on what they communicate to the reader.

Add the `structure/work-note` rule to flag statements focused on the work process itself: what was attempted, what was discovered, work progress status, or a prior state.

Control the rule by document type, because explaining past attempts and why they failed is central to design documents and decision records. `work_notes_ok` in `structure.toml` governs this setting.

### Rationale

Across 240 prominent READMEs, 307 sentences with forward-looking wording were classified by location.

| Location | Count | Share |
|---|---|---|
| Under a dedicated heading | 125 | 41% |
| Within explanatory prose | 182 | 59% |

Among the 182 sentences placed within explanatory prose, **none** described internal work progress. Of those, 17 gave the reader an actionable step and the remainder described features or plans.

The subject of the sentence—not its vocabulary—separates these categories.

| Pattern | README | Reference (en) | Reference (ja) | Rust RFC | ADR |
|---|---|---|---|---|---|
| What was attempted | 0.4% | 0.2% | 0.1% | 4.5% | 3.2% |
| What was discovered | 0% | 0.3% | 0.1% | 7.5% | 1.6% |
| Work progress status | 0% | 0% | 0.1% | 0% | 0.5% |
| A prior state | 0% | 0.3% | 0.3% | 5.0% | 0.5% |

Across reader-facing documentation, these patterns appear in 0.1% to 0.4% of files. In Rust RFCs and ADRs they range from 1.6% to 7.5%, because documenting the investigation itself is the purpose of those formats. That difference justifies toggling the rule by doctype.

Candidate patterns for `we refactored` and its Japanese counterpart were excluded.

In English, 8 of 11 occurrences pointed readers toward a destination or updated name—for example, `moved to their own
repositories`, `renamed Redis Open Source with the v8.0 release`—telling them where to look or what term to use. The remaining three consisted of an FAQ heading and a naming anecdote.

In Japanese, the pattern cannot be separated from an ordinary verb phrase: `フォーカスを移動した` (meaning the focus moved) matches it, and all 43 matches in the reference corpus were of this type.

| Corpus | Applied doctype | Findings |
|---|---|---|
| 240 READMEs | `readme` | 0 |
| 200 Rust RFCs | `design` | 0 |
| 190 ADRs | `decision` | 0 |
| 652 reference (en) | `reference` | 3 (0.5%) |
| 798 reference (ja) | `reference` | 5 (0.6%) |

### Consequences

An identical sentence is treated as relevant content in one document type and extraneous commentary in another: `We tried another approach and it did not work` triggers a finding in a README yet passes in a decision record.

This behavior differs fundamentally from vocabulary checks. Whereas `register/rare-wago` relies purely on frequency without examining context, `structure/work-note` alters its verdict on identical wording according to the document type.

Documents with no declared type are skipped because whether they target external readers cannot be determined. Listing a file under `[paths]` in `.suikou/structure.toml` enables the rule for it.

Plans and calls for contributions placed under dedicated headings are not flagged. Because 41% of prominent open-source READMEs organize them that way, readers can skip them easily. Rather than banning plans from READMEs, the rule expects a heading to announce them.
## D-41 Per-doctype threshold calibration and rule adjustments

### Context

Applying a single set of metric thresholds across both entry-point pages and reference documentation produced widespread false positives. Under the default configuration, `en.mattr100` flags 70% of 240 well-known open-source READMEs versus 15% of 652 reference documents, while `en.simile_marker_per_1k` flags 74% of those READMEs.

A 70% flag rate across an entire document genre indicates a miscalibrated threshold rather than pervasive writing defects. User feedback confirmed this distinction: an entry-point README is less formal than technical pages stored under `docs/`.

### Decision

Profiles support per-doctype adjustments capable of changing a threshold, disabling a metric, or suppressing a local rule.

Every adjustment must declare a `reason`, and a test enforces this requirement across all built-in entries. Each adjustment records an explicit agreement with the user instead of an automatic judgment by the tool.

Configuration layers merge in a fixed sequence: built-in profile, built-in doctype adjustment, project-level override, and project-level doctype adjustment.

### Rationale

Empirical measurements covered 240 English READMEs (199 organizations across 8 languages) and 150 Japanese READMEs (144 organizations).

| Metric | README | Reference | Ratio |
|---|---|---|---|
| `en.mattr100` | 70% | 15% | 4.7 |
| `en.prose_ratio` | 32% | 11% | 2.9 |
| `en.simile_marker_per_1k` | 74% | 45% | 1.6 |

Continuous prose proportion varies threefold depending on the document genre.

| Corpus | Measured `prose_ratio` |
|---|---|
| 240 READMEs | 0.170 |
| 652 reference (en) | 0.427 |
| 200 Rust RFCs | 0.544 |

Lists and code blocks dominate READMEs, leaving 17% prose compared with 43% in reference pages and 54% in Rust RFCs. The default `en.prose_ratio` threshold of 0.65 places every README below the acceptable bound.

Applying per-doctype adjustments lowers README flag rates while leaving the reference corpus unchanged.

| Metric | README (before) | README (after) | Reference |
|---|---|---|---|
| `en.mattr100` | 70% | 6% | 15% |
| `en.prose_ratio` | 32% | 7% | 11% |
| `en.simile_marker_per_1k` | 74% | 0% | 45% |

Across the 150 Japanese READMEs, adjusted flag rates fall between 0% and 2%.

Differences in register do not shift all metrics in the same direction.

| Metric | README (ja) | Reference (ja) |
|---|---|---|
| `ja.kango_ratio` | 0.620 | 0.521 |
| `ja.mattr100` | 0.752 | 0.674 |
| `ja.renyo_te_ratio` | 10.16 | 6.00 |

Japanese READMEs contain more Sino-Japanese compounds than reference documents because feature and component names concentrate compound vocabulary. Uniformly relaxing all limits for entry-point pages would therefore misalign with the data; each metric requires its own direction of adjustment.

### Consequences

Evaluation thresholds for a given metric vary by doctype. To preserve traceability of which rules govern each document, doctype adjustments cannot add new metrics; adding metrics belongs exclusively to global profile overrides.

No local rule is suppressed by default for any built-in doctype. The most frequent local findings on READMEs—`maint/list-lead-in` with 340 hits and `maint/parallel-items` with 271 hits—reflect established style guidance that remains valid in READMEs. Projects can still use the suppression mechanism in their own configuration, even though built-in profiles leave it unused.

Files with no declared doctype receive no adjustments. Enabling doctype adjustments requires either mapping the file under `[paths]` in `.suikou/structure.toml` or specifying `doctype:` in its front matter.

## D-42 Dedicated log doctype and warning-aware verification

### Context

Adding `structure/work-note` produced 11 findings across this repository: 7 in `TASKS.md`, 3 in `decisions.md`, and 1 in `metrics.md`.

Ten of those findings were false positives. Because CLAUDE.md designates `TASKS.md` and `decisions.md` as the proper destinations for work records, the rule was paradoxically instructing a decision log to relocate its own entries to the decision log.

Both files carried the `reference` doctype solely because it imposes no mandatory sections. As a result, the type system conflated document purpose with section requirements.

A separate verification bug appeared alongside this issue. Because `suikou check --quiet` exits 1 only for errors and exits 0 when warnings occur, checks gated solely on the exit code missed 47 warnings despite CLAUDE.md requiring zero errors and zero warnings.

### Decision

A dedicated `log` doctype is added for record-keeping documents such as decision logs, task lists, and change histories. It enables `work_notes_ok` and mandates no required sections.

Both `TASKS.md` and `decisions.md` are reassigned to this doctype.

Verification counts reported findings directly instead of checking the command exit code.

### Rationale

The `log` doctype adds no new checks; it only relaxes `reference`. Even without measurement on an external corpus, relaxing a restriction does not introduce the false-positive risk of adding a new rule.

Reassigning the log files cleared their warnings, whereas the single remaining finding in `metrics.md` proved valid on inspection: a reference document defining terms included a historical remark stating that substitution had previously been decided by position. Rewriting that passage as a definition and linking D-31 for the historical account reduced work-note findings to zero across the three documents.

The other 47 warnings originated in this repository's prose after sections drafted during that session caused the files to exceed the warning thresholds. Every vocabulary replacement was selected by comparing frequencies in the reference corpus.

| Original word | Reference corpus | Replacement | Reference corpus |
|---|---|---|---|
| `読み手` | 0 | `読者` | 13 |
| `向き` | 54 | `方向` | 155 |
| `測る` | 0 | `測定` | 63 |
| `要る` | 0 | `必要` | 3,503 |
| `入り口` | 1 | `起点` | 20 |

Intuition alone gave no indication that `読み手` was unusual; only corpus frequency counts showed that it has zero occurrences in the reference corpus.

### Consequences

Total findings across the repository's documentation dropped from 58 to 0.

Because the exit code reflects errors alone, the verification command checks output lines instead.

```sh
for f in <all Markdown>; do suikou check "$f" --format text; done | grep -c ':[0-9]*:[0-9]*:'
```

Doctype definitions must separate document purpose from required section structure. Both `reference` and `log` require no specific sections yet fulfill different roles. The `work_notes_ok` flag is the first field to encode that distinction, and additional fields can handle comparable requirements for other rules.

## D-43 Word-count gating for density metrics and dropping uncalibratable metrics per doctype

### Context

Following the README recalibration in D-41, evaluating the remaining doctypes uncovered a similar problem on ADRs. Running the `decision` doctype across 190 ADRs triggered `en.mattr100` on 45%, `en.prose_ratio` on 45%, and `en.simile_marker_per_1k` on 75%.

Further analysis showed that the simile metric's high flag rate was not driven by doctype. Instead, its trigger rate declines monotonically as word count increases.

| Word count | Flag rate |
|---|---|
| 0 to 400 | 86% |
| 400 to 800 | 47% |
| 800 to 1,250 | 24% |
| 1,250 to 2,000 | 16% |
| 3,000 to 5,000 | 5% |

These measurements span 1,042 documents drawn from the reference corpus, Rust RFCs, and ADRs.

### Decision

A word-count gate is applied to `en.simile_marker_per_1k`: documents shorter than 1,250 words are exempt from threshold comparison.

For `decision`, `en.prose_ratio` is disabled, while `en.mattr100` is recalibrated for both `design` and `decision`.

### Rationale

Both analytical limits and corpus measurements support this cutoff. Given a threshold of 0.8 per 1,000 words, a single marker meets that threshold at 1000 ÷ 0.8 = 1,250 words. Below that length, any document has either zero markers or enough with one to pass, leaving no usable granularity; empirical flag rates increase sharply below the same cutoff.

The gate runs during threshold comparison rather than metric computation. Because density is well defined regardless of length, preserving the raw calculation keeps golden tests compatible with the reference implementation.

Placement differs from `MIN_TE_FOR_RATIO` in `ja.renyo_te_ratio` because the underlying issues differ: that gate prevents computing an undefined ratio over a tiny denominator, whereas this gate suppresses comparisons on a well-defined number that lacks resolution.

Once gated, flag rates fell across every doctype, removing any need to disable the metric per doctype.

| Doctype | Before | After |
|---|---|---|
| `readme` | 74% | 14% |
| `design` | 16% | 6% |
| `decision` | 75% | 4% |
| `reference` | 45% | 1% |

The 1% reference-corpus rate aligns with the metric's documented human range of 0.84 to 2.54. Because the default threshold of 0.8 separates human prose from machine output rather than using a Tukey fence, `suikou baseline` reporting that it cannot calibrate this metric on any corpus signals a methodological difference rather than a broken metric.

The remaining 14% rate on READMEs reflects true positives. The 34 flagged READMEs have a median of 2,370 prose words compared with 1,241 for the 57 unflagged ones, ruling out insufficient prose: long, prose-heavy READMEs simply omit markers on their comparisons. The per-doctype exclusion for `readme` was therefore removed.

For `decision`, `en.prose_ratio` is dropped because `suikou baseline corpus/cache/adrs` computes a calibrated bound of -0.18, outside the metric's valid range. Because the distribution is too wide to bound reliably, disabling the metric avoids inventing an arbitrary cutoff. Its default human range of 0.66 to 0.97 reflects prose-heavy documentation and does not fit terse, template-structured records.

By contrast, `en.mattr100` calibrated to 0.76 for `design` and 0.88 for `decision`. Both proposals and decision records shift terminology with each topic and exhibit low lexical repetition.

| Doctype | `mattr100` | `prose_ratio` | `simile` |
|---|---|---|---|
| `readme` | 6% | 7% | 14% |
| `design` | 0% | 4% | 6% |
| `decision` | 0% | 0% | 4% |
| `reference` | 15% | 11% | 1% |

### Consequences

High flag rates trace back to distinct root causes, each paired with its own fix.

| Cause | Remedy |
|---|---|
| Insufficient resolution at short document lengths | Gate the metric across all doctypes |
| Distribution shifts across doctypes | Adjust the threshold per doctype |
| Distribution too dispersed to bound | Disable the metric for that doctype |

Initially blaming doctype for simile flags overlooked the real driver, which was document length. Applying a per-doctype exclusion prematurely leaves the underlying defect intact elsewhere: excluding simile markers for `readme` in D-41 left the 75% flag rate on `decision` hidden. Isolating the root cause must precede selecting a remedy.

Because no Japanese corpus of ADRs or design proposals exists, Japanese thresholds remain unadjusted: `ja.mattr100` and `ja.prose_ratio` keep their defaults for these doctypes.

## D-44 Undated GitHub documents do not form a human reference corpus

### Context

Japanese thresholds for `design` and `decision` remained at their defaults because no Japanese corpus of ADRs or design documents existed.

GitHub code search supplied a candidate corpus of 123 files from 95 organizations, filtered to contain at least 300 hiragana and katakana characters. Default thresholds produced very high flag rates on these files.

| Metric | 67 Japanese ADRs | 56 Japanese design docs |
|---|---|---|
| `ja.mattr100` | 86% | 82% |
| `ja.prose_ratio` | 88% | 87% |
| `ja.list_ratio` | 82% | 82% |
| `ja.renyo_per_1k` | 43% | 39% |

Although calibration would lower those rates, verifying whether the files were actually human-written came first.

### Decision

No thresholds are calibrated from this corpus, leaving the Japanese settings for `design` and `decision` at their defaults.

The collected files are retained because filtering by each file's own commit timestamp may make the subset usable.

Every subsequent effort to assemble a human reference corpus must first define a method for verifying that its documents predate widespread LLM use.

### Rationale

The tool's own diagnostics quantified the contamination. The `register/rare-wago` rule, designed as an indicator of machine-generated prose, flags native-Japanese terms repeated where professional technical writing avoids them.

| Corpus group | `register/rare-wago` | `maint/parallel-items` |
|---|---|---|
| Reference corpus, concept pages, human translation (67) | 4% | 35% |
| Kubernetes task pages, human translation (40) | 20% | 52% |
| Procedures collected from GitHub (160) | 35% | 61% |
| ADRs collected from GitHub (67) | 31% | 71% |

GitHub-sourced groups trigger these machine-writing markers far more often than human translations, reaching 31% to 35% on `register/rare-wago`—eight times the 4% rate of the reference corpus.

Genre differences and contamination both contribute to these figures. Genre accounts for part of the spread, as concept pages register 4% while task pages reach 20%. Comparing task pages directly against GitHub procedures still leaves a gap of 20% versus 35%, showing that genre alone does not explain the difference.

Among 25 sampled ADRs, 8 came from repositories created before 2023. Because a repository's creation date does not indicate when a specific ADR was written, filtering by repository creation date is insufficient.

Calibrating against a contaminated corpus shifts thresholds toward machine-generated style, reversing the purpose of the thresholds.

### Consequences

Validating corpus provenance falls on whoever supplies input to `suikou baseline`. Because `baseline` assumes every input document is human-authored, it performs no authorship verification itself.

Existing diagnostics provide a practical screen: comparing `register/rare-wago` and `maint/parallel-items` flag rates against the reference corpus exposes severe contamination. Any sample running eight times above the reference rate is unfit as a human baseline.

Provenance records for the stored corpora appear below.

| Corpus | Source | Human authorship |
|---|---|---|
| `pro_ja`, 798 | Japanese translations of k8s, MDN, Vue, React, and the Rust book | Human translation |
| `pro_en`, 652 | English originals of the same documentation | Human |
| `rfcs`, 200 | Rust RFCs | Human; proposals span years before LLMs |
| `adrs`, 190 | Published English ADRs | Unverified |
| `readmes`, 240 | READMEs selected by star count | Unverified |
| `adrs_ja`, 123 | GitHub code search | **Contaminated** |
| `howto_ja`, 200 | GitHub code search | **Partly contaminated** |

The same check was applied to the English `adrs` and `readmes` corpora. Because `register/rare-wago` is limited to Japanese, maint rules acted as the proxy indicator.

| Corpus group | `maint/parallel-items` | `maint/trailing-etc` |
|---|---|---|
| `pro_en`, 652 (reference) | 51% | 4% |
| `rfcs`, 200 | 58% | 17% |
| `adrs`, 190 | 28% | 5% |
| `readmes`, 240 | 43% | 24% |

No English group approaches the eightfold gap seen in Japanese, and `adrs` scores below the reference corpus at 28% compared with 51%. The calibrations adopted in D-41 and D-43 therefore rely on corpora that pass this screen.

On `maint/trailing-etc`, `readmes` reaches 24% versus 4% in the reference corpus, six times the reference rate. Ending feature lists with `etc.` is an established README convention that predates LLMs, and selecting repositories by star count includes many older files, attributing this elevation to genre.

Because D-28 rejected adding an English counterpart to `register/rare-wago`, English screening depends solely on maint rules, yielding weaker contamination detection than in Japanese.

## D-45 Restricting M2 to forward-referencing counts

### Context

M2 flags item counts written in running prose because adding or deleting an item invalidates the stated number.

The earlier implementation scanned prose without checking what each count referenced. TASKS.md documented the resulting false positives—flagging ordinary explanatory prose that states a count without a following list—and cited `tests/golden/input/ja_prose.md`:

```
無効化の方法は大きく分けて二つある。時間で切る方法と、変更を検知して切る方法である。
```

Here, the count and the listed items resolve in the same sentence. Adding a third method requires rewriting that sentence, so no separate text can go stale.

### Decision

M2 triggers only when a stated count points forward to content outside the sentence.

A sentence qualifies when it contains an explicit forward-pointing cue—以下, 次の, or 下記—that introduces upcoming items.

A sentence also qualifies when a list, heading, or table follows immediately, forming a separate structure that can be edited independently of the prose.

Finally, a sentence qualifies when subsequent prose introduces items using enumerating markers such as 第一に, 1つ目, or まず.

### Rationale

All 168 findings across 798 reference documents were classified by marker presence and following structure.

| | List | Heading | Prose | Nothing |
|---|---|---|---|---|
| With 以下/次の | 38 | 4 | 14 | 6 |
| Without | 17 | 2 | 81 | 6 |

Of the 81 cases without a forward-pointing marker that were followed by prose, only 3 contained an enumerating marker, while the other 78 resolved in the same sentence.

An earlier tally defined false positives strictly as matches not followed by a list, which produced a 67% rate. Reviewing the concrete examples refuted that definition because prose goes stale in the exact same way when headings follow:

```
次の3つの命名規則がよく使われます。

### DNSサブドメイン名
```

Adding a fourth convention makes 次の3つ inaccurate. Whether the counted elements are formatted as list items or headings does not change the failure mode; what matters is whether the count references content located elsewhere.

### Consequences

Flagged documents in the reference corpus fell from 112 to 67, and the golden-test case is no longer flagged.

Defining a rule's scope requires deriving the trigger condition from the failure mode the rule prevents. M2 exists to prevent stale prose counts when items are added or removed, a risk that arises only when the count and the counted items can be modified separately. Testing solely for a following list captured only a subset of that condition.

The English rule applies the same structure, pairing `following`, `below`, and `these` as forward-pointing cues with `first`, `second`, and `one is` as enumerating markers. Because the English reference corpus was not classified in the same way, these lists were transferred from the Japanese analysis rather than measured independently.
## D-46 Document types define target section scale without enforcing lint rules

### Context

Previously, document type definitions specified which sections to include and what question each section should answer, without indicating expected section length or composition. Writers had to decide section sizing on their own, producing uneven proportions across documents.

### Decision

Each section definition may include an optional `scale` field, which `plot` and `brief` output as structural guidance.

This field is not a lint rule, so `check` ignores it completely.

Rather than prescribing absolute character counts, it describes relative length ordering between sections and the expected balance of prose and lists within each section.

### Rationale

Section sizes were measured across 221 READMEs from prominent open-source repositories. An initial sample of 240 files included 19 Chinese READMEs, which fall outside the languages this tool targets and were excluded.

| Section | Count | Median characters | Interquartile range | Bullet lines | Prose lines |
|---|---|---|---|---|---|
| License | 96 | 212 | 109 to 452 | 0 | 2 |
| Documentation | 91 | 349 | 171 to 854 | 0 | 1 |
| Contributing | 157 | 386 | 202 to 766 | 0 | 2 |
| Install | 203 | 455 | 186 to 954 | 0 | 3 |
| Usage | 72 | 480 | 211 to 920 | 0 | 2 |
| About | 32 | 668 | 403 to 969 | 0 | 3 |
| Configuration | 54 | 739 | 327 to 1568 | 0 | 7 |
| Features | 78 | 1032 | 495 to 1815 | 6 | 2 |

Median lengths span roughly five times from License to Features, while both the relative ordering and the internal composition remain consistent: Features is the only section driven primarily by bullet lists, and Configuration contains the highest number of prose lines.

**Section length cannot serve as a lint rule because quartiles span 2.5 to 3 times the median.** Human-authored documentation varies too widely to form separable distributions, ruling out a reliable threshold. Candidate metrics rejected on the same grounds appear in D-43 and under `[rejected]` in `corpus/baselines.toml`.

Requiring a code block in installation instructions is also unsupportable. Out of 203 Install sections, only 100 include a code block (49%); the remainder link to external wikis, describe build workflows in prose, or list downloadable binaries.

### Measurement errors corrected after the first pass

The initial measurement pass suffered from methodological errors that skewed section counts and medians.

| Error | Impact |
|---|---|
| Counting sections with nearly empty bodies | Parent sections whose real content resided in subsections entered the dataset |
| Matching headings via substring search | `What's New` was misclassified under About |
| Retaining Chinese READMEs | Documents in an unsupported language entered the dataset |

Correcting these issues reduced the About sample from 49 to 32 sections and shifted the median length of Features from 731 to 1032 characters. The measurement script is stored at `research/structure/measure_granularity.py`.

### Consequences

Only `readme` defines `scale` values, because no other document types have been measured. A unit test restricts the set of types providing scale metadata strictly to `readme`.

Another test verifies that every `scale` entry supplies both English and Japanese text so neither language's output omits the guidance.

Subtypes for library versus application READMEs were not introduced. Corpus analysis showed no distinct clusters: only 10% of READMEs include an API section, 14% show a package-manager installation command, and 56% match neither pattern, indicating a continuous spectrum of section compositions rather than discrete variants.

## D-47 Add a document type for staged proposals and decouple section presence from heading level

### Context

Previously, `design`—modeled on the Rust RFC template—served as the sole document type for technical proposals.

Proposals for incrementally rolled-out features impose distinct operational requirements. Kubernetes Enhancement Proposals (KEPs), for instance, must address test plans, graduation criteria, rollback paths, version skew, and monitoring before a staged release can proceed.

### Decision

Introduce an `enhancement` document type modeled on the KEP template.

Make Summary, Motivation, Goals, and Proposal required sections.

Evaluate section presence independent of heading depth, while restricting section-order validation to top-level headings.

### Rationale

A corpus of 265 published KEPs was analyzed across 19 SIGs, capped at 25 documents per SIG. Files still containing unmodified template boilerplate were removed, eliminating 212 of the 402 initial candidates.

| Section | Presence rate |
|---|---|
| Summary | 98% |
| Motivation | 95% |
| Goals | 94% |
| Proposal | 94% |
| Graduation Criteria | 92% |
| Implementation History | 91% |
| Non-Goals | 87% |
| Risks and Mitigations | 77% |
| Test Plan | 76% |
| Design Details | 71% |
| Alternatives | 71% |
| Production Readiness Review | 61% |
| Version Skew Strategy | 58% |
| Upgrade / Downgrade Strategy | 57% |
| Drawbacks | 53% |
| User Stories | 39% |

Because omitting any single required section triggers a diagnostic, the required subset was selected using cumulative trigger rates rather than individual section frequencies.

| Required sections | Trigger rate |
|---|---|
| Summary | 2% |
| + Motivation | 6% |
| + Goals + Proposal | 9% |
| + Non-Goals | 15% |
| + Graduation Criteria + Test Plan | 29% |

Selecting the 9% configuration aligns the false-positive rate with the other document types.

Operational sections remain optional while retaining their guiding questions and descriptions so `plot` and `brief` continue prompting authors to consider them, following the approach established in D-31 and D-32.

### Decoupling section presence from heading level

When first evaluated against the corpus, the new document type triggered diagnostics on 262 of 265 KEPs (98%).

The cause was heading depth: KEPs nest `Goals` beneath `## Motivation` as a `###` subheading, whereas section matching inspected only top-level headings.

**Heading depth is a structural formatting choice rather than a semantic absence.** Placing goals inside the motivation section reflects how an author groups related ideas, not that goals are missing.

D-33 had rejected matching lower-level headings out of concern that a subheading inside one section might match a different section definition and distort order checks.

Separating presence checks from order checks eliminates that conflict. Presence matching scans headings at every depth, while order validation examines top-level headings exclusively, since relative ordering between a nested subheading and a top-level heading is undefined.

After this separation, the trigger rate on `enhancement` dropped from 98% to 8%.

### Consequences

Trigger rates declined across all existing document types as well.

| Document type | Evaluation corpus | Trigger rate |
|---|---|---|
| `readme` | 240 READMEs | 4% |
| `design` | 200 Rust RFCs | 8% |
| `decision` | 190 ADRs | 7% |
| `enhancement` | 265 KEPs | 8% |
| `howto` | 41 Kubernetes task pages | 0% |

The 11 MADR documents previously set aside in D-33 pass under the separated logic; they nest consequences under `Decision
Outcome`, representing a hierarchy choice rather than an omission.

D-33 also noted that splitting document types by template was the alternative to relaxing heading-level matching. Introducing `enhancement` implements that alternative as well, resulting in both changes being adopted.

The document-type decision tree gains a branch that distinguishes `design` from `enhancement` based on whether the feature rolls out in stages. A test enforces that every defined document type remains reachable in the decision tree.

## D-48 Align README headings with convention and restrict M2 counts to the enclosing section

### Context

Applying the README granularity analysis from D-46 to this repository's own README exposed a naming mismatch.

Five of the repository README's seven headings barely appeared across the 221 notable open-source READMEs.

| Repository heading | Frequency across 221 READMEs |
|---|---|
| `Installing` | 1% |
| `Using it` | 0% |
| `How it is built` | 0% |
| `Layout` | 0% |
| `State` | 0% |
| `Documentation` | 25% |
| `License` | 41% |

Readers struggle to locate standard information when headings use unfamiliar labels.

The `Status` section suffered from a separate maintenance issue: the same status table was duplicated in three places—`CLAUDE.md` and both README files—and the README copies had drifted out of sync, showing 8 rows instead of 13 and omitting `plot`, the glossary, and the release workflow.

### Decision

Rename the headings to match open-source conventions: `Install`, `Usage`, `How it works`, and `Status`.

Move `Layout` into `development.md`, because repository directory structure is contributor documentation.

Remove the duplicated `Status` table from both READMEs and replace it with a pointer to `CLAUDE.md`.

### Rationale

The replacement names reflect the dominant conventions in the 221-README corpus: `Install` (41%), `Usage` (33%), and `Documentation` (25%).

A `Layout` section enumerating repository paths appeared in only 1 of the 221 READMEs; prospective users do not rely on internal file trees when evaluating adoption.

Reducing `Status` to a short reference eliminates synchronization drift across three copies by maintaining a single canonical source.

Although `Usage` measures 1,052 characters against a corpus third quartile of 920, the maintainer elected to leave it as written, consistent with D-46's principle that section scale is guidance rather than an enforceable rule.

### Fixing an M2 false positive

Editing the Japanese README uncovered a false positive in the M2 rule condition introduced in D-45:

    部分ごとの状況は CLAUDE.md にある。
    ここに写しを置くと古くなる。実際に、取り除く前は5つの部分が遅れていた。

    ## ライセンス

Because a heading appeared immediately afterward, the rule interpreted the numeric count as referencing upcoming items. In reality, `## ライセンス` begins an unrelated sibling section; the previous section had simply ended.

**A numeric count can refer only to content nested within its own section.** A subsequent heading triggers the rule only when its heading level is deeper than the heading enclosing the count.

In the reference corpus, this distinction produces 1 case in each direction, so corpus frequency alone is inconclusive. The justification rests on scope semantics: any rule that flags a retrospective count solely because it appears in a section's closing paragraph is flawed regardless of how seldom it fires.

The fix is implemented consistently across all three codebases—Rust, the textlint preset, and `research/mcheck.py`—without changing the golden test counts.

### Consequences

Running the doctype evaluation tooling against this repository's own documentation was what surfaced the heading drift. Measurement infrastructure does not automatically improve existing prose; applying it to the project's own files requires deliberate follow-up work.

Duplicating tables across multiple files is avoided: between `CLAUDE.md` and the README, one document serves as the single source of truth and the other links to it.

## D-49 Keep Hugo for the documentation site because Zensical lacks native multilingual builds

### Context

A proposal evaluated migrating the documentation site to Zensical, a Rust-and-Python static site generator built by the creators of Material for MkDocs.

The repository builds English and Japanese documentation in a single pipeline, configuring `defaultContentLanguage` for English and mapping `[languages]` to `content/en` and `content/ja`.

### Decision

Continue building the documentation site with Hugo.

### Rationale

Zensical does not support multiple languages within a single build, as its documentation explicitly notes:

> Zensical only supports setting a canonical language for the entire project.

Because `docs_dir` accepts only a single directory path and the site language is configured globally, switching languages requires manual links pointing to separately built sites.

Plugins do not close this gap either. As of version 0.1.1, Zensical's documentation specifies that it neither executes arbitrary MkDocs plugins nor exposes the MkDocs plugin API.

The `mkdocs-static-i18n` plugin is absent from Zensical's supported, unsupported, and planned lists, and configuration blocks for unlisted plugins are silently ignored. Although a Module API is planned, it will target the paid Spark tier before broader release.

On the roadmap, `Language organization` and `Language publishing` do not appear under `Now available`, `In development`, or `Up next`, and the roadmap explicitly disclaims any committed timelines or priority order. Project updates also state that Zensical will remain on 0.x releases while its module architecture matures.

### Findings from a prototype evaluation

Version 0.1.1 was installed and tested locally to determine whether migration was viable. The trial uncovered two undocumented behaviors alongside other technical details, recorded here in case the migration is revisited.

| Evaluation area | Finding |
|---|---|
| Publishing under a subpath | Supported; generated links are relative, requiring no base-path option |
| Language-selector configuration key | `[[project.extra.alternate]]`; omitted from official documentation |
| `site_dir` | Must reside inside the project directory; external paths are rejected |
| Custom front matter | Preserved intact, leaving `doctype` unaffected |
| Build performance and size | 0.26s build time, producing 592K of output versus Hugo's 796K |

Splitting the site into two separate projects—each with its own configuration under `docs/en` and `docs/ja`—works technically. Both builds succeed and emit the appropriate `<html lang>` attribute for each language.

### Reasons for declining the migration

Maintaining two separate projects would require duplicating the `nav` structure across two configuration files, directly contradicting the single-source principle established in D-48.

Although Zensical offers Material-style visual polish, built-in search, and the removal of 57 lines of custom layouts, those gains do not outweigh unscheduled multilingual support, an indefinite 0.x release phase, and the absence of plugin extensibility prior to the Module API.

This decision can be revisited once i18n enters the roadmap's active status table.

### Consequences

The no-external-theme policy in `docs/hugo.toml` remains in effect, along with the four custom layout templates totaling 57 lines.

Client-side search remains absent as well: although Zensical generates a `search.json` index automatically, gaining search does not justify manually stitching together two single-language builds.

## D-50 Existing rules separate AI-authored documentation in Japanese but not in English

### Context

Addressing the gap identified in T21 required evaluating genuinely AI-authored documentation whose provenance could be confirmed from git history: repositories where 60% or more of recent commits include `Co-Authored-By: Claude`.

Separate corpora were gathered for each language, totaling 140 English documents and 150 Japanese documents.

### Decision

Add no new lint rules, as the existing rules already separate AI-authored text from human writing in Japanese.

In English, the gap postulated in T21 does not materialize in empirical data, while in Japanese the existing rule set already addresses it.

### Rationale

Across the 140 AI-authored English documents, the four work-note patterns (`structure/work-note`) appear at 1%, compared with 0% in human READMEs. The four candidate patterns show no separation either, and 注意書き occurs at a higher rate in human-authored documents.

In Japanese, by contrast, existing rules differentiate cleanly across three corpora ordered by confidence in their authorship attribution.

| Corpus | Authorship provenance | Documents |
|---|---|---|
| AI-authored READMEs | Confirmed from commit history | 37 |
| Japanese READMEs from GitHub | Unverified; D-44 detected AI contamination | 150 |
| Reference corpus | Professional translations | 150 |

| Rule | AI-authored | GitHub | Reference |
|---|---|---|---|
| `register/rare-wago` | 70% | 34% | 14% |
| `glossary/variant` | 64% | 45% | 12% |
| `register/prefer-katakana` | 43% | 34% | 30% |
| `maint/numbered-heading` | 13% | 10% | 1% |

**All four rules rank the three corpora in the exact order of their authorship certainty.**

The GitHub corpus flagged as mixed in D-44 falls consistently between the confirmed AI-authored set and the human reference set. Because all three datasets were assembled independently, this monotonic alignment corroborates both findings.

### Controlling for document genre

Before controlling for document type, observed ratios appeared substantially larger.

| Rule | Uncontrolled | Genre-controlled |
|---|---|---|
| `register/rare-wago`, against the reference corpus | 13.6x | 2.1x |
| `maint/numbered-heading`, English, against human READMEs | 6.3x | 1.7x |

In English, the AI-authored sample contained a higher proportion of `docs/` pages, whereas the human baseline consisted solely of READMEs. Without genre matching, a document-type difference would nearly have been misattributed to AI authorship. Comparisons must therefore pair documents of the same genre.

### Corpus defects found through manual review

Inspecting files directly prior to running measurements uncovered sampling flaws in the English dataset.

| Defect | Scope |
|---|---|
| Language filter checked only for kana | 23 Korean and 3 Chinese files were misclassified as English |
| Agent instruction files | 42 of 200 files (21%): orchestrator runbooks and `.memory/` records |

Agent instruction files are intentionally written as notes directed at an automated agent rather than user-facing documentation, placing them outside the target genre. Both filters were applied to the Japanese collection so all 150 files are Japanese documentation with instruction files removed.

### Consequences

This result provides independent confirmation for D-28's decision against implementing English style rules: across 140 AI-authored English documents, none of the evaluated patterns separate AI prose from human prose.

In Japanese, the existing rules already distinguish the corpora effectively, so no additions are warranted.

These findings remain subject to methodological limitations. Every sampled document was committed to a public repository with human review in the loop, so unreviewed draft notes do not appear in the data. Additionally, authorship share is computed over the last 100 commits rather than a repository's full history, and six retained repositories contain fewer than 20 commits, making their commit-share estimates weak evidence.

## D-51 Adopt the neon prose style as this repository's baseline

### Context

Commit 7d07fc5, dated 2026-11-24, rewrote 26 files in this repository. Gemini (neon) performed the rewrite as a wholesale style conversion: opening paragraphs became definitional sentences, headings became nominal phrases, paragraphs stopped being hard-wrapped, and native Japanese vocabulary shifted toward loanwords and Sino-Japanese compounds. Median sentence length rose from 31 to 42 characters and median body line length from 33 to 91.

The pre-rewrite prose satisfied the existing thresholds. The rewritten prose exceeded several Japanese thresholds while the English side improved, dropping from 25 positional findings to 1.

The table below places the rewritten prose against the measured human and AI ranges in `corpus/baselines.toml`.

| Metric | Human native | AI, 4 models | Before | neon |
|---|---|---|---|---|
| renyo_te_ratio | 1.79–2.00 | 3.62–15.94 | 1.42 | 9.00 |
| renyo_per_1k | 2.29–2.71 | 4.81–6.75 | 2.48 | 4.60 |
| kango_ratio | 0.326–0.397 | 0.455–0.519 | 0.456 | 0.550 |
| mattr100 | 0.569–0.581 | 0.629–0.665 | 0.585 | 0.611 |
| demonstrative_per_1k | 5.69–6.47 | 1.38–3.39 | 8.14 | 5.72 |
| keishiki_meishi_per_1k | 11.10–11.16 | 4.99–6.95 | 10.71 | 8.84 |

`renyo_te_ratio` lands in the middle of the AI range and `kango_ratio` exceeds it. `demonstrative_per_1k` moved into the human native range.

### Decision

Adopt the neon style as the baseline for this repository's documentation. Confine threshold relaxation to the project-level override in `.suikou/profile.toml` and leave the built-in `oss` profile unchanged, so no relaxed threshold reaches a user's document.

Two rule-precision fixes do reach users, deliberately. `register/rare-wago` now exempts tokens whose surface is entirely katakana, and `structure/missing-section` now accepts `事前準備` as the prerequisites section. Both only reduce false positives.

Derive the thresholds from the measured maximum across this repository's nine Japanese documents plus roughly 3% of headroom, rather than from `suikou baseline` directly. The Tukey upper fence that `baseline` applies (Q3 + 1.5×IQR) finds outliers within a corpus; it does not set a style baseline, and over nine samples it lands far above the maximum. For `ja.renyo_te_ratio` the calibrated 23.21 is 1.8 times the measured maximum of 12.75 and exceeds even the top of the measured AI range at 15.94, so adopting it would stop that metric detecting AI prose entirely.

Leave `ja.list_ratio` and `ja.prose_ratio` at their defaults. Measured at 0.167 against a default of 0.25 and 0.870 against 0.75, neither produced a single finding. Relaxing a metric that never fired would be relaxation without evidence.

Do not relax `ja.demonstrative_per_1k`. `baseline` refuses to calibrate it, reporting that the derived bound falls outside the value range and could never fire. The default of 4.5 stands, and demonstratives were added to the three documents that fell below it.

Do not change the English rules. The rewrite improved the English side without any rule change.

### Rationale

Recalibrating the built-in thresholds from the neon measurements is rejected. Doing so would set `renyo_te_ratio` to 23.21, placing the entire measured AI range of 3.62–15.94 below the threshold, and `kango_ratio` behaves the same way. Those two metrics would stop detecting AI prose, which changes what the tool claims.

The adopted thresholds and the detection power each one costs are below.

| Metric | Default | Measured max | Adopted | AI, 4 models | Detection |
|---|---|---|---|---|---|
| ja.renyo_te_ratio | 3.0 | 12.75 | 13.2 | 3.62–15.94 | upper part of the range survives |
| ja.renyo_per_1k | 4.0 | 7.16 | 7.4 | 4.81–6.75 | lost |
| ja.mattr100 | 0.60 | 0.6235 | 0.64 | 0.629–0.665 | mostly lost |
| ja.kango_ratio | 0.52 | 0.5871 | 0.59 | 0.455–0.519 | lost |

For `ja.renyo_per_1k`, `ja.mattr100`, and `ja.kango_ratio` the loss is unavoidable: the measured maximum already sits inside or above the AI range, so any threshold admitting this style sits there too. `CLAUDE.md` carries a 連用中止 density of 7.16, higher than any of the four models measured. That loss is accepted knowingly. `ja.renyo_te_ratio` is the exception, because its measured maximum falls inside the AI range, so adopting 13.2 preserves detection above that point.

Hand-setting per-doctype values under `[doctype.*]` is also rejected. The log, design, howto, overview, and readme doctypes each have exactly one sample in this repository, so values derived from them would overfit. D-41 introduced per-doctype adjustment on the strength of 240 READMEs and 652 reference documents, not one file. The threshold at which the tool refuses its own calibration must not be circumvented by hand.

Three false positives were corrected alongside the adoption. `register/prefer-katakana` fired 27 times on `検査→テスト`, but `検査` here denotes the lint inspection while `テスト` denotes `cargo test`, so the substitution would introduce an error. `register/rare-wago` flagged `糸`, the lexeme of `イト`, after the dictionary split `トレイト` into `トレ` and `イト`; the existing guard excluded only single-character katakana, letting two characters through. Widening the guard also exempts native Japanese words spelled in katakana, but measurement found that trade to be near-free: across 13 formal nouns and common native words, standalone occurrences numbered 1 in 798 professional Japanese translations and 0 in 150 AI-authored Japanese documents. The single hit is `ヒト` in a physiology context, where it names the species by scientific convention. Formal vocabulary (`否`, `留まる`, `内訳`) and writing-domain vocabulary (`表記ゆれ`, `書き出し`) were also false positives, because the reference corpus consists of Japanese translations of Kubernetes, MDN, and Vue — web documentation that is thin in both registers.

Not every flagged word became an allow entry. Each was counted against the 798-document reference corpus, and words absent from it were replaced instead: `改める` (0) became `変更する` (255), `上がる` became `増加する` (45), and `取りこぼし` (0) became `見落とし` (4). `改める` and `上がる` belong to the body-motion class that step 6 of CLAUDE.md's vocabulary procedure names. Minority spellings were normalized too: `保守` (2) to `メンテナンス` (17) and `書式` (4) to `フォーマット` (11), and both left the allow list. `食い違う` is absent from the corpus but appears in 12 places, so its replacement was deferred and recorded in `TASKS.md`.

`structure/missing-section` collided with nominal-phrase headings. neon renamed the prerequisites section to `事前準備`, which was absent from the synonym list, so `事前準備` was added. The header of `structure.toml` already records the same principle for `register/prefer-katakana`: a document written the way the tool advises must not be rejected by another of its rules.

Per-paragraph measurement is deferred to separate work. Measured at a paragraph threshold of 0.64, `ja.kango_ratio` separates the corpora 4.3-fold, flagging 3.6% of pre-rewrite paragraphs against 15.5% of neon's. `ja.renyo_te_ratio` cannot work per paragraph: the `MIN_TE_FOR_RATIO` gate yields a value for only 38 of 562 paragraphs. `ja.renyo_per_1k` is measurable but cannot localize, because the excess spreads across half the paragraphs. `TASKS.md` carries the detail.
