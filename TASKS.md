# TASKS.md

作業の順序と、それぞれの完了条件。上から順に進める。

## T1 コンパイルを通す（完了）

既定の feature で `cargo test --all` が通る。
`cargo fmt --all -- --check` と `cargo clippy --all-targets -- -D warnings` も通る。

行ったことを記す。

- `metrics/ja.rs` の漢語率のテストの期待値を 0.5 の厳密一致に直した。
  実装ではなく期待値を直した理由は `docs/DECISIONS.md` の D-12 にある
- `markdown.rs` の `block_before` を `rfind` に置き換えた。
  返す値は変わらない。`filter().last()` は全体を走査するため clippy が警告していた
- `rules/mod.rs` の `ItemEnding` の重複除去を `HashSet` に置き換えた
- `tokenizer.rs` の入れ子の `if` を畳み、`map_or` を `is_none_or` に置き換えた
- `cargo fmt --all` を適用した

## T2 lindera を繋ぐ（完了）

```sh
cargo test --all --features lindera-unidic
```

`verify_schema` は実辞書に対して通った。
lindera 6.0.0 と UniDic の素性の並びは、`tokenizer.rs` の定数のままでよい。
「設定」の語種が `漢`、「し」の活用形が `連用形-一般` で返ることを
`tokenizer::lindera_tests` が固定している。

着手時、`mod lindera_impl` が非公開のままで再エクスポートされておらず、
`LinderaMorphology` はクレートの外から参照できなかった。
辞書ありのビルドが通っていたのは、この型を誰も構築していなかったためである。
dead_code の警告がそれを示していた。`pub use` を足して解消した。

### 辞書の読み込み時間

```sh
cargo run --release --features lindera-unidic --example dict_load
```

| 版 | 読み込みと自己診断 | 1文の解析 |
|---|---|---|
| release | 0.68 から 0.70 ミリ秒 | 9.6 マイクロ秒 |
| debug | 6.2 ミリ秒 | 49 マイクロ秒 |

埋め込み辞書は複製を伴わずに読まれるため、起動コストは無視できる水準にある。
hooks から Write と Edit のたびに起動する用途でも、辞書の読み込みは律速にならない。
daemon モード（T9）の根拠を辞書の読み込みに置くことは、この測定では支持されない。
daemon を入れるかどうかは、プロセスの起動そのものの費用を別に測ってから決める。

## T3 ゴールデンテストを繋ぐ（M 系と文書指標は完了）

`crates/suikou-core/tests/golden_rules.rs` が M 系の件数を、
`golden_metrics.rs` が文書指標を、`tests/golden/expected/` の値と突き合わせる。
英語は辞書なしで、日本語は `--features lindera-unidic` で走る。

M 系は5つの入力すべてで整数が一致する。
文書指標は、参照実装が値を出す3文書すべてで許容範囲に収まる。

突き合わせの過程で決めたことを挙げる。

- M2 と M6 を出現ごとに数える形に変えた。根拠は `docs/DECISIONS.md` の D-13
- 語数と文数の突き合わせに相対誤差を導入した。根拠は D-14
- 形式名詞の対象範囲が参照実装と仕様で食い違っている。根拠と影響は D-15

### 残っているもの

`unescape.json` は突き合わせていない。
期待値が spaCy の文分割に基づいており、Rust に英語の文分割がないためである。
`docs/METRICS.md` は英語の文分割を spaCy に従うと定めているため、
ここを埋めるには英語の文分割を実装して定義を決め直すことになる。

突き合わせで露見したが、ゴールデンの入力が踏んでいない食い違いを挙げる。

- `research/mcheck.py` の M2 英語の正規表現には `key \w+` と `main \w+` がある。
  `docs/METRICS.md` の M2 の定義にはこれがなく、Rust 実装も持たない。
  仕様と参照実装のどちらを正本にするかを決めていない
- 参照実装は M6 を箇条書きの行では数えない。ループの構造による副産物に見える。
  Rust は全ブロックを対象とする。METRICS.md は M6 の対象範囲を定めていない

## T4 check サブコマンドを実装する

`suikou check <path>` を動くようにする。

構成は次のとおり。

- 対象ファイルを読み、`Document::parse` に渡す
- 言語を判定し、対応する指標と規則を走らせる
- プロファイルの閾値と突き合わせ、外れたものだけを `Report::document` に積む
- 規則の結果を `Report::local` に積む
- `--format` に応じて Markdown か JSON を出す
- `--quiet` は exit code だけを返す

形態素解析器の取得は `crates/suikou-cli/src/morphology.rs` にある。
辞書なしのビルドで日本語を要求された場合はここで落ちる。
`selftest` が呼び出し側になっているため、そのまま使える。

完了条件は、`tests/golden/input/ja_maintainability.md` に対して
M1 から M7 の指摘が出て、Markdown の形が `docs/DESIGN.md` の例と一致すること。

exit code の割り当ては次のとおり。

- 指摘なしは 0
- warning か info だけなら 0
- error があれば 1
- 実行に失敗したら 2

## T5 brief サブコマンドを実装する

生成前にプロンプトへ入れる制約ブロックを出力する。

内容の決め方は実測に基づく。
構造の規則は簡潔に書き、時点依存語だけは禁止語を列挙する。
項目数の明示、見出しの連番、手動番号は指示がなくても発生しないため入れない。
語彙と文構造の指標は指示で変わらないため入れない。

根拠は `docs/DECISIONS.md` の該当節にある。

`--detail` の水準は `concise` と `detailed` の二つとする。
簡潔な指示には出力を縮める副作用があるため、既定値の選択には判断が要る。
当面は `balanced` を既定とし、構造の規則を簡潔に、時点依存語を列挙する形にする。

完了条件は、出力をそのままプロンプトに貼れる形であること。

## T6 baseline サブコマンドを実装する

コーパスから閾値を較正してプロファイルを生成する。

`corpus/fetch.sh` で取得したコーパスに対して走らせ、
`corpus/baselines.toml` の値を再現できることを確認する。

完了条件は、生成したプロファイルの閾値が、
同梱のプロファイルと同じ桁に収まること。

## T7 textlint の局所ルールを実装する

`packages/textlint-rule-preset-tech-maintainability` に M1 から M7 を実装する。

Rust 側と同じ判定を JS で書き直すことになる。
判定の定義は `docs/METRICS.md` にある。

既存のプリセットとの重複に注意する。
`@textlint-ja/textlint-rule-preset-ai-writing` の `no-ai-colon-continuation` と
M1 は目的が異なるが、同じ箇所を二重に指摘する場合がある。

完了条件は、`tests/golden/input/` の各ファイルに対して
Rust 実装と同じ件数の指摘が出ること。

## T8 terms サブコマンドを実装する

文書群から分野の用語を抽出して `.suikou/terms.toml` に書く。

比喩の層がこれを使う。terms に載る語は確立した用語として扱い、
比喩の判定から除外する。

辞書を同梱せず文書から作るのは、
プロジェクトごとに用語が違うためであり、
同時に辞書が古びる問題への回答でもある。

## T9 MCP サーバと daemon

`check` と `brief` をツールとして公開する。
daemon は辞書の読み込みを一度に抑えるためのもので、
hooks から高頻度で起動する用途で要る。

WSL2 での動作確認が必要になる。

## T10 リリースと配布（ワークフローは整備済み。公開は未実施）

`.github/workflows/release.yml` がタグ `v*` の push で動く。
検証、3ターゲットのビルド、梱包、下書きのリリース作成までを行う。
公開は下書きを見てから手で行う。

サブコマンドが `todo!()` のうちはタグを打たない。
今タグを打つと、起動して即座に落ちるバイナリが配られる。

手動実行でもビルドと梱包までは走るため、ワークフロー自体の検証は先にできる。

### 手元で確認した範囲

`act` で `workflow_dispatch` を走らせ、`verify` と musl の `build` を通した。
macOS と Windows のランナーは act では動かせないため、この二つは未検証のままである。

初回の実行で、梱包されたバイナリに辞書が入っていないことが分かった。
経緯と対処は `docs/DECISIONS.md` の D-18 にある。
`act` で回していなければ、辞書のない配布物がスモークテストを通っていた。

既存の `ci.yml` も `act` で通した。rust と python の両ジョブが成功し、
参照実装が `rules.json` の期待値を再現することも確認できている。

### GitHub でしか分からないこと

- `aarch64-apple-darwin` のビルドと `tar -cJf` の可用性
- `x86_64-pc-windows-msvc` のビルドと `7z` の可用性
- `actions/upload-artifact` と `gh release create` の実際の挙動

## 保留している判断

実装と評価を経てから決める。

- `brief` の出力をスキルが毎回呼ぶか、SKILL.md に静的に埋めるか
- 閾値をジャンル別に分ける粒度。ジャンルをどう判定するかを含む
- M2 の適用範囲。リストが後続しない純粋な説明文で数を述べる場合も検出される。
  `tests/golden/input/ja_prose.md` がその例である
- 日本語の人間コーパスの補強。現状は解説書に偏っており、手順書型が足りない。
  形式名詞と漢語率の severity を上げられるかがこれにかかっている
