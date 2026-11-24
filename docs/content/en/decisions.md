---
title: "Decisions"
weight: 30
doctype: "log"
---

This page is a record of each point where a decision could have gone another way, together
with the reason it went the way it did. It is kept so that a later reader does not have to
repeat the same investigation.

## D-01 Metrics over the whole document are kept out of textlint

textlint attaches each finding to the position of a node.
A metric computed over the whole document has no node it could be attached to.
Moreover, most of these metrics report that something is missing: too few formal nouns, too
few demonstratives, or too few simile markers. Something that is missing has no position.
A shortage of formal nouns, for example, is a property of the whole document and not of any
single line.

The first attempt implemented these metrics as textlint rules that reported against the
Document node. That was a mistake. Placing the analysis inside textlint restricts it to
JavaScript, where no tokenizer returns the word origin, and without the word origin the
kango ratio cannot be computed. Giving up a validated metric because of how the code is
organized puts the priorities in the wrong order.

Instead, the front-end tool calls textlint and combines the output of textlint with its own
analysis. The front-end tool can also act as the MCP server, and an agent is better served by
a single combined report in any case.

## D-02 Rust is the implementation language, and Python is limited to calibration and exploration

Every metric that passed validation can be implemented in pure Rust, because lindera-unidic
returns every UniDic feature, including the word origin and the inflected form.

Python was needed only for participial clauses, nominalization, modifier stacking, and
dependency distance, and measurement rejected all four.
The deeper the parse a metric required, the more its result depended on the design of the
experiment, and the less the effect size from earlier research held up.

The work is therefore divided into two tiers. Metrics that have been settled are implemented
in Rust, while exploration and calibration are done in Python. A metric is moved into Rust
only after it has proven useful in Python.

When the tool runs from Claude Code hooks on every Write and every Edit, startup time becomes
the dominant cost. spaCy takes two to five seconds to load, so Python cannot be part of the
production path.

## D-03 Lines are processed directly, without a Markdown parser

The reference implementation under `research/` processes text line by line, and the golden
values can be matched only by doing the same.
Fewer dependencies also mean fewer failures caused by a difference between versions.

## D-04 The tokenizer is isolated behind a trait

Both the lindera API and the order of the UniDic features change from version to version.
Confining the version-dependent code to a single struct limits the code that a version
change can affect to a few dozen lines.

Keeping lindera out of the default features also lets the rules and the metrics be tested
without downloading a dictionary, so a change can be checked more quickly.

## D-05 The feature order is checked at startup

Pinning a version still leaves the risk that an upgrade causes a failure no one notices.
If the order of the features changes, the word origin and the inflected form are read from
the wrong positions. A value is still returned, but it is the wrong value.

The kango ratio and the renyo count both depend on those two fields, so if the program keeps
running on incorrect values, the findings themselves become incorrect.
It is better to analyze a known word at startup and to exit with an error when the result
does not match.

## D-06 Thresholds are not divided by register

Two models were used to test whether the plain and polite registers change the metrics.
The direction of the difference did not agree between the models, and no consistent effect
appeared. Any effect of register is outweighed by the difference between models.

Register detection is kept, but only to decide whether a rule that depends on the register,
such as the rule on sentences ending in a noun, applies. There is no need for a second table
of thresholds.

## D-07 Formal nouns and the kango ratio are reported at info

Both metrics vary widely with the kind of document.
In procedures, formal nouns come close to the threshold, and in explanations, the kango
ratio approaches the value for native human writing. The kango ratio also depends strongly
on the topic.

The Japanese human corpus consists mostly of expository books and contains few procedures.
Both metrics remain at info until baselines are available for each kind of document, and
both will move up to warning once those baselines exist.

## D-08 An effect size does not transfer from one task to another

For participial clauses in English, a reported rate of 5.3 times the human rate suggested a
threshold of nine to ten. When measured, the range overlapped the human range, so the
metric could not be used. Nominalization behaved in the same way.

The likely cause is a difference in experimental design.
The earlier study gave the model a human-written passage and asked it to continue in the
same register, whereas this work asked the model to write a document.

Earlier research can suggest hypotheses, but its effect sizes cannot be carried over to a
different task.

## D-09 The maintainability layer is not calibrated to the human average

The AWS S3 user guide is the only corpus that writes list lead-ins correctly.
In the Linux kernel, incomplete lead-ins outnumber complete ones.
If this layer were calibrated to the human average, it would lose its purpose.

This layer is not meant to remove AI style. Its purpose is to make a document better than
most documents written by people.

## D-10 What a prompt can prevent is separated from what only a linter can catch

A condition that included an instruction about formatting was compared with a condition
that did not. The formatting disappeared completely, while vocabulary and sentence structure
did not change.

The same experiment was run with an instruction about maintainability.
The structural rules were followed completely under a short instruction, whereas
time-dependent words dropped to zero only under a detailed one.
Stating the number of items, numbering headings, and numbering list items by hand did not
occur even without any instruction.

`brief` therefore includes only the constraints that an instruction can enforce.
A constraint that has no effect wastes tokens and risks making the model overcorrect.

As a side effect, the maintainability instruction alone improved the document metrics.
Requiring a lead-in added prose to the output, so the prose ratio and the list ratio moved
toward the human range.

## D-11 The term list is built from the documents instead of being shipped

The hardest task in the metaphor layer is distinguishing a live metaphor from a dead one.
Cache and handshake are established terms in the field, not metaphors.
A sentence that says a buffer behaves like a waiting room marks the comparison, whereas a
sentence that calls the buffer a waiting room does not.

A word that becomes widely known for being singled out becomes less frequent.
The words singled out early in 2024 declined immediately afterward. A word list shipped with
the tool goes out of date.

A list built from your own documents does not go out of date.
Besides the fact that each project uses different terms, this property is what addresses
the problem of an outdated word list.

## D-12 The kango ratio test had the wrong expected value

`kango_ratio_counts_only_content_words` was failing.
For the input 設定を変更すること, the implementation returned 0.5, and the test expected a
value greater than 0.5.

`TASKS.md` says that when a test fails, the implementation is fixed. This case is an
exception. The comment in the test counted three content words, 設定, 変更, and こと, but the
`FakeMorphology` specification also registers する as a verb, which makes four.
The metric definition includes in the divisor every content word whose origin is 和, 漢, 外,
or 混, so the verb する is counted as well, and the value of 2/4 that the implementation
returns is the value the definition requires.
Changing the implementation to match the test would violate the definition.

The purpose of the test is to confirm that particles are excluded from the divisor.
Changing the expected value to exactly 0.5 keeps that purpose intact.
Including the particle を in the divisor would give 0.4, so the two cases can still be told
apart.

The implementation is fixed when it departs from the definition.
The expected value is fixed when the expected value departs from it.

## D-13 M2 and M6 count every occurrence

The reference implementation counts each occurrence within a line, whereas the Rust
implementation reported no more than one finding per block.
In `tests/golden/input/en_maintainability.md`, a single line contains two of the listed
words, and the reference returned 2 where Rust returned 1.

Counting every occurrence was chosen, and the reason comes from a constraint of this project.
If only one of two findings on a line is reported, a document edited to fix the reported one
fails `check --quiet` again.
That would defeat the design goal of never linting and fixing one layer at a time.

The unused `column` field on `Position` shows that a position for each occurrence was
intended from the beginning.
Giving each occurrence its own column makes two findings on the same line distinguishable.

M7 looks only at the end of a line, so it can report at most one finding per line, and this
change does not affect it.

## D-14 Word and sentence counts are compared by relative error

The comparison policy allowed an absolute error of 0.02 for a ratio and 0.5 for a density,
and it required exact equality for the M rules. A word count fits none of these categories.

The word count of `en_prose.md` was 445 in the reference and 444 in Rust.
The cause is `trade-off`. spaCy splits this word into three tokens and discards the
punctuation, which yields two words, while splitting on whitespace yields one.
The file contains exactly one hyphenated word, which accounts for the entire difference.

The difference comes from how the two tokenizers define a word, not from an error in the
implementation. Requiring exact equality would mean requiring two different definitions to
produce the same number. A relative error of one percent catches systematic drift and
tolerates everything else.

## D-15 The definition and the reference measure formal nouns over different text

The metric definition measures vocabulary over prose and the text of list items, and it
lists formal nouns among the vocabulary metrics. The Rust implementation follows the
definition.

The reference implementation, `research/jametrics.py`, reports demonstratives both over
prose alone and over prose combined with list items.
For formal nouns, however, it reports only the figure for prose alone.

On `tests/golden/input/ja_maintainability.md`, prose alone gives 14.98, and prose combined
with list items gives 15.77.
The difference of 0.79 is larger than the tolerance of 0.5 allowed for a density.

The implementation must not be changed in response. The definition gives its reason: some
models put almost all of their output into bullet lists, and for those models a prose-only
measurement discards more than eighty percent of the text.

The mismatch lies with the threshold instead.
The human range of 11.095 to 11.163 and the D5 threshold of 8.5 were both measured under the
prose-only definition. Including list items raises the value, so D5 now fires less often
than it did when it was calibrated.
This is a further reason why keeping D5 at info, as decided in D-07, was correct.

The mismatch remains unresolved until the calibration is redone.
Redoing it requires adding the figure for prose combined with list items to
`research/jametrics.py` and measuring the corpus again.

## D-16 The tool ships as a single binary with the dictionary embedded

To decide how to distribute the tool, the size of the embedded dictionary was measured.

| Contents | Size |
|---|---|
| Binary with the dictionary | 191 MB |
| The same, compressed with xz | 26 MB |
| Binary without the dictionary | 1.1 MB |

`strip` makes no difference, because most of the size consists of dictionary data, not
symbols.

Keeping the dictionary outside the binary and downloading it on the first run is also
possible. lindera 6.0.0 provides `load_dictionary_from_path`, which reads a dictionary
without copying it. Embedding was chosen nonetheless, because of how the tool is used. For a
tool that runs from hooks on every Write and every Edit, reliability depends on how few ways
it has to fail.

An external dictionary adds three ways to fail: the dictionary can be missing, corrupted,
or of the wrong version. An upgrade that replaces the dictionary, for instance, would leave
the binary reading features from the wrong positions. Self-diagnosis detects a version
mismatch, but detecting it still leaves the user unable to proceed. A tool that needs no
network access after download fits this use better.

The unit of distribution is one archive for each operating system.
Combining every operating system into one archive would make it 78 MB compressed and would
force every user to download builds for systems they do not use.

The targets are x86_64 Linux with musl, macOS on Apple Silicon, and x86_64 Windows.
Linux builds use musl so that the version of glibc does not matter.
A tool that fails on an older distribution or inside a container is a serious problem when
hooks call it. Intel Macs and ARM Linux are not targeted, and anyone who needs those builds
compiles from source.

## D-17 The dictionary feature is disabled by default

`suikou-cli` gained the `lindera-unidic` feature, and the feature remains disabled by
default. Enabling it by default would pull the dictionary into `cargo test --all` and break
the premise of D-04 that the core can be tested without a dictionary.

Two measures cover the resulting gap.
The release build enables the feature explicitly and runs the binary to confirm that the
dictionary is present before packaging.
A build without the dictionary exits immediately inside `morphology::load` when Japanese
analysis is requested.

Silently returning an empty analysis is not acceptable.
An empty tokenization drives the kango ratio, the renyo count, and the formal noun count to
zero. Zero lies below the thresholds, so some findings fire and others do not, and one way
through that mixture ends in a false report that nothing needs fixing.

`suikou --version` distinguishes the two builds.
A user who cannot identify the binary in use cannot find out why Japanese text was not
analyzed.

## D-18 The release artifact is verified against the dictionary itself

Running the release workflow under `act` revealed this problem.
The packaged binary was only 1.2 MB, yet `--version` reported that the dictionary was
embedded. The dictionary is 191 MB, so both statements cannot be true.

The cause was dead code elimination.
Every subcommand was still a `todo!()`, so no code path reached `morphology::load`, and the
linker removed the embedded dictionary together with it.
After a code path that reaches it was added, the binary grew to 191.5 MB and returned a real
analysis.

The smoke test was also at fault.
The `--version` string is chosen at compile time by `#[cfg]`, so it reports only whether the
feature was enabled and says nothing about whether the dictionary is present.
A binary without a dictionary passes that check.

A `selftest` subcommand now loads the dictionary and has it analyze a sentence.
The release is verified by how the artifact behaves rather than by the settings it was built
with, which is the same reasoning as verifying the feature order at startup.

As a side effect, `morphology::load` gained a caller, so the warning suppression in
`crates/suikou-cli/src/morphology.rs` could be removed.
Users also gain a way to confirm that the binary they have is what it claims to be.

## D-19 Line numbers in findings refer to the file before preprocessing

Running `suikou check` on this project's own documents revealed this problem.
The reported line numbers were four lines above the actual lines, and four is exactly the
number of lines in the front matter that had been removed.

Preprocessing deleted the front matter and the fenced code blocks entirely, so every line
after them moved up and the line numbers no longer matched the file.

Each removed span is now replaced with the same number of newlines.
Empty lines are not counted, so the number of non-empty lines and the list ratio do not
change.

A finding from a linter has to refer to a line in the file the user has open.
A finding with a correct value but the wrong position is of no use to anyone.

## D-20 The documentation is a Hugo site with English as the default language

The documentation moved from files placed directly under `docs/` to
`docs/content/{en,ja}/`, and English is the default language.

No external theme is used; a minimal set of layouts is kept in the repository instead.
This keeps the number of dependencies low, and it also applies the project's policy of
pinning versions to the documentation.

Keeping a single canonical copy of each page also means that the site and the repository
never hold the same content twice.

## D-21 The detail level of brief is set by a flag, not by a profile field

The design document describes an option in which a profile stores the detail level for
brief. This implementation does not adopt that option and controls the level of detail only
through the flag `--detail concise|balanced|detailed`.

The reasoning is as follows. Profile is a struct that holds metric thresholds. Adding a
field specific to brief would give the struct two responsibilities: holding values that
calibration writes back, and holding a value that only selects the shape of the output.
Mixing the two kinds of value in one struct would make it unclear which values
`suikou baseline` is supposed to write, so this implementation keeps Profile limited to
holding thresholds.

The flag also gained a third level that the original plan did not include. TASKS.md
initially listed only concise and detailed, but during implementation a middle level,
balanced, was added, because the structural rules each fit in one short sentence while the
time-dependent words still had to be listed in full every time. Concise shortens that
combination of short sentences and a full list even further, and detailed adds the
reasoning behind each rule.

The profile-based option remains on the list of open questions. Whether to move the detail
level into a profile will be decided only when an actual request for a per-project default
arrives.

## D-22 The baseline threshold is set with the Tukey outlier fence

Implementing `suikou baseline` required a method for choosing the threshold value.
`docs/content/en/design.md` places each threshold between the upper end of the human range
and the lower end of the AI range, as with the 0.60 threshold for `ja.mattr100`. Baseline,
however, sees only a human corpus, so it has no way of knowing where the AI range lies.

The chosen approach derives a boundary from the human distribution alone, namely a point
that the distribution itself would classify as an outlier.

That boundary is the Tukey outlier fence. The interquartile range is the distance between
the first and third quartiles, and the fence lies `FENCE_K` times that range beyond the
nearer quartile. The coefficient of 1.5 comes from Tukey's definition of an "outlier" and is
widely used in statistics, which makes it easier to justify than a margin invented for this
project. A looser coefficient of 3.0 defines Tukey's "far out" points, but with the small
sample sizes that baseline usually handles, that wider fence would place the threshold too
far out to be meaningful, so this project does not use it.

The quartiles are computed with the hinge method. The sorted sample is split at the median
into two halves, and the median of each half becomes Q1 and Q3. When the sample has an odd
number of values, the middle value is excluded from both halves. A percentile method with
linear interpolation also exists, but the hinge method is simple enough that the expected
values in a test can be verified by hand.

A metric with fewer than four samples is not calibrated. With three samples or fewer, one
half of the split contains a single value, so the interquartile range reflects that one
value instead of the spread of the distribution. Filling an uncalibrated metric with zero or
a placeholder would hide the lack of evidence from anyone reading the profile, so the metric
is omitted from the output, and the reason is written to standard error.

The first implementation had a defect. A metric in the below direction is a ratio or a count
per thousand, and either one is zero or greater by definition. When the samples are tightly
clustered, Q1 - 1.5*IQR can fall below zero. Calibrating on this project's own Japanese
documents (`docs/content/ja/*.md`, `README.ja.md`, `CLAUDE.md`, `TASKS.md`) produced exactly
that result: a fence of -4.96 for `ja.demonstrative_per_1k` and -4.98 for
`ja.keishiki_meishi_per_1k`. A density or a ratio can never be negative, so no real document
can satisfy the below check (`value < threshold`) against a negative threshold, and the
rule can never fire. The profile still parses as valid TOML, so running `suikou check` with
it reports nothing, and a check that looks only for "nothing reported" mistakes this defect
for a successful calibration.

Clamping the fence to zero would not have fixed the problem. A density or a ratio already
has a lower bound of zero, so a threshold of exactly zero still requires a value strictly
below zero, which never occurs; clamping only replaces one unreachable threshold with
another. Instead, a below-direction fence at or below zero is now treated as a calibration
failure in the same way as too few samples: the metric is omitted from the output, and the
reason is written to standard error.

The above direction does not receive the same treatment. All metrics share the same lower
bound of zero, but they do not share an upper bound: a ratio cannot exceed 1, while a count
per thousand has no fixed maximum. Adding a separate maximum for each metric to the
calibration logic would bring knowledge specific to each metric into code that is otherwise
generic, so the fix applies only to the below direction.

`guidance`, `direction`, and `severity` are all copied unchanged from the bundled oss
profile. If baseline rewrote that text on every run, its output would become outdated the
next time someone edited oss.toml, so calibration changes only `value`.

The corpus that `corpus/fetch.sh` downloads is not available in this environment, because
licensing keeps it out of the repository. As a result, it remains unverified whether this
method reproduces `corpus/baselines.toml` or the order of magnitude of the bundled
profiles. The documents under `tests/golden/input/` are too few to serve as a substitute; no
single metric there reaches the minimum of four samples. TASKS.md keeps this item open until
someone runs `suikou baseline` on a real human corpus and compares the result with
`corpus/baselines.toml`.

A below-direction threshold that survives calibration has been confirmed to work.
Calibrating on the documents listed above kept `ja.prose_ratio` (fence 0.69) in the
profile. Running `suikou check --profile` with that profile fires the rule on a document
that consists mostly of bullet lists and does not fire it on a document of prose alone;
both results were confirmed against actual command output.

## D-23 Judgment calls made while writing the textlint preset

Writing `packages/textlint-rule-preset-tech-maintainability` (T7) required several judgment
calls that could not be combined under one heading. Each one is described below in its own
paragraph, so that the reasoning behind one decision is kept separate from the reasoning
behind the next.

**Under a standard configuration, textlint does not print `maint/` as the rule prefix.**
The instructions for this task asked for rule IDs that match the Rust side, such as
`maint/list-lead-in`. Reading textlint's own code for loading presets showed that the
prefix textlint prints is derived from the configuration key in `.textlintrc`, with the
leading `preset-` removed. When this package is loaded in the standard way, as
`"preset-tech-maintainability": true`, every finding is printed with the prefix
`tech-maintainability/`. A configuration key must begin with `preset-` to be recognized as
a preset, and that same `preset-` is always removed before the remainder of the key becomes
the prefix, so no configuration key can produce a `maint/` prefix through the standard
loader.

The name of each rule (the part after the prefix, such as `list-lead-in`) does match the
Rust side, as requested. Only the prefix follows textlint's own convention. Changing it
would require renaming the package to something like `textlint-rule-preset-maint`, which
would no longer match its directory and would confuse readers more than the mismatched
prefix does. The package name therefore stays aligned with its directory, and the README
states clearly that the printed prefix of a finding is `tech-maintainability/`. The golden
tests avoid the prefix question entirely: they call `@textlint/kernel` directly and
register each rule by hand under a `maint/` prefix, so the constraint described above has
no effect on that comparison.

**Rules match against the raw source, not the textlint AST.** textlint passes a rule the
Markdown divided into nodes such as paragraphs and headings. A paragraph that soft-wraps
across several lines becomes a single Paragraph node inside textlint. `markdown.rs`, in
contrast, builds one block for each line, and a soft-wrapped continuation remains a
separate line. The difference appears in a paragraph such as lines 14 and 15 of
`en_maintainability.md`, where one sentence spans two lines. Matching against the text of
the joined node would combine those two lines into one string and allow a regular expression
to match across words that the Rust side never joins.

There were two options: match against the text of textlint's Paragraph nodes, or port the
line-based block builder to JavaScript and take nothing from textlint except the raw source
string. The first option looks more like a typical textlint rule, but the number of findings
diverges whenever the granularity of the nodes differs from that of the lines. The second
option was chosen. `src/lib/blocks.js` ports the block builder from `markdown.rs`, including
its regular expressions, and each rule reads the raw source through
`context.getSource(node)` and passes it through this local block builder before making any
judgment. textlint's role is reduced to reading the file, applying the configuration, and
reporting; it does not parse the structure of the document.

The cost of this choice is that reported line and column numbers are computed against the
preprocessed string instead of the raw source. A position in the raw file can differ from
the corresponding position in the preprocessed string wherever an escape sequence was undone
or inline markup was removed. `src/lib/position.js` searches for the matched text within the
target line from left to right and maps it back, approximately, to a character position in
the raw source. The approximation exists so that no finding is lost, not to guarantee an
exact position. The comparison with `tests/golden/` checks counts only; the accuracy of
positions is not part of that requirement.

**IPADIC merges UniDic's four taigen categories into one part of speech.** For Japanese,
M1 and M5 check whether the last token at the end of a line or an item is a taigen, that
is, a noun-like word that can end a sentence by itself. suikou-core uses UniDic and treats a
token as taigen when its top-level part of speech is noun, suffix, pronoun, or adjectival
noun stem. kuromojin, the standard analyzer among textlint tools, uses IPADIC, which does
not make this four-way distinction at the top level.

Testing kuromojin 3.0.1 directly showed that words UniDic classifies as suffix, pronoun, or
adjectival noun stem all appear under IPADIC as nouns, with pronoun, suffix, or
adjectival-noun stem as their finer subcategory. "これ" (this) was returned as noun/pronoun,
and "静か" (quiet) was returned as noun/adjectival-noun-stem. Under IPADIC, the four-way
UniDic check therefore reduces to a single condition: the top-level part of speech is noun.
`isTaigen` in `src/lib/morphology-ja.js` is based on this test.

The check for a verb or auxiliary in the ren'youkei required the same kind of verification.
UniDic subdivides that form, as in "連用形-一般", while IPADIC returns the plain string
"連用形" with nothing appended, as confirmed with "使い" and "し". The Rust-side check,
`cform.startsWith("連用形")`, can therefore be ported without change.

**Messages follow the language of the document, whereas the Rust side always uses Japanese.**
`finding()` in suikou-core returns a Japanese message even for an English document, as
confirmed from the output of `suikou check` on `en_maintainability.md`.

Some users of the textlint preset write only English documents, and a message they cannot
read is of no help to them. The judgment logic and the regular expressions stay aligned with
the Rust side, but the preset chooses a Japanese or English message to match the language it
detects. Both sets of messages are stored in `src/lib/messages.js`. The text of the messages
plays no role in the comparison of counts, so this difference does not affect the
completion criterion of the golden tests.

**Tests use `node:test` directly instead of textlint-tester.**
textlint-tester assumes that Mocha's global `describe` and `it` exist. When those globals
are absent, it falls back to a substitute that simply calls the test function without
waiting for it. That substitute never awaits the Promise returned by a test case, so under
`node --test` some assertions reported success before the linting they depended on had
finished.

Two options were considered: adding Mocha as a dependency so that textlint-tester could be
kept, or writing a thin layer directly on `@textlint/kernel`. The option that added no
dependency was chosen. `test/support/lint.js` wraps the kernel in a few lines, registers
every rule under the `maint/` prefix, and awaits the result directly. This works with the
asynchronous test functions of `node --test` without any adapter.

**The overlap with `no-ai-colon-continuation` from
`@textlint-ja/textlint-rule-preset-ai-writing` is deliberately not suppressed.** That rule
also uses kuromojin to check whether the text before a colon ends in a predicate, which is
similar to the Japanese check in M1.

The two rules serve different purposes: that rule flags a stylistic pattern typical of
AI-written prose, while M1 flags a list structure that breaks when an item is added or
removed. Even so, both can fire on the same line. No mechanism lets one suppress the other.
A user who enables both presets reads both findings and can judge which one applies to the
document; suppressing one on the user's behalf would remove information instead of adding
it. The overlap is documented in the README instead.

## D-24 Term extraction divides words by structure as well as by part of speech

In Japanese, a run of consecutive nouns is grouped into a single compound word.
UniDic assigns a bound suffix such as 素 or 性 its own part of speech, separate from noun,
so treating only nouns as part of a compound split 形態素解析 into 形態 and 解析 and reduced
保守性 to 保守. This problem was found by running the tool on its own documentation.

Treating the suffix tag like the noun tag fixes that split, but it also allows a counter
such as つ to begin a compound by itself. UniDic provides no finer tag that would separate a
counter from an ordinary derivational suffix, so maintaining a list of counters was not an
option. The rule relies on structure instead: a suffix can only extend a run that already
contains a noun. A suffix separated from the run by a numeral, such as the つ in 3つ, never
begins a run by itself.

UniDic treats a space only as a boundary between words and does not keep the space as a
token. Joining two Latin-script words without a space therefore creates a word that does not
appear in the source text. Running the tool on its own documentation showed this: Linux
kernel, which appears in several places in `design.md`, came out as Linuxkernel. The join
now inserts a space between two adjacent Latin-script words, and inserts no space between a
Latin-script word and a Japanese word, as in 仕様 attached to API. Inside a Japanese sentence
that pair is normally written without a space, so the join does not add one either.

The list that filters out generic words reuses `KEISHIKI_MEISHI` from `metrics/ja.rs`
instead of creating a new list. A generic word that slips into the glossary only loses its
exclusion from the metaphor check, which is a small cost. Missing a genuine term of the
field costs more, so the balance favors keeping words in.

Running the tool on its own documentation revealed a further problem: single-character
words occupied the top of the word list. Of 395 words, 54 were a single character, and the
top twenty began with 文 (50), 形 (49), 語 (48), 指示 (43), and 値 (36), continued with 地 (28)
and 数 (27), and reached 版 (22), 行 (22), and 層 (20). 地 is the remainder left after の split
地の文 into two words, and the other words are common Japanese words that appear frequently in
any kind of writing and are not specific to any field.

Listing the words to exclude by name was rejected, because choosing them would itself be
guesswork. The fix relies on a structural property instead, namely the number of characters
in the assembled word, and a word of one character is excluded. Excluding one-character words
is common practice in Japanese term extraction: most of the meaning is carried by words of
several characters, while a one-character word is the smallest possible unit, tends to have
many meanings, and rarely identifies a single field. A two-character word made of one token,
such as 文書, passes through this filter unchanged. After the fix, the top words for this
repository's own documentation were 指示, 規則, 指標, 人間, 文書, 日本語, 指摘, 閾値, 辞書, and 実装,
and none of them was a single character.

English has neither a part-of-speech tagger nor a sentence splitter here. The extraction
therefore relies on words written entirely in capitals, capitalized words that do not begin a
sentence, and inline code that consists of a single identifier. Table rows are excluded from
this scan. A table cell does not contain a sentence, so the start-of-sentence signal never
applies there, and the same label repeats down a column often enough to pass the minimum
occurrence count. Running the tool on its own English documentation showed table values
entering the glossary in this way.

A heading that begins with an ID has the same problem. In this file's own heading,
D-04, The tokenizer hides behind a trait, D-04 was treated as the first word, so the
following word, The, was read as if it were in the middle of a sentence. The fix continues
to treat the position as the start of a sentence for as long as words written entirely in
capitals appear at the beginning of the text.

A heading with two ID-like words next to each other has a further problem. In this file's
own heading, D-13, M2 and M6 count every occurrence, nothing separates D-13 from M2, so the
two merge into a single word, D-13 M2. The word M2 on its own loses that occurrence and can
fall below the minimum occurrence count. The error is an omission, a real word left out,
rather than an invention, a word the source never contained, so it is recorded as a known
limitation and not fixed. Fixing it would require a rule that decides how far two adjacent
ID-like words should be merged, and no rule has been found that does this without breaking a
pair such as AWS S3, where two short adjacent words together name one product.

The minimum occurrence count is two by default. A word that appears only once offers no way
to distinguish a term of the field from an incidental phrase, and checking a term for
consistent use requires the same word to appear more than once in the first place. This
default is derived from those two purposes, not from measurement.

## D-25 MCP is implemented by hand on JSON-RPC instead of with a new dependency, and there is no daemon

T9 required two judgment calls: how to implement the MCP server, and whether to build a
daemon.

**MCP is implemented with `serde_json` alone.** The choice was between adding a dedicated
crate such as `rmcp` and writing by hand the three methods that this server answers:
`initialize`, `tools/list`, and `tools/call`.

The server handles only those three requests and uses none of resources, prompts, or
notifications sent by the server. A dedicated crate would require pinning a crate version
for features the server never calls, which does not fit this repository's practice of
pinning dependencies with `=` and rerunning `verify_schema` on every update. `serde_json`
is already a dependency of both crates, so the number of new dependencies is zero. The
stdio transport also comes down to one JSON-RPC message per line, which is simple enough
that writing the transport by hand carried little risk. The implementation is in
`crates/suikou-cli/src/mcp.rs`; it calls `check::analyze` and `brief::run` and does nothing
else, so the judgment logic is not duplicated between the CLI and the MCP server.

**There is no daemon.** The original reason for a daemon was to load the dictionary only
once. The loading time measured in T2, 0.68-0.70 ms in a release build, did not support that
reason, and TASKS.md required measuring the cost of process startup itself before deciding
either way. A release build (`--features
lindera-unidic`) ran `suikou check` on a short document fifty times, measuring the
wall-clock time of each run; `suikou --version` was also measured over fifty runs for
comparison. The table below lists the measured times on WSL2.

| command | average cost | max cost |
|---|---|---|
| `suikou --version` | about 1.5-1.6 ms | about 1.9-2.4 ms |
| `suikou check` (with the dictionary, short document) | about 3.7-3.9 ms | about 4.3-4.7 ms |

`suikou --version` never loads the dictionary, so its time represents process startup
alone: reading the roughly 200 MB executable, linking it, and parsing its arguments. The
additional time for `suikou check` is about two milliseconds, which covers loading the
dictionary (0.7 ms) and measuring one short document. Both times are negligible compared
with the seconds that one LLM turn takes, which is the setting in which a hook runs on every
Write and Edit. A daemon would add complexity of its own — for example, managing a socket or
a named pipe, deciding how to detect file changes, and serializing concurrent callers — and
the time it would save is too small to justify that complexity. `suikou daemon` remains as a
subcommand; running it prints the measured times and this decision instead of doing any
work. Keeping the subcommand, instead of removing it entirely, lets a user who runs it see
directly why it does nothing.

## D-26 Native words out of register are detected by comparing frequencies with a reference corpus

When a generative model translates English into Japanese, it sometimes replaces a
Sino-Japanese word with a native word. 確認する becomes 確かめる, and 必要である becomes 要る.
The result is too informal for technical documentation and slows the reader down.

Which register is appropriate was decided without building a table of word pairs, because
such a table would require listing every difference in phrasing across every field. The rule
compares frequencies with a corpus of professional Japanese translation instead.

Measurement supports that decision.
In the Kubernetes documentation alone, 使う appears 320 times in the Sonnet translation and 33
times in the professional translation, which suggests a rule that replaces 使う with 使用.
Extending the corpus to Kubernetes, MDN, and Vue, however, shows 使う 461 times in professional
translation. Candidates such as 次 for 以下, 仕組み for メカニズム, and 既定 for デフォルト were
rejected for the same reason. MDN uses 既定, so the convention differs from field to field.

A finding requires two statistics.
The log-likelihood ratio of Dunning (1993) shows whether the difference is significant, and
the Log Ratio of Hardie (2014) shows how large it is. Significance alone is not enough,
because in a corpus of this size even an ordinary word used slightly more often becomes
significant.

The minimum Log Ratio was determined by measurement.
Twelve professional pages that were not used to build the reference were compared with this
repository's Japanese pages. A minimum of 4 produced findings for 9 words in the
professional pages, a minimum of 6 produced 1, and a minimum of 7 produced none.
The design treats a warning on human-written text as a false positive, so the minimum is set
where the professional pages produce no findings, at the cost of missing some real ones.

Building the reference corpus is subject to two conditions.
First, tutorials must be excluded: the Rust Book translation contains 確かめる 36 times, which
would bring that word into the reference. This agrees with the measured result that the kind
of document affects the metrics more than the register does.
Second, a word must appear in two or more fields, because a word confined to one field is
part of that field's vocabulary and cannot serve as a reference.

`.suikou/register-allow.toml` lists the words that a project accepts.
The reference was built from web documentation, which contains little vocabulary from
measurement and arithmetic, and each project fills that gap itself. This follows the same
approach as D-11, in which the glossary is built from the documents at hand.

## D-27 The guidance attached to the kango ratio was wrong

D7 told the writer to replace kango with native verbs. Measurement contradicts that advice.

The table below compares the word origin of content words in professional translation and in
this repository.

| | Native | Sino-Japanese | Loanword | Mixed |
|---|---|---|---|---|
| Professional translation | 0.384 | 0.329 | 0.280 | 0.007 |
| This repository | 0.470 | 0.463 | 0.050 | 0.017 |

Professional translators do not keep the kango ratio low by using native verbs. They keep it
low by using established loanwords, and their proportion of native words is in fact lower
than this repository's. The guidance now says to use the established loanword and to avoid
stringing kango nouns together.

The wrong guidance caused real harm.
A rewrite that followed it turned 検証 into 確かめ, 制約 into 縛り, and 抽出 into 取り出し, and the
register of the text deteriorated. The rule in D-26 exists to detect that kind of
deterioration.

Reducing a metric to zero is not the goal.
D7 has the severity info, which marks it as advisory, and treating it as a requirement was
the original mistake. A document in this repository must produce no errors and no warnings.
Anything reported at info is read as advice and nothing more.

## D-28 The register rule could not be applied to English

Two experiments attempted to apply the method of D-26 to English. Neither succeeded, so no
English rule is provided.

The first experiment compared documents with a reference corpus.
A frequency table was built from 652 pages and 510,000 words of Kubernetes, MDN, and Vue
documentation in English, and the same statistics were applied.
Fourteen professional English pages that were not used for the reference still produced
20 findings even with the minimum raised to 9.
The top of both lists consisted of field terminology: portworx, cinder, and ebs on the
professional side, and prose, corpus, and metric on this repository's side.
Restricting the comparison to words that appear in all three fields still left words such as
operations, which belong to the topic of a page rather than to its register.

The cause lies in the structure of the vocabulary.
In Japanese, the condition that the word origin is native removes field terminology almost
automatically, because field terminology consists of Sino-Japanese words and loanwords.
English vocabulary has no such marker. A document about a particular topic uses that topic's
vocabulary far more often than a general corpus does, so frequency alone cannot separate
topic vocabulary from register vocabulary.

The second experiment held the topic constant.
A model translated professional Japanese translations back into English, and the result was
compared with the original English across 36 pages and 30,000 words on each side.
Not a single word reached significance. Against a critical value of 15.13, the highest value
was 13.2 for your and the next was 11.1 for will, and the original English uses both words
more often than the model does, so the difference does not even point the opposite way.
The same design applied to Japanese gave 使う 320 occurrences against 33, with a
log-likelihood ratio of 320.

This result has the same character as D-08.
An effect size must not be carried from one task to another, and a method that works in one
language must not be carried to another language either.

Only word choice was tested in these experiments.
MATTR, the prose ratio, and the simile markers measure different properties, and they
remain in place.

`research/lexshift/` reproduces both experiments.
`build_reference_en.py` builds the English frequency table, and `translate_back.sh` produces
the back-translation. The unsuccessful experiments are kept so that nobody has to run them
again.

## D-29 A Sino-Japanese word is replaced with the loanword when the corpus favors the loanword

D-26 decided against a table of word pairs, because the preferred form reverses from one
field to another. That decision is revised for pairs of a Sino-Japanese word and a katakana
loanword. A counterexample no longer disqualifies a pair: the form that prevails across all
the fields combined is the form to follow.

Several mechanical ways of identifying synonyms were tried and abandoned.
Using an English word as a pivot and linking whatever co-occurs in the aligned paragraph
finds collocations instead of synonyms, producing pairs such as 永続 with ボリューム and 番号
with ポート. Lift and the Dice coefficient give the same kind of result.
Substitution is a matter of one word taking the place of another, not of co-occurrence, and
a paragraph is the wrong unit in which to observe it.

Listing the candidates and deciding among them were therefore separated. A person lists the
candidates and measurement decides, which is the same approach used in D-26. Of 97
candidates, 53 were kept.

Three conditions determine whether a pair is kept. The loanword must occur at least twice as
often as the Sino-Japanese word, the pair must occur at least twenty times in total, and the
Sino-Japanese word must occur no more than sixty times.

The third condition excludes words with more than one meaning.
Measurement shows that professionals use 対象, 対応, 状態, and 場合 often, and not always with
the meaning of the loanword. This repository uses 対象 to mean the scope that a metric
covers, which has nothing to do with an object. Limiting the table to words that
professionals rarely use keeps each pair reliable.

A word that follows a noun is skipped because it is part of a compound; replacing the 版 in
英語版 or 第3版 would be wrong.

`.suikou/register-allow.toml` exempts a word that a project uses with a different meaning.
This repository exempts 記録, 一覧, 比率, 経路, 分岐, and 実体; for example, 比率 here means a
ratio such as the kango ratio, not a rate.


## D-30 Structure serves as a guardrail, and the outline is written before the prose

Rules that work at the level of the sentence cannot repair a document whose structure is
wrong. A missing section, an order that does not follow, or a paragraph that makes three
claims at once cannot be seen while reading one sentence at a time.

The structural rules apply to both languages.
The set of sections, their order, and the paragraph as a unit are properties of the argument
a document makes, and they are independent of the vocabulary of any particular language.
Measurement confirms this. Paragraphs of seven or more sentences account for 0.27 percent of
paragraphs in professional English and 0.59 percent in professional Japanese.
In this respect the result differs from D-28, where the register rule could not be applied
to English.

The doctypes and their required sections were taken from existing standards instead of being
invented. RFC 7322 specifies required sections in a recommended order. The sections of each
doctype come from the Rust RFC template, the Kubernetes Enhancement Proposal, Michael
Nygard's Architecture Decision Record, and Diátaxis. The limit on paragraph length follows
the Google developer documentation style guide and is set at six sentences.
The constraint against adding rules based on intuition applies here as well.

Only a missing section is reported as an error. Order, paragraph length, and an empty
section are reported as warnings. A document that lacks a required section cannot answer
the question its reader came with, whereas order and paragraph length describe a path
through the argument and are not rules that must never be broken.

Section matching considers only the top level of headings.
A leading level-one heading is the title, not a section, so it is excluded.
Considering deeper levels lets a subheading inside one section match a different section
and disrupts the order check, which is exactly what happened with the English design
document in this repository.

Only the first section can be satisfied by the lead paragraph instead of a heading.
The abstract in RFC 7322 appears between the title and the table of contents without a
heading of its own, and a README follows the same pattern. Requiring a `## What it is`
heading at that point reads worse than the paragraph it would replace.

### The outline is approved before the prose is written

`suikou plot` writes a template into `.suikou/plans/`.
The template contains the section headings, the question each section answers, and an empty
line for the claim that each section makes. A section whose claim cannot be written in one
line has not been thought through, and the plot reveals this before the body is written.

The plot is kept out of version control, because it records the thinking and is not a
deliverable. CI therefore cannot see it, and the comparison with the body runs only on a
local machine and inside an agent's loop. This is not a weakness. A structural mistake is
supposed to be caught before writing begins, not blocked by a check just before release.

### Guidance in addition to detection

All of the rules described so far worked in one direction: find a violation and then
remove it. A list of violations shows what to avoid but does not show what to write.

The template therefore includes the question each section answers and what the section
should contain, and `suikou plot` and `suikou brief --doctype` present this to the writer
before the body is written. The text of the guidance is kept in one place,
`structure::writing_rules`, so that it cannot diverge from the text of the findings.

Token output does not increase. The template is printed only when `--doctype` is given.

## D-31 Measurement removed two required sections from the how-to type

The required sections of each doctype were taken from an existing framework instead of
being invented. Diátaxis describes a how-to guide, and prerequisites, steps, and a
verification became the three required sections. Two of the three failed as soon as they
were applied to professional documentation.

Applying `howto` to 120 Kubernetes task pages reported `Steps` as missing on all 120 and
`Verify` as missing on 113. Not a single page passed.

Counting the headings shows why. Of the H2 headings on those pages, 111 are
`{{%/* heading "prerequisites" */%}}` and 84 are `{{%/* heading "whatsnext" */%}}`. The rest are
specific to each page, such as `Create a namespace` and `Edit a Secret`, and each one names
the operation it covers.

The steps of a how-to guide are its body, not a section within it. Diátaxis says that a
how-to guide is a sequence of steps, not that a heading reading `Steps` must appear above
them. Requiring that heading would remove from the outline the one thing it conveys, namely
what each step does. That contradicts the guidance that the headings alone should convey the
line of the argument.

`steps` and `verify` are therefore no longer required. `prerequisites` remains required.

### A second measurement revealed two more defects

In the first run, all 120 pages satisfied `prerequisites`. They did not actually satisfy it;
the rule was too loose to detect the gap.

Which section the lead paragraph could substitute for was decided by position. Any document
with an opening paragraph therefore satisfied its first required section, whatever that
section was. Applying `howto` to 120 concept pages reported nothing on any of them, which
means the type was not distinguishing anything.

Whether a section can be satisfied by an opening paragraph depends on the nature of the
section, not on its position. Only a section that states the subject now carries `lead_ok`.
Prerequisites and context cannot be satisfied by an opening paragraph. After the change, 117
of the 120 concept pages report a missing section, and the type distinguishes documents
correctly.

Section synonyms were matched only against the language of the document. The Japanese
translation of Kubernetes keeps the `{{%/* heading "prerequisites" */%}}` shortcode as it is,
so matching only Japanese synonyms reported a missing section on 110 of 111 pages. Matching
now uses the synonyms of both languages, because the language of a document and the
language of its headings need not be the same.

### Rates after the fixes

| Corpus | Pages | Missing a required section |
|---|---|---|
| English task pages | 120 | 9 (7.5%) |
| Japanese task pages | 111 | 6 (5.4%) |
| English concept pages run as `howto` | 120 | 117 (97.5%) |
| Japanese concept pages run as `howto` | 120 | 116 (96.7%) |

The two languages fall within the same range. When the rule is applied to documents of
another type, it fires on nearly every page, and when it is applied to documents of its own
type, it rarely fires. That contrast is what shows that the type works. The pages that fire
really do lack `Before you begin`, and several declare `task` while being explanations;
`Dependency on Docker explained` is one example. These are places where Kubernetes departs
from its own template, not false positives of this tool.

### Removed from detection but kept in the guidance

The question and the note on what the section should contain remain in the doctype.
`suikou plot` and `suikou brief --doctype howto` still ask how the task is done and how the
reader can tell that it worked.

What can be detected and what is worth asking before writing are not the same.
Telling the reader how to confirm the result is a sign of a good procedure, but the presence
of a heading reading `Verify` does not measure it. Separating those two things is the core
of this decision.

### Only the how-to type could be measured against outside documents

The required sections of `design`, `decision`, and `overview` have been tested only against
the documents in this repository. Those documents were written to match the doctype, so the
test confirms nothing.

Measuring against an outside corpus requires documents that declare a type. Kubernetes
declares `content_type`, whose values are `task`, `concept`, `reference`, and `tutorial`,
and only `task` corresponds to a type defined here. Diátaxis explanation and tutorial would
be the types for the other two, but no type will be added until there is something to
measure it against.

## D-32 The design type failed against the Rust RFCs it was based on

The required sections of `design` were said to come from the Rust RFC template. To test
that claim, the type was applied to 200 RFCs from the top level of `text/` in
`rust-lang/rfcs`, taken from the highest numbers downward so that the template would already
be established. Applying a type to its own source is the strictest test available.

The first run reported `Design` as missing on all 200, `Non-goals` on 199, and `Scope` on
198. Each of the three had a different cause.

### A mismatch of names

The 200 reports for `Design` did not come from a wrong requirement but from a wrong name.

The Rust RFC template calls that section `Guide-level explanation` and
`Reference-level explanation`, and the corpus contains 157 of the first and 148 of the
second. `Explanation` was added to the synonyms, since both names contain it.
`Implementation` and `Proposal` were added as well, because design documents use these names
widely.

### A required section that the template does not contain

`Scope` and `Non-goals` do not appear in the Rust RFC template. They come from the
Kubernetes Enhancement Proposal, so two templates had been merged into one set of
requirements.

Both sections are now optional, and both keep their question and their note on what the
section should contain. `plot` and `brief` continue to ask how far the proposal goes and what
it will not do, as in D-31.

### Two defects in the parser

There was no limit on the indentation of a heading. Four or more spaces make a line code,
and CommonMark does not recognize a heading there. A `#` inside an indented code block was
read as a level-one heading, which made level one the top level of that document and hid its
real sections. Indentation is now limited to three spaces.

Fenced code blocks were removed with a regular expression. A long fence that contained a
shorter fence was closed at the inner fence, and the code after that point was read as
prose, which turned `# [dependencies]` into a heading. A line-by-line scan replaces the
regular expression: a fence closes only on a line that uses the same marker, is at least as
long, and has nothing after it. This is what CommonMark specifies.

Both fixes are ported to the textlint preset, and the two implementations return identical
M-rule counts on real RFCs.

### The rate after the fixes

21 of the 200 RFCs report a missing section, or 10.5 percent. Eighteen of those are not
feature proposals at all: they are roadmaps, project group charters, team changes, and
policy RFCs, which are stored in `text/` but do not follow the feature template. The
remaining three use a section name of their own, such as `Proposed additions` or `Solution`.

No more synonyms will be added. Adding section names without limit would bring the count
close to zero, but that would be fitting the rule to the corpus rather than describing a
template. A name is added only if it appears in a published template.

## D-33 The decision type held up, and section order could be measured

The required sections of `decision` were taken from Michael Nygard's Architecture Decision
Record. To test them, the type was applied to 190 ADRs collected from 28 repositories.
`research/structure/fetch_adrs.py` finds them starting from the first record that
`adr-tools` creates, and it excludes indexes and templates, which are not decision records.

| Rule | Out of 190 |
|---|---|
| Missing a required section | 20 (10.5%) |
| Sections out of order | 0 |
| Long paragraph | 6 (3.2%) |
| Empty leaf section | 2 (1.1%) |

Where `howto` and `design` failed, this type held up. The difference is whether one
template was adopted as a whole or several were merged. `design` had merged the Rust RFC
with the Kubernetes Enhancement Proposal, and `howto` had misread a description in Diátaxis
as a list of section names. `decision` adopts the ADR as it stands.

### Section order could be measured for the first time

Judging order requires at least two of the doctype's sections to be present. On Kubernetes
task pages, only 7 of 120 contained two, so the measurement showed nothing. Among the ADRs,
189 of 190 contain two, and the rule fires on none of them.

The order of context, decision, and consequences matches how people actually write. That is
the evidence for keeping this order as a recommendation.

### The remaining 20

Eleven follow MADR, which uses `Context and Problem Statement` and `Decision Outcome` and
places the consequences one level below `Decision Outcome`. Section matching stops at the
top level, so a consequences section nested under another heading is not found.

That limitation stays. Considering deeper levels lets a subheading inside one section match
another section and disrupts the order check, which is what happened in D-31. Supporting
MADR would require a separate doctype, not a deeper search.

Of the other nine, some contain nothing but `Status` and `Context`: they are decision
records that state neither the decision nor its consequences, which is a genuine gap. Others
write `Proposal` where the template says `Decision`.

No more synonyms will be added here either, following the same limit set in D-32.

## D-34 The natural-japanese rules were measured, and none was adopted

`coji/natural-japanese` (MIT) is a skill for writing readable Japanese in a work setting.
It provides 28 detectors, a catalogue of forbidden phrases, and a collection of
translationese patterns. Whether any of it could be adopted was decided in the way this
repository decides such questions.

A rule is added only if a measurement shows that the human and AI ranges do not overlap, or
if a clause in a published style guide supports it, with a citation. Candidates may be
proposed by hand, but measurement makes the decision. D-26 established that procedure.

### How the measurement was done

`corpus/cache/k8s/` contains a human translation and an AI translation of the same
documents, paired, so that differences in subject matter cannot explain a difference in the
results. The 798-document reference corpus was examined as well. The scripts are in
`research/translationese/`.

### Results

| Candidate | Human | AI | Reference | Verdict |
|---|---|---|---|---|
| 「することができる」 | 2.43 | 0.85 | — | The difference points the other way |
| Ten translationese patterns | near zero | near zero | — | Do not occur in technical prose |
| A conjunction opening a paragraph | 1.6% | 1.2% | 1.6% | Does not separate human from AI |
| Three identical sentence endings | 11.8% | 10.5% | 6.0% | Does not separate human from AI |
| Double negative | 0.16 | 0.11 | 0.16 | Does not separate human from AI |
| Three 「の」 in a row | 3.67 | 3.03 | 2.84 | Humans use it more |
| A 48-phrase catalogue | 1 at most | 5 at most | — | Below the G² threshold |

The figures are occurrences per 10,000 characters, except for the two given as a percentage
of paragraphs.

None was adopted. `corpus/baselines.toml` records each one under `[rejected]` together with
its reason.

### Why none of them separated human from AI

The phrase catalogue was designed for essays and business writing. Phrases such as
「いかがでしたか」 and 「見ていきましょう」 never appear in technical documentation; of the 48
phrases, only 19 occur anywhere in this corpus. A different kind of document brings out
different habits in a model, which is the same kind of result as in D-28.

The translationese patterns require attention to what they are compared with. The human
side of this corpus is a professional translation of English documentation, so it consists
of translation by nature, and this corpus cannot show whether a text reads as translated.
What could be measured was whether the AI translation reads as more translated than the
human translation, and it does not.

### What was adopted

No rule was adopted, but the evidence matrix was.
`corpus/reports/research/prose-rhythm-evidence-matrix.md` compares the published literature
with its own measurements for sentence-length variance, burstiness, autocorrelation, TTR,
and perplexity, and marks each one as counter-evidential.

The matrix reaches, independently, the same conclusion as this repository. For the
coefficient of variation of sentence length, it measures a false-positive rate of 32.4
percent on human text at the threshold in use. For autocorrelation, it finds an effect in
the opposite direction from the hypothesis and lists the detector as a candidate for
removal.

A different author reaching the same conclusion from a different corpus strengthens this
position, so the matrix is recorded as a source next to the `[rejected]` list.

### Both tools applied to the same document

A list of rules does not show what a tool actually detects. A Japanese design document of
the kind a model writes was therefore checked with both tools.

| | suikou | natural-japanese |
|---|---|---|
| Findings | 13 across 5 rules, plus 4 document-level directions | 6 across 5 categories |
| Rules in common | none | none |

The two tools found entirely different things. suikou reported an item count stated in
prose, numbered headings, list items that were not parallel, words that depend on a point in
time, and a Sino-Japanese word where the loanword is preferred; each is a place that breaks
on the next edit or goes out of date. natural-japanese reported clichés, an inanimate
subject, translationese, and rhythm; each is a place where the prose reads unnaturally.

Both tools flagged the same opening sentence, but for different reasons: one for a
time-dependent word and the other for a cliché.

On a professional page, the Japanese translation of a Kubernetes task, suikou reported two
findings and natural-japanese reported one. Neither tool produced much output there.

### Counts over a paired corpus

The tools were applied to twelve human translations and twelve AI translations of the same
documents.

| | Human | AI | Ratio |
|---|---|---|---|
| suikou | 43 | 62 | 1.44 |
| natural-japanese | 87 | 82 | 0.94 |

The breakdown shows where natural-japanese's count comes from. `translationese` reports 13 on
the human side against 2 on the AI side, and `translationese_morph` reports 19 against 2.
Both lean toward the human translation, which agrees with the measurement above: the human
side is a professional translation of English, so it consists of translation by nature.

### The one rule whose difference pointed the expected way

`repeated_sentence_lead`, which fires when the first two morphemes of a sentence repeat six
times in one document, was the only candidate that fired more often on the AI side. suikou
has no equivalent rule, so it was measured in the same way as the others.

| Set | Documents | Documents firing | Per 100 sentences |
|---|---|---|---|
| Human translation | 39 | 35% | 14.2 |
| AI translation | 78 | 50% | 16.4 |
| Reference corpus | 60 | 26% | 10.6 |

The direction is as expected, but the ranges overlap. The ratio of AI to human is 1.15, and
26 percent of the reference corpus fires. A rule is added only when the ranges do not
overlap, and these ranges do overlap.

The form of the output does not fit either. The rule marks every sentence it matches, which
produced 17 findings from a single document of 71 sentences, contrary to the constraint that
a rule shows only a limited number of positions.

The rule was not adopted, but its figures are kept on record as the only candidate whose
difference pointed the expected way.

## D-35 Dependence on a point in time is judged by the presence of an anchor, not by tense

A model adapts too closely to the context it is in. When it writes a document during a
session, it treats the moment of that session as if it were the permanent present of the
document. The sentence `現在レイテンシが悪化しており` is true only on the day it was written.

The justification for M6 is moved to this mechanism, away from the idea that documents go
stale. This explains why this rule alone is not followed under a short instruction and
requires every forbidden word to be listed, which `brief` has established by measurement.

### Tense does not predict whether a sentence goes stale

One proposal was to remove temporal expressions and keep documents timeless. That proposal
is not adopted.

| Sentence | Tense | Goes stale |
|---|---|---|
| `2026年3月に2.5倍になった` | Past | Never |
| `現在は640ミリ秒である` | Non-past | By tomorrow |

What separates the two sentences is whether an absolute anchor is present.

Targeting tense would break a doctype that works. The context section of a decision record
is written in the past tense, and D-33 measured that this type holds across 190 published
ADRs. The ordering of steps in a procedure also takes the form of a tense, but it does not
depend on when the text is read.

### Making the exception checkable

An exception left to the writer's judgment cannot be checked, so it is converted into a
question of whether an anchor is present.

If the same sentence contains a calendar year, a version number, or a numbered document, the
rule does not fire. `lindera 6.0.0 で現在の素性の順序を確認した` states a past fact and never
goes stale.

The anchor is checked sentence by sentence; a year in a neighboring sentence does not anchor
this one. Only a form prefixed with `v` or a three-part number counts as a version, so that
a decimal number is not mistaken for one. Treating `2.5 times` as an anchor removes every
finding in its sentence, which is what happened while this change was being implemented.

### What a word list cannot detect

A period measured from the present contains a number, so no list of words can cover it.
`直近3ヶ月`, `過去2年`, and `先週` are matched as a pattern instead.

This gap was confirmed on a real document. When applied to a design document written by a
model, M6 detected `現在`, `現時点`, `今後`, and `既存の` but missed
`直近3ヶ月でリクエスト数が約2.5倍に増加した`.

The added words complete the set that Google's guidance on timeless documentation names with
`recently`, `soon`, and `new`. They fall under a clause that was already cited, not under a
new hypothesis.

### Measurement

| Corpus | Before | After |
|---|---|---|
| 60 Japanese task pages | 119 findings / 36 documents | 123 / 36 |
| 60 reference documents | 179 / 39 | 185 / 39 |
| 39 AI translations | 85 / 26 | 92 / 27 |

The increase is small, because reference documentation is already written without reference
to a particular moment. Across 200 documents, the anchor rule suppressed one finding:
`既存のHyper-V分離サポートは、v1.10の試験的な機能であり`, a sentence that states what was true
at `v1.10` and therefore does not go stale.

Every document in this repository passes both before and after the change.

### What is not targeted

The past tense itself, descriptions of something ongoing, and expressions of order are not
targeted. A procedure needs its order, and a decision record needs its history. Removing
them would only take information away from the reader.

## D-36 The report becomes a tenth shorter while conveying more

One of the constraints on this tool is that it must not increase the number of tokens it
outputs. That number was measured rule by rule and compared with what each rule provides.

### What each rule detects

The table below gives findings per document across 39 AI translations and the 39 human
translations of the same pages.

| Rule | AI | Human | Ratio | Chars per finding |
|---|---|---|---|---|
| `maint/time-dependent` | 2.36 | 2.26 | 1.05 | 59 |
| `register/prefer-katakana` | 1.36 | 0.64 | 2.12 | 84 |
| `register/rare-wago` | 1.33 | 0.41 | 3.25 | 60 |
| `maint/parallel-items` | 0.44 | 0.41 | 1.06 | 69 |
| `structure/long-paragraph` | 0.23 | 0.18 | 1.29 | 88 |
| `maint/item-count` | 0.18 | 0.10 | 1.75 | 93 |
| `maint/list-lead-in` | 0.00 | 0.46 | 0.00 | 92 |

The register rules separate human from AI text best. The M rules are based on a published
style guide and fire on human writing as well; D-09 accepted this by design, and this
measurement does not contradict it.

By number of findings, `maint/time-dependent` is the largest. With a ratio of 1.05, it says
nothing about who wrote the document. It is kept because it is based on a published clause
and because acting on its findings improves the document.

### An excerpt that did not show the match

The text shown next to a position was the first 60 characters of the line. When the match
was further along the line, the excerpt did not contain it. When a line contained several
matches, the same 60 characters appeared once for each match: three identical excerpts for
three findings, with nothing to indicate which word to change.

The match is now shown first, followed by a narrow window of the surrounding text. The width
differs by language, because fourteen Japanese characters contain roughly a clause while
fourteen English characters contain about two words. English uses twenty characters, which
is the width at which the total size of the report equals what the fixed width of sixty
produced.

### A number in a title is an identifier

M3 detects a heading numbered by hand, because sections must be renumbered when one is added
or removed. That does not apply to a title: an ADR title is written as `# 3. Title`, and the
number identifies the record. 155 of 190 published ADRs were firing on it. Excluding the
leading level-one heading from the rule reduced that number to 11.

### The tool contradicted its own advice

`register/prefer-katakana` recommends the loanword, while the section synonyms listed only
Sino-Japanese forms. A decision record with the heading `## コンテキスト` was told that its
context section was missing, so one rule rejected what another rule had recommended. The
synonyms now include the loanword forms.

### The result

| Corpus | Before | After |
|---|---|---|
| 60 reference documents | 71,129 | 61,447 |
| 60 ADRs | 38,704 | 30,972 |
| 60 RFCs | 58,745 | 58,597 |
| 39 AI translations | 46,979 | 42,272 |

The total went from 215,557 characters to 193,288, a reduction of a tenth. The savings come
from numbered titles that no longer fire and from the narrower window. At the same time,
every position now shows the text it matched, so the report conveys more while printing
less.

## D-37 Word choice and orthography are decided separately, and the glossary is published

Choosing a word by ear tends toward a forced native equivalent. D-27 records an instance of
this: `検証` became `確かめ` in an attempt to lower the kanji ratio.
`register/rare-wago` detects such a word after it has been written, but nothing guided the
choice at the time of writing.

Reading the part of [yomiyasu](https://github.com/nanaism/yomiyasu)
(MIT License, Copyright (c) 2026 nanaism) that covers formal writing revealed a pair of
questions for making that choice. Nothing is quoted from it; only the structure of the
judgment was adopted, and the wording here is original.

### Two questions

Ask the following questions in order.

1. Would someone writing in this field choose this word for a specification or a procedure?
2. Does the word translate back into English one word at a time?

A yes to the second question and a no to the first indicate a phrase copied word by word
from English.

The key point of the first question is that repetition proves nothing. The vocabulary of one
AI-written document carries over into the next, so finding the word elsewhere in this
repository is no evidence that it is established. The documents in this repository were
written by a model, so this caution applies to them as well.

### Why word choice and orthography are kept separate

Each rests on a different authority, so the glossary keeps them apart.

| Kind | Settles | Authority |
|---|---|---|
| `term` | Which word to use | Corpus frequency and the PLaMo Translate rendering |
| `orthography` | How the word is written | The JTCA guideline, checked word by word against the corpus |

Measurement made the separation necessary. When asked to translate the same word in two
sentences, PLaMo Translate produced both `コンピュータ` and `コンピューター`. It writes
`サーバー`, `ユーザー`, and `クラスター` with the long-vowel mark, and `ブラウザ`, `パラメータ`, and
`フォルダ` without it.

A translation model can decide which word to use, but it cannot decide how the word is
written. That requires a published rule.

### Where the rule and the measurement disagree

The third edition of the JTCA guideline on loanword spelling writes the English endings
-er, -or, -ar, and -y with the long-vowel mark.

Measured over 4.38 million characters, the guideline holds for -er, -or, and -ar. For the
-y ending the result is the opposite: `プロパティ` occurs 870 times and `プロパティー` 3 times.
`コンテナ` ends in -er, yet this field has settled on the form without the mark, by 1783 to 95.

Each word is therefore decided by measurement instead of being derived from the guideline,
and a word whose form disagrees with the guideline is marked `measured-against-jtca` so that
the disagreement remains on record.

### The glossary is published in a portable format

`crates/suikou-core/data/glossary_ja.toml` contains the source.
`glossary/suikou-ja.prh.yml` and `glossary/suikou-ja.csv` are generated from it.

prh was chosen because it is the widely used tool for handling variant spellings in Japanese
and because its dictionary format works in textlint without changes. The generated
dictionary was loaded into textlint, which reported the variants and corrected them with
`textlint --fix`.

### Single-character calques are not covered by this rule

`glossary/variant` covers orthography and calques of more than one character.

Searching for a single character by substring matching also finds the `題` inside `問題`.
Counted in that way, the reference corpus contains 613 occurrences of `題`, nearly all of
which are part of `問題`, `課題`, and `話題` rather than the calque itself. Handling this
correctly requires morphological analysis, and `prefer-katakana` already analyzes words in
that way.

### This repository was inconsistent

Running the rule found `サーバ` in four of this repository's own documents, which now use
`サーバー` as the glossary specifies.

The rule fires on 7 of 60 reference documents and on 10 of 60 Kubernetes task pages. These
are not false positives: the reference corpus is itself inconsistent, with 591 `サーバー`
against 130 `サーバ`, and the purpose of the rule is to point to the less common form.

## D-38 Orthography is decided by counting each word, with the long vowel where the count is inconclusive

### Context

The policy for settling variant spellings changed three times.

The first policy omitted the long vowel everywhere, as in `サーバ` and `コンピュータ`. The
second added it everywhere, on the grounds that the third edition of the JTCA katakana
guideline (2015) specifies the long-vowel mark for `-er`, `-or`, `-ar` and `-y`.

Neither policy matched what the reference corpus actually does.

### Decision

Each word is counted in the reference corpus, and the more frequent form is used. Only when
the count is inconclusive — the more frequent form occurs less than twice as often as the
other, or the word occurs fewer than 50 times in total — is the long-vowel form used.

### Rationale

The three policies were compared over 12 words and 8,540 occurrences.

| Policy | Disagrees with usage | Words wrong |
|---|---|---|
| Long vowel throughout | 45.9% | 5 |
| No long vowel throughout | 54.1% | 7 |
| Counted per word | 9.5% | 0 |

Each uniform policy is wrong about half the time. `サーバー` is the more frequent form in the
reference corpus, and so is `コンピュータ`, so any single uniform policy gets one of them
wrong.

The long vowel is the default because the JTCA guideline specifies it, and there is no
reason to go against a published rule when the count does not decide.

### Consequences

Orthography is now kept separate from word choice, because each rests on a different
authority: word choice depends on how narrowly the meaning is defined, and orthography
depends on usage and a published rule. `[[orthography]]` and `[[term]]` in
`glossary_ja.toml` correspond to the two.

A translation model can decide word choice but not orthography. PLaMo Translate produced
both `コンピュータ` and `コンピューター` for the same word in different sentences. On word
choice, it agreed with human translators on 90.6% of 200 aligned paragraphs, and all three
disagreements were a different spelling of the same word, never a wrong word.

Words that do not appear in the corpus cannot be counted. The default exists so that such a
word does not leave the question unresolved.

## D-39 The README type requires only the lead paragraph, and the form of the opening becomes a rule

### Context

Documents generated by a model carry the writer's own notes into the README: the state of
the work, what turned out to be true after trying something, and a statement of what the
document is for. The reader has no use for any of this.

Diátaxis states that most documentation problems arise when a single piece of content tries
to meet more than one need at the same time. A note to oneself and a README meet different
needs.

Whether the `overview` type fits a README had never been measured. TASKS.md listed the
question as open.

### Decision

A new `readme` type is added, and it requires only the lead paragraph.

`usage` is removed from the required sections of `overview`, and its synonyms are extended.

A new rule, `structure/lead-opener`, reports opening forms that a document should avoid.

### Rationale

240 READMEs were collected from well-known open source projects. They were selected by star
count, divided across 8 programming languages, and limited to 3 per organization, so that no
single ecosystem or organization would be mistaken for the general form. 199 organizations are
represented.

Applying `overview` reported a missing required section in 205 of them (85%). This is the
same situation as in D-31, where the steps section was removed from `howto`, and in D-32,
where the scope section was removed from `design`.

| Section | Identified by heading |
|---|---|
| Install | 62% |
| Contributing | 53% |
| License | 49% |
| Documentation | 41% |
| Usage | 22% |

Even the most common section is identified by its heading in only 62% of the READMEs.
Requiring it would reject two READMEs out of five. A README does not express its structure
through its headings.

It expresses its structure through the lead paragraph.

| Measured | Value |
|---|---|
| Has a lead paragraph | 78% (187/240) |
| Of those, within 300 characters | 83% (156/187) |
| Median length of the lead | 121 characters |
| Median position of the first heading | 13% into the document |

With only the lead paragraph required, 11 of the 240 fire (5%). Three of those are Chinese
READMEs, which this tool does not cover. One is RxJava, which places the date of the next
release in a heading before saying what RxJava is.

Headings in the form of a question were added to the synonyms; `## What's Rails?` is the case
that prompted the addition. Such a synonym can also match `What's new`, but across the 240
READMEs that happens once, while the question form is matched 21 times.

The opening rule includes only the forms that occurred zero times across 831 lead
paragraphs.

| Form | README | Rust RFC | ADR | Reference (en) |
|---|---|---|---|---|
| Opens on generalities | 0/199 | 0/20 | 0/10 | 0/602 |
| Opens on how it came about | 0/199 | 0/20 | 0/10 | 0/602 |
| Declares what the document is for | 0/199 | 0/20 | 0/10 | 0/602 |
| Refers to itself | 1/199 | 0/20 | 0/10 | **37/602** |

In Japanese, the first three forms were likewise absent from 653 lead paragraphs in the
reference corpus.

Self-reference (`This document describes`, `本文書は`) is not included in the rule. It occurs
in 6% of the English reference corpus, where it works well in API documentation. It is the
most recognizably machine-like of the four openings, but the measurement shows that it is in
normal use.

Applied to 2,080 real documents, the rule fires zero times. The documents are 240 READMEs,
200 Rust RFCs, 190 ADRs, and 652 English and 798 Japanese reference documents.

### Consequences

Notes to oneself in a README are now detected by where the document begins, not by
vocabulary. Words such as `currently` and `TODO` are also used by well-known open source
projects: 45 of the 240 contain `currently`. There was no basis for forbidding them.

That measurement, however, counted two different things as one. In well-known open source
projects, `currently` and `Roadmap` address the reader: they describe what is planned, help
the reader decide whether to adopt the project, and ask for help. They are not the writer's
notes. It is no surprise that vocabulary could not separate the two — what separates them is
not the word but what the sentence tells the reader. D-40 addresses this.

What belongs in the lead paragraph, in contrast, is governed by a clear norm. Use the subject
as the grammatical subject and state what it is. Do not open with background, intent, or
history.

The rule does not detect the notes themselves. Notes placed in the middle of a document are
beyond its reach. The lead paragraph is where reading begins, and requiring it to begin with
the subject moves such notes further back in the document. Removing them from there is the
role of the norm that asks each section to answer the question its heading poses.

## D-40 The test is what a sentence tells the reader, not which words it uses

### Context

D-39 concluded that there was no basis for detecting the writer's notes by vocabulary: 45 of
240 well-known READMEs contain `currently`, and 48 have a Status or Roadmap heading.

That measurement counted two different things as one.

In well-known open source projects, `currently` and `Roadmap` address the reader. They
describe what is planned, help the reader decide whether to adopt the project, and ask for
help. They give the reader something to do next.

What an agent writes points in the opposite direction: it is context for the agent's own
later use, and it gives the reader nothing.

The words are the same while their purpose differs, so it is natural that vocabulary could
not separate them.

### Decision

The test is what the sentence tells the reader.

A new rule, `structure/work-note`, reports sentences about the work itself: what was tried,
what was discovered, the state of the work, and a former state.

Whether the rule applies depends on the doctype. In a design document or a decision record,
what was tried and why it failed is the message itself. `work_notes_ok` in `structure.toml`
expresses this.

### Rationale

Across 240 well-known READMEs, 307 sentences containing forward-looking words were sorted by
where they appear.

| Placement | Count | Share |
|---|---|---|
| Under a heading that announces it | 125 | 41% |
| Mixed into explanatory prose | 182 | 59% |

Of the 182 sentences in prose, **none** was about the work. 17 gave the reader something to do,
and the rest described a feature or a plan.

What separates the sentences is their subject, not their vocabulary.

| Form | README | Reference (en) | Reference (ja) | Rust RFC | ADR |
|---|---|---|---|---|---|
| What was tried | 0.4% | 0.2% | 0.1% | 4.5% | 3.2% |
| What was discovered | 0% | 0.3% | 0.1% | 7.5% | 1.6% |
| The state of the work | 0% | 0% | 0.1% | 0% | 0.5% |
| A former state | 0% | 0.3% | 0.3% | 5.0% | 0.5% |

In documents addressed to readers, these forms range from 0.1% to 0.4%. In Rust RFCs and
ADRs, they range from 1.6% to 7.5%, because there the same form is the message. That is the
basis for switching the rule by doctype.

`we refactored` and its Japanese equivalent were removed from the candidates.

In English, 8 of 11 occurrences pointed the reader somewhere: `moved to their own
repositories`, `renamed Redis Open Source with the v8.0 release` — telling the reader where
to look or what to call something. The remaining three were an FAQ heading and an anecdote
about a name.

In Japanese, the form cannot be distinguished from an ordinary verb: `フォーカスを移動した`
(the focus moved) matches, and all 43 matches in the reference corpus were of that kind.

| Corpus | Doctype applied | Fires |
|---|---|---|
| 240 READMEs | `readme` | 0 |
| 200 Rust RFCs | `design` | 0 |
| 190 ADRs | `decision` | 0 |
| 652 reference (en) | `reference` | 3 (0.5%) |
| 798 reference (ja) | `reference` | 5 (0.6%) |

### Consequences

The same sentence is now content in one doctype and clutter in another:
`We tried another approach and it did not work` is reported in a README and passes in a
decision record.

This rule differs in kind from the vocabulary rules. `register/rare-wago` is decided by
frequency and never considers context. `structure/work-note` reaches different verdicts on
the same words depending on the type of document in which they appear.

A document that declares no type is not checked, because it cannot be known whether it
addresses a reader. Listing the document under `[paths]` in `.suikou/structure.toml`
enables the rule.

Plans and calls for help under their own heading are not reported. 41% of well-known open
source projects do exactly this, which lets the reader skip them. Nothing here asks a
README to drop its plans; it asks only that a heading announce them.

## D-41 How strictly a norm applies is decided for each doctype

### Context

The same set of thresholds was applied both to entry-point documents and to reference
documents.

With the default thresholds, `en.mattr100` fires on 70% of 240 READMEs from well-known open
source projects, compared with 15% of 652 reference documents, and
`en.simile_marker_per_1k` fires on 74% of the READMEs.

A metric that fires on 70% of a genre indicates a threshold that does not fit, not a
property of the documents. The user made the same point: an entry-point README and the
documents under `docs/` are not expected to be equally formal.

### Decision

The profile supports adjustments for each doctype: replacing a threshold, dropping a metric,
and silencing a local rule.

An adjustment requires a `reason`, and a test checks that every built-in adjustment has one.
An adjustment records an agreement with the user; it is not something the tool decides on
its own.

Settings are applied in this order: the built-in profile, the built-in doctype adjustment,
the project's own settings, and then the project's own doctype adjustment.

### Rationale

240 READMEs in English (199 organizations, 8 programming languages) and 150 in Japanese (144
organizations) were measured.

| Metric | README | Reference | Ratio |
|---|---|---|---|
| `en.mattr100` | 70% | 15% | 4.7 |
| `en.prose_ratio` | 32% | 11% | 2.9 |
| `en.simile_marker_per_1k` | 74% | 45% | 1.6 |

The share of running prose varies by a factor of three across kinds of document.

| Corpus | Measured `prose_ratio` |
|---|---|
| 240 READMEs | 0.170 |
| 652 reference (en) | 0.427 |
| 200 Rust RFCs | 0.544 |

A README is 17% prose, and the rest consists of lists and code; a reference document is 43%
prose, and a Rust RFC 54%. With the default `en.prose_ratio` threshold of 0.65, every README
falls below the threshold.

After the adjustment, the firing rates drop, while the rates on the reference corpus remain
unchanged.

| Metric | README before | README after | Reference |
|---|---|---|---|
| `en.mattr100` | 70% | 6% | 15% |
| `en.prose_ratio` | 32% | 7% | 11% |
| `en.simile_marker_per_1k` | 74% | 0% | 45% |

Across 150 Japanese READMEs, the adjusted firing rates fall between 0% and 2%.

The difference in formality does not always point the same way.

| Metric | README (ja) | Reference (ja) |
|---|---|---|
| `ja.kango_ratio` | 0.620 | 0.521 |
| `ja.mattr100` | 0.752 | 0.674 |
| `ja.renyo_te_ratio` | 10.16 | 6.00 |

Japanese READMEs contain more Sino-Japanese vocabulary than the reference corpus, because
naming features and components produces many compounds. Relaxing every threshold for an
entry point would therefore be wrong; the direction of the difference depends on the metric.

### Consequences

The same metric is now judged against different thresholds depending on the doctype. To keep
it possible to trace which document was judged by which metric, a doctype adjustment cannot
add a metric; only the project-wide settings can.

No local rule is silenced by default for any doctype. The rules that fire most often on
READMEs, `maint/list-lead-in` with 340 occurrences and `maint/parallel-items` with 271, are
based on published style guidance and remain valid for a README. The mechanism is available
to projects but is not used in the built-in settings.

A document that declares no type receives no adjustment. Listing it under `[paths]` in
`.suikou/structure.toml`, or writing `doctype:` in its front matter, enables the adjustment.

## D-42 Documents that collect records get their own type, and verification includes warnings

### Context

After `structure/work-note` was introduced, it fired 11 times on this repository: 7 times in
`TASKS.md`, 3 times in `decisions.md`, and 1 time in `metrics.md`.

Ten of those findings were errors in the rule. CLAUDE.md designates `TASKS.md` and
`decisions.md` as the places where accounts of the work belong, so the rule was telling a
decision record to move its account into the decision record.

Both documents had declared the `reference` type, which had been chosen only because it
requires no sections. The type system was treating the purpose of a document and its
required sections as a single dimension.

A flaw in the verification process was found at the same time. `suikou check --quiet` exits
with 1 only when there is an error; a warning exits with 0. Checking every document by that
exit code alone had missed 47 warnings, while the requirement set in CLAUDE.md is that there
be no errors and no warnings.

### Decision

A `log` type is added for documents that collect records: a decision log, a task list, or a
change history. It has `work_notes_ok` and requires no sections.

`TASKS.md` and `decisions.md` are moved to this type.

Verification counts findings instead of reading the exit code.

### Rationale

The `log` type adds no detection; it only relaxes `reference`. It has not been measured
against an outside corpus, but relaxing a type carries none of the risk that adding a rule
does.

After the move, work-note fires zero times across the three documents. The finding in
`metrics.md` remained and turned out to be correct on inspection: a sentence explaining that
substitution by the lead paragraph had once been decided by position appeared in a document
of definitions. It was rewritten as a definition, and the account was replaced with a
reference to D-31.

The remaining 47 warnings concerned this repository's own prose, which the sections written
in that session had pushed over the thresholds. Each replacement word was chosen by counting
occurrences in the reference corpus.

| Word used | Reference corpus | Replacement | Reference corpus |
|---|---|---|---|
| `読み手` | 0 | `読者` | 13 |
| `向き` | 54 | `方向` | 155 |
| `測る` | 0 | `測定` | 63 |
| `要る` | 0 | `必要` | 3,503 |
| `入り口` | 1 | `起点` | 20 |

That `読み手` does not occur anywhere in the reference corpus was not apparent until it was
counted. Nothing about the word seemed wrong by ear.

### Consequences

The number of findings across all documents went from 58 to 0.

The verification procedure changed. The exit code reflects only errors, so it is not used.

```sh
for f in <all Markdown>; do suikou check "$f" --format text; done | grep -c ':[0-9]*:[0-9]*:'
```

A type needs to record the purpose of a document separately from whether the document
requires sections. `reference` and `log` both require no sections but serve different
purposes. `work_notes_ok` is the first field that expresses that difference; if the same
need arises for another rule, another field will be added.

## D-43 Density metrics get a minimum word count, and metrics that cannot be calibrated are dropped per doctype

### Context

After D-41 adjusted the thresholds for READMEs, the other types were measured. When the
`decision` type was applied to 190 ADRs, `en.mattr100` fired on 45%, `en.prose_ratio` on
45%, and `en.simile_marker_per_1k` on 75% — the same situation READMEs had been in.

Examining the simile metric showed that the problem was not the doctype. Its firing rate
changes steadily with the length of the document.

| Words | Fires |
|---|---|
| 0 to 400 | 86% |
| 400 to 800 | 47% |
| 800 to 1,250 | 24% |
| 1,250 to 2,000 | 16% |
| 3,000 to 5,000 | 5% |

These rates were measured over 1,042 documents from the reference corpus, Rust RFCs, and
ADRs.

### Decision

`en.simile_marker_per_1k` gets a minimum word count: below 1,250 words, the value is not
compared with the threshold.

`en.prose_ratio` is dropped for `decision`. `en.mattr100` is recalibrated for `design` and
`decision`.

### Rationale

The minimum follows from both theory and measurement. The threshold is 0.8 per 1,000 words,
so a single marker reaches it at 1000 ÷ 0.8 = 1,250 words. Below that length, a document
either has no marker or exceeds the threshold with one, so the metric cannot express any
difference. The measured firing rates also rise sharply below the same point.

The minimum is applied at the comparison, not in the calculation. The density can be
calculated for a document of any length, and leaving the calculation unchanged allows the
golden test to keep checking it against the reference implementation.

`MIN_TE_FOR_RATIO` in `ja.renyo_te_ratio` guards a ratio that cannot be defined when its
divisor is small; the new minimum guards a value that can be defined but cannot express a
difference. The reasons differ, so the checks are placed differently.

With the minimum in place, the rate improved for every type, and no type needed the metric
to be dropped.

| Doctype | Before | After |
|---|---|---|
| `readme` | 74% | 14% |
| `design` | 16% | 6% |
| `decision` | 75% | 4% |
| `reference` | 45% | 1% |

The 1% on the reference corpus agrees with the human range of 0.84 to 2.54 recorded in the
metric's own documentation. The default threshold of 0.8 is based on separating human from
machine writing, which is a different method from a Tukey fence — so when `suikou baseline`
reports that it cannot calibrate this metric on any corpus, that reflects the difference in
method, not a defect in the metric.

The remaining 14% on READMEs consists of correct findings. The 34 READMEs that fire have a
median of 2,370 words of prose, compared with 1,241 for the 57 that do not fire, so the cause
is not a shortage of prose: long READMEs with a lot of prose simply do not mark their
comparisons. The per-doctype exclusion was therefore removed from `readme` as well.

`en.prose_ratio` is dropped for `decision` because `suikou baseline corpus/cache/adrs`
reports a calibrated bound of -0.18, which lies outside the metric's range: the distribution
is too wide to set a boundary. Instead of choosing an arbitrary number, the metric is not
applied. Its default basis, a human range of 0.66 to 0.97, was measured on documentation
consisting mostly of prose and does not apply to a short record that follows a template.

`en.mattr100` could be calibrated: 0.76 for `design` and 0.88 for `decision`. Both a proposal
and a decision record change their vocabulary with their subject and repeat few words.

| Doctype | `mattr100` | `prose_ratio` | `simile` |
|---|---|---|---|
| `readme` | 6% | 7% | 14% |
| `design` | 0% | 4% | 6% |
| `decision` | 0% | 0% | 4% |
| `reference` | 15% | 11% | 1% |

### Consequences

When a metric fires too often, the cause is one of three kinds.

| Cause | What to do |
|---|---|
| No resolution at this document length | Set a minimum for the metric, regardless of doctype |
| The distribution differs by doctype | Change the threshold for each doctype |
| The distribution is too wide to set a boundary | Do not apply the metric to that doctype |

The doctype was suspected first, but the simile metric turned out to depend on length.
Applying a per-doctype exclusion first leaves the same defect in every other type: when D-41
dropped the simile metric for `readme`, the 75% on `decision` went unnoticed. The cause must
be identified before the remedy is chosen.

No Japanese corpus of ADRs or proposals exists, so the Japanese side has not been adjusted:
`ja.mattr100` and `ja.prose_ratio` remain at their defaults for these types.

## D-44 Documents collected from GitHub after the spread of LLMs do not make a human reference corpus

### Context

The Japanese thresholds for `design` and `decision` were still at their defaults, because no
Japanese corpus of ADRs and design documents was available.

Such a corpus was collected through GitHub code search: 123 files from 95 organizations,
filtered to those containing at least 300 hiragana and katakana characters. With the default
thresholds, the firing rates were extreme.

| Metric | 67 Japanese ADRs | 56 Japanese design docs |
|---|---|---|
| `ja.mattr100` | 86% | 82% |
| `ja.prose_ratio` | 88% | 87% |
| `ja.list_ratio` | 82% | 82% |
| `ja.renyo_per_1k` | 43% | 39% |

Calibration would lower those rates. Before calibrating, however, the corpus was checked to
see whether it consisted of human writing.

### Decision

No thresholds are calibrated from this corpus. The Japanese side of `design` and `decision`
remains at its defaults.

The collected corpus is kept. Filtering it by the commit date of each file may make it
usable.

Any future collection of a human reference corpus must first determine how to establish
that the writing predates the spread of LLMs.

### Rationale

The contamination was measured with this tool's own rules. `register/rare-wago` reports
native Japanese words that are repeated where professional technical writing does not use
them — a rule that was introduced as a marker of machine writing.

| Group | `register/rare-wago` | `maint/parallel-items` |
|---|---|---|
| Reference corpus, concept pages, human translation (67) | 4% | 35% |
| Kubernetes task pages, human translation (40) | 20% | 52% |
| Procedures collected from GitHub (160) | 35% | 61% |
| ADRs collected from GitHub (67) | 31% | 71% |

The groups collected from GitHub show the machine markers more strongly than human
translation does: 31% to 35% on `register/rare-wago`, eight times the 4% of the reference
corpus.

Two effects are mixed here. Genre accounts for part of the difference, since concept pages
fire at 4% and task pages at 20%. Comparing task pages with task pages still leaves 20%
against 35%, so holding genre constant does not remove the gap.

Of 25 sampled ADRs, 8 came from repositories created before 2023. The creation date of a
repository is not the date on which its ADRs were written, so that date alone cannot filter
the corpus.

Calibrating from a contaminated group would move the thresholds toward machine style, which
would reverse the purpose of the thresholds.

### Consequences

Whoever passes a corpus to `suikou baseline` must establish what that corpus is. `baseline`
treats every document it receives as human writing and has no way to tell otherwise.

The tool already provides the means to check: compare the firing rates of
`register/rare-wago` and `maint/parallel-items` with those on the reference corpus. A group
that fires eight times as often is not a human reference corpus.

The origin of each corpus already in hand is recorded below.

| Corpus | Where it came from | Human writing? |
|---|---|---|
| `pro_ja`, 798 | Japanese translations of k8s, MDN, Vue, React, the Rust book | Human translation |
| `pro_en`, 652 | The English originals of the same | Human |
| `rfcs`, 200 | Rust RFCs | Human; the proposals span years before LLMs |
| `adrs`, 190 | Published ADRs in English | Unverified |
| `readmes`, 240 | READMEs chosen by star count | Unverified |
| `adrs_ja`, 123 | GitHub code search | **Contaminated** |
| `howto_ja`, 200 | GitHub code search | **Partly contaminated** |

The same measurement was applied to `adrs` and `readmes` on the English side. Because
`register/rare-wago` works only for Japanese, the maint rules were used in its place.

| Group | `maint/parallel-items` | `maint/trailing-etc` |
|---|---|---|
| `pro_en`, 652 (reference) | 51% | 4% |
| `rfcs`, 200 | 58% | 17% |
| `adrs`, 190 | 28% | 5% |
| `readmes`, 240 | 43% | 24% |

Nothing comes close to the eightfold gap seen on the Japanese side, and `adrs` fires less
often than the reference, at 28% against 51%. The calibrations in D-41 and D-43 rest on
corpora that pass this check.

`readmes` fires at 24% against 4% on `maint/trailing-etc`, six times as often. Ending a list of
features with `etc.` is a README convention that long predates LLMs, and selecting by star
count includes many old documents, so this difference is interpreted as a matter of genre.

Only the maint rules are available for English, because D-28 decided that no English
counterpart to `register/rare-wago` would be implemented. The means of measuring
contamination is therefore weaker for English than for Japanese.

## D-45 M2 applies only when the count refers forward

### Context

M2 reports a count of items stated in running prose, because adding or removing an item
makes the prose wrong.

The implementation ran on all prose and never checked what the count referred to. TASKS.md
recorded the result — that the rule also fires on ordinary explanation that states a count
with no list after it — and pointed to `tests/golden/input/ja_prose.md` as an example:

```
無効化の方法は大きく分けて二つある。時間で切る方法と、変更を検知して切る方法である。
```

Here the count and the items it counts are completed within the same sentence. Adding a
third method would mean rewriting that sentence, so the count and the items cannot fall out
of step.

### Decision

M2 applies only when the count refers forward, which is indicated in one of three ways.

The sentence contains 以下, 次の, or 下記, which announces what follows.

A list, a heading, or a table follows — a structure that can be edited separately from the
sentence.

Prose containing an enumerating word (第一に, 1つ目, まず) follows.

### Rationale

The 168 firings across 798 reference documents were classified.

| | List | Heading | Prose | Nothing |
|---|---|---|---|---|
| With 以下/次の | 38 | 4 | 14 | 6 |
| Without | 17 | 2 | 81 | 6 |

Of the 81 firings without a marker and followed by prose, only 3 contain an enumerating
word. The other 78 complete the count within the same sentence.

The first analysis treated "no list follows" as the condition for a false positive and
produced a rate of 67%. That was wrong. Reading the cases shows that the prose becomes just
as wrong when a heading follows, as in this example:

```
次の3つの命名規則がよく使われます。

### DNSサブドメイン名
```

If a fourth convention is added, 次の3つ becomes wrong. Whether the counted items are list
items or headings makes no difference to how the text breaks. What matters is whether the
count refers to something elsewhere.

### Consequences

Firings across the reference corpus fell from 112 documents to 67, and the golden example is
no longer reported.

Deciding where a rule applies means deriving the condition from the reason for the rule. The
reason for M2 is that adding or removing an item makes the prose wrong, and that happens only
when the count and the items can be edited separately. Whether a list follows was only part
of that condition.

The English side uses the same condition, with `following`, `below`, and `these` as the
forward-referring words and `first`, `second`, and `one is` as the enumerating words. The
English reference corpus was not classified in this way, so those word lists were taken
over from the Japanese side rather than measured.

## D-46 How much to write belongs to the doctype, but not to the rules

### Context

A doctype specified which sections to use and the question each section answers. It did not
specify how much to write, which left that to the writer and made the sections uneven.

### Decision

Each section can have a `scale`, which `plot` and `brief` print.

`check` never reports on it. It is not a rule.

The scale is expressed as the relative size of the sections and the composition of each,
not as an absolute number of characters.

### Rationale

221 READMEs from well-known open source projects were measured; the 19 Chinese READMEs among
the 240 collected were excluded because they are outside the languages this tool handles.

| Section | Count | Median characters | First to third quartile | Bullet lines | Prose lines |
|---|---|---|---|---|---|
| License | 96 | 212 | 109 to 452 | 0 | 2 |
| Documentation | 91 | 349 | 171 to 854 | 0 | 1 |
| Contributing | 157 | 386 | 202 to 766 | 0 | 2 |
| Install | 203 | 455 | 186 to 954 | 0 | 3 |
| Usage | 72 | 480 | 211 to 920 | 0 | 2 |
| About | 32 | 668 | 403 to 969 | 0 | 3 |
| Configuration | 54 | 739 | 327 to 1568 | 0 | 7 |
| Features | 78 | 1032 | 495 to 1815 | 6 | 2 |

The median length differs by a factor of five from the license section to the features
section, while the relative order and composition are stable: features is the only section
that consists mainly of bullets, and configuration contains the most prose.

**It is not a rule because the quartiles spread to 2.5 or 3 times the median.** Human
documents vary too widely to separate from machine documents, and a value that does not
separate them cannot serve as a threshold. D-43 and `[rejected]` in `corpus/baselines.toml`
list the candidates that were dropped for the same reason.

A code block cannot be required either. Of 203 install sections, 100 contain one (49%). The
others point to a wiki, explain the build system in prose, or list the available downloads.

### The measurement was corrected three times

The first measurement contained three errors.

| Fault | Effect |
|---|---|
| Sections with an almost empty body were counted | Sections whose content sits in subsections entered the sample |
| Headings were classified by substring | `What's New` landed in the about section |
| Chinese READMEs were included | A language outside the tool's scope entered the sample |

After these errors were corrected, the number of about sections fell from 49 to 32, and the
median length of the features section changed from 731 to 1032.
`research/structure/measure_granularity.py` contains the procedure.

### Consequences

Only `readme` has a `scale`, because the other types have not been measured. A test ensures
that `readme` is the only type with one.

A test also ensures that each `scale` is provided in both languages, because a scale provided
in only one language would be missing from the output in the other.

No README variants were added. Whether libraries and applications differ was measured, and
they do not: only 10% have an API section and 14% show installation through a package
manager, while 56% fall into neither group. The composition of sections varies continuously
rather than forming clusters.

## D-47 A type for staged proposals is added, and the presence of a section no longer depends on heading level

### Context

`design`, taken from the Rust RFC template, was the only type for design documents.

The same word, however, covers different requirements. A Kubernetes Enhancement Proposal asks
about operating what is proposed: the test plan, the graduation criteria, the rollback,
version skew, and monitoring. Anything rolled out in stages cannot be released until those
are settled.

### Decision

An `enhancement` type is added, based on the KEP template.

Summary, Motivation, Goals, and Proposal are required.

Whether a section exists is judged regardless of heading level; order is judged only on
top-level headings.

### Rationale

265 published KEPs were measured, drawn from 19 SIGs with at most 25 per SIG. KEPs that still
contained unedited template text were excluded; 212 of 402 candidates were removed by that
filter.

| Section | Present |
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

The required set was chosen by its combined firing rate, because a single missing section
fires the rule, so the individual rates do not decide the question.

| Required set | Fires |
|---|---|
| Summary | 2% |
| + Motivation | 6% |
| + Goals + Proposal | 9% |
| + Non-Goals | 15% |
| + Graduation Criteria + Test Plan | 29% |

The set with a 9% firing rate was chosen, which places it in the same range as the other
types.

The operational sections remain optional but keep their questions and guidance, so `plot`
and `brief` continue to ask about them — the same treatment that D-31 and D-32 established.

### The presence of a section no longer depends on heading level

When the type was applied as first defined, it fired on 262 of 265 KEPs (98%).

The cause was heading level. A KEP places `Goals` under `## Motivation` as a `###` heading,
and section matching considered only the top level.

**The level of a heading is a choice of formatting, not of meaning.** Placing the goals
under the motivation is a decision about where they belong, not a sign that they are missing.

D-33 had declined to relax this limitation, on the grounds that "looking at lower levels
makes a subheading inside one section match another and corrupts the order judgement".

Separating presence from order removes that reason. Presence ignores the heading level, and
order considers only top-level headings, because ordering a nested heading against a
top-level heading has no meaning.

With the two judgments separated, the firing rate of `enhancement` fell from 98% to 8%.

### Consequences

The other types were affected as well, and all of their firing rates went down.

| Doctype | Corpus | Fires |
|---|---|---|
| `readme` | 240 READMEs | 4% |
| `design` | 200 Rust RFCs | 8% |
| `decision` | 190 ADRs | 7% |
| `enhancement` | 265 KEPs | 8% |
| `howto` | 41 Kubernetes task pages | 0% |

The 11 MADR documents that D-33 had set aside now pass: they place the consequences under
`Decision
Outcome`, which is a matter of placement, not of absence.

D-33 had also named splitting the type by template as the alternative to relaxing the
limitation. Adding `enhancement` is that alternative, so in the end both were done.

The selection tree gained a branch, in which `design` and `enhancement` are distinguished by
whether the proposal is rolled out in stages. A test ensures that every type can be reached
from the tree.

## D-48 README headings use the conventional names, and M2 counts only within a section

### Context

D-46 built a way to measure how much a README section contains, and that measurement was
then applied to this repository's own README.

Five of its seven headings almost never appear among 221 READMEs from well-known open source
projects.

| Heading here | Present in the 221 |
|---|---|
| `Installing` | 1% |
| `Using it` | 0% |
| `How it is built` | 0% |
| `Layout` | 0% |
| `State` | 0% |
| `Documentation` | 25% |
| `License` | 41% |

A reader cannot find a section under a name the reader does not know.

The `Status` section had a second problem. The same table appeared in three places —
`CLAUDE.md` and both READMEs — and the two copies in the READMEs had become outdated, with 8
rows instead of 13 and without `plot`, the glossary, or the release workflow.

### Decision

The headings use the conventional names: `Install`, `Usage`, `How it works`, and `Status`.

`Layout` moves to `development.md`, because it is material for contributors.

The `Status` table is removed from the READMEs, which now refer to `CLAUDE.md`.

### Rationale

The conventional names were chosen by how often they appear among the 221 READMEs: `Install`
41%, `Usage` 33%, and `Documentation` 25%.

`Layout` lists repository paths and appears in only 1 of the 221. It is not something a
reader considers when deciding whether to adopt a project.

`Status` was reduced to a sentence because the three copies were not being kept consistent.
Any copy eventually becomes outdated, so the information is kept in one place.

`Usage` is 1,052 characters long, compared with a measured third quartile of 920. The user
decided to keep it, and D-46 had already established that the scale is guidance, not a rule.

### Fixing a false positive in M2

During this work, the M2 condition added in D-45 fired incorrectly on this repository's own
README:

    部分ごとの状況は CLAUDE.md にある。
    ここに写しを置くと古くなる。実際に、取り除く前は5つの部分が遅れていた。

    ## ライセンス

Because a heading followed, the count was treated as referring forward. However,
`## ライセンス` is not what the count refers to; the section simply ended there.

**A count can refer only to what is inside its own section.** A following heading is now
counted only when it is deeper than the heading under which the count appears.

The reference corpus contains 1 case of each kind, so frequency decides nothing here. The
basis for the change is the range a count can refer to. A rule that fires whenever a count
appears in the last paragraph of a section is defective, however rarely that happens.

All three implementations include the fix — Rust, the textlint preset, and
`research/mcheck.py` — and the golden counts are unchanged.

### Consequences

Applying the doctype machinery to this repository's own documents is what revealed that the
headings had departed from convention. Building a way to measure does not fix the documents;
applying it is separate work.

The same table is not placed in several locations. Either `CLAUDE.md` or the README is the
source, and the other refers to it.

## D-49 The documentation site stays on Hugo, because Zensical cannot handle two languages

### Context

A proposal was made to move the documentation site to Zensical, a site generator written in
Rust and Python by the makers of Material for MkDocs.

This site builds English and Japanese together, with `defaultContentLanguage` set to
English and `[languages]` pointing to `content/en` and `content/ja`.

### Decision

The site stays on Hugo.

### Rationale

Zensical cannot handle two languages in a single build, and its documentation says so:

> Zensical only supports setting a canonical language for the entire project.

`docs_dir` takes a single path, and the language is a project-wide setting. A language
selector would consist of hand-written links to separate sites.

A plugin cannot fill the gap either. As of 0.1.1, the documentation states that Zensical does
not run arbitrary MkDocs plugins and does not expose the MkDocs plugin API.

`mkdocs-static-i18n` appears on none of the lists of supported, unsupported, or planned
plugins, and configuration for a plugin that is not listed is silently ignored. A Module API
is planned, but it will be available first in the paid Spark tier.

The roadmap includes `Language organization` and `Language publishing`, but neither appears
in the `Now available`, `In development`, or `Up next` table, and the roadmap states that it
commits to no dates and no order. The project's own update says that it will remain at 0.x
while the module system is being developed.

### What was established by trying it

Version 0.1.1 was installed and run to determine whether the move was possible. Two points
that the documentation does not mention came to light, and they are recorded here for anyone
who reconsiders the question.

| Question | Result |
|---|---|
| Publishing under a subpath | Works; output is relative, so no base-path setting is needed |
| The language-selector key | `[[project.extra.alternate]]`; the documentation does not name it |
| `site_dir` | Must sit inside the project; anything outside is rejected |
| Custom front matter | Preserved, so `doctype` is unaffected |
| Build | 0.26s, 592K of output against Hugo's 796K |

A setup of two projects — one configuration file each under `docs/en` and `docs/ja` — does
work. Both projects were built, and `<html lang>` is generated correctly for each language.

### Why the move was declined

Two projects would mean maintaining `nav` in two configuration files. D-48 had just
established that the same table is not placed in several locations, and this setup would
contradict it.

There are benefits — the appearance of Material, search, and the removal of 57 lines of
hand-written layout — but multilingual support is not scheduled, the project states that it
will remain at 0.x, and plugins cannot fill the gap until the Module API arrives. Weighing
these factors, staying on Hugo is the better choice.

The question will be reconsidered when i18n appears in the status table of the roadmap.

### Consequences

The policy in `docs/hugo.toml` of using no external theme remains in place, as do the four
hand-written layouts totaling 57 lines.

The site also continues to have no search. Zensical includes a `search.json`, but not at the
cost of assembling the two languages by hand.

## D-50 The rules separate human from AI writing in Japanese but not in English

### Context

Closing the gap identified in T21 required documents actually written by an AI, with
authorship established from the commit log: repositories in which 60% or more of recent
commits contain `Co-Authored-By: Claude`.

140 English and 150 Japanese documents were collected separately.

### Decision

No rule is added. The existing rules already separate human from AI writing in Japanese.

The gap identified in T21 is not a real gap in English, and in Japanese the existing rules
already cover it.

### Rationale

The four note forms (`structure/work-note`) do not appear in English documentation written
by an AI: they occur in 1% of the 140 documents, against 0% of human READMEs. The four
candidate forms do not separate the two groups either, and 注意書き occurs more often in the
human documents.

In Japanese, the existing rules do separate the groups. Three groups whose authorship is
established with different degrees of certainty were measured with the same rules.

| Group | Authorship | Count |
|---|---|---|
| AI-authored READMEs | Settled from the commit log | 37 |
| Japanese READMEs from GitHub | Unknown; D-44 found them contaminated | 150 |
| Reference corpus | Professional translation | 150 |

| Rule | AI-authored | GitHub | Reference |
|---|---|---|---|
| `register/rare-wago` | 70% | 34% | 14% |
| `glossary/variant` | 64% | 45% | 12% |
| `register/prefer-katakana` | 43% | 34% | 30% |
| `maint/numbered-heading` | 13% | 10% | 1% |

**All four rules rank the three groups in the same order as the certainty of their attribution.**

The group that D-44 found contaminated falls between the other two, and the three groups were
collected by separate procedures, so their agreement supports both findings.

### Holding the kind of document constant

Before the kind of document was held constant, the gaps appeared much larger.

| Rule | Uncontrolled | Controlled |
|---|---|---|
| `register/rare-wago`, against the reference corpus | 13.6x | 2.1x |
| `maint/numbered-heading`, English, against human READMEs | 6.3x | 1.7x |

On the English side, the AI corpus contained more `docs/` pages, while the human side
contained only READMEs. A difference in the kind of document very nearly got reported as a
difference in authorship. Comparisons are therefore made between documents of the same kind.

### Reading the corpus revealed its defects

Reading the documents before measuring them revealed two defects in the English corpus.

| Defect | Extent |
|---|---|
| The language test only looked for kana | 23 Korean and 3 Chinese documents counted as English |
| Agent instruction files | 42 of 200 (21%): orchestrator runbooks and `.memory/` records |

An instruction file is, by design, a set of notes addressed to an agent, so it is not the
kind of document in question. The Japanese collection applied both filters: all 150
documents are in Japanese, and instruction files are excluded.

### Consequences

The decision in D-28 not to implement English style rules is supported by a second line of
evidence: across 140 English documents written by an AI, no form separates the groups.

In Japanese, the existing rules work, and nothing needs to be added.

Two limitations remain. First, these are documents committed to public repositories, so a
person reviewed them at some point; notes that exist in a draft before review do not appear
here. Second, the share is measured over the most recent 100 commits instead of the whole
history, and six of the repositories kept have fewer than 20 commits, so their shares are
weak evidence.
