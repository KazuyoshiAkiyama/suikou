---
title: "Writing"
weight: 7
doctype: "reference"
---

This reference defines the standards and procedures for authoring documentation with suikou, covering word selection, README structure, and prompts provided to language models.

Architecture details and fixed invariants appear in [Development]({{< relref "development.md" >}}). Threshold calibration for custom repositories appears in [Tuning]({{< relref "tuning.md" >}}).

## Selecting terminology

Terminology decisions rely on established usage rather than personal intuition. Authors verify standard expressions before finalizing wording.

### Fixed evaluation order

Evaluating vocabulary rules in an arbitrary sequence produces inconsistent results, so checks follow a fixed order.

**Constraints** apply first to filter out invalid candidates. **Preferences** apply next to rank the remaining options, followed by spelling conventions and glossary recording.

| Step | Check | Category | Verification method |
|---|---|---|---|
| 1 | Defined term status | Constraint | Inspect `.suikou/defined.toml`; if an entry exists, adopt it and skip remaining steps |
| 2 | Candidate enumeration | — | Collect plausible options; intuition is permitted at this stage |
| 3 | Domain attestation | Constraint | Count occurrences in the reference corpus; discard candidates with no matches |
| 4 | Noun-verb compatibility | Constraint | Review the isolated collocation directly |
| 5 | Semantic specificity | Preference | Measure translation dispersion and favor the candidate that converges |
| 6 | Bodily imagery | Preference | Between equally specific candidates, favor the one with weaker physical imagery |
| 7 | Orthography | — | Count spelling variants in the reference corpus; default to the long vowel when counts do not decide |
| 8 | Glossary entry | — | Record the chosen term and supporting measurement in `glossary_ja.toml` |

Running constraints before preferences avoids spending effort ranking candidates that fail basic validity checks. A candidate rejected at step 4 returns the process to step 2, and replacing a verb requires repeating step 5.

### Step 3: reference corpus frequency

```sh
grep -c "根因" corpus/cache/pro_ja/*.md
```

Because the reference corpus consists of professional translations, its counts reflect vocabulary actually selected in published technical prose. Absence from the corpus indicates that the field does not use the term.

Corpus lookup cannot compensate for a genre mismatch. The reference corpus contains API documentation and lacks normative or limit-setting statements, which prevented establishing a corpus-backed substitute for を置く (recorded in T19).

### Step 4: isolated collocation review

```sh
cargo run --release --features lindera-unidic --example collocation -- <human side> <AI side>
```

This command extracts noun-particle-verb triples via the morphological analyzer and outputs pairs that appear in the AI text but not in the human reference. Because no automated check determines valid collocations reliably, a human reviewer evaluates the reported pairs.

### Step 5: translation dispersion

```sh
plamo-translate server &
python research/glossary/narrow.py --ja2en 置く 配置する 設定する
python research/glossary/narrow.py --en2ja place configure deploy
```

Each translation direction measures a distinct property.

| Direction | Property measured | Use case |
|---|---|---|
| `--ja2en` | Number of English translations produced from one Japanese term | Gauging the semantic breadth of a Japanese term |
| `--en2ja` | Number of Japanese translations produced from one English term | Selecting a single standard Japanese equivalent |

Each candidate is inserted into a minimal sentence frame rather than a full corpus sentence, because real corpus context introduces domain-specific subject matter that prevents controlled measurement.

| Word | Convergence | Translation |
|---|---|---|
| 配置する | 100% | place×6 |
| 作成する | 100% | create×6 |
| 作る | 83% | create×5 make×1 |
| 置く | 67% | place×4 set×1 establish×1 |
| 設定する | 67% | set×4 configure×2 |

Because 置く and 設定する show identical breadth, substituting one for the other does not reduce ambiguity; 配置する provides the narrower choice. Prior to this tool, earlier recommendations had proposed replacing one broad verb with another equally broad verb.

Measurement is impossible when a candidate's translation collides with a noun already present in the frame. Because `configure` translates to 設定 and the frame 設定を already contains that noun, the tool reports an unmeasurable case instead of outputting a score.

### Step 6: fainter physical imagery

When two candidates are equally specific, choose the term with weaker bodily imagery. **Expressions evoking physical movement are a strong marker of AI-generated text.** Technical documentation describes mechanisms rather than physical manipulation of objects.

| Avoid | Prefer |
|---|---|
| 外す | 除外する |
| 落とす | 除去する |
| 拾う | 検出する |
| 寄せる | 統一する |
| 突き合わせる | 照合する |

This repository contained the left-column verbs at 8 to 32 times their frequency in the reference corpus; for example, `外す` appeared at 0.06 per 10,000 characters in the reference corpus versus 1.90 here.

The `register/rare-wago` rule cannot flag these verbs because they still occur occasionally in the reference corpus and the rule's minimum threshold is 128 times the reference rate. Detection therefore requires a preference rule, enforced by `glossary/variant`, rather than a frequency check. The same criterion applies to verbs not listed in the table: check whether the word evokes a hand or body movement.

### Avoiding over-correction

Terms are never replaced solely to reduce the kanji ratio. Standard technical Japanese words such as `確認`, `使用`, `停止`, and `動作` belong in technical prose and remain untouched; D-27 documents the regression caused when they were replaced.

Content-free formal expressions such as `多角的` are a separate case and warrant revision. When particles have been omitted to concatenate Sino-Japanese words consecutively, keep those words intact and restore the missing particles and verbs.

### Step 8: glossary recording

Finalized word choices are recorded in `crates/suikou-core/data/glossary_ja.toml` together with the supporting measurement. Running `research/glossary/build_glossary.py` regenerates the portable glossary outputs under `glossary/`.

Orthographic rules are maintained separately from terminology choices because each relies on a different authority.

### Why repetition is not evidence

Step 3 evaluates whether a technical writer in the field would select a term in a specification or procedure, not merely whether the term is intelligible in conversation.

Repeated occurrences within the same document or across this repository do not validate a term. Because AI-generated documents reproduce vocabulary across subsequent drafts, internal repetition fails to demonstrate established domain usage, and that limitation applies to this repository's model-authored documentation.

Automated statistics cannot resolve the collocation candidates flagged in step 4. Corpus-based ranking across 32,822 verbs reaches 80% accuracy only within roughly the top 600 candidates (Nakano and Tomiura, 2011), so human judgment remains necessary (recorded in T19).

## Authoring a README

### Defining the subject in the lead

An analysis of 240 READMEs from prominent open-source projects showed that document structure is defined primarily by the opening lead paragraph rather than by section headings.

| Metric | Measurement |
|---|---|
| Presence of a lead paragraph | 78% |
| Lead paragraphs within 300 characters | 83% |
| Median lead length | 121 characters |

State what the project is within 300 characters, placing the project name as the grammatical subject.

```markdown
# suikou

suikou is a linter for technical documentation. It handles Japanese and English.
```

Do not open with background, intent, or history. Across 831 lead paragraphs sampled from READMEs, Rust RFCs, ADRs, and the reference corpus, these openings appeared zero times; `structure/lead-opener` flags them.

### Optional section headings

Even the most frequent section, installation, appears in only 62% of the surveyed READMEs. Accordingly, the `readme` doctype mandates only the lead paragraph and requires no specific headings.

A README still answers standard reader questions, which `suikou plot README.md --doctype readme` outlines.

| Section | Question |
|---|---|
| What it is | What is the project in one sentence? |
| Install | How do users install it? |
| Usage | What is the minimal way to run it? |
| Documentation | Where is detailed documentation located? |
| Contributing | How can contributors participate? |
| License | Under what license terms is it distributed? |

### Per-doctype formality thresholds

A single threshold configuration does not fit both READMEs and reference pages. Measured running prose accounts for 17% of a README, 43% of a reference document, and 54% of a Rust RFC.

Per-doctype overrides are configured in `.suikou/profile.toml`.

```toml
[doctype.readme]
reason = "Agreed with the user: an entry point is mostly lists"
rules = ["structure/long-paragraph"]

[doctype.readme.thresholds."ja.kango_ratio"]
value = 0.65
```

Including a `reason` field is effectively mandatory, as an automated test verifies that every built-in override specifies one. Any threshold relaxation must reflect an explicit agreement with the user rather than an independent adjustment.

Formality shifts do not move uniformly in a single direction. Japanese READMEs exhibit a higher Sino-Japanese proportion than the reference corpus (0.62 versus 0.52), so relaxing every metric uniformly would be incorrect; each metric must be evaluated individually (see D-41).

### Excluding author notes

Work-in-progress status, empirical observations from testing, and unresolved defects do not belong in a README because readers do not need internal tracking notes.

According to Diátaxis, most documentation failures arise when a single document attempts to satisfy multiple distinct needs simultaneously. Personal working notes and a README serve separate purposes.

Each category of information belongs in its designated location.

| Content | Destination |
|---|---|
| Design trade-offs that could have been decided either way | `docs/content/*/decisions.md` |
| Unverified figures or deferred options | `TASKS.md` |
| Detailed explanations | `docs/content/` |

Lexical rules cannot reliably exclude author notes. Terms such as `currently` and `TODO` also appear in prominent open-source READMEs — 45 of the 240 surveyed projects contain `currently` — leaving no empirical basis for banning them. Only the opening structure can be validated reliably, as documented in D-39.

## Working with PLaMo Translate

PLaMo Translate is an optional local translation model used when drafting documentation from scratch or verifying vocabulary choices.

### Model selection for this machine

| Hardware constraint | Model | Rationale |
|---|---|---|
| 8 GB of VRAM or less, or no GPU | `plamo-3-translate-2b` at Q4_K_M, 1.81 GB | Default on Linux; starts rapidly and is adequate for sentence-level checks |
| 12 GB of VRAM or more | `plamo-2-translate` 10B at Q4, about 6.2 GB | Capable of paragraph-level translation |

Because this machine has 8 GB of VRAM, it uses the 2B model.

### Scope of each model size

The 2B model is restricted to individual words and sentences and is never used to rewrite paragraphs or entire documents. The 10B model handles paragraph-level text, though its startup latency warrants running the resident server.

Raw translation output from either model is never copied directly into documentation. In one test, the model translated `lint` as `糸くず` (fabric fuzz); consult the model solely to observe its word choice.

### Execution setup

```sh
export PLAMO_TRANSLATE_CLI_LLAMA_SERVER=<path to llama-server>
plamo-translate server &
plamo-translate --from English --to Japanese --input "..."
```

An individual invocation takes 5.1 seconds without the resident server versus 1.3 seconds with it, so start the server first when querying multiple words in a batch.

`llama-server` requires build b11960 or higher. Because prebuilt binaries depend on glibc 2.38, this machine compiles it from source.

## Repository documentation workflow

Documentation in this repository adheres to the same rules that suikou checks.

### Finalizing structure before drafting

Authoring begins with an outline rather than full prose.

```sh
suikou plot docs/content/en/example.md --doctype design
```

This command creates a plan file under `.suikou/plans/` listing each section heading alongside the question that section must answer. Enter each section's claim in one line and obtain approval before writing the prose; any claim that cannot be stated in one line indicates an under-developed section.

Plan files remain excluded from version control because they capture intermediate planning rather than final deliverables. When the document structure changes during drafting, update the plan file accordingly, since `structure/plot-mismatch` flags discrepancies between the document and its plan.

Prompts generated for language models also incorporate the target doctype.

```sh
suikou brief --doctype design --lang en
```

The generated brief outputs each section's guiding question and affirmative writing guidelines before listing prohibitions.

### Declaring the doctype

Markdown pages under `docs/content/` specify `doctype:` in their front matter. Files that cannot contain front matter declare their doctype under `[paths]` in `.suikou/structure.toml`. Specifying a doctype enables verification of mandatory sections and their recommended ordering.

### Authoring rules applied in this repository

Documentation in this repository follows these writing conventions.

- Precede every bullet list with a complete sentence that does not connect grammatically to the list items.
- Omit item counts from surrounding prose.
- Omit sequential numbers from section headings.
- Avoid time-dependent expressions unless qualified by a specific year or version.
- Avoid bare metaphors; use an explicit simile or a concrete example instead.
- Do not repeat rare native Japanese words avoided in professional technical writing.
- Limit each paragraph to a single claim, and split any paragraph exceeding six sentences.
- Write each section to answer the question implied by its heading.
- Escape Hugo shortcodes when quoting them, because Hugo evaluates shortcodes even inside code spans.

Run `suikou check` on each page prior to committing it. A page passes when it has no errors or warnings; info-level diagnostics serve as advice.

Reducing every metric to zero is not an objective. The kango ratio is reported at info level; an earlier attempt to reduce it to zero turned 検証 into 確かめ and degraded the register across all Japanese pages (recorded in D-27). To reduce the kango ratio, substitute an established loanword rather than a native verb.
