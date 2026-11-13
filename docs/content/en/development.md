---
title: "Development"
weight: 40
doctype: "reference"
---

This page covers building the project, pinning versions, handling the tokenizer, and
running the golden tests. Each of these is a constraint that work in this repository has
to respect.

## Building and testing

The commands for checking the core are listed below. The first command is the one to
reach for most of the time, and the second command is the one to reach for when the
dictionary matters.

```sh
cargo test --all                           # No tokenizer. Fast. Start here
cargo test --all --features lindera-unidic # With the dictionary. Slower
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

The default features leave lindera out, so the core logic can be checked without
downloading a dictionary. Leaving lindera out keeps the loop for checking a change
short. For example, a change to the rules can be checked in seconds.

The release build names that feature explicitly.

```sh
cargo build --release -p suikou-cli --features lindera-unidic
```

A build without the dictionary handles English only.
Asking that build for Japanese makes that build fail on the spot rather than return an
empty analysis. `suikou selftest` reports which build is in hand.

## Pinning versions

Pinning versions has a reason behind it, and that reason is set out here. The reason is
that a version change can break a value without breaking a build.

Every dependency is pinned with `=`, and `Cargo.lock` is committed.
Every version is confirmed to exist on crates.io before that version is written down.

lindera gets the strictest treatment.
The lindera API and the order of the UniDic features both change between versions.
When that order shifts, the word origin and the inflected form break silently.
The kango ratio and the renyo count both depend on those two fields, so running on broken
values makes the findings themselves wrong.

After a lindera upgrade, `LinderaMorphology::verify_schema` has to pass.
That check pushes a known sentence through the tokenizer and confirms that the part of
speech, the word origin, and the inflected form come from the expected positions.
On a mismatch that check reports which constant to fix and exits.
Failing beats returning a wrong value quietly.

## Handling the tokenizer

The tokenizer changes behavior between versions, so the tokenizer is kept behind a
boundary.

Version-dependent code lives only in `LinderaMorphology`, inside
`crates/suikou-core/src/tokenizer.rs`.
Every other module reaches the tokenizer through the `Morphology` trait.
Tests use `FakeMorphology`, so the logic can be checked with no dictionary present at
all.

A new metric must not cross that boundary.
Crossing that boundary means the tests stop running without a dictionary.

Measured in a release build, loading the embedded dictionary takes 0.7 milliseconds.
Analyzing one sentence after that takes 9.6 microseconds.

```sh
cargo run --release --features lindera-unidic --example dict_load
```

## Keeping the scope of each metric straight

Vocabulary metrics, meaning MATTR, the kanji ratio, the kango ratio, demonstratives, and
formal nouns, run over `Document::body()`.
Sentence-structure metrics, meaning sentence length, commas, renyo, and the te-form, run
over `Document::prose()`.

A vocabulary metric must never run over prose alone.
Some models put almost everything into bullet lists, and for those models prose alone
throws away more than eighty percent of the document.

## Keeping the preprocessing order

`preprocess` in `markdown.rs` depends on the order of its steps.
Moving the step that undoes backslash escapes later breaks sentence splitting.
The AWS doc_source corpus is written in that escaped form, and leaving the escapes in
place inflates the mean sentence length by a factor of three or more.

`tests/golden/input/en_escaped.md` pins that behavior.

Each removal leaves the same number of newlines behind, so a finding points at the line
the user has open. A correct value at the wrong position is a finding nobody can use.

## Measuring how often a rule fires

Adding a rule or moving a threshold calls for a measurement against the reference corpus.

```sh
cargo run --release --example structure_rate -- corpus/cache/pro_en en
cargo run --release --example structure_rate -- corpus/cache/pro_ja ja
```

`suikou check` truncates the positions it prints per rule, so the count cannot be read
off the CLI output. The example calls the rules directly. What comes out is recorded in
`TASKS.md`.

`corpus/fetch.sh` fetches the corpus, which licensing keeps out of the repository.

Measuring the required sections needs a corpus that declares a type.

```sh
python research/structure/fetch_typed.py corpus/cache/typed 120
```

That collects Kubernetes pages grouped by their own `content_type:`. Map those values
onto the doctypes under `[front_matter]` in `.suikou/structure.toml`, then run `check`
as usual. `research/structure/structure.toml` holds that configuration.

The size of the report is compared before and after a rule changes. Build the previous
revision in a `git worktree`, run both over the same corpus, and total the characters.

The `design` type is measured against the Rust RFCs.

```sh
python research/structure/fetch_rfcs.py corpus/cache/rfcs 200
```

Running a type against its own source is the strictest test available.

The `decision` type is measured against published ADRs.

```sh
python research/structure/fetch_adrs.py corpus/cache/adrs 200 12
```

`research/structure/structure-adr.toml` holds that configuration.

## Choosing a word

A word is not chosen by ear. The common way of saying it is looked up first.

### Two questions

Ask them in order.

1. Would someone writing in this field choose this word in a specification or a procedure?
2. Does it map word for word back into English?

The first asks what gets written, not what gets understood in speech. A yes to the second
and a no to the first marks a phrase copied word by word from English: `根因` for root
cause, `時間窓` for time window, `基準線` for baseline.

Repetition within the same document, or across this repository, is not evidence for the
first question. Vocabulary from one AI-written document carries into the next, so
repetition says nothing about what is established.

### Where the evidence comes from

Two sources answer the first question.

```sh
grep -c "根因" corpus/cache/pro_ja/*.md
plamo-translate --from English --to Japanese --input "The root cause was a missing index."
```

The reference corpus is professional translation, so it shows the word that was chosen in
writing. PLaMo Translate is a model built for translation, so it shows the Japanese put
against an English term. Where both point at the same word, that word is the common one.

### Leave ordinary Sino-Japanese alone

No word is swapped merely to lower the kanji ratio. `確認`, `使用`, `停止`, and `動作` are
ordinary in technical Japanese and are not replaced with native equivalents; D-27 records
what happened when they were. Formal words carrying no content, such as `多角的`, are a
separate matter and do get revisited.

### A settled word goes into the glossary

What the lookup settles is written into `crates/suikou-core/data/glossary_ja.toml`
alongside the figure that settled it. `research/glossary/build_glossary.py` regenerates
the portable forms under `glossary/`.

Orthography is kept apart from word choice, because the authority differs. Orthography
starts from the JTCA guideline and is confirmed per word against the reference corpus.

## Using PLaMo Translate

PLaMo Translate is a translation model that runs locally. It is optional, and it is used
when a document is written from scratch and when a word choice needs confirming.

### Which size this machine takes

| Condition | Model | Why |
|---|---|---|
| 8 GB of VRAM or less, or no GPU | `plamo-3-translate-2b` at Q4_K_M, 1.81 GB | The Linux default; loads quickly and suffices at the sentence level |
| 12 GB of VRAM or more | `plamo-2-translate` 10B at Q4, about 6.2 GB | Reaches the paragraph level |

This machine has 8 GB, so it takes the 2B.

### What each size is used for

The 2B is held to words and sentences, never to rewriting a paragraph or a whole document.
The 10B reaches paragraphs but loads slowly enough to want the resident server.

Neither size has its output pasted into a document. One probe rendered `lint` as `糸くず`,
the fluff on fabric. What is read is which word it chose, and no more.

### Running it

```sh
export PLAMO_TRANSLATE_CLI_LLAMA_SERVER=<path to llama-server>
plamo-translate server &
plamo-translate --from English --to Japanese --input "..."
```

A call costs 5.1 seconds without the resident server and 1.3 seconds with it, so a batch
of words starts the server first.

`llama-server` has to be at b11960 or above. The published builds require glibc 2.38, so this
machine builds it from source.

## Golden tests

The golden tests exist to keep the values in line with the reference implementation.
The reference implementation is the Python under `research/`, and the golden tests pin
what that implementation returns.

`crates/suikou-core/tests/golden_rules.rs` compares the rule counts, and
`golden_metrics.rs` compares the document metrics, against `tests/golden/expected/`.
English runs without a dictionary, and Japanese needs `--features lindera-unidic` to run.

lindera and fugashi are different tokenizers, so the comparison allows a margin.
A ratio allows an absolute error of 0.02, a density per thousand characters allows 0.5,
and the M rules demand an exact integer match.
A word or sentence count allows a relative error of one percent, because the count
depends on how the tokenizer is defined.

When a metric falls outside that margin, the question of which implementation is right
gets settled case by case, and the reasoning goes on the decisions page.
The Python side is not automatically right.

## Not adding rules on a hunch

**This is the heaviest constraint in the repository.**

A hunch about style turns out to be wrong most of the time once someone measures it.
Many of the hypotheses raised while designing this project were rejected by measurement.
Research on translationese found the same thing about its own hypotheses.

Adding a rule means showing one of the following.

- A measurement over a human corpus and over AI output, with ranges that do not overlap.
- A clause in a published style guide, cited.

`corpus/baselines.toml` records the rejected metrics together with the reason under
`[rejected]`.
**Nothing recorded there may be reintroduced.**
The coefficient of variation of sentence length, the loanword ratio, and participial
clauses and nominalization in English all sit on that list.
Each one is widely described as a marker of AI prose, and each one overlaps the human
range once it is measured.

## Holding down token use

Holding down token use puts the constraints below on the report. Each constraint exists
to keep the report small enough to read in one pass.

The report format carries constraints.

- The explanation for a rule is printed once per rule and never repeated per line.
- At most `Report::MAX_POSITIONS_PER_RULE` positions are printed per rule.
- A metric inside its threshold is not printed at all.
- A document metric becomes an instruction rather than a number.

These exist so that the linter and the model never have to pass a document back and forth
layer by layer.
Anyone changing the format should confirm that the change leaves that purpose intact, and
that the report still fits in one pass.

## Writing documentation here

The documentation in this repository follows the rules the tool enforces.

### Settle the structure before writing

A document starts with its outline rather than its prose.

```sh
suikou plot docs/content/en/example.md --doctype design
```

That writes a file under `.suikou/plans/` holding the section headings and the question
each section answers. Fill in the claim each section makes, one line each, and get that
approved before the body is written. A claim you cannot state in one line marks a section
you have not thought through.

The plot stays out of version control, because it is a document of the thinking rather
than a deliverable. If the structure moves while the body is written, move the plot with
it; `structure/plot-mismatch` reports the difference.

The instruction handed to a model carries the doctype too.

```sh
suikou brief --doctype design --lang en
```

The question behind each section and the rules for writing come out ahead of the
prohibitions.

### Declare the doctype

Pages under `docs/content/` declare `doctype:` in the front matter.
A document that cannot carry front matter is declared under `[paths]` in
`.suikou/structure.toml`. Declaring a doctype adds the checks for required sections and
their recommended order.

### The rules followed here

- Open every list with a complete sentence that does not bind grammatically to the items.
- Keep the number of items out of the prose.
- Leave section numbers off headings.
- Avoid a word that depends on a moment in time; anchor it to a year or a version if you must.
- Avoid a bare metaphor, reaching for a simile or a worked example instead.
- Do not repeat a native word that professional technical Japanese avoids.
- Make one claim per paragraph, splitting the paragraph once it passes six sentences.
- Write each section as an answer to the question its heading raises.
- Quote a Hugo shortcode in its escaped form; Hugo runs it even inside a code span.

Run `suikou check` over a page before committing that page.
The bar is a page free of errors and warnings. Anything at info is advice.

Driving a metric to zero is not the goal.
The kango ratio sits at info, and an attempt to zero it turned 検証 into 確かめ and wrecked
the register of every Japanese page. D-27 records what happened.
To lower the kango ratio, reach for the established loanword rather than a native verb.
