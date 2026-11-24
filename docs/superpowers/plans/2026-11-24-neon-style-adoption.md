# neon 文体の採用 実装計画

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** neon の文体（7d07fc5）をこのリポジトリの文書の基準として採用し、日本語文書の error と warning をゼロ件にする。

**Architecture:** 3層に分けて進める。偽陽性はルールの実装とデータで是正する。文体の採用は `.suikou/profile.toml` のプロジェクト単位の閾値で行い、`oss.toml` の既定には触れない。較正では解消できない `ja.demonstrative_per_1k` は、閾値を保ったまま文書を直す。

**Tech Stack:** Rust（`crates/suikou-core`）、TOML の設定とデータ、Markdown の文書。

**Spec:** `docs/superpowers/specs/2026-11-24-neon-style-adoption-design.md`

## Global Constraints

- `crates/suikou-core/profiles/oss.toml` を変更してはならない。利用者向けの既定の閾値であり、この採用の非目標である。
- 英語のルールと閾値を変更してはならない。
- `corpus/baselines.toml` の `[rejected]` に載る指標を導入してはならない。`ja.gairaigo_ratio` が「人間と重なる」として登録済みである。
- 検査結果の判定に終了コードを使ってはならない。`--quiet` は error でのみ 1 を返す。件数で確認する。
- 合格基準は error と warning がゼロ件であること。info は参考値として残してよい。
- 日本語の計測には辞書が必要である。検証用のバイナリは `cargo build --release --features lindera-unidic` で作る。
- `cargo test --all`、`cargo test --all --features lindera-unidic`、`cargo fmt --all`、`cargo clippy --all-targets -- -D warnings` のすべてを通す。
- 緩和の経緯は `reason` と決定記録の両方に残す。独断で基準を緩和してはならない。

## Review Focus

- **短い文書における `ja.demonstrative_per_1k`。** `docs/content/ja/_index.md` の本文は 561字で指示詞2件であり、1件の増減で指標が 1.78 動く。短い文書ではこの指標が雑音になる。Task 5 でこの不安定さを記録する。
- **`.suikou/profile.toml` の指標名の誤記。** `Profile::apply` は知らない指標名をそのまま足す仕様である（`profile.rs:88-92` の註釈に明記）。`ja.renyo_per_1k` を `ja.renyo_per1k` と書いても、エラーにならず既存の閾値が緩まないまま新しい閾値が増える。Task 3 でこの経路を試験する。
- **`reason` を書かないプロジェクト単位の緩和。** `reason` を足したあと、空のまま緩和できてしまうと CLAUDE.md の要求が形骸化する。一方で強制すると既存の利用者の設定が読めなくなる。Task 3 で空を許す側に決め、試験で固定する。要求はツールではなく CLAUDE.md が担う。
- **カタカナ表層のガードによる検出漏れ。** 和語をカタカナで書いた語（「コト」「ハズ」のような強調）が和語の判定から外れる。`register/rare-wago` の検出力が落ちる方向である。Task 2 の試験はこの取引を受け入れる側を固定する。カタカナの和語は技術文書にほぼ現れず、未知語の誤解析を通す損のほうが大きい。
- **allow リストが本物の問題を隠すこと。** `検査` を除外すると、`テスト` が正しい文脈の `検査` も黙る。Task 1 で、除外した語が別の規則にも影響しないことを確認する。

---

### Task 1: 偽陽性の是正 — allow リストへの追記

**Files:**
- Modify: `.suikou/register-allow.toml`

**Interfaces:**
- Consumes: なし。
- Produces: なし。`crates/suikou-cli/src/check.rs:38` が読む `words` 配列へ追記するだけである。この配列は `register/rare-wago` と `register/prefer-katakana` の両方の `allow` に渡る。

- [ ] **Step 1: 現在の指摘件数を記録する**

Run:
```sh
cargo build --release --features lindera-unidic
for f in CLAUDE.md README.ja.md docs/content/ja/*.md; do
  ./target/release/suikou check "$f" --format text | grep -c 'register/'
done
```
Expected: 合計 54 件（`register/prefer-katakana` 40 件、`register/rare-wago` 14 件）。この数を控える。

- [ ] **Step 2: カタカナ対応表から外す語を追記する**

`words` 配列の「対応表（カタカナ語の優先）から外す語」の節へ、註釈つきで追記する。各行の註釈は、その語をこの文書群で別の意味に使っている理由を書く。

| 語 | 註釈に書く理由 |
|---|---|
| `検査` | lint の検査を指す。`テスト` は `cargo test` を指す |
| `事例` | 技術文書で定着している |
| `保守` | メンテナンスと併用する |
| `書式` | 出力の書式を指す |
| `層` | レイヤーと併用する |
| `走査` | 文字列の走査を指す |

- [ ] **Step 3: 和語の指摘から外す語を追記する**

`words` 配列の前半へ、註釈つきで追記する。`偏る` は登録済みだが名詞形の `偏り` が漏れているため、両方を並べる。

| 語 | 註釈に書く用例 |
|---|---|
| `定める` | 規約が定める |
| `満つ` | 4件に満たない。`満たない` の終止形 |
| `偏り` | 分布の偏り。`偏る` の名詞形 |
| `食い違う` | 期待値と定義が食い違う |
| `絞り込む` | 条件で候補を絞り込む |
| `上がる` | 測定値が上がる |
| `改める` | 名前を改める |
| `揃う` | 標本が揃う |
| `揃える` | 表記を揃える |

- [ ] **Step 4: 件数が減ったことを確認する**

Run:
```sh
for f in CLAUDE.md README.ja.md docs/content/ja/*.md; do
  ./target/release/suikou check "$f" --format text | grep 'register/'
done
```
Expected: `検査→テスト`、`事例→ケース`、`保守→メンテナンス`、`書式→フォーマット`、`層→レイヤー`、`走査→スキャン`、および Step 3 の9語が消える。`トレイト` を `糸` と解析する3件（`docs/content/ja/development.md`）だけが残る。この3件は Task 2 で扱う。

- [ ] **Step 5: 除外が他の規則に波及していないことを確認する**

Run: `./target/release/suikou check docs/content/ja/metrics.md --format text | grep -c ':[0-9]*:[0-9]*:'`
Expected: `register/` 以外の指摘件数が Step 1 の時点と変わらない。

- [ ] **Step 6: Commit**

```bash
git add .suikou/register-allow.toml
git commit -m "fix: この文書群で別の意味に使う語を語彙の指摘から外す"
```

---

### Task 2: カタカナ表層を和語の判定から外す

**Files:**
- Modify: `crates/suikou-core/src/register.rs:225-234`（`is_target`）
- Test: `crates/suikou-core/src/register.rs` の `mod tests`

**Interfaces:**
- Consumes: なし。
- Produces: `is_target(pos1: &str, pos2: &str, goshu: &str, surface: &str) -> bool` の判定を変える。シグネチャは変えない。

**背景:** lindera は `トレイト` を `トレ` と `イト` に分割し、`イト` の語種を `和`、語彙素を `糸` として返す。既存のガードは1文字のカタカナだけを除いているため、2文字の `イト` が通る。`docs/content/ja/development.md` でこれが3件の誤検出になる。再現は次のとおりである。

```sh
printf 'トレイトを使う。トレイトを定義する。トレイトを実装する。トレイトの境界を書く。トレイトは便利である。トレイトを経由する。\n' > /tmp/t.md
./target/release/suikou check /tmp/t.md --format text --lang ja
```

- [ ] **Step 1: 失敗する試験を書く**

`mod tests` へ追記する。既存の `numerals_are_not_reported` と同じ形にする。

```rust
#[test]
fn a_katakana_surface_is_not_reported_as_wago() {
    // lindera は「トレイト」を「トレ」「イト」に割り、「イト」の語種を「和」、
    // 語彙素を「糸」として返す。未知のカタカナ語に対する解析の破綻である。
    let m = FakeMorphology::from_spec("イト|名詞|普通名詞||和 を|助詞|格助詞||和 使う|動詞|一般|終止形-一般|和");
    let d = Document::parse("イトを使う。\n\nイトを使う。\n\nイトを使う。\n");
    assert!(check(&d, &m, &reference(), &HashSet::new()).is_none());
}
```

- [ ] **Step 2: 試験が失敗することを確認する**

Run: `cargo test -p suikou-core a_katakana_surface_is_not_reported_as_wago`
Expected: FAIL。`check` が指摘を返すため `is_none()` が成り立たない。

- [ ] **Step 3: `is_target` のガードを一般化する**

1文字のカタカナに限っていた判定を、表層がすべてカタカナ（長音符を含む）の語に広げる。既存の註釈を、1文字の断片だけでなく未知のカタカナ語の誤解析も除く理由に書き換える。

- [ ] **Step 4: 試験が通ることを確認する**

Run: `cargo test -p suikou-core register`
Expected: PASS。既存の `register` の試験もすべて通る。

- [ ] **Step 5: 実文書で3件が消えたことを確認する**

Run:
```sh
cargo build --release --features lindera-unidic
./target/release/suikou check docs/content/ja/development.md --format text | grep -c 'register/'
```
Expected: 0。

- [ ] **Step 6: 全体の検査を通す**

Run: `cargo test --all && cargo test --all --features lindera-unidic && cargo fmt --all && cargo clippy --all-targets -- -D warnings`
Expected: すべて成功。ゴールデンテストの期待値は M1〜M7 と文書指標だけを持ち、語彙の規則を含まないため変更は不要である。

- [ ] **Step 7: Commit**

```bash
git add crates/suikou-core/src/register.rs
git commit -m "fix: カタカナ表層の語を和語の判定から外す"
```

---

### Task 3: プロジェクト単位の緩和に `reason` を持たせる

**Files:**
- Modify: `crates/suikou-core/src/profile.rs:61-74`（`ProfileOverride`）
- Test: `crates/suikou-core/src/profile.rs` の `mod override_tests`（`profile.rs:195`。`mod tests` ではない）

**Interfaces:**
- Consumes: なし。
- Produces: `ProfileOverride` に `pub reason: String` を足す。`#[serde(default)]` を付け、既存の `.suikou/profile.toml` を持たない利用者が壊れないようにする。`DoctypeAdjust::reason` と同じ形にする。

**背景:** `ProfileOverride` は `disable`、`thresholds`、`doctype` の3つだけを持ち、`reason` を持たない。serde は既定で未知の欄を黙って捨てるため、`reason` を書いても記録されない。CLAUDE.md は緩和の経緯を `reason` に残すことを求めており、黙って捨てられる欄は要求を形骸化させる。

- [ ] **Step 1: 失敗する試験を書く**

```rust
#[test]
fn a_project_override_carries_its_reason() {
    let ov: ProfileOverride = toml::from_str(
        "reason = \"neon の文体を採用した。経緯は D-51 にある\"\n\
         [thresholds.\"ja.renyo_per_1k\"]\nvalue = 9.36\n",
    )
    .expect("読めていない");
    assert_eq!(ov.reason, "neon の文体を採用した。経緯は D-51 にある");
}

#[test]
fn an_override_without_a_reason_still_loads() {
    // 既存の利用者は reason を書いていない。欄を足しても壊してはならない。
    // 空のまま緩和できることは承知の上で許す。強制すると既存の設定が読めなくなる。
    let ov: ProfileOverride =
        toml::from_str("[thresholds.\"ja.renyo_per_1k\"]\nvalue = 9.36\n").expect("読めていない");
    assert!(ov.reason.is_empty());
}

#[test]
fn an_unknown_metric_name_is_added_rather_than_ignored() {
    // 指標名を誤記しても既存の閾値は緩まない。誤記が増えるだけである。
    let mut p = Profile::builtin("oss").expect("プロファイルが無い");
    let before = p.thresholds["ja.renyo_per_1k"].value;
    let ov: ProfileOverride = toml::from_str(
        "[thresholds.\"ja.renyo_per1k\"]\nvalue = 9.36\ndirection = \"above\"\n",
    )
    .expect("読めていない");
    p.apply(&ov).expect("重ねられていない");
    assert_eq!(p.thresholds["ja.renyo_per_1k"].value, before);
    assert!(p.thresholds.contains_key("ja.renyo_per1k"));
}
```

- [ ] **Step 2: 試験が失敗することを確認する**

Run: `cargo test -p suikou-core profile`
Expected: FAIL。`ProfileOverride` に `reason` が無いためコンパイルが通らない。

- [ ] **Step 3: `ProfileOverride` に `reason` を足す**

`DoctypeAdjust::reason` と同じ形にする。欄の註釈に、プロジェクト単位の緩和がツールの既定を置き換えるものではないことを書き残す理由を記す。

- [ ] **Step 4: 試験が通ることを確認する**

Run: `cargo test -p suikou-core profile`
Expected: PASS。

- [ ] **Step 5: 全体の検査を通す**

Run: `cargo test --all && cargo fmt --all && cargo clippy --all-targets -- -D warnings`
Expected: すべて成功。

- [ ] **Step 6: Commit**

```bash
git add crates/suikou-core/src/profile.rs
git commit -m "feat: プロジェクト単位の閾値の緩和に reason を持たせる"
```

---

### Task 4: `.suikou/profile.toml` に較正した閾値を置く

**Files:**
- Create: `.suikou/profile.toml`

**Interfaces:**
- Consumes: Task 3 の `ProfileOverride::reason`。
- Produces: なし。

- [ ] **Step 1: 較正をやり直して値を確かめる**

Run:
```sh
./target/release/suikou baseline CLAUDE.md README.ja.md docs/content/ja/*.md --lang ja --name suikou-docs
```
Expected: 次の6件が出る。`ja.demonstrative_per_1k` と `ja.keishiki_meishi_per_1k` は「較正した境界が値域の外に出て、この閾値は永久に発火しない。較正しない」として拒否される。これは正しい挙動であり、この2件は緩和の対象としない。

| 指標 | 既定 | 較正値 |
|---|---|---|
| ja.renyo_te_ratio | 3.0 | 23.21 |
| ja.renyo_per_1k | 4.0 | 9.36 |
| ja.mattr100 | 0.60 | 0.640 |
| ja.kango_ratio | 0.52 | 0.590 |
| ja.list_ratio | 0.25 | 0.283 |
| ja.prose_ratio | 0.75 | 0.767 |

- [ ] **Step 2: `.suikou/profile.toml` を書く**

`reason` に次の4点を書く。較正の標本がこのリポジトリの日本語文書9件であること、`baseline.rs` が4件未満を拒否する条件を満たすこと、これが `oss.toml` の既定を置き換えるものではないこと、経緯が D-51 にあること。

`[thresholds]` には上の6件の `value` だけを書く。`direction`、`severity`、`guidance` は組み込みのままにする。`ThresholdPatch` は書いた欄だけを差し替える仕様であり、写すと組み込みの変更に追従しなくなる。

ファイル冒頭の註釈に、`ja.demonstrative_per_1k` を緩和しない理由（較正が拒否された）と、代わりに文書を直す方針を書く。

- [ ] **Step 3: 文書の警告が消えたことを確認する**

Run:
```sh
for f in CLAUDE.md README.ja.md docs/content/ja/*.md; do
  echo "--- $f"
  ./target/release/suikou check "$f" --format text | grep -E 'warning|error'
done
```
Expected: `ja.demonstrative_per_1k` の warning が `docs/content/ja/_index.md`（3.565）、`docs/content/ja/design.md`（3.317）、`docs/content/ja/tuning.md`（3.433）の3件だけ残る。他の warning と error はゼロ件。`ja.keishiki_meishi_per_1k` の info は残ってよい。

- [ ] **Step 4: 英語側が変わっていないことを確認する**

Run:
```sh
for f in README.md docs/content/en/*.md; do
  ./target/release/suikou check "$f" --format text | grep -c ':[0-9]*:[0-9]*:'
done
```
Expected: 変更前と同じ件数。`.suikou/profile.toml` は `ja.` の指標だけを書いており、英語に影響しない。

- [ ] **Step 5: Commit**

```bash
git add .suikou/profile.toml
git commit -m "feat: このリポジトリの文書に neon の文体の閾値を当てる"
```

---

### Task 5: 指示詞を補う

**Files:**
- Modify: `docs/content/ja/_index.md`
- Modify: `docs/content/ja/design.md`
- Modify: `docs/content/ja/tuning.md`

**Interfaces:**
- Consumes: Task 4 の `.suikou/profile.toml`。
- Produces: なし。

**背景:** `ja.demonstrative_per_1k` は緩和の対象としない。neon の書き直しはこの指標を中央 5.72 へ改善し、人間ネイティブの範囲 5.69–6.47 に入れている。文書を直す方向が採用した文体に反しない。対象は `DEMONSTRATIVES`（`metrics/ja.rs:12`）の `これ`、`それ`、`この`、`その`、`これら`、`それら`、`こちら`、`そちら`、`ここ`、`そこ` である。閾値は 4.5 で、指摘は本文1000字あたりの件数で出る。

| 文書 | 本文の字数 | 現在の指示詞 | 現在値 | 4.5 に要る追加 |
|---|---|---|---|---|
| `docs/content/ja/_index.md` | 561 | 2 | 3.565 | 1 |
| `docs/content/ja/design.md` | 9,044 | 30 | 3.317 | 11 |
| `docs/content/ja/tuning.md` | 2,330 | 8 | 3.433 | 3 |

- [ ] **Step 1: 名詞の再掲を指示詞へ置き換える**

既定の `guidance` は「段落間の接続に指示詞を使い、名詞の再掲を減らす」である。語を足すのではなく、段落の冒頭で直前の段落の主題を名詞で繰り返している箇所を指示詞に替える。文体の規約（主題を主語にする、一段落に一つの主張）を崩さない。

- [ ] **Step 2: 3件とも閾値に収まったことを確認する**

Run:
```sh
for f in docs/content/ja/_index.md docs/content/ja/design.md docs/content/ja/tuning.md; do
  echo "--- $f"
  ./target/release/suikou check "$f" --format text | grep -E 'warning|error'
done
```
Expected: 出力なし。

- [ ] **Step 3: 日本語文書の全体が合格することを確認する**

Run:
```sh
for f in CLAUDE.md README.ja.md docs/content/ja/*.md; do
  n=$(./target/release/suikou check "$f" --format text | grep -cE 'warning|error')
  echo "$n $f"
done
```
Expected: すべて 0。

- [ ] **Step 4: 短い文書での不安定さを記録する**

`TASKS.md` に、`ja.demonstrative_per_1k` が短い文書で雑音になる件を追記する。`docs/content/ja/_index.md` の本文は 561字で指示詞2件であり、1件の増減で指標が 1.78 動く。本文の字数に下限を設けるかどうかは未検討であることを書く。実測の裏付けがないため、ここでは規則を変えない。

- [ ] **Step 5: Commit**

```bash
git add docs/content/ja/_index.md docs/content/ja/design.md docs/content/ja/tuning.md TASKS.md
git commit -m "docs: 名詞の再掲を指示詞に替える"
```

---

### Task 6: 決定記録に D-51 を追記する

**Files:**
- Modify: `docs/content/ja/decisions.md`
- Modify: `docs/content/en/decisions.md`

**Interfaces:**
- Consumes: Task 4 の較正値。
- Produces: なし。

- [ ] **Step 1: 日本語の D-51 を書く**

既存の D 項目の形に合わせる。`## D-51 <見出し>` の次に `### 文脈`、`### 決定` を置く。D-50 の直後に足す。neon の文体で書く（見出しは体言止めの名詞句、段落は1行、冒頭は定義文）。

内容に次を含める。採用した文体とその出所（7d07fc5）。人間コーパスと AI コーパスの実測範囲に対する neon の位置、特に `ja.renyo_te_ratio` が AI 範囲の中央に入り `ja.kango_ratio` が AI 範囲を超えること。既定を引き直す案を採らなかった理由（AI 4モデルの実測範囲が丸ごと閾値の下に収まり、2つの指標が検出力を失う）。緩和の適用範囲がこのリポジトリの文書に限られること。`ja.demonstrative_per_1k` を緩和せず文書を直した理由。段落単位の計測を別の案件として切り分けたこと。

- [ ] **Step 2: 英語の D-51 を書く**

`docs/content/en/decisions.md` に同じ内容を英語で足す。英語側は書き直しによって局所指摘が25件から1件へ減っており、英語のルールを変更しないことを明記する。

- [ ] **Step 3: 決定記録が合格することを確認する**

Run:
```sh
./target/release/suikou check docs/content/ja/decisions.md docs/content/en/decisions.md --format text | grep -cE 'warning|error'
```
Expected: 0。

- [ ] **Step 4: 残った案件を `TASKS.md` に書く**

段落単位の `kango_ratio` を局所指摘として追加する案を追記する。実測は閾値 0.64 で書き直し前 3.6% に対し neon 15.5% であり、4.3倍に分かれる。指す先は D-27 の「漢語の名詞を重ねない」である。`ja.renyo_te_ratio` は `MIN_TE_FOR_RATIO` の門により562段落のうち38段落しか値が出ず段落単位では成立しないこと、`ja.renyo_per_1k` は超過が段落の半数に散るため位置を指せないことも書く。

- [ ] **Step 5: 全体の検査を通す**

Run: `cargo test --all && cargo test --all --features lindera-unidic && cargo fmt --all && cargo clippy --all-targets -- -D warnings`
Expected: すべて成功。

- [ ] **Step 6: Commit**

```bash
git add docs/content/ja/decisions.md docs/content/en/decisions.md TASKS.md
git commit -m "docs: neon の文体の採用と閾値の緩和を D-51 に記録する"
```
