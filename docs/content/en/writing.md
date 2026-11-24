---
title: "Writing"
weight: 7
doctype: "reference"
---

This page is a reference for the norms and procedures to follow when writing a document with suikou. It covers how to choose a word, how to structure a README, and what instruction to give the model.

[Development]({{< relref "development.md" >}}) explains how the tool is built and which invariants must stay fixed. [Tuning]({{< relref "tuning.md" >}}) explains how to fit the thresholds to your own documents.

## How to choose a word

Do not choose a word by intuition. First find out how the idea is usually expressed, and then choose.

### The order the checks run in

If the rules are applied in whatever order they come to mind, the result changes from one
attempt to the next. The order is therefore fixed.

**Constraints** are applied first. Each constraint decides whether a candidate is usable at
all, so the list of candidates gets shorter. **Preferences** are applied next and rank the
remaining candidates. The spelling is settled and the record is written last.

| Step | What is checked | Kind | Method |
|---|---|---|---|
| 1 | Is it a name the project has defined? | Constraint | Look in `.suikou/defined.toml`; if the name is there, use it and skip the remaining steps |
| 2 | List the candidates | — | Write down whatever comes to mind; intuition is acceptable at this step |
| 3 | Is it used in this field? | Constraint | Check its frequency in the reference corpus; drop any candidate that never occurs |
| 4 | Does the noun combine with the verb? | Constraint | Read the noun and verb pair in isolation |
| 5 | Is the meaning narrow? | Preference | See how widely its translations vary; choose the word whose translations converge |
| 6 | Does it evoke a physical action? | Preference | Among words of equal narrowness, choose the one that evokes the weaker image |
| 7 | Settle the spelling | — | Count each spelling in the reference corpus; if the counts do not decide, use the long vowel |
| 8 | Record the result | — | Add it to `glossary_ja.toml` with the measured value that decided it |

Constraints are applied first because ranking a candidate and then discarding it wastes
the ranking. If a candidate fails at step 4, return to step 2. If you replace the verb, you
must also repeat step 5.

### How step 3 checks frequency in the reference corpus

```sh
grep -c "根因" corpus/cache/pro_ja/*.md
```

The reference corpus consists of professional translations, so it shows which words
translators actually chose when writing. If a word never occurs in it, writers in the field
do not use that word.

This method does not work when the genre differs. The reference corpus consists of API
documentation and contains no sentences that set a norm or a limit. For this reason, no
replacement for を置く could be justified; T19 records the details.

### How step 4 reads a pair in isolation

```sh
cargo run --release --features lindera-unidic --example collocation -- <human side> <AI side>
```

This command uses the morphological analyzer to count noun-particle-verb triples and lists
the pairs that appear on the AI side but not on the human side. No automatic method can
decide which pairing is correct, so a person reads the list and decides.

### How step 5 measures the spread of translations

```sh
plamo-translate server &
python research/glossary/narrow.py --ja2en 置く 配置する 設定する
python research/glossary/narrow.py --en2ja place configure deploy
```

Each direction of translation measures something different.

| Direction | What it measures | When to use it |
|---|---|---|
| `--ja2en` | How many English words one Japanese word is translated into | To find out how broad the meaning of a Japanese word is |
| `--en2ja` | How many Japanese words one English word is translated into | To decide which word to standardize on |

The word is placed in a minimal sentence frame before translation. A sentence taken from a
corpus brings its own topic words into the translation, so it cannot be used for the
measurement.

The table below shows measured examples.

| Word | Converges | Translation |
|---|---|---|
| 配置する | 100% | place×6 |
| 作成する | 100% | create×6 |
| 作る | 83% | create×5 make×1 |
| 置く | 67% | place×4 set×1 establish×1 |
| 設定する | 67% | set×4 configure×2 |

置く and 設定する are equally broad, so replacing one with the other does not reduce
ambiguity; the word to choose is 配置する. Before this tool was built, the suggestion had
been to replace one broad word with another broad word.

When the translation of the word being tested is the same as a noun in the frame, no
measurement is possible. For example, `configure` translates to 設定, which already appears
in the frame as 設定を. In that case the tool reports that it cannot measure the word
instead of printing a value.

### How step 6 prefers the weaker image

When two candidates are equally narrow, choose the one that evokes a physical action less
strongly. **Words of this kind make a text read as AI-written.** Technical writing
explains how a mechanism works; it is not about moving objects by hand.

| Avoid | Choose |
|---|---|
| 外す | 除外する |
| 落とす | 除去する |
| 拾う | 検出する |
| 寄せる | 統一する |
| 突き合わせる | 照合する |

Measurement showed that this repository used the words on the left 8 to 32 times as often
as the reference corpus. For example, `外す` occurs 0.06 times per 10,000 characters in the
reference corpus and 1.90 times in this repository.

`register/rare-wago` cannot detect these words, because they do appear in the reference
corpus a few times and the rule's lower limit is 128 times the reference rate. A frequency
rule cannot reach them, but a preference can, and `glossary/variant` provides that
preference. Words not listed in the table are avoided for the same reason; when unsure, ask
whether the word suggests a movement of the hand or the body.

### Errors in the opposite direction

Do not replace a word only to lower the kanji ratio. `確認`, `使用`, `停止` and `動作` are
standard in technical Japanese and are kept as they are; D-27 records what happened when
they were replaced.

Formal words with no real content, such as `多角的`, are a separate problem and are reviewed
on their own. Where particles were omitted to string Sino-Japanese words together, keep the
words and restore the particles and verbs.

### How step 8 records the result in the glossary

A word settled by this process is added to `crates/suikou-core/data/glossary_ja.toml`
together with the measured value that settled it. `research/glossary/build_glossary.py`
regenerates the general-purpose files under `glossary/`.

The spelling is recorded separately from the word choice, because each is based on a
different authority.

### Why repeated use is not evidence

Step 3 asks whether a writer in this field would choose the word for a specification or a
procedure. It does not ask whether the word would be understood in conversation.

The fact that a word appears again in the same document, or elsewhere in this repository,
is not evidence. The vocabulary of one AI-written document carries over into the next, so
repeated use does not show that a word is established. The documents in this repository
were written by a model, so this caution applies to them as well.

No automatic method can choose the correct verb for a pair that step 4 reports. A method
that ranks candidates from 32,822 verbs by corpus statistics reaches only 80% accuracy even
within the top 600 or so (Nakano and Tomiura, 2011), so a person makes the decision. T19
records the details.

## How to write a README

### Put the subject in the lead paragraph

A measurement of 240 READMEs from well-known open source projects found that the structure
of a README is carried by its lead paragraph, not by its headings.

| Measured item | Value |
|---|---|
| READMEs with a lead paragraph | 78% |
| Of those, leads within 300 characters | 83% |
| Median length of the lead | 121 characters |

In the lead paragraph, use the project name as the subject and state what the project is.
Keep the paragraph within 300 characters.

```markdown
# suikou

suikou is a linter for technical documentation. It handles Japanese and English.
```

Do not begin with background, purpose or history. These three kinds of opening occurred
zero times in 831 lead paragraphs taken from READMEs, Rust RFCs, ADRs and the reference
corpus. `structure/lead-opener` detects them.

### Do not require headings

Even install, the section most often identified by its heading, is found in only 62% of the
READMEs. The `readme` type therefore requires only the lead paragraph.

Each section still has a question to answer, however. `suikou plot README.md --doctype readme`
lists these questions.

| Section | Question |
|---|---|
| What it is | What is it? Can it be described in one sentence? |
| Install | How do I install it? |
| Usage | What is the quickest way to use it? |
| Documentation | Where can I find the details? |
| Contributing | How can I contribute? |
| License | Under what terms can it be used? |

### Set the level of formality for each doctype

A README and a reference document cannot share one set of thresholds. Running prose makes
up 17% of a README, 43% of a reference document, and 54% of a Rust RFC.

Record the adjustment in `.suikou/profile.toml`.

```toml
[doctype.readme]
reason = "Agreed with the user: an entry point is mostly lists"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

Treat `reason` as nearly mandatory; a test confirms that every built-in adjustment includes
one. Any relaxation is recorded as the result of an agreement with the user, and is never
decided unilaterally.

The difference in formality does not always point the same way. Japanese READMEs contain
more Sino-Japanese vocabulary than the reference corpus, at 0.62 against 0.52. Relaxing every
threshold would therefore be wrong, and the direction must be checked for each metric. D-41
gives the details.

### Keep the author's notes out of the README

Do not put the state of the work, what you learned by trying something, or what is still
unfixed into a README. Readers of a README have no use for that information.

Diátaxis states that most documentation problems arise when a single piece of content tries
to meet more than one need at the same time. A note to yourself and a README meet different
needs.

Each kind of note has a designated place.

| Content | Where it goes |
|---|---|
| A judgment that could have gone either way | `docs/content/*/decisions.md` |
| A value you are unsure of, or an option you set aside | `TASKS.md` |
| A detailed explanation | `docs/content/` |

Such notes cannot be detected by vocabulary. Well-known open source projects also use
`currently` and `TODO`, and 45 of the 240 READMEs contain `currently`, so there is no basis
for banning these words. Only the opening of the document can be checked. D-39 records the
details.

## How to use PLaMo Translate

PLaMo Translate is a model dedicated to translation that runs on your own machine. It is
optional and is used in two situations: when a document is written from scratch, and when a
word choice needs to be confirmed.

### Which model size to use on this machine

| Condition | Model | Reason |
|---|---|---|
| 8 GB of VRAM or less, or no GPU | `plamo-3-translate-2b` at Q4_K_M, 1.81 GB | The default on Linux; it loads quickly and is adequate for single sentences |
| 12 GB of VRAM or more | `plamo-2-translate` 10B at Q4, about 6.2 GB | It can handle whole paragraphs |

This machine has 8 GB of VRAM, so it uses the 2B model.

### What each model size is used for

The 2B model is limited to words and sentences and is never used to rewrite a paragraph or
a whole document. The 10B model can handle paragraphs, but it loads slowly, so it is run as
a resident server.

The output of either model is never pasted into a document. In one test, the model
translated `lint` as `糸くず`, meaning the fluff on fabric. Only the word the model chose is
read from the output, and nothing more.

### How to run it

```sh
export PLAMO_TRANSLATE_CLI_LLAMA_SERVER=<path to llama-server>
plamo-translate server &
plamo-translate --from English --to Japanese --input "..."
```

Each call took 5.1 seconds without the resident server and 1.3 seconds with it. To check a
batch of words, start the server first.

`llama-server` must be version b11960 or later. The published binaries require glibc 2.38,
so on this machine it is built from source.

## Rules for writing documentation in this repository

The documentation in this repository follows the same rules that the tool enforces.

### Settle the structure before writing

When you create a document, write its outline before you write its prose.

```sh
suikou plot docs/content/en/example.md --doctype design
```

This command creates a file under `.suikou/plans/` that lists the section headings and the
question each section answers. Write the claim of each section in one line, and have the
claims approved before you write the body. If you cannot state a section's claim in one
line, you have not yet thought that section through.

The plot is kept out of version control, because it records the thinking and is not a
deliverable. If the structure changes while you write the body, update the plot to match;
`structure/plot-mismatch` reports any difference between the two.

When you build the instruction for a model, specify the doctype as well.

```sh
suikou brief --doctype design --lang en
```

The instruction lists the question for each section and the writing rules before the
prohibitions.

### Declare the doctype

Pages under `docs/content/` declare `doctype:` in their front matter.
A document that cannot have front matter is registered under `[paths]` in
`.suikou/structure.toml`. Declaring a doctype enables the checks for required sections and
their recommended order.

### The rules followed here

Every document in this repository follows the rules below.

- Introduce every list with a complete sentence that is not grammatically joined to the items.
- Do not state the number of items in the prose.
- Do not number headings.
- Do not use words whose meaning depends on the time of reading; if you must, tie the statement to a year or a version.
- Do not use a metaphor on its own; use a simile or a worked example instead.
- Do not repeatedly use native words that professional technical Japanese avoids.
- Limit each paragraph to one claim, and split it once it exceeds six sentences.
- Write each section as an answer to the question its heading poses.
- Quote a Hugo shortcode in escaped form, because Hugo executes it even inside a code span.

Run `suikou check` on a page before you commit it.
The page passes when it has no errors and no warnings. Findings at info level are advisory.

The goal is not to reduce any metric to zero.
The kango ratio is reported at info, and an attempt to reduce it to zero changed 検証 into
確かめ and damaged the register of every Japanese page. D-27 records what happened.
To lower the kango ratio, use an established loanword instead of a native verb.
