---
title: "Decisions"
weight: 30
doctype: "reference"
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

Keeping the dictionary outside and fetching it on first run is possible. lindera 6.0.0
offers `load_dictionary_from_path`, which reads without copying. Embedding won anyway, and
the reason is the intended use. For a tool launched from hooks on every Write and every
Edit, the number of failure paths is the reliability.

An external dictionary adds three paths, namely absent, corrupt, and mismatched. For
instance, an upgrade that replaces the dictionary leaves the binary reading features from
the wrong position. Self-diagnosis catches the mismatch, and catching it still leaves the
user stuck. Needing no network after download suits this use better.

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

That first implementation carried a defect. A below-direction metric is a ratio or a
per-thousand count, and both stay at or above zero by construction. When the samples
cluster tightly, Q1 - 1.5*IQR can fall below zero. Calibrating against this project's own
Japanese documents (`docs/content/ja/*.md`, `README.ja.md`, `CLAUDE.md`, `TASKS.md`)
produced exactly that: a fence of -4.96 for `ja.demonstrative_per_1k` and -4.98 for
`ja.keishiki_meishi_per_1k`. Since a density or a ratio never goes negative, no real
document can satisfy the below check (`value < threshold`) against a negative threshold,
so the rule can never fire. The profile still parses as valid TOML, so running
`suikou check` against it reports nothing, and a check that only reads "nothing reported"
mistakes this defect for a successful calibration.

Clamping the fence to zero was not the fix. A density or a ratio is already bounded at
zero, so a threshold of exactly zero still asks for a value strictly less than zero, which
still never happens; clamping trades one unreachable threshold for another. Instead, a
below fence at or under zero is now treated as a calibration failure, the same way too few
samples is: that metric is dropped from the output and the reason goes to standard error.

The above direction gets no matching treatment. Every metric shares the same lower bound
(zero), but metrics do not share an upper bound: a ratio tops out at 1, while a
per-thousand count has no fixed ceiling. Bringing a per-metric ceiling into the
calibration logic would mix metric-specific knowledge into code that otherwise stays
generic, so the fix stays on the below side only.

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

A below-direction threshold that does survive calibration has been confirmed to work.
Calibrating against the documents above left `ja.prose_ratio` (fence 0.69) in the profile,
and running `suikou check --profile` with it against a document that is mostly bullet
lists fires the rule, while running it against a prose-only document does not, both
confirmed against real command output.

## D-23 Judgment calls in the textlint preset

Writing `packages/textlint-rule-preset-tech-maintainability` (T7) raised several
judgment calls that did not fit under one heading. Each judgment call is laid out below,
and each judgment call gets its own paragraph so that the reasoning for one judgment call
does not blur into the reasoning for the next judgment call.

**textlint does not print `maint/` as the rule prefix under a standard config.** The
brief for this task asked for rule IDs that match the Rust side, `maint/list-lead-in`
and so on. Reading textlint's own preset-loading code showed that the prefix textlint
prints comes from the config key written in `.textlintrc`, with a leading `preset-`
stripped back off. Loading this package the standard way, as
`"preset-tech-maintainability": true`, prints every finding under the prefix
`tech-maintainability/`. A config key has to start with `preset-` to be recognized as a
preset at all, and that same `preset-` is always the part stripped away before the rest
of the key becomes the prefix, so no config key reaches a `maint/` prefix through the
standard loader.

Each rule's own name (the part after the prefix, such as `list-lead-in`) does match the
Rust side, as asked. Only the prefix follows textlint's own convention instead, and
bending that would mean renaming the package itself to something like
`textlint-rule-preset-maint`, disagreeing with its own directory and confusing readers
more than the prefix mismatch does. The package name stays aligned with its directory,
and the README states plainly that a finding's printed prefix reads
`tech-maintainability/`. The golden-parity tests sidestep the prefix question
altogether: they call `@textlint/kernel` directly and register each rule under a
`maint/` prefix by hand, so the constraint above never touches that comparison.

**Match against the raw source, not the textlint AST.** textlint hands a rule the
Markdown broken into nodes such as paragraphs and headings. A paragraph that soft-wraps
across lines becomes a single Paragraph node inside textlint. `markdown.rs`, by
contrast, builds one block per line, and a soft-wrapped continuation stays a separate
line. The mismatch shows up on a paragraph such as lines 14 and 15 of
`en_maintainability.md`, where one sentence spans two lines. Matching against the joined
node text would join those two lines into one string, letting a regex match span words
the Rust side never joins.

Two paths were open: match against the text of textlint's Paragraph nodes, or port the
line-based block builder to JavaScript and let textlint supply nothing but the raw
source string. The first path looks more like a standard textlint rule, but the count
of findings drifts whenever the node granularity differs from the line granularity. The
second path was taken. `src/lib/blocks.js` ports the block builder from `markdown.rs`,
regular expressions included, and each rule reads the raw source through
`context.getSource(node)` and runs it through this local block builder before judging
anything. textlint's job becomes reading the file, honoring configuration, and
reporting; it does no parsing of the document structure on its own.

The cost of this choice is that reported line and column numbers are computed against
the preprocessed string rather than the raw one. A position in the raw file can drift
from a position in the preprocessed one wherever an escape sequence was undone or
inline markup was stripped. `src/lib/position.js` searches for the matched text within
the target line, left to right, and ties it back to a character position in the raw
source as an approximation. The approximation exists so that a finding is never
dropped, not to guarantee an exact position. The check against `tests/golden/` compares
counts only; position accuracy is not part of that bar.

**IPADIC folds UniDic's four taigen categories into one part of speech.** The Japanese
judgment for M1 and M5 asks whether the last token at the end of a line or item is a
taigen, a noun-like word that can end a sentence on its own. suikou-core runs on UniDic
and treats a token as taigen when its top-level part of speech is noun, suffix,
pronoun, or adjectival noun stem. kuromojin, the standard analyzer in the textlint
ecosystem, runs on IPADIC, which has no such four-way split at that level.

Probing kuromojin 3.0.1 directly showed that words UniDic classifies as suffix, pronoun,
or adjectival noun stem all surface under IPADIC as a noun whose finer subcategory
reads pronoun, suffix, or adjectival-noun stem instead. "これ" (this) came back as
noun/pronoun and "静か" (quiet) came back as noun/adjectival-noun-stem. Under IPADIC the
four-way UniDic check collapses into one condition: that the top-level part of speech
reads noun. `isTaigen` in `src/lib/morphology-ja.js` rests on this probe.

The check for a verb or auxiliary ending in the ren'youkei needed the same kind of
verification. UniDic subdivides that form, as in "連用形-一般", while IPADIC reports it
as the bare string "連用形" with no suffix, confirmed against "使い" and "し". The
Rust-side check, `cform.startsWith("連用形")`, ports over unchanged.

**Messages follow the document's language; the Rust side always answers in Japanese.**
`finding()` in suikou-core returns a Japanese message even for an English document, a
behavior confirmed against the output of `suikou check` on `en_maintainability.md`.

Some users of the textlint preset write only English documents, and a message they
cannot read does not help them. The judgment logic and the regular expressions stay
aligned with the Rust side, but the preset picks a Japanese or English message to match
the language it detects. Both live in `src/lib/messages.js`. Message text plays no part
in the count comparison, so this divergence does not touch the golden-test completion
bar.

**Tests call `node:test` directly instead of going through textlint-tester.**
textlint-tester assumes Mocha's global `describe` and `it`. Where those globals are
absent, it falls back to a stand-in that just calls the test function without waiting
for it. That stand-in never awaits the Promise a test case returns, so running it under
`node --test` let some assertions report success before the linting they depended on
had even finished.

Two paths were weighed: add Mocha as a dependency to keep textlint-tester, or write a
thin layer directly on `@textlint/kernel`. The path that added no dependency won.
`test/support/lint.js` wraps the kernel in a few lines, registers every rule under the
`maint/` prefix, and awaits the result directly. That fits `node --test`'s async test
functions without any adapter.

**The overlap with `no-ai-colon-continuation` from
`@textlint-ja/textlint-rule-preset-ai-writing` is left unsuppressed.** That rule also
uses kuromojin to check whether the text before a colon ends in a predicate, which is
close to the Japanese half of M1.

The purpose differs: that rule flags a stylistic
pattern typical of AI-written prose, and M1 flags a list structure that breaks when an
item is added or removed. The two can still fire on the same line. No mechanism was
added to let one suppress the other. A user who enables both presets has read both and
is in a position to judge which finding fits their document; silencing one on their
behalf would remove information rather than add it. The overlap is written down in the
README instead.

## D-24 The term extraction cuts words by structure, not part of speech alone

Japanese groups a run of consecutive nouns into a single compound word.
UniDic tags a bound suffix such as 素 or 性 with its own part of speech, separate from
noun, so treating nouns alone split 形態素解析 into 形態 and 解析, and left 保守性 as just
保守. Running the tool against its own documentation is what surfaced the break.

Adding the suffix tag to the noun tag fixes that split, but it also lets a counter such
as つ start a compound on its own. UniDic offers no finer-grained tag that would separate
a counter from an ordinary derivational suffix, so building a list of counters was not an
option. The rule leans on structure instead: a suffix only extends a run that already
holds a noun. A suffix cut off by a numeral, for example the つ in 3つ, never starts a run
by itself.

UniDic marks a space only as a boundary between words and keeps no word for the space
itself. Joining two Latin-script words without a space builds a word the source text
does not have. Running the tool against its own documentation showed this: Linux
kernel, which appears in several places in `design.md`, came out as Linuxkernel. The
join now adds a space between two Latin-script words that sit next to each other, and
adds no space between a Latin-script word and a Japanese word, as in 仕様 attached to
API. That pair is normally written with no space inside a Japanese sentence, so the
join adds no space there either.

The list that filters out generic words reuses `KEISHIKI_MEISHI` from `metrics/ja.rs`
rather than building a new one. A generic word that slips into the glossary only loses
its exclusion from the metaphor check, a small cost. Missing a genuine term from the
field costs more, so the balance favors keeping words in.

Running the tool against its own documentation surfaced a further problem:
single-character words filled the top of the word list. Out of 395 words, 54 words
were a single character, and the top twenty words were 文 (50), 形 (49), 語 (48),
指示 (43), 値 (36), then 地 (28) and 数 (27), then 版 (22), 行 (22), and 層 (20). 地
is what is left after の split 地の文 into two words, and the rest of these words are
ordinary Japanese words that appear often in any piece of writing and belong to no
one field.

Naming the words to drop was ruled out, since naming them is itself a guess. The fix
uses structure instead: the number of characters in the assembled word. A word that
comes out to one character is dropped. Dropping one-character words is common
practice in Japanese term extraction, since a word of several characters carries
most of the meaning, while a one-character word is the smallest possible word and
tends to carry many meanings, rarely pointing to one field. A word of two characters
built from one token, for example 文書, passes through this filter untouched. After
the fix, the top words in the word list for this repository's own documentation were
指示, 規則, 指標, 人間, 文書, 日本語, 指摘, 閾値, 辞書, and 実装, with no
one-character word among them.

English has neither a part-of-speech tagger nor a sentence splitter. The extraction
leans on words written in capitals alone, a capitalized word that does not sit at the
start of a sentence, and inline code that reads as a bare identifier. Table rows are
excluded from this scan. A table cell carries no sentence, so the start-of-sentence
signal never fires there, and the same label repeats down a column often enough to
clear the occurrence floor. Running the tool against its own English documentation
showed table values leaking into the glossary this way.

A heading that opens with an id carries the same problem. This file's own heading,
D-04, The tokenizer hides behind a trait, treats D-04 as the first word, so the word
The that follows reads as though it sat mid-sentence. The fix keeps treating the
position as sentence-initial for as long as capital-only words keep appearing at the
start of the text.

A heading with two id-like words side by side carries a further problem. This
file's own heading, D-13, M2 and M6 count every occurrence, has no mark between D-13
and M2, so the two words merge into one word, D-13 M2. The word M2 on its own loses
that occurrence and can fall under the occurrence floor. The cost falls on the
missing side, a real word left out, rather than on the invented side, a word the
source never wrote, so this stays a known limit rather than a fix. Fixing it needs a
rule that decides how far to merge two id-like words next to each other, and no such
rule has turned up that would do this without breaking a pair such as AWS S3, where
two short words next to each other name one product.

The occurrence floor defaults to two. A word seen once gives no way to tell a term of
the field apart from an incidental phrase, and checking a term for consistency needs
the same word to appear more than once in the first place. This default comes from
that pair of goals rather than from measurement.

## D-25 MCP hand-writes JSON-RPC instead of adding a dependency, and no daemon

T9 needed two judgment calls: how to implement the MCP server, and whether to build a
daemon.

**MCP is implemented with `serde_json` alone.** The choice stood between adding a
dedicated crate such as `rmcp` and hand-writing the three methods this server answers:
`initialize`, `tools/list`, and `tools/call`.

The set of requests it handles stops at those three; it uses none of resources, prompts,
or server-initiated notifications. A
dedicated crate would pin a crate version for features the server never calls, out of
step with this repository's habit of pinning dependencies with `=` and rerunning
`verify_schema` on every update. `serde_json` is already a dependency of both crates,
so the new dependency count is zero. The stdio transport also reduces to one JSON-RPC
message per line, a shape simple enough that hand-writing the transport carried little
risk. The implementation lives in `crates/suikou-cli/src/mcp.rs`; the implementation
calls `check::analyze` and `brief::run` and nothing else, so the judgment logic is not
written twice between the CLI and the MCP paths.

**No daemon.** The daemon's original justification was capping the cost of loading
the dictionary once. The cost measured in T2, 0.68-0.70 ms in release, did not
support that justification, and TASKS.md called for measuring the cost of process
startup itself before deciding either way. A release build (`--features
lindera-unidic`) ran `suikou check` against a short document fifty times, measuring
wall-clock cost per run; `suikou --version` served as a comparison, its cost also
measured over fifty runs. The table below lists the measured cost, from WSL2.

| command | average cost | max cost |
|---|---|---|
| `suikou --version` | about 1.5-1.6 ms | about 1.9-2.4 ms |
| `suikou check` (with the dictionary, short document) | about 3.7-3.9 ms | about 4.3-4.7 ms |

`suikou --version` never loads the dictionary, so its cost stands for the cost of
process startup on its own: reading the roughly 200 MB executable, linking the
executable, and parsing its arguments. The added cost in `suikou check` runs a couple
of milliseconds, the combined cost of loading the dictionary (0.7 ms) and measuring
one short document. Both costs are negligible next to the seconds an LLM turn takes,
the setting a hook that fires on every Write and Edit runs in. A daemon would add
complexity of its own — for example, managing a socket or a named pipe, deciding how
to detect file changes, and serializing concurrent callers — for a cost saving too
small to justify that complexity. `suikou daemon` stays as a subcommand; running it
prints the measured cost and this judgment instead of doing work. Keeping the
subcommand, rather than removing the subcommand outright, lets a user who types it
see directly why it does nothing.

## D-26 Off-register native words are found by comparing frequencies against a reference corpus

A generative model translating English into Japanese sometimes swaps a Sino-Japanese
word for a native one. 確認する becomes 確かめる, and 必要である becomes 要る.
The result reads too soft for technical documentation, and it slows the reader down.

What counts as the right register was settled without building a table of word pairs.
A table would mean enumerating every difference in phrasing across every field.
The rule compares frequencies against a corpus of professional Japanese translation
instead.

Measurement backs that decision.
Looking at Kubernetes alone, 使う appears 320 times in the Sonnet translation against 33
in the professional one, which invites a rule saying 使う should become 使用.
Widening the corpus to Kubernetes, MDN, and Vue shows 使う 461 times in professional
translation. Candidates such as 次 for 以下, 仕組み for メカニズム, and 既定 for
デフォルト fell the same way. MDN uses 既定, so the convention differs by field.

Two statistics decide a finding.
The log-likelihood ratio from Dunning (1993) says whether the difference is significant,
and the Log Ratio from Hardie (2014) says how large it is.
Significance alone is not enough, because using an ordinary word slightly more often
reaches significance on a corpus this size.

The Log Ratio floor came from measurement.
Twelve professional pages held out of the reference were compared against this
repository's Japanese pages.
A floor of 4 reports 9 words in the professional pages, a floor of 6 reports 1, and a
floor of 7 reports none.
The design treats a warning on human-written text as a false positive, so the floor sits
where the professional pages come back clean, at the cost of missing some findings.

Building the reference carries two conditions of its own.
A tutorial must stay out: the Rust Book translation carries 確かめる 36 times, which would
pull it into the reference. That agrees with the measured finding that the kind of
document moves the metrics more than the register does.
A word also has to appear in two fields or more, because a word confined to one field
belongs to that field rather than to the reference.

`.suikou/register-allow.toml` carries the words a project accepts.
The reference is built from web documentation, which is thin on the vocabulary of
measurement and arithmetic, and the project covers that gap itself.
That is the same shape as D-11, where the glossary is built from the documents at hand.

## D-27 The guidance attached to the kango ratio was wrong

D7 told the writer to open kango into native verbs. Measurement contradicts that advice.

Comparing the word origin of content words between professional translation and this
repository gives the table below.

| | Native | Sino-Japanese | Loanword | Mixed |
|---|---|---|---|---|
| Professional translation | 0.384 | 0.329 | 0.280 | 0.007 |
| This repository | 0.470 | 0.463 | 0.050 | 0.017 |

Professionals do not hold the kango ratio down by reaching for native verbs.
They hold it down by using the established loanword, and their native share is in fact
lower than this repository's.
The guidance now reads: reach for the established loanword, and stop stacking kango nouns.

The wrong guidance did real damage.
Following it through a rewrite turned 検証 into 確かめ, 制約 into 縛り, and 抽出 into
取り出し, and the register collapsed.
The rule in D-26 exists to catch that collapse.

Driving a metric to zero is not the goal.
D7 carries the severity info, which marks it as advisory, and treating it as a gate was
the original mistake.
A document in this repository has to come back free of errors and warnings.
Anything at info is read as advice and nothing more.

## D-28 The register rule did not carry over to English

Two experiments tried to bring the method from D-26 to English. Neither worked, so no
English rule ships.

The first compared a document against a reference corpus.
A frequency table was built from 652 pages and 510,000 words of Kubernetes, MDN, and Vue
in English, and the same statistics ran against it.
Fourteen professional English pages held out of the reference still return 20 findings
with the floor raised to 9.
The top of both lists is field terminology: portworx, cinder, and ebs on the professional
side, and prose, corpus, and metric on this repository's side.
Narrowing to words that appear in all three fields leaves words such as operations, which
belong to the topic of the page rather than to its register.

The cause sits in the structure of the lexicon.
In Japanese, the condition "the word origin is native" removes field terminology almost
mechanically, because field terminology is Sino-Japanese or a loanword.
English marks nothing of the kind. A document about a topic uses the vocabulary of that
topic far more often than a general corpus does, so frequency alone cannot separate the
vocabulary of the topic from the vocabulary of the register.

The second experiment held the topic fixed.
Professional Japanese translations were translated back into English by the model and
compared against the original English, across 36 pages and 30,000 words on each side.
Not one word reached significance. Against a critical value of 15.13, the largest was
your at 13.2 and the next was will at 11.1, and the original uses both more often than
the model does, so the difference does not even run the other way.
The same design in Japanese put 使う at 320 against 33, with a log-likelihood ratio of 320.

This result has the same shape as D-08.
An effect size must not be carried across tasks, and a method that holds in one language
must not be carried across languages either.

Only word choice was tested here.
MATTR, the prose ratio, and the simile markers measure something else, and they stand.

`research/lexshift/` reproduces both experiments.
`build_reference_en.py` builds the English table and `translate_back.sh` produces the
back-translation.
The failed side is kept so that nobody runs the same experiment twice.

## D-29 A Sino-Japanese word loses to the loanword when the corpus says so

D-26 decided against a table of word pairs, because the convention reverses between
fields. That decision is revised for pairs of a Sino-Japanese word and a katakana
loanword. A counter-example no longer disqualifies a pair: the side that wins across the
fields as a whole is the side to follow.

Several mechanical routes to synonymy were tried and dropped.
Using an English word as a pivot and linking whatever co-occurs in the aligned paragraph
finds collocations rather than synonyms, producing pairs such as 永続 with ボリューム and
番号 with ポート. Lift and the Dice coefficient behave the same way.
Substitution is about alternation, not co-occurrence, and a paragraph is the wrong unit
to see it.

Listing the candidates and deciding among them were therefore separated. A person lists
them and measurement decides, which is the shape D-26 already used. Of 97 candidates, 53
were kept.

Three conditions decide. The loanword has to outnumber the Sino-Japanese word at least
two to one, the pair has to reach twenty occurrences, and the Sino-Japanese side has to
stay at or below sixty.

That third condition keeps words with more than one sense out.
Measurement shows professionals using 対象, 対応, 状態, and 場合 heavily, and not always
in the sense the loanword carries. This repository uses 対象 for the scope a metric
covers, which is not an object at all.
Restricting the table to words professionals barely use keeps each pair trustworthy.

A word preceded by a noun is skipped as part of a compound, since rewriting the 版 inside
英語版 or 第3版 would be wrong.

`.suikou/register-allow.toml` exempts a word a project uses in another sense. This
repository exempts 記録, 一覧, 比率, 経路, 分岐, and 実体; 比率 means a ratio such as the
kango ratio, not a rate.


## D-30 Structure is a guardrail, and the outline comes before the prose

Rules at the level of the sentence cannot rescue a document whose structure is wrong.
A missing section, an order that does not follow, a paragraph carrying three claims at
once: none of these are visible while reading one sentence at a time.

The structural rules apply to both languages.
The set of sections, their order, and the unit of a paragraph are properties of the
argument a document makes, and they sit outside the vocabulary of any one language.
Measurement bears this out. Paragraphs of seven sentences or more run at 0.27 percent
in professional English and at 0.59 percent in professional Japanese.
This is where the result parts from D-28, where the register rule failed to carry over.

The doctypes and their required sections were taken from existing standards rather than
invented. RFC 7322 sets out required sections in a recommended order. The sections of
each doctype come from the Rust RFC template, the Kubernetes Enhancement Proposal,
Michael Nygard's Architecture Decision Record, and Diátaxis. The paragraph ceiling
follows the Google developer documentation style guide at six sentences.
The constraint against rules invented from a hunch holds here as well.

Only a missing section is an error. Order, paragraph length, and an empty section are
warnings. A document without a required section cannot answer the question its reader
arrived with, whereas order and paragraph length are a path through the argument rather
than a rule that must never be broken.

Section matching looks at the top level of headings alone.
A leading level-one heading is the title rather than a section, so it is dropped.
Looking deeper lets a subheading inside one section match a different section and throws
the order check off, which is exactly what happened to the English design document in
this repository.

Only the first section can be answered by the lead paragraph instead of a heading.
The abstract in RFC 7322 sits between the title and the table of contents without a
heading of its own, and a README takes the same shape. Forcing a `## What it is` there
reads worse than the paragraph it replaces.

### The outline is approved before the prose

`suikou plot` writes a template into `.suikou/plans/`.
It carries the section headings, the question each section answers, and a blank line for
the claim that section makes. A section whose claim cannot be written in one line has not
been thought through, and the plot surfaces that before the body exists.

The plot is kept out of version control, because it is a document of the thinking rather
than a deliverable. CI therefore cannot see it, and the comparison runs only on a local
machine and inside an agent loop. That is not a weakness. A structural mistake is meant
to be caught before the writing starts, not stopped at the gate before release.

### Guidance, not only detection

Every rule up to here pushed in one direction: find a violation, then remove it.
A list of violations settles what to avoid. It does not settle what to write.

So the template carries the question each section answers and what belongs in it, and
`suikou plot` and `suikou brief --doctype` put that in front of the writer before the
body exists. The text of the guidance lives in one place, `structure::writing_rules`, so
that it cannot drift from the text of the findings.

Token output does not grow. The template appears only when `--doctype` is given.

## D-31 Measurement took two required sections out of the how-to type

The required sections of each doctype were taken from an existing framework rather than
invented. Diátaxis describes a how-to guide, and prerequisites, steps, and a check became
the three required sections. Two of the three fell the moment they met professional
documentation.

Running `howto` against 120 Kubernetes task pages reported `Steps` absent on all 120 and
`Verify` absent on 113. Not one page came back clean.

Counting the headings explains it. Of the H2 headings on those pages, 111 are
`{{%/* heading "prerequisites" */%}}` and 84 are `{{%/* heading "whatsnext" */%}}`. The rest are
particular to the page: `Create a namespace`, `Edit a Secret`, and so on, each naming the
operation it covers.

The steps of a how-to are its body, not a section of it. What Diátaxis says is that a
how-to guide is a sequence of steps, not that a heading reading `Steps` has to sit above
them. Forcing that heading would strip from the outline the one thing it carries, which
is what each step does. That works against the guidance that the headings alone should
carry the line of the argument.

So `steps` and `verify` are no longer required. `prerequisites` stays required.

### Measuring again turned up two more defects

The first run had all 120 pages satisfying `prerequisites`. They were not satisfying it;
the rule was too loose to notice.

Which section the lead paragraph could stand in for was decided by position. Any document
with an opening paragraph therefore satisfied its first required section, whatever that
section was. Running `howto` against 120 concept pages reported nothing on any of them,
which is a type doing no work at all.

Whether a section can be answered by an opening paragraph is a property of the section,
not of its position. Only a section that states the subject now carries `lead_ok`.
Prerequisites and context are not answered by an opening paragraph. After the change, 117
of the 120 concept pages report a missing section, and the type discriminates.

Section synonyms were matched against the document's own language alone. The Japanese
translation of Kubernetes keeps the `{{%/* heading "prerequisites" */%}}` shortcode as
written, so matching Japanese synonyms alone reported a missing section on 110 of 111
pages. Matching now runs against the synonyms of both languages, because the language of
a document and the language of a heading need not agree.

### Rates after the repairs

| Corpus | Pages | Missing a required section |
|---|---|---|
| English task pages | 120 | 9 (7.5%) |
| Japanese task pages | 111 | 6 (5.4%) |
| English concept pages run as `howto` | 120 | 117 (97.5%) |
| Japanese concept pages run as `howto` | 120 | 116 (96.7%) |

The two languages land in the same band. Against documents of another type the rule fires
on nearly every page, and against documents of its own type it stays nearly silent. That
gap is what shows the type doing its work. Reading the pages that fire, they genuinely
carry no `Before you begin`, and several declare `task` while reading as explanation:
`Dependency on Docker explained` is one. These are places where Kubernetes departs from
its own template, not false positives here.

### Out of the detection, still in the guidance

The question and the note on what belongs there stay in the doctype. `suikou plot` and
`suikou brief --doctype howto` still ask how it is done and how the reader knows it
worked.

What can be detected and what is worth asking before writing are not the same set.
Telling the reader how to confirm the result is a mark of a good procedure; the presence
of a heading reading `Verify` is not a way to measure it. Separating those two is what
this decision turns on.

### Only the how-to type could be measured from outside

The required sections of `design`, `decision`, and `overview` have been tried against the
documents in this repository and nowhere else. Those documents were written to match the
doctype, so the exercise confirms nothing.

Measuring against an outside corpus needs documents that declare a type. Kubernetes
declares `content_type`, whose values are `task`, `concept`, `reference`, and `tutorial`,
and only `task` lines up with a type defined here. Diátaxis explanation and tutorial
would be the types for the other two, and a type will not be added until there is
something to measure it against.

## D-32 The design type failed against the Rust RFCs it was taken from

The required sections of `design` were said to come from the Rust RFC template. Testing
that claim meant running the type against 200 RFCs from the top level of `text/` in
`rust-lang/rfcs`, taken from the highest numbers down so that the template had settled.
Running a type against its own source is the strictest test available.

The first run reported `Design` absent on all 200, `Non-goals` on 199, and `Scope` on 198.
Each of the three had a different cause.

### A mismatch of names

The 200 for `Design` were not a wrong requirement but a wrong name.

The Rust RFC template calls that section `Guide-level explanation` and
`Reference-level explanation`, and the corpus carries 157 of the first and 148 of the
second. `Explanation` went into the synonyms, since both names contain it.
`Implementation` and `Proposal` went in beside it, as names design documents use widely.

### Requiring a section the template has no room for

`Scope` and `Non-goals` are absent from the Rust RFC template. They belong to the
Kubernetes Enhancement Proposal, so two templates had been merged into one requirement.

Both are now optional, and both keep their question and their note on what belongs there.
`plot` and `brief` go on asking how far it goes and what it will not do, as in D-31.

### Two defects in the parser

Heading indentation had no ceiling. Four spaces or more make a line code, and CommonMark
does not read a heading there. A `#` inside an indented code block became a level-one
heading, which made level one the top level of that document and left its real sections
invisible. Indentation is now capped at three spaces.

Fenced code blocks were removed with a regular expression. A long fence holding a shorter
one inside it closed at the inner fence, and the code past that point was read as prose,
which turned `# [dependencies]` into a heading. A line scan replaces it: a fence closes
only on a line of the same marker, at least as long, with nothing after it. That is what
CommonMark specifies.

Both fixes are ported into the textlint preset, and the two implementations return
identical M-rule counts on real RFCs.

### The rate after the repairs

21 of the 200 report a missing section, or 10.5 percent. Eighteen of those are not
feature proposals at all: roadmaps, project group charters, team changes, and policy
RFCs, which sit in `text/` without following the feature template. The remaining three
use a section name of their own, such as `Proposed additions` or `Solution`.

Adding synonyms stops here. Keep adding section names and the count approaches zero, but
that is fitting the corpus rather than describing a template. A name earns its place by
appearing in a published template.
