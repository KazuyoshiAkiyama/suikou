---
title: "Decisions"
weight: 30
---

This page exists so that a later reader never has to repeat the same investigation.
Each place where the call could have gone another way gets recorded here.

## D-01 Document-wide metrics stay out of textlint

textlint reports against the position of a node.
A metric computed across the whole document has no node to attach to.
Most of those metrics report something missing, namely scarce formal nouns, scarce
demonstratives, and scarce simile markers. Something missing has no position at all.
A scarce formal noun, for example, is spread across the whole document rather than
sitting on one line.

The first attempt implemented them as textlint rules reporting on the Document node.
That was wrong. Living inside textlint ties the analysis to JavaScript, which leaves no
tokenizer that returns the word origin, which makes the kango ratio impossible to compute.
Dropping a validated metric for the convenience of the implementation inverts the order of
priorities.

The front-end tool calls textlint and merges the output of textlint with its own
analysis. The front-end tool can serve MCP itself, and one merged report suits an agent
better anyway.

## D-02 Rust is the language, and Python is limited to calibration and search

Every metric that survived validation can be written in pure Rust, because
lindera-unidic returns every UniDic feature, including the word origin and the inflected
form.

Python was needed for participial clauses, nominalization, modifier stacking, and
dependency distance. Measurement rejected all four.
The deeper the parse a metric needs, the more sensitive it proved to the design of the
experiment, and the less the earlier effect size carried over.

The work therefore splits into two tiers. A settled metric is written in Rust, and search
and calibration happen in Python. A metric moves into Rust only after Python proves that
metric out.

Running from Claude Code hooks on every Write and every Edit makes launch cost dominant.
spaCy takes two to five seconds to load, so Python cannot sit in the production path.

## D-03 Lines rather than a Markdown parser

The reference implementation under `research/` works line by line, and matching the golden
values means working the same way.
Fewer dependencies also means fewer accidents caused by a version difference.

## D-04 The tokenizer hides behind a trait

The lindera API and the order of the UniDic features both change between versions.
Confining the version-dependent code to one struct shrinks the breakable surface to a few
dozen lines.

Keeping lindera out of the default features also means the rules and the metrics can be
tested without downloading a dictionary, which makes checking a change faster.

## D-05 The feature order is verified at startup

Pinning a version still leaves the problem of a silent break on upgrade.
When the feature order shifts, the word origin and the inflected form get read from the
wrong position. A value comes back, and the value that comes back is wrong.

The kango ratio and the renyo count both depend on those two fields, so running on broken
values makes the findings themselves wrong.
Pushing a known word through the tokenizer and failing on a mismatch is the better
outcome.

## D-06 Thresholds do not split by register

Whether the plain and polite registers move the metrics was tested against two models.
The direction did not agree across models, and no systematic effect appeared.
The difference is absorbed by the difference between models.

Register detection stays, but only to decide whether a register-dependent rule such as
the noun-stopped sentence applies. The threshold table does not need a second copy.

## D-07 Formal nouns and the kango ratio sit at info

Both metrics swing widely with the kind of document.
Formal nouns touch the threshold in procedures, and the kango ratio approaches the native
human value in explanations. The kango ratio is sensitive to the subject as well.

The Japanese human corpus leans toward expository books and lacks procedures.
Both metrics stay at info until baselines exist per kind of document, and both move up to
warning once they do.

## D-08 An effect size does not carry across tasks

For participial clauses in English, a report of 5.3 times the human rate suggested a
threshold of nine to ten. Measurement put the range on top of the human range, so the
metric was unusable. Nominalization behaved the same way.

The cause appears to be the design of the experiment.
The source handed over a human passage and asked for a continuation in the same register,
while this work asked for a document to be written.

Earlier work is a source of hypotheses. Its effect sizes do not transfer.

## D-09 The maintainability layer does not match the human average

The AWS S3 user guide is the only corpus that writes list lead-ins correctly.
Incomplete lead-ins dominate the Linux kernel.
Matching the human average would empty this layer of meaning.

This layer does not remove AI style. It makes a document better than most documents
written by humans.

## D-10 Separate what a prompt can prevent from what only a linter can catch

A condition with a formatting instruction was compared against one without.
Formatting disappeared completely, and vocabulary and sentence structure did not move.

The same experiment ran for the maintainability instruction.
The structural rules held completely under a short instruction, and time-dependent words
reached zero only under a detailed one.
Stating the item count, numbering headings, and hand-numbering items never occur even
without an instruction.

`brief` therefore carries only what an instruction can enforce.
A constraint that does not work wastes tokens and risks over-correction.

As a second effect, the maintainability instruction alone improved the document metrics.
Asking for a lead-in produced prose, and the prose ratio and the list ratio moved toward
the human range.

## D-11 The term list is built from the documents rather than shipped

Telling a live metaphor from a dead one is the hardest part of the metaphor layer.
Cache and handshake are established terms in the field, not metaphors.
A sentence that says a buffer behaves like a waiting room marks the comparison, and a
sentence that calls the buffer a waiting room does not.

A word that becomes famous for being called out drops in frequency.
The words called out early in 2024 fell right afterward. A shipped word list ages.

A list built from your own documents does not age.
Beyond the fact that terms differ between projects, that property is the structural
answer to an aging word list.

## D-12 The kango ratio test had the wrong expectation

`kango_ratio_counts_only_content_words` was failing.
For the input 設定を変更すること the implementation returned 0.5 and the test expected
more than 0.5.

`TASKS.md` says to fix the implementation when a test fails. This case is the exception.
The comment on the test counted three content words, namely 設定, 変更, and こと, but the
`FakeMorphology` spec also registers する as a verb, which makes four.
The metric definition puts every content word whose origin is 和, 漢, 外, or 混 into the
divisor, so the verb する counts too, and the 2/4 the implementation returns is what the
definition asks for.
Matching the implementation to the test would break the definition.

The intent of the test is to confirm that particles stay out of the divisor.
Changing the expectation to exactly 0.5 preserves that intent.
Putting the particle を into the divisor would give 0.4, so the two cases remain
distinguishable.

An implementation gets fixed when the implementation departs from the definition.
An expectation gets fixed when the expectation does.

## D-13 M2 and M6 count every occurrence

The reference implementation counts occurrences per line, while the Rust implementation
reported at most one per block.
In `tests/golden/input/en_maintainability.md`, one line holds two of the listed words at
once, and the reference returned 2 against the 1 from Rust.

Counting every occurrence won, and the reason comes from this project's own constraint.
Reporting one of two findings on a line means the document, edited to clear the reported
one, fails `check --quiet` again.
That collapses the design goal of never linting and fixing layer by layer.

The unused `column` field on `Position` shows that per-occurrence positions were the
original intent.
Giving each occurrence a column makes two findings on one line distinguishable.

M7 anchors to the end of a line, so at most one can fire per line, and the change does not
touch it.

## D-14 Word and sentence counts compare on relative error

The comparison policy allowed an absolute error of 0.02 on a ratio, 0.5 on a density, and
exact equality on the M rules. A word count fits none of those.

The word count of `en_prose.md` came out at 445 from the reference and 444 from Rust.
The cause is `trade-off`. spaCy splits it into three tokens and drops the punctuation,
giving two words, while splitting on whitespace gives one.
The file holds exactly one hyphenated word, which matches the difference exactly.

That is a difference of definition between tokenizers, not an error in the implementation.
Demanding exact equality would demand that two different definitions produce one number.
A relative error of one percent catches systematic drift and tolerates the rest.

## D-15 The scope of the formal noun metric differs between the definition and the reference

The metric definition measures vocabulary over prose plus the body of list items, and it
lists formal nouns among them. The Rust implementation follows that.

The reference implementation `research/jametrics.py` emits demonstratives both over prose
alone and over prose plus list items.
For formal nouns, it emits the prose-only figure and nothing else.

Measured on `tests/golden/input/ja_maintainability.md`, prose alone gives 14.98 and prose
plus list items gives 15.77.
The difference of 0.79 exceeds the 0.5 allowed for a density.

The implementation must not change here. The definition states its reason.
Some models put almost everything into bullet lists, and measuring prose alone throws away
more than eighty percent of the text for those models.

The mismatch belongs to the threshold instead.
The human range of 11.095 to 11.163 and the D5 threshold of 8.5 were both measured under
the prose-only definition.
Including list items raises the value, so D5 now fires less readily than it did at
calibration time.
Keeping D5 at info, decided in D-07, holds up for this reason as well.

The mismatch stays open until the calibration is redone.
Redoing it means adding the prose-plus-list figure to `research/jametrics.py` and
measuring the corpus again.

## D-16 Ship one binary with the dictionary embedded

Deciding how to ship meant measuring the embedded dictionary.

| Contents | Size |
|---|---|
| Binary with the dictionary | 191 MB |
| The same, compressed with xz | 26 MB |
| Binary without the dictionary | 1.1 MB |

`strip` does nothing, because the bulk is dictionary data rather than symbols.

Keeping the dictionary outside and fetching it on first run is possible.
lindera 6.0.0 offers `load_dictionary_from_path`, which reads without copying.
Embedding won anyway, and the reason is the intended use.
For a tool launched from hooks on every Write and every Edit, the number of failure paths
is the reliability.
An external dictionary adds three paths, namely absent, corrupt, and mismatched.
For instance, an upgrade that replaces the dictionary leaves the binary reading features
from the wrong position.
Self-diagnosis catches the mismatch, and catching it still leaves the user stuck.
Needing no network after download suits this use better.

The unit of distribution is one archive per operating system.
Bundling every system into one archive reaches 78 MB compressed and forces every user to
download the systems they do not use.

The targets are x86_64 Linux on musl, Apple Silicon macOS, and x86_64 Windows.
Linux uses musl so that no version of glibc matters.
A tool that fails on an older distribution or inside a container hurts when hooks call it.
Intel Mac and ARM Linux are out. Anyone who needs them builds from source.

## D-17 The dictionary feature stays off by default

`suikou-cli` gained `lindera-unidic`, and the feature stays off by default.
Turning it on by default would pull the dictionary into `cargo test --all` and break the
premise from D-04 that the core can be tested without one.

Two measures cover the gap instead.
The release build names the feature explicitly and runs the binary to confirm the
dictionary before packaging.
A build without the dictionary fails on the spot inside `morphology::load` when asked for
Japanese.

Returning an empty analysis silently is not acceptable.
An empty tokenization drives the kango ratio, the renyo count, and the formal noun count
all to zero.
Zero sits below the thresholds, so some findings fire and others do not, and one path
through that mixture ends in a false report of nothing to fix.

`suikou --version` tells the two builds apart.
A user who cannot identify the binary in hand cannot trace why Japanese went unanalyzed.

## D-18 Verify the artifact against the dictionary itself

Running the release workflow under `act` exposed this.
The packaged binary came to 1.2 MB while `--version` claimed the dictionary was embedded.
The dictionary is 191 MB, so the two cannot both be true.

The cause is dead code elimination.
With every subcommand still a `todo!()`, nothing reached `morphology::load`, and the
linker dropped the embedded dictionary along with it.
Adding a path that reaches it brought the binary to 191.5 MB and produced a real analysis.

The smoke test was at fault too.
The `--version` string is a compile-time branch on `#[cfg]`, so it reports whether the
feature was enabled and nothing about whether the dictionary exists.
A binary with no dictionary passes that check.

A `selftest` subcommand now loads the dictionary and asks it to analyze a sentence.
Verification runs against the behavior of the artifact rather than the settings of the
build, which is the same reasoning behind verifying the feature order at startup.

As a second effect, `morphology::load` gained a caller, so the suppression in
`crates/suikou-cli/src/morphology.rs` came out.
Users also gain a way to confirm that the binary in hand is what it claims to be.

## D-19 Finding line numbers match the file before preprocessing

Running `suikou check` on this project's own documents exposed this.
Reported line numbers pointed four lines above the real ones, and four is exactly the
length of the front matter that had been stripped.

Preprocessing removed the front matter and the fenced code blocks outright, so every line
below them moved up and the numbers stopped matching the file.

Each removal now leaves the same number of newlines behind.
Empty lines are not counted, so the count of non-empty lines and the list ratio do not
change.

A linter finding has to point at a line in the file the user has open.
A correct value at the wrong position is a finding nobody can use.

## D-20 The documentation is a Hugo site with English as the default

The documentation moved from files directly under `docs/` into `docs/content/{en,ja}/`,
and English is the default language.

No theme is pulled in from outside, and a minimal set of layouts lives in the repository.
That keeps the dependency count down, and it holds the project's policy on pinned
versions on the documentation side too.

Fixing one canonical copy of each page also means the site and the repository never carry
the same content twice.

## D-21 brief's detail level lives on a flag, not a profile field

The design document sketches an option where a profile carries the detail level for
brief. This implementation sets that option aside and controls detail through a
`--detail concise|balanced|detailed` flag alone.

Here is the reasoning. Profile is a struct that holds metric thresholds. Adding a
brief-specific field to it would split the struct across two jobs: values that
calibration writes back, and a value that only picks an output shape. Mixing the two
kinds of value in one struct blurs the range `suikou baseline` is supposed to touch, so
this implementation keeps Profile to the single job of holding thresholds.

The flag also grew a third level beyond the original sketch. TASKS.md first listed only
concise and detailed, for example, but the implementation split out a middle level,
balanced, once the structural rules turned out to fit one short sentence each while the
time-dependent words still needed a full list every time. Concise trims that list-plus-
sentences form further, and detailed adds the reasoning behind each rule.

The profile-based option stays on the list of open questions. Whether to move detail
onto a profile is a call this project can make once a real request for a per-project
default shows up, not before.

## D-22 The baseline threshold uses the Tukey outlier fence

Implementing `suikou baseline` called for a way to pick the threshold value.
`docs/content/en/design.md` places each threshold between the top of the human range and
the bottom of the AI range, for example the 0.60 line drawn for `ja.mattr100`. Baseline,
though, only ever sees a human corpus, so it never learns where the AI range sits.

The approach chosen derives a boundary from the human distribution alone: a point that
the distribution itself would call an outlier.

That boundary follows the Tukey outlier fence. The interquartile range is the gap between
the first and third quartiles, and the fence sits `FENCE_K` times that range beyond the
nearer quartile. The coefficient 1.5 comes from Tukey's definition of an "outlier" and is
a standard choice in statistics, easier to justify than an ad-hoc margin invented for this
project. A looser coefficient of 3.0 defines Tukey's "far out" points, but with the small
sample counts baseline typically works with, that wider fence would push the threshold too
far out to mean much, so this project does not use that coefficient.

The quartiles come from the hinge method: split the sorted sample at the median into two
halves and take the median of each half as Q1 and Q3. An odd-length sample excludes the
middle value from both halves. A percentile method with linear interpolation exists too,
but the hinge method is simple enough that its expected values can be checked by hand in a
test.

A metric with fewer than four samples goes uncalibrated. With three samples or fewer, one
half of the split collapses to a single value, so the interquartile range tracks that one
value rather than the shape of the distribution. Filling an uncalibrated metric with zero
or a placeholder would hide the missing evidence from whoever reads the profile, so that
metric is left out of the output, and the reason goes to standard error instead.

`guidance`, `direction`, and `severity` all carry over from the bundled oss profile
unchanged. Rewriting that text on every baseline run would leave the baseline output
stale the next time someone edits oss.toml, so calibration touches only `value`.

The corpus that `corpus/fetch.sh` would fetch is not available here; licensing keeps it
out of the repository. That leaves the reproduction of `corpus/baselines.toml`, and of the
bundled profiles' order of magnitude, unverified. The documents under
`tests/golden/input/` are too few to substitute either, for instance no single metric
there reaches the four-sample minimum. TASKS.md records this as open until someone runs
`suikou baseline` against a real human corpus and checks the result against
`corpus/baselines.toml`.
