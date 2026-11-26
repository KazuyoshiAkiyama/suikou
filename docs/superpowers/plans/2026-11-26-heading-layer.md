# 見出しの検査レイヤー 実装計画

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 見出しに当てる規則を `headings.rs` に集約し、章タイトルの用言止めを検出する `structure/heading-form` を追加する。

**Architecture:** `crates/suikou-core/src/headings.rs` を新設し、`BlockKind::Heading` だけを読む規則をそこへ置く。`maint/numbered-heading` と `maint/time-dependent` の見出し分を `rules/mod.rs` から移し、未接続の `glossary/defined-name` を配線する。型による除外は組み込みプロファイルの `[doctype.<型>] rules` を使い、規則側に型の分岐を持たせない。

**Tech Stack:** Rust 2021、lindera 6.0.0 + UniDic（`--features lindera-unidic`）、`FakeMorphology` による辞書なしテスト、`toml`、`serde`

**Spec:** `docs/superpowers/specs/2026-11-26-heading-layer-design.md`

## Global Constraints

- 形態素解析は `Morphology` トレイト経由でのみ使う。lindera に直接依存してはならない。
- 既定の feature（lindera なし）で `cargo test --all`、`cargo fmt --all`、`cargo clippy --all-targets -- -D warnings` のすべてが通ること。
- `tests/golden/expected/rules.json` と `tests/golden/input/*.md` は書き換えない。
- `maint/time-dependent` と `maint/numbered-heading` の判定内容、識別子、文面は変更しない。
- 組み込みプロファイルの `[doctype.<型>]` には必ず `reason` を書く。テストが理由の存在を検証している。
- 1つのルールで報告する箇所は `Report::MAX_POSITIONS_PER_RULE`（20）件までとする。`rules/mod.rs` の `finding` ヘルパと同じ切り詰めを行う。
- **1つの規則の識別子に対して `LocalFinding` は1件だけ返す。** `report.rs:136` の `to_markdown` は `local` の要素ごとに節を出力するため、同じ識別子で2件返すと説明文が二重に出て、位置の上限が producer ごとになる。本文と見出しの双方に当たる規則は、見出しのレイヤーが `Vec<Position>` を返し、本文側の producer が1件に束ねる。
- `structure/heading-form` は `lang == Lang::Ja` のときだけ呼ぶ。英語の経路の `UnusedMorphology::tokenize` は `unreachable!` である（`crates/suikou-cli/src/morphology.rs:20`）。
- `.suikou/defined.toml` は作らない。

## Review Focus

- 標題だけの文書（`# 題` のみ、節が無い）に `structure/heading-form` を当てたとき、`sections` が空を返してもパニックしないこと。
- 見出しが `###` から始まる文書で、`sections` が最も浅い階層を返す定義どおりに第3階層を章タイトルとして扱うこと。階層を2に固定した実装は誤りである。
- 日本語を含まない見出し（`## ja.renyo_per_1k`、`## JSON`）が `structure/heading-form` を発火させないこと。最後の形態素が補助記号になり、用言ではないため既定でも通るが、意図した除外であることをテストで固定する。
- 括弧を閉じていない見出し（`## 設定する（任意`）で、末尾を整える処理が無限ループや範囲外アクセスを起こさないこと。
- `maint/time-dependent` が標題（先頭の第1階層の見出し）にも当たり続けること。移動前の挙動がそうなっている。
- 本文と見出しの双方に `maint/time-dependent` の対象語がある文書で、`report.local` に `maint/time-dependent` の `LocalFinding` が1件しか現れないこと。2件返すと md 出力で説明文が二重になる。
- 英語と判定された文書に日本語の章タイトルが混ざっていても、形態素解析器が呼ばれずパニックしないこと。

---

## ファイル構成

| ファイル | 責務 |
|---|---|
| `crates/suikou-core/src/headings.rs` | 新設。見出しに当てる規則の入口と判定 |
| `crates/suikou-core/src/lib.rs` | `pub mod headings;` を追加 |
| `crates/suikou-core/src/rules/mod.rs` | M3 を削除し、M6 は見出しの位置を `headings` から受け取る |
| `crates/suikou-core/src/tokenizer.rs` | `Token::is_yougen` を追加 |
| `crates/suikou-core/src/check.rs` | `headings::check` を `evaluate_with_glossary` から呼ぶ |
| `crates/suikou-core/profiles/{oss,service}.toml` | `howto` と `log` の `rules` に除外を追加 |
| `crates/suikou-core/tests/golden_rules.rs` | `count` が `headings::check` の結果も連結する |
| `skills/suikou/SKILL.md`、`skills/ja-style/SKILL.md` | 見出しを体言止めへ書き直す |
| `docs/content/{ja,en}/metrics.md` | 見出しのレイヤーの節を追加 |
| `crates/suikou-core/src/glossary.rs` | `check_defined` が本文と見出しの双方を1件に束ねる |
| `crates/suikou-core/examples/structure_rate.rs` | `structure/heading-form` の発火率を計測する |
| `docs/content/{ja,en}/decisions.md` | D-52 を追加 |
| `TASKS.md` | 発火率の計測値と、`brief` への追加の見送りを記録 |

---

### Task 1: 用言の判定

**Files:**
- Modify: `crates/suikou-core/src/tokenizer.rs:43-45`（`is_taigen` の隣）

**Interfaces:**
- Produces: `Token::is_yougen(&self) -> bool`

- [ ] **Step 1: 失敗するテストを書く**

`tokenizer.rs` の `mod tests` に置く。`FakeMorphology::from_spec` の記法は `表層|品詞1|品詞2|活用形|語種` である。

```rust
#[test]
fn a_verb_an_auxiliary_and_an_adjective_are_yougen() {
    let m = FakeMorphology::from_spec(
        "する|動詞|非自立可能|終止形-一般|和 ない|助動詞||終止形-一般|和 \
         広い|形容詞|一般|終止形-一般|和 方法|名詞|普通名詞||漢 ）|補助記号|括弧閉||記号",
    );
    let yougen = |s: &str| m.tokenize(s).last().unwrap().is_yougen();
    assert!(yougen("する"));
    assert!(yougen("ない"));
    assert!(yougen("広い"));
    assert!(!yougen("方法"));
    assert!(!yougen("）"));
}
```

- [ ] **Step 2: 失敗を確認する**

Run: `cargo test --all is_yougen`
Expected: FAIL。`no method named is_yougen`

- [ ] **Step 3: `Token::is_yougen(&self) -> bool` を実装する**

`is_taigen` と同じ形で、`pos1` が `動詞`、`助動詞`、`形容詞` のいずれかに一致するかを返す。

- [ ] **Step 4: 通ることを確認する**

Run: `cargo test --all is_yougen`
Expected: PASS

- [ ] **Step 5: コミットする**

```bash
git add crates/suikou-core/src/tokenizer.rs
git commit -m "feat: 用言で文を止めているかの判定を Token に足す"
```

---

### Task 2: 見出しの末尾を整える

**Files:**
- Create: `crates/suikou-core/src/headings.rs`
- Modify: `crates/suikou-core/src/lib.rs`（`pub mod headings;`）

**Interfaces:**
- Produces: `headings::trim_tail(text: &str) -> &str`

- [ ] **Step 1: 失敗するテストを書く**

```rust
#[test]
fn the_tail_of_a_heading_is_trimmed() {
    assert_eq!(trim_tail("設定を変更する（任意）"), "設定を変更する");
    assert_eq!(trim_tail("T14 ルールを導入する（済み）"), "T14 ルールを導入する");
    assert_eq!(trim_tail("検証の方法。"), "検証の方法");
    assert_eq!(trim_tail("register/rare-wago: 文体にそぐわない和語"), "register/rare-wago: 文体にそぐわない和語");
    assert_eq!(trim_tail("List lead-ins (M1)"), "List lead-ins");
    assert_eq!(trim_tail("設定する（任意"), "設定する（任意");
    assert_eq!(trim_tail("（）"), "");
    assert_eq!(trim_tail("   "), "");
}
```

- [ ] **Step 2: 失敗を確認する**

Run: `cargo test --all the_tail_of_a_heading`
Expected: FAIL。`headings` モジュールが無い

- [ ] **Step 3: `trim_tail` を実装する**

落とす対象は、末尾の空白（半角と全角）、`。`、`：`、`:`、`、`、`！`、`？`、`.`、および末尾が `）` または `)` のときの対応する開き括弧までである。括弧の除去と記号の除去を、文字列が変わらなくなるまで繰り返す。開き括弧が無い閉じ括弧では何も落とさず、繰り返しを打ち切る。

- [ ] **Step 4: 通ることを確認する**

Run: `cargo test --all the_tail_of_a_heading`
Expected: PASS

- [ ] **Step 5: コミットする**

```bash
git add crates/suikou-core/src/headings.rs crates/suikou-core/src/lib.rs
git commit -m "feat: 見出しの末尾の括弧と句読点を落とす処理を足す"
```

---

### Task 3: structure/heading-form

**Files:**
- Modify: `crates/suikou-core/src/headings.rs`

**Interfaces:**
- Consumes: `Token::is_yougen`（Task 1）、`trim_tail`（Task 2）、`crate::structure::sections(doc) -> Vec<Outline<'_>>`（`Outline` は `block: &Block`、`level: usize` を持つ）
- Produces: `headings::RULE_HEADING_FORM: &str = "structure/heading-form"`、`headings::check_form(doc: &Document, morph: &dyn Morphology) -> Option<LocalFinding>`

- [ ] **Step 1: 失敗するテストを書く**

`morph()` ヘルパには `する`、`ない`、`方法`、`構成`、`k`、`設計` を含む `FakeMorphology` を用意する。

```rust
#[test]
fn a_section_heading_ending_in_a_predicate_is_reported() {
    let d = Document::parse("# 標題\n\n## 構成を設計する\n\n本文である。\n");
    let f = check_form(&d, &morph()).expect("発火していない");
    assert_eq!(f.rule_id, RULE_HEADING_FORM);
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.positions.len(), 1);
    assert_eq!(f.positions[0].line, 3);
}

#[test]
fn a_nominal_section_heading_is_not_reported() {
    let d = Document::parse("# 標題\n\n## 検証の方法\n\n本文である。\n");
    assert!(check_form(&d, &morph()).is_none());
}

#[test]
fn the_title_is_not_a_section_heading() {
    // 標題が用言で終わっても節ではない。
    let d = Document::parse("# 構成を設計する\n\n本文である。\n");
    assert!(check_form(&d, &morph()).is_none());
}

#[test]
fn a_document_without_sections_is_not_reported() {
    let d = Document::parse("# 標題\n\n本文である。\n");
    assert!(check_form(&d, &morph()).is_none());
}

#[test]
fn the_shallowest_level_is_the_section_even_when_it_is_the_third() {
    let d = Document::parse("### 構成を設計する\n\n本文である。\n");
    let f = check_form(&d, &morph()).expect("第3階層が章タイトルになっていない");
    assert_eq!(f.positions[0].line, 1);
}

#[test]
fn a_nested_heading_is_not_a_section_heading() {
    let d = Document::parse("# 標題\n\n## 検証の方法\n\n### 構成を設計する\n\n本文である。\n");
    assert!(check_form(&d, &morph()).is_none());
}

#[test]
fn a_heading_without_japanese_is_not_reported() {
    let d = Document::parse("# 標題\n\n## ja.renyo_per_1k\n\n本文である。\n");
    assert!(check_form(&d, &morph()).is_none());
}

#[test]
fn a_parenthesised_suffix_does_not_hide_the_predicate() {
    let d = Document::parse("# 標題\n\n## 構成を設計する（任意）\n\n本文である。\n");
    assert!(check_form(&d, &morph()).is_some());
}
```

- [ ] **Step 2: 失敗を確認する**

Run: `cargo test --all --lib headings`
Expected: FAIL。`check_form` が無い

- [ ] **Step 3: `check_form` を実装する**

`structure::sections(doc)` の各要素について、`trim_tail(&o.block.text)` を取る。空であれば飛ばす。日本語の文字（ひらがな、カタカナ、漢字）を1字も含まなければ飛ばす。残ったものを `morph.tokenize` に渡し、最後の形態素が `is_yougen` であれば `Position { line: o.block.line, column: 1, text: <整えた見出しの先頭60字> }` を積む。文面は「章タイトルが用言で終わっている。体言止めの名詞句にする」とする。位置が空なら `None` を返す。切り詰めは `MAX_POSITIONS_PER_RULE` に従う。

- [ ] **Step 4: 通ることを確認する**

Run: `cargo test --all --lib headings`
Expected: PASS（8件）

- [ ] **Step 5: 実文書で確かめる**

Run: `cargo run -q --features lindera-unidic -- check skills/suikou/SKILL.md skills/ja-style/SKILL.md --format text 2>&1 | grep -c heading-form`
Expected: この時点では配線前のため `0`。配線は Task 5 で行う。

- [ ] **Step 6: コミットする**

```bash
git add crates/suikou-core/src/headings.rs
git commit -m "feat: 章タイトルの用言止めを検出する structure/heading-form を足す"
```

---

### Task 4: M3 の移動と M6 の位置の受け渡し

**Files:**
- Modify: `crates/suikou-core/src/rules/mod.rs:515-560`（`check_all` の見出しの分岐と `m6.extend`）
- Modify: `crates/suikou-core/src/headings.rs`
- Modify: `crates/suikou-core/tests/golden_rules.rs:31`

**Interfaces:**
- Consumes: `rules::RULE_NUMBERED_HEADING`（すでに `pub`）。`re_m3`、`re_m6_ja`、`re_m6_en`、`re_relative_period`、`re_relative_period_en`、`pos_of_unanchored_matches` は `rules/mod.rs` で private であるため `pub(crate)` へ上げる。`pos_of_unanchored_matches` の実シグネチャは `fn(b: &Block, re: &Regex, lang: Lang) -> Vec<Position>`（`rules/mod.rs:353`）
- Produces: `headings::check_numbered(doc: &Document) -> Option<LocalFinding>`、`headings::time_dependent_positions(doc: &Document, lang: Lang) -> Vec<Position>`

- [ ] **Step 1: 失敗するテストを書く**

`headings.rs` に置く。

```rust
#[test]
fn a_numbered_section_heading_is_reported_here() {
    let d = Document::parse("# 標題\n\n## 2. 検証の方法\n\n本文である。\n");
    let f = check_numbered(&d).expect("発火していない");
    assert_eq!(f.rule_id, crate::rules::RULE_NUMBERED_HEADING);
    assert_eq!(f.positions[0].line, 3);
}

#[test]
fn a_numbered_title_is_not_reported() {
    // ADR の `# 3. 題` は文書識別子である。
    let d = Document::parse("# 3. 題\n\n本文である。\n");
    assert!(check_numbered(&d).is_none());
}

#[test]
fn a_time_dependent_word_in_a_heading_yields_a_position() {
    let d = Document::parse("# 標題\n\n## 現在の仕様\n\n本文である。\n");
    let p = time_dependent_positions(&d, Lang::Ja);
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].line, 3);
}

#[test]
fn a_time_dependent_word_in_the_title_yields_a_position() {
    // 移動前の挙動を保つ。標題も古びる。
    let d = Document::parse("# 現在の仕様\n\n本文である。\n");
    assert_eq!(time_dependent_positions(&d, Lang::Ja).len(), 1);
}

#[test]
fn an_anchored_heading_yields_no_position() {
    let d = Document::parse("# 標題\n\n## lindera 6.0.0 で確認した現在の順序\n\n本文である。\n");
    assert!(time_dependent_positions(&d, Lang::Ja).is_empty());
}
```

`rules/mod.rs` の `mod tests` に、束ねが1件であることを固定するテストを追加する。

```rust
#[test]
fn time_dependent_findings_from_prose_and_headings_are_one() {
    let d = Document::parse("## 現在の構成\n\n最新の版では挙動が変わる。\n");
    let f = check_all(&d, Lang::Ja, &morph());
    let n = f.iter().filter(|x| x.rule_id == RULE_TIME_DEPENDENT).count();
    assert_eq!(n, 1, "{f:?}");
    let t = f.iter().find(|x| x.rule_id == RULE_TIME_DEPENDENT).unwrap();
    assert_eq!(t.positions.len(), 2, "{t:?}");
}
```

`rules/mod.rs` の `detects_numbered_heading` は削除する。同じ入力に対するテストが `headings.rs` へ移るためである。

- [ ] **Step 2: 失敗を確認する**

Run: `cargo test --all --lib headings 2>&1 | tail -20`
Expected: FAIL。`check_numbered` と `time_dependent_positions` が無い

- [ ] **Step 3: `check_all` から M3 を外し、M6 に見出しの位置を渡す**

`check_all` の `BlockKind::Heading` の分岐、`m3` の収集、`is_title` の算出、`RULE_NUMBERED_HEADING` の `finding` 呼び出しを削除する。`m6.extend` の2行は `b.kind != BlockKind::Heading` のときだけ実行する形に変える。ループの後、`finding(RULE_TIME_DEPENDENT, ...)` を呼ぶ前に `m6.extend(headings::time_dependent_positions(doc, lang))` を加え、`(line, column)` で並べて重複を落とす処理をその後に移す。

- [ ] **Step 4: `headings.rs` に2つの関数を実装する**

`check_numbered` は `doc.blocks` の見出しを順に見て、先頭の第1階層の見出しを標題として外し、残りに `re_m3` を `block.raw.trim()` へ当てる。文面と重大度は削除前の `finding` 呼び出しにあるものをそのまま使う。`time_dependent_positions` は見出しのすべてのブロック（標題を含む）に `pos_of_unanchored_matches` を語の一覧と相対期間の両方で当て、連結して返す。並べ替えと重複の除去は呼び出し側が行う。

- [ ] **Step 5: ゴールデンテストの `count` を直す**

`golden_rules.rs:31` の `let findings = rules::check_all(doc, lang, morph);` を、`check_all` の結果に `headings::check_numbered(doc)` の結果を連結した `Vec<LocalFinding>` に変える。M6 は `check_all` が1件に束ねたまま返すため連結の対象にしない。

- [ ] **Step 6: 通ることを確認する**

Run: `cargo test --all 2>&1 | tail -15`
Expected: PASS。失敗0件

Run: `cargo test --all --features lindera-unidic 2>&1 | tail -15`
Expected: PASS。失敗0件。`golden_rules` の M3 と M6 が参照実装と一致すること

- [ ] **Step 7: コミットする**

```bash
git add crates/suikou-core/src/rules/mod.rs crates/suikou-core/src/headings.rs crates/suikou-core/tests/golden_rules.rs
git commit -m "refactor: 見出しに当てる M3 と M6 の判定を headings へ移す"
```

---

### Task 5: 入口の関数と配線

**Files:**
- Modify: `crates/suikou-core/src/headings.rs`
- Modify: `crates/suikou-core/src/glossary.rs`（`check_defined` を本文と見出しの双方に当てる）
- Modify: `crates/suikou-core/src/check.rs:94-107`（`evaluate_with_glossary` の中、`register` を呼ぶ箇所の隣）

**Interfaces:**
- Consumes: `check_form`、`check_numbered`、`glossary::Defined`、`glossary::defined() -> &'static Vec<Defined>`
- Produces: `headings::check(doc: &Document, lang: Lang, morph: &dyn Morphology) -> Vec<LocalFinding>`、`headings::defined_positions(doc: &Document, list: &[glossary::Defined]) -> Vec<Position>`

- [ ] **Step 1: 失敗するテストを書く**

`headings.rs` に置く。`PanicMorphology` は `tokenize` で `panic!` するテスト専用の型として同じテストモジュール内に定義する。

```rust
struct PanicMorphology;
impl crate::tokenizer::Morphology for PanicMorphology {
    fn tokenize(&self, _text: &str) -> Vec<crate::tokenizer::Token> {
        panic!("英語の経路で形態素解析器が呼ばれた")
    }
}

#[test]
fn the_entry_point_collects_the_heading_only_rules() {
    let d = Document::parse("# 標題\n\n## 2. 構成を設計する\n\n本文である。\n");
    let f = check(&d, Lang::Ja, &morph());
    let ids: Vec<&str> = f.iter().map(|x| x.rule_id.as_str()).collect();
    assert!(ids.contains(&RULE_HEADING_FORM), "{ids:?}");
    assert!(ids.contains(&crate::rules::RULE_NUMBERED_HEADING), "{ids:?}");
}

#[test]
fn an_english_document_never_touches_the_tokenizer() {
    // 英語の経路の形態素解析器は unreachable! である。呼んだら落ちる。
    let d = Document::parse("# 日本語の標題\n\n## 構成を設計する\n\nBody text here.\n");
    let f = check(&d, Lang::En, &PanicMorphology);
    assert!(!f.iter().any(|x| x.rule_id == RULE_HEADING_FORM), "{f:?}");
}

#[test]
fn a_defined_name_called_otherwise_in_a_heading_yields_a_position() {
    let list = vec![crate::glossary::Defined {
        name: "doctype".to_string(),
        instead_of: vec!["文書型".to_string()],
        note: String::new(),
    }];
    let d = Document::parse("# 標題\n\n## 文書型の一覧\n\n本文である。\n");
    let p = defined_positions(&d, &list);
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].line, 3);
}

#[test]
fn an_empty_defined_list_yields_no_position() {
    let d = Document::parse("# 標題\n\n## 文書型の一覧\n\n本文である。\n");
    assert!(defined_positions(&d, &[]).is_empty());
}
```

`glossary.rs` の `mod target_tests` に、本文と見出しが1件に束なることを固定するテストを追加する。

```rust
#[test]
fn defined_name_findings_from_prose_and_headings_are_one() {
    let list = vec![Defined {
        name: "doctype".to_string(),
        instead_of: vec!["文書型".to_string()],
        note: String::new(),
    }];
    let d = Document::parse("# 標題\n\n## 文書型の一覧\n\n文書型を宣言する。\n");
    let f = check_defined(&d, &list).expect("発火していない");
    assert_eq!(f.rule_id, RULE_DEFINED);
    assert_eq!(f.positions.len(), 2, "{f:?}");
}
```

- [ ] **Step 2: 失敗を確認する**

Run: `cargo test --all --lib 2>&1 | grep -E "^error|test result" | head`
Expected: FAIL。`check` と `defined_positions` が無い

- [ ] **Step 3: `defined_positions` と `check` を実装する**

`defined_positions` は見出しのブロックに対し、`glossary::check_defined` の本文側と同じ照合を行って `Position` を返す。照合の本体は `glossary.rs` に `fn defined_hits(blocks: &[&Block], list: &[Defined]) -> Vec<Position>` として切り出し、`pub(crate)` とする。`check` は `check_numbered` を言語を問わず呼び、`check_form` を `lang == Lang::Ja` のときだけ呼び、`Option` を `Vec` へ平坦化して返す。`defined_positions` は `check` に含めない。本文側の producer が束ねるためである。

- [ ] **Step 4: `glossary::check_defined` を本文と見出しの双方に当てる**

`check_defined` が `defined_hits` を本文のブロックと見出しのブロックの双方に対して呼び、位置を連結し、`(line, column)` で並べて重複を落としてから1件の `LocalFinding` を返す形にする。

- [ ] **Step 5: `evaluate_with_glossary` から呼ぶ**

`crates/suikou-core/src/check.rs` の `evaluate_with_glossary` で、`rules::check_all` の直後に `headings::check(doc, lang, morph)` の結果を `local` へ追加する。続けて `glossary::check_defined(doc, glossary::defined())` の結果も追加する。`check_form` の言語の絞りは `headings::check` の内側にあるため、ここで分岐しない。

- [ ] **Step 6: 通ることを確認する**

Run: `cargo test --all 2>&1 | tail -15`
Expected: PASS

Run: `cargo run -q --features lindera-unidic -- check skills/suikou/SKILL.md skills/ja-style/SKILL.md --format text 2>&1 | grep -c heading-form`
Expected: `12`

Run: `cargo run -q --features lindera-unidic -- check docs/content/ja/tuning.md --format json 2>&1 | grep -c heading-form`
Expected: `0`。`tuning.md` は `howto` 型だが除外は Task 6 で入れる。この時点では章タイトルが全件体言止めであるため0件になる

- [ ] **Step 7: コミットする**

```bash
git add crates/suikou-core/src/headings.rs crates/suikou-core/src/glossary.rs crates/suikou-core/src/check.rs
git commit -m "feat: 見出しの検査を evaluate から呼び、defined-name を配線する"
```

---

### Task 6: 型による除外

**Files:**
- Modify: `crates/suikou-core/profiles/oss.toml`、`crates/suikou-core/profiles/service.toml`（`[doctype.howto]` と `[doctype.log]` を追加）

**Interfaces:**
- Consumes: `Profile::for_doctype` が返す `silenced`（`profile.rs:188`）

- [ ] **Step 1: 失敗するテストを書く**

`profile.rs` の `mod override_tests` に置く。

```rust
#[test]
fn heading_form_is_silenced_for_howto_and_log_in_both_profiles() {
    for p in ["oss", "service"] {
        let profile = Profile::builtin(p).unwrap();
        for name in ["howto", "log"] {
            let (_, silenced) = profile.for_doctype(name, None);
            assert!(
                silenced.contains(&"structure/heading-form".to_string()),
                "{p}/{name} で黙らせていない: {silenced:?}"
            );
        }
    }
}

#[test]
fn heading_form_stays_on_for_reference_design_and_decision() {
    for p in ["oss", "service"] {
        let profile = Profile::builtin(p).unwrap();
        for name in ["reference", "design", "decision"] {
            let (_, silenced) = profile.for_doctype(name, None);
            assert!(
                !silenced.contains(&"structure/heading-form".to_string()),
                "{p}/{name}"
            );
        }
    }
}
```

- [ ] **Step 2: 失敗を確認する**

Run: `cargo test --all heading_form_is_silenced 2>&1 | tail -10`
Expected: FAIL

- [ ] **Step 3: 両プロファイルに除外を書く**

`oss.toml` と `service.toml` の双方に `[doctype.howto]` と `[doctype.log]` を新設し、`rules = ["structure/heading-form"]` と `reason` を書く。`reason` には設計の「型による除外」の表の根拠を入れる。`howto` は `howto_ja` の発火率 25.1%、`log` は `TASKS.md` の章タイトル32件のうち27件である。`[doctype.decision]` には触らない。

- [ ] **Step 4: 通ることを確認する**

Run: `cargo test --all 2>&1 | tail -10`
Expected: PASS

Run: `cargo run -q --features lindera-unidic -- check docs/content/ja/decisions.md TASKS.md docs/content/ja/tuning.md --format json 2>&1 | grep -c heading-form`
Expected: `0`

- [ ] **Step 5: 発火率を計測する**

`crates/suikou-core/examples/structure_rate.rs` に `structure/heading-form` を加える。`development.md:75` が規則の追加時に求める手順である。

Run: `cargo run --release --features lindera-unidic --example structure_rate -- corpus/cache/pro_ja ja`
Expected: `structure/heading-form` が発火する文書の割合が 10% 前後。桁が違う場合は判定の実装を疑う

Run: `cargo run --release --features lindera-unidic --example structure_rate -- corpus/cache/howto_ja ja`
Expected: 25% 前後。設計の実測と一致すること

- [ ] **Step 6: 計測値を `TASKS.md` に記録する**

`development.md:75` が「計測値は `TASKS.md` に記録する」と定めている。`pro_ja` と `howto_ja` の発火率、および `brief` への追加を見送った旨を書く。

- [ ] **Step 7: コミットする**

```bash
git add crates/suikou-core/profiles/ crates/suikou-core/src/profile.rs crates/suikou-core/examples/structure_rate.rs TASKS.md
git commit -m "feat: 手順と記録の型で heading-form を黙らせ、発火率を計測する"
```

---


### Task 7: skills の見出しの書き直し

**Files:**
- Modify: `skills/suikou/SKILL.md`（第2階層5件）
- Modify: `skills/ja-style/SKILL.md`（第2階層7件と、見出し規約の条文 `skills/ja-style/SKILL.md:57`）

**Interfaces:**
- Consumes: Task 6 までの `structure/heading-form`

- [ ] **Step 1: 指摘を数える**

Run: `cargo run -q --features lindera-unidic -- check skills/suikou/SKILL.md skills/ja-style/SKILL.md --format text`
Expected: `structure/heading-form` が12件

- [ ] **Step 2: 見出しを体言止めの名詞句へ書き直す**

節の中身を名詞句で言い直す。指示の強さが落ちる箇所では、見出しではなく本文の先頭に動詞の文を置く。`skills/ja-style/SKILL.md:57` の条文は、文書の種類で形を決める形から階層で決める形へ書き換える。章タイトルは体言止め、第3階層以下は体言止めを既定として手順の一歩と禁止事項のときだけ用言とし、根拠として第3階層の用言率（`pro_ja` 20.9%、`howto_ja` 15.0%）を添える。

- [ ] **Step 3: 指摘が消えたことを確認する**

Run: `cargo run -q --features lindera-unidic -- check skills/suikou/SKILL.md skills/ja-style/SKILL.md --format json`
Expected: `severity` が `error` または `warning` の項目が0件。`grep 'warning'` で判定してはならない。`--format text` は severity を印字しない

- [ ] **Step 4: コミットする**

```bash
git add skills/
git commit -m "docs: スキルの章タイトルを体言止めに直す"
```

---

### Task 8: 文書と決定の記録

**Files:**
- Modify: `docs/content/ja/metrics.md:43`（指標ごとの対象範囲の表）、`:278` 以降（構造の規則）
- Modify: `docs/content/ja/metrics.md` の M3 と M6 の節（対象範囲に見出しが入ることを明記）
- Modify: `docs/content/en/metrics.md`（同じ位置）
- Modify: `docs/content/ja/decisions.md`、`docs/content/en/decisions.md`（末尾に D-52）

**Interfaces:**
- Consumes: Task 1 から 7 までの実装

- [ ] **Step 1: `metrics.md` に見出しのレイヤーの節を書く**

日本語と英語の双方に書く。内容は、見出しに当たる規則の一覧（`structure/heading-form`、`maint/numbered-heading`、`maint/time-dependent`、`glossary/defined-name`）、`structure/heading-form` の判定手順（`sections` の定義、末尾の整え方、日本語を含まない見出しの除外、最後の形態素の品詞）、型による除外とその根拠である。「指標ごとの対象範囲」の表には、文書指標と文体の規則が見出しを対象外とすることを1行で加える。

- [ ] **Step 2: D-52 を日英の `decisions.md` に書く**

記録する論点は6つである。人間と AI で分離しない値（`pro_ja` 4.8% 対 `ai_docs_ja` 6.6%）を規則の根拠にした理由と、この値を文書指標にしてはならないこと。英語に規則を作らなかった理由（`pro_en` に体言止めと述語止めの二分が無い）。`howto` と `log` を除外し `decision` を除外しなかった根拠の表。型を宣言していない文書にも当てる判断とその帰結。見出しのレイヤーを分けた理由（`maint/time-dependent` が偶然当たり、`glossary/defined-name` は未接続であった）。規則の識別子ごとに `LocalFinding` を1件に束ねる制約。節の構成は D-51 に揃える。

- [ ] **Step 3: 検査する**

Run: `cargo run -q --features lindera-unidic -- check docs/content/ja/metrics.md docs/content/ja/decisions.md docs/content/en/metrics.md docs/content/en/decisions.md --format json`
Expected: `severity` が `error` または `warning` の項目が0件

- [ ] **Step 4: 静的検査とテストを通す**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all && cargo test --all --features lindera-unidic`
Expected: すべて PASS

- [ ] **Step 5: コミットする**

```bash
git add docs/
git commit -m "docs: 見出しの検査レイヤーを文書化し D-52 を記録する"
```
