# suikou

技術文書の lint とハーネス。日本語と英語に対応する。

生成AIが書いた技術文書には、単語の選び方にとどまらない特徴がある。
比喩の多用、文末表現の偏り、接続のこなれなさ、
そして必要以上に具体的で保守しにくい構造がこれにあたる。
suikou はこれらを検出し、同時に文書の保守性を高める。

## 設計の要点

### 規則はすべて実測か明文の規範に基づく

直感で決めた規則は入れていない。
直感に基づく文体特徴は、実証すると大半が外れる。
翻訳調の研究とAI文の研究という独立した二つの領域で、同じことが確認されている。

閾値の根拠は `corpus/baselines.toml` にある。
採用しなかった指標と、その理由も記録してある。

### 検出対象は二つに分かれる

行や文の単位で真偽が決まり位置を持つものと、
文書全体でしか計算できず位置を持たないものがある。
後者の多くは欠落の指摘であり、原理的に位置がない。

この分割が「一度の実行で、一度の修正で済ませる」という要件の実装形になっている。
局所指摘がどこを直すかを伝え、文書指標から合成した方針がどう直すかを伝える。

### 保守性の層は AI 臭さと独立している

M1 から M7 は Google developer documentation style guide の明文規定に基づく。
AI 以前から存在する問題であり、モデルの世代交代でも陳腐化しない。
この層だけを単体で使うこともできる。

## 文書

| 文書 | 内容 |
|---|---|
| `CLAUDE.md` | 作業を始めるときに最初に読む。守るべき制約 |
| `TASKS.md` | 作業の順序と完了条件 |
| `docs/DESIGN.md` | 設計の全体。実測値と根拠を含む |
| `docs/METRICS.md` | 指標の厳密な定義。移植に使う |
| `docs/DECISIONS.md` | 決定の経緯 |

## 構成

| 場所 | 内容 |
|---|---|
| `crates/suikou-core` | 解析の中核 |
| `crates/suikou-cli` | CLI と MCP サーバ |
| `packages/` | textlint の局所ルール |
| `skills/suikou` | エージェント向けのスキル |
| `research/` | 較正と探索に使う Python 実装 |
| `corpus/` | ベースラインの実測値と取得スクリプト |

## インストール

### ビルド済みバイナリ

GitHub Releases に OS ごとのアーカイブを置いてある。
辞書を同梱してあるため、取得したあとはネットワークを必要としない。

| OS | アーカイブ |
|---|---|
| Linux (x86_64) | `suikou-v<version>-x86_64-unknown-linux-musl.tar.xz` |
| macOS (Apple Silicon) | `suikou-v<version>-aarch64-apple-darwin.tar.xz` |
| Windows (x86_64) | `suikou-v<version>-x86_64-pc-windows-msvc.zip` |

Linux は musl で静的にリンクしてあるため、glibc の版に依存せずに動く。

```sh
tar xf suikou-v<version>-<target>.tar.xz
cd suikou-v<version>-<target>
install -m 755 suikou ~/.local/bin/
```

Windows では zip を展開し、`suikou.exe` を PATH の通った場所に置く。

取得したファイルは同じリリースにある `SHA256SUMS` で検証できる。

```sh
sha256sum -c SHA256SUMS --ignore-missing
```

入ったバイナリが期待どおりかは `suikou selftest` で確かめられる。
辞書を実際に読んで解析させるため、辞書の欠けた版を掴んでいれば分かる。

```sh
suikou selftest
```

アーカイブには `SKILL.md` を入れてある。
エージェントから使う場合は、これをスキルとして取り込む。

### ソースから入れる

```sh
cargo install --git https://github.com/KazuyoshiAkiyama/suikou suikou-cli --features lindera-unidic
```

`--features lindera-unidic` を省くと辞書を含まない版が入る。
その版は英語だけを扱い、日本語を指定すると明示的に落ちる。
黙って空の解析結果を返すと、漢語率も連用中止も 0 になり「指摘なし」という誤った結論が出るためである。

どちらの版が入っているかは `suikou --version` で判別できる。
辞書の有無でバイナリの大きさが変わる。同梱版は約190MB、辞書なしは約1MBである。

## 使い方

```sh
# 書き始める前に
suikou brief --profile oss --lang ja

# 書いた後に
suikou check docs/guide.md

# 自前のコーパスから閾値を作り直す
suikou baseline 'docs/**/*.md'
```

## 開発

```sh
cargo test --all          # 形態素解析なしで本体のロジックを検証する
cargo test --all --features lindera-unidic
```

既定の feature には lindera を含めていない。
辞書を落とさずに本体のテストが回るため、変更の検証が速い。

### バージョンの固定

依存は `=` で厳密に固定してある。`Cargo.lock` もコミットする。

lindera は特に厳密に扱う。API の形と UniDic の素性の並びがバージョンで変わり、
並びがずれると語種と活用形が静かに壊れるためである。
漢語率と連用中止がこの二つに依存しているため、
壊れた値のまま動くと、指摘の内容そのものが誤りになる。

### 形態素解析の扱い

バージョンに依存するコードは `crates/suikou-core/src/tokenizer.rs` の
`LinderaMorphology` だけに閉じ込めてある。
ほかのモジュールは `Morphology` トレイト越しにしか形態素解析を使わない。
テストでは `FakeMorphology` を使うため、辞書がなくてもロジックを検証できる。

`LinderaMorphology::new` は起動時に `verify_schema` を呼ぶ。
既知の語を流して、品詞、語種、活用形が想定の位置から取れることを確かめる。
ずれていれば、どの定数を直すべきかを示して落ちる。
黙って誤った値を返すより落ちる方がよい。

### 指標の仕様とゴールデンテスト

指標の厳密な定義は `docs/METRICS.md` にある。
`research/` の Python 実装が参照実装であり、
`tests/golden/` がその出力を固定している。

## 状態

解析の中核まで。CLI はまだ動かない。
test、fmt、clippy が既定の feature でも `--features lindera-unidic` でも通る。
ゴールデンテストが、M 系の件数と文書指標を `research/` の参照実装と突き合わせている。

| 部分 | 状態 |
|---|---|
| ブロック抽出と前処理 | 実装済み。単体テストあり |
| M1 から M7 | 実装済み。単体テストあり |
| 日本語の文書指標 | 実装済み。`Morphology` 越しに動く |
| 英語の文書指標 | 実装済み |
| 出力スキーマと Markdown 整形 | 実装済み |
| lindera の接続 | 実装済み。lindera 6.0.0 で検証済み |
| ゴールデンテスト | M 系と文書指標は接続済み |
| CLI の各サブコマンド | 未実装。`main.rs` の `todo!` |
| textlint の局所ルール | 未実装 |
