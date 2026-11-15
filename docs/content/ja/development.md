---
title: "開発"
weight: 40
doctype: "reference"
---

作り方、バージョンの固定方、形態素解析の扱い、そしてゴールデンテストについて書く。
これらはいずれも、このリポジトリで作業するときに遵守する事項である。

## 置き場

| 場所 | 内容 |
|---|---|
| `crates/suikou-core` | 解析の中核 |
| `crates/suikou-cli` | CLI と MCP サーバー |
| `packages/` | textlint の局所ルール |
| `skills/suikou` | エージェント向けのスキル |
| `research/` | 較正と探索に使う Python |
| `corpus/` | もとになった値と、取ってくるスクリプト |

## 作りとテスト

本体を確認するときは、次のコマンドを使う。

```sh
cargo test --all                           # 形態素解析なし。速い。まずこれ
cargo test --all --features lindera-unidic # 辞書を含む。遅い
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

既定の feature に lindera を入れていない。
そのため辞書を取得せずに本体を確認できて、修正したところを確認するのが速い。

配布用のビルドでは、その feature を書くことになる。

```sh
cargo build --release -p suikou-cli --features lindera-unidic
```

辞書を含まないバージョンは英語だけを扱うものである。
そのバージョンに日本語を求めた場合は、空の解析結果を返さずにその場で異常終了する。
どちらのバージョンがこの環境にあるかは `suikou selftest` で確認できる。

## バージョンを固定する

バージョンを固定することには理由がある。その理由をここに書く。

依存はすべて `=` で固定して、`Cargo.lock` も置くことにする。
そのバージョンは crates.io で実在を確認してから書く。

lindera はいちばん厳密に扱うことにする。
API の形と UniDic の素性の順序が、バージョンで変わるためである。
その順序が一致しなくなると、語種と活用形が気付かれずに壊れる。
漢語率と連用中止がこの二つに依存しているため、
壊れた値のまま動くと、指摘の内容そのものが誤りになる。

lindera を更新した場合は、`LinderaMorphology::verify_schema` を必ず通す。
知っている文を流して、品詞と語種と活用形が思った位置から取れることを確認する。
一致せずいた場合は、どの定数を直せばよいかを示して異常終了する。
気付かれずに誤った値を返すよりも、異常終了する方がよいためである。

## 形態素解析の扱い

形態素解析は、バージョンによって振る舞いが変わるものである。

バージョンに依存するコードは、`crates/suikou-core/src/tokenizer.rs` の
`LinderaMorphology` だけに置いてある。
ほかのモジュールは `Morphology` トレイト越しにしか形態素解析を使わない。
テストでは `FakeMorphology` を使うため、辞書がない場合でも確認できる。

指標を追加するときも、この境を越えない。
越えると、辞書なしでテストが回らなくなる。

測定した値では、埋め込んだ辞書の読み込みが release で 0.7 ミリ秒だった。
1文の解析はそのうえで 9.6 マイクロ秒だった。

```sh
cargo run --release --features lindera-unidic --example dict_load
```

## 指標の対象範囲を混ぜない

語彙の指標（MATTR、漢字率、漢語率、指示詞、形式名詞）は `Document::body()` を使う。
文の構成の指標（文長、読点、連用中止、テ形接続）は `Document::prose()` を使う。

語彙の指標を地の文だけで測定してはならない。
出力の大半が箇条書きになるモデルがあるためである。
その場合、本文の8割以上を破棄することになる。

## 前処理の順序を変えない

`markdown.rs` の `preprocess` は順序に意味がある。
バックスラッシュのエスケープを戻す処理を後ろへ動かすと、文分割が成り立たない。
AWS の doc_source がこの形式であり、戻さない場合の平均文長は3倍以上にふくらむ。

`tests/golden/input/en_escaped.md` がこの振る舞いを固定している。

取り除いた範囲は、同じ数の改行に置き換えることにする。
そうすることで、指摘が使う人の開いている行を指すようになる。
値が正しくても、位置が一致せずいればその指摘は役に立たない。

## 発火率を測る

規則を追加したり閾値を変更したりした場合は、参照コーパスに対する発火率を計測する。

```sh
cargo run --release --example structure_rate -- corpus/cache/pro_en en
cargo run --release --example structure_rate -- corpus/cache/pro_ja ja
```

`suikou check` は規則ごとに示す位置を打ち切るため、CLI の出力からは件数を数えられない。
この例は規則を直に呼ぶ。計測した値は `TASKS.md` に記録する。

コーパスは `corpus/fetch.sh` で取得する。ライセンスの都合でリポジトリには入れない。

必須の節を計測する場合は、型の宣言を持つコーパスが要る。

```sh
python research/structure/fetch_typed.py corpus/cache/typed 120
```

Kubernetes の `content_type:` ごとに頁を集める。
`.suikou/structure.toml` の `[front_matter]` でこちらの型に読み替えてから `check` を当てる。
その設定は `research/structure/structure.toml` にある。

レポートの字数は、規則を変更したときに前後で比べる。
`git worktree` で変更前のバージョンをビルドし、同じコーパスに当てて字数を合計する。

`design` の型は Rust RFC で計測する。

```sh
python research/structure/fetch_rfcs.py corpus/cache/rfcs 200
```

型の出どころそのものに当てるため、最も厳しい確認になる。

`decision` の型は公開されている ADR で計測する。

```sh
python research/structure/fetch_adrs.py corpus/cache/adrs 200 12
```

設定は `research/structure/structure-adr.toml` にある。

## ゴールデンテスト

ゴールデンテストは、参照の実装と値を統一するためのものである。

`crates/suikou-core/tests/golden_rules.rs` が規則の件数を、
`golden_metrics.rs` が文書指標を、`tests/golden/expected/` の値と照合する。
英語は辞書なしで走り、日本語は `--features lindera-unidic` で実行される。

lindera と fugashi は別のトークナイザであるため、照合には許容範囲を置く。
比率は絶対の誤差 0.02、1000文字あたりの密度は 0.5、M 系の規則は完全一致を求める。
語数と文数はトークナイザの定義に依存するため、相対の誤差 1% とする。

幅を超えた指標が出た場合は、どちらの実装が正しいかをそのつど決める。
そしてその根拠を決定の記録に残すことにする。Python の側が常に正しいとは限らない。

## 直感で規則を足さない

**これがこのリポジトリでいちばん重い制約である。**

文体の特徴について直感で立てた仮説は、確認すると大半は成立しない。
このプロジェクトでも、設計の途中で立てた仮説の多くが測定した値で落ちている。
翻訳調の研究でも同じことが起きている。

規則を追加するときは、次のいずれかの根拠を示す。

- 人間のコーパスと AI の出力の測定した値。その幅が重ならないこと。
- 明文のスタイルガイドの規約。その出典を示すこと。

`corpus/baselines.toml` の `[rejected]` に、除外した指標とその理由が書いてある。
**ここに載っている指標を導入してはならない。**
文長の変動係数、外来語率、英語の分詞節と名詞化がこれにあたる。
いずれも「AI らしさの指標」として広く言われているものだが、測定すると人間の範囲と重なる。

## トークンの出る量を増やさない

トークンの出る量を抑えることには、次のような制約がある。

レポートの形には制約がある。

- ルールの説明はルールにつき一度だけ出力する。行ごとに繰り返さない。
- ひとつのルールにつき示す位置は `Report::MAX_POSITIONS_PER_RULE` 件までとする。
- 閾値の内に収まっている指標は出力しない。
- 文書指標は数ではなく指示の文に変換して出力する。

この制約は、レイヤーごとに lint と修正を往復させないための設計である。
その形を変更するときは、この目的を壊していないかどうかを確認する。
