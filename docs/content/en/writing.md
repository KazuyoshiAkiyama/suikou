---
title: "Writing"
weight: 7
doctype: "reference"
---

The norms and procedures for writing a document with suikou: how a word is chosen, how a README is laid out, and what instruction the model is handed.

How the tool is built, and the invariants that must not move, are in [Development]({{< relref "development.md" >}}). Fitting the thresholds to your own documents is in [Tuning]({{< relref "tuning.md" >}}).

## Choosing a word

A word is not chosen by ear. The common way of saying it is looked up first.

### The order the checks run in

Applying the rules in whatever order they come to mind gives an unstable answer, so the
order is fixed.

**Constraints** come first: each one settles whether a candidate holds at all, and the
field narrows. **Preferences** come second and rank what is left. Orthography and the
record come last.

| Step | What is checked | Kind | Method |
|---|---|---|---|
| 1 | Is it a defined name? | Constraint | Read `.suikou/defined.toml`; if it is there, that settles it and the rest is skipped |
| 2 | List the candidates | — | Whatever comes to mind; ear is allowed here |
| 3 | Is it written in this field? | Constraint | Frequency in the reference corpus; drop a candidate with none |
| 4 | Does the noun take the verb? | Constraint | Read the pair on its own |
| 5 | Is the meaning narrow? | Preference | How the translation spreads; take the one that converges |
| 6 | Does it lean on bodily imagery? | Preference | Among equally narrow words, take the one with the fainter image |
| 7 | Settle the orthography | — | Count per word in the reference corpus; the long vowel where counting does not decide |
| 8 | Record it | — | Write it into `glossary_ja.toml` with the figure that settled it |

Constraints run first because ranking a candidate and then discarding it is wasted work.
A candidate that fails step 4 goes back to step 2, and swapping the verb makes step 5
start over.

### Step 3: frequency in the reference corpus

```sh
grep -c "根因" corpus/cache/pro_ja/*.md
```

The reference corpus is professional translation, so it shows the word chosen in writing.
None of a word there means the field does not write it.

It does not reach a mismatch of genre. The reference is API documentation and never sets
a norm or a limit, which is why no replacement for を置く could be grounded; T19 records
that.

### Step 4: read the pair on its own

```sh
cargo run --release --features lindera-unidic --example collocation -- <human side> <AI side>
```

That counts noun-particle-verb triples through the morphological analyzer and lists the
pairs present on the AI side and absent on the human side. Nothing settles the right
pairing automatically, so a person reads what comes out.

### Step 5: how the translation spreads

```sh
plamo-translate server &
python research/glossary/narrow.py --ja2en 置く 配置する 設定する
python research/glossary/narrow.py --en2ja place configure deploy
```

The two directions measure different things.

| Direction | What it measures | When it is used |
|---|---|---|
| `--ja2en` | How many English words one Japanese word splits into | To learn how broad a Japanese word is |
| `--en2ja` | How many Japanese words one English word splits into | To decide which one to standardise on |

The word goes into a minimal frame. A corpus sentence brings its own subject matter along
and nothing can be measured through it.

| Word | Converges | Translation |
|---|---|---|
| 配置する | 100% | place×6 |
| 作成する | 100% | create×6 |
| 作る | 83% | create×5 make×1 |
| 置く | 67% | place×4 set×1 establish×1 |
| 設定する | 67% | set×4 configure×2 |

置く and 設定する are equally broad, so moving from one to the other removes no ambiguity;
配置する is the one to take. Before this tool existed the suggestion had been to move a
broad word to another broad word.

Where the word's own translation collides with a frame noun there is nothing to measure.
`configure` translates to 設定, which the frame 設定を already holds. The tool then reports
that it cannot measure rather than printing a figure.

### Step 6: take the fainter image

Where two candidates come out equally narrow, take the one whose bodily image is fainter.
**Words of this kind mark a text as AI-written.** Technical prose explains a mechanism; it
does not move objects around.

| Avoid | Take |
|---|---|
| 外す | 除外する |
| 落とす | 除去する |
| 拾う | 検出する |
| 寄せる | 統一する |
| 突き合わせる | 照合する |

This repository used the words on the left 8 to 32 times more than the reference corpus:
`外す` runs 0.06 per 10,000 characters there and 1.90 here.

`register/rare-wago` cannot reach them, because the reference does use them a little and
its floor is 128 times the reference rate. Frequency does not get there; a preference
does, and `glossary/variant` carries it. A word absent from the table is avoided for the
same reason: ask whether it suggests a movement of the hand or the body.

### The mistake in the other direction

No word is swapped merely to lower the kanji ratio. `確認`, `使用`, `停止` and `動作` are
ordinary in technical Japanese and stay; D-27 records what happened when they did not.

Formal words carrying no content, such as `多角的`, are a separate matter and do get
revisited. Where particles were dropped to pack Sino-Japanese words together, the words
stay and the particles and verbs come back.

### Step 8: record it in the glossary

What the lookup settles goes into `crates/suikou-core/data/glossary_ja.toml` alongside the
figure that settled it. `research/glossary/build_glossary.py` regenerates the portable
forms under `glossary/`.

Orthography is kept apart from word choice, because the authority differs.

### Repetition is not evidence

What step 3 asks is whether someone writing in this field would choose the word in a
specification or a procedure, not whether it is understood in speech.

Finding the word again in the same document, or elsewhere in this repository, is not
evidence. Vocabulary from one AI-written document carries into the next, so repetition
says nothing about what is established. These documents were written by a model, so this
applies to them.

Nothing settles the right pairing for what step 4 reports. Ranking candidates from 32,822
verbs by corpus statistics reaches 80% accuracy only within the top 600 or so (Nakano and
Tomiura, 2011), so a person decides. T19 records it.

## Writing a README

### The lead holds the subject

Measuring 240 READMEs from notable open source showed the structure sitting in the lead
paragraph rather than in the headings.

| Measured | Value |
|---|---|
| Has a lead paragraph | 78% |
| Of those, within 300 characters | 83% |
| Median length of the lead | 121 characters |

Make the name the subject and say what it is, within 300 characters.

```markdown
# suikou

suikou is a linter for technical documentation. It handles Japanese and English.
```

Do not open on background, intent or history. Those three openings occurred zero times
across 831 lead paragraphs drawn from READMEs, Rust RFCs, ADRs and the reference corpus.
`structure/lead-opener` reports them.

### No heading is required

The most commonly identified section, install, is found in 62% of them. The `readme` type
requires the lead and nothing else.

The questions remain, though. `suikou plot README.md --doctype readme` lays them out.

| Section | Question |
|---|---|
| What it is | What is it, in one sentence? |
| Install | How is it installed? |
| Usage | What is the shortest way to use it? |
| Documentation | Where is the detail? |
| Contributing | How does someone contribute? |
| License | Under what terms? |

### Formality is settled per doctype

One set of thresholds cannot serve a README and a reference document. A README is 17%
running prose, a reference document 43%, a Rust RFC 54%.

The adjustment goes in `.suikou/profile.toml`.

```toml
[doctype.readme]
reason = "Agreed with the user: an entry point is mostly lists"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

`reason` is close to mandatory: a test checks that every built-in adjustment carries one. A
loosening is recorded as something agreed with the user, never decided here.

Formality does not differ in one direction. Japanese READMEs carry more Sino-Japanese than
the reference corpus, 0.62 against 0.52. Loosening everything would be wrong; the direction
is read per metric. D-41 has the detail.

### The writer's notes stay out

The state of the work, what turned out to be true on trying it, what has not been fixed
yet: none of it belongs in a README, because the reader has no use for it.

Diátaxis holds that most documentation problems come from content trying to serve more than
one need at once. A note to self and a README serve different needs.

Each kind has its place.

| Content | Where it goes |
|---|---|
| A judgement that could have gone either way | `docs/content/*/decisions.md` |
| A figure you are unsure of, an option set aside | `TASKS.md` |
| The fuller explanation | `docs/content/` |

Vocabulary cannot catch it. `currently` and `TODO` are used by notable open source too, 45
of the 240 containing `currently`, so there is no ground for forbidding them. Where the
document starts is the only part that can be checked. D-39 records it.

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
