<!-- この文書にも suikou の lint がかかる。
編集する前に末尾の「この文書を書くときの注意」を見る。 -->
# textlint-rule-preset-tech-maintainability

[suikou](https://github.com/KazuyoshiAkiyama/suikou) の M1 から M7 を textlint の規則にした
プリセットである。日本語と英語の両方を扱い、文書ごとに言語を自動で判定する。

[English](README.md)

## 規則の中身

規則の出典は Google developer documentation style guide の Lists、
Headings、Timeless
documentation の各節である。生成AI以前からある問題を扱うため、モデルが世代を重ねても古びない。
規則の厳密な定めは
[`docs/content/ja/metrics.md`](../../docs/content/ja/metrics.md)（「M 系の規則」の節）にある。

左の列はこのパッケージが内部で使う名前であり、対応する Rust 側のルールID
（`crates/suikou-core/src/rules/mod.rs` に定めた `maint/list-lead-in` など）の
接頭辞を除いた部分と一致させてある。textlint が実際に表示するIDはこれとは
異なる接頭辞を持つ。詳しくは後述の「ルールID」を見る。

| ルール名 | 何を見つけるか |
|---|---|
| `list-lead-in` | 箇条書きの導入文が項目と文法的に結合していて、項目を増減すると破綻する |
| `item-count` | 地の文に項目数を書いていて、項目を増減すると本文が合わなくなる |
| `numbered-heading` | 見出しに手動の節番号が付いている |
| `manual-number` | 箇条書きの項目本文が手動の番号で始まっている |
| `parallel-items` | 同じ箇条書きの中で項目の末尾の形が揃っていない |
| `time-dependent` | 読む時点によって意味が変わる語を使っている |
| `trailing-etc` | 地の文または項目が「など」「etc.」で終端している |

## 入れ方

```sh
npm install --save-dev textlint textlint-rule-preset-tech-maintainability
```

`.textlintrc` にこのパッケージを足す。

```json
{
  "rules": {
    "preset-tech-maintainability": true
  }
}
```

個々のルールは、このパッケージの名前空間越しに無効にできる。

```json
{
  "rules": {
    "preset-tech-maintainability": {
      "manual-number": false
    }
  }
}
```

## ルールID

textlint は、プリセットの指摘に付ける接頭辞を `.textlintrc` に書いた設定キーから作る。
先頭の `preset-` を取り除いた文字列がその接頭辞になる。このパッケージを上の書き方どおり
読み込むと、指摘のIDはすべて `tech-maintainability/list-lead-in` のように
`tech-maintainability/` から始まる。これは textlint 自身の命名規則がそう動いている
だけであって、このパッケージが何かを上書きしているわけではない。設定キーは `preset-` で
始まらないとプリセットとして認識されず、認識された場合はその `preset-` が必ず剥がされる
ため、標準の読み込み経路のどんな設定キーを選んでも `maint/list-lead-in` という形には
届かない。

Rust 側のバイナリが使う ID（`maint/list-lead-in`、
`maint/item-count` など）は、
上の表にある個々のルール名と、判定ロジックそのものの正本であり続ける。`maint/` という
接頭辞そのものは `suikou check` の出力に属するものであって、このパッケージが
表示するものではない。このパッケージ自身のテスト（`test/support/lint.js`）は
`@textlint/kernel` を直に呼んで各ルールを `maint/` というIDで手動登録しているが、これは
標準のプリセット読み込みを経由しない書き方であり、`suikou check --format json` との
ゴールデンテストの突き合わせでルールIDを一対一にそろえるためだけに使っている。
経緯は [`docs/content/ja/decisions.md`](../../docs/content/ja/decisions.md) の D-23 にある。

## Rust 側の実装との関係

suikou の解析の中核は Rust の crate
（[`crates/suikou-core`](../../crates/suikou-core)）にあり、
`suikou` バイナリとして配る。
このパッケージは、すでに textlint を使っている人のために、同じ規則を別の言語で
もう一度実装したものである。判定ロジックと正規表現は
`crates/suikou-core/src/rules/mod.rs` から一字一句移植した。
思いつきで足したものはない。

二つの実装には、作りのうえで一つだけ異なる点がある。
`suikou check` は原文を直接、
物理行ごとに1ブロックとして扱う行ベースの手順（`markdown.rs`）でブロックへ分ける。

このパッケージも同じことをする。
各ルールは `context.getSource()` でファイル全体の原文を受け取り、
その手順を JavaScript に移植したものにかけてから判定する。
textlint自身が持つ Markdown の AST は歩かない。

ソフトラップで複数行にまたがる段落は、textlintの中ではひとつの結合したノードになる。
結合後の文字列に対して判定すると、
Rust 側では決して結び付けない語同士が正規表現の一致範囲に入ってしまう。
行ベースの手順を移植したことで、二つのツールの指摘の件数が一致する。
このリポジトリの[ゴールデンテスト](../../tests/golden/)は、
そのファイルごとの一致を確認している。

前処理も段ごとに移植してある。中身がコードである Hugo のショートコードも同様である。
`{{< highlight yaml >}}` の中には YAML のコメントがあり、
先に取り除かないと行頭の `#` が見出しとして読まれる。
どちらの実装も `highlight` と `mermaid`、`codelang` を宣言した `tab` を取り除き、
閉じが見つからない開きには手を付けない。

日本語の形態素解析も、もう一つの違いである。suikou-core は lindera 経由で UniDic を読む。

このパッケージは [kuromojin](https://github.com/azu/kuromojin) 経由で IPADIC を読む。
kuromojin は textlint 界隈の標準である。IPADIC には、
UniDic が持つ「接尾辞」「代名詞」「形状詞」に当たる独立した品詞大分類がなく、
この3つはいずれも品詞細分類を持つ「名詞」として現れる。
`list-lead-in` と `parallel-items` が使う体言止めの判定は、
その結果として IPADIC ではひとつの条件にまとまる。この理由付けと、
それを確認した実測は[`docs/content/ja/decisions.md`](../../docs/content/ja/decisions.md) の D-23 に書いてある。

構造の規則はこのパッケージに入れない。
`structure/missing-section` とその周辺の規則は、節が文書に無いことを問うものであり、
無い節には指摘を結び付けるノードがない。
文書の全体で測定する指標を textlint に収めない理由と同じであり、経緯は D-01 にある。
これらの規則は `suikou` の側が受け持つ。

もう一つの違いは作りの都合ではなく、意図してそうしてある。`suikou check` は、文書が
英語であっても説明文を常に日本語で出す。このパッケージは、検出した文書の言語に
説明文の言語を合わせる。その場にいる誰も読めない説明文は、その人たちの役に立たない
ためである。

## 例

導入文が項目と文法的に結合していると、下の箇条書きを直したときに破綻する。

```markdown
対象となるファイルは:

- `/etc/app/main.conf`
- `/etc/app/conf.d/*.conf`
```

`list-lead-in` は、この導入文が助詞で終端していて完全な文でも体言止めの
コロンでもないため指摘する。導入文を「対象となるファイルは次のとおり。」
のような完全な文に書き直せば、箇条書き自体を変えなくても指摘は消える。

項目の末尾の形が揃っていない箇条書きも、同じ理由で壊れやすい。

```markdown
- 設定ファイルの構文を検証する。
- サービスを再読み込み
```

`parallel-items` は、一方が句点で終端、もう一方が終端していないために
この箇条書きを指摘する。すべての項目を完全な文で終えるか、すべての項目を
言い切らない形で揃えれば指摘は消える。どちらの指摘も、箇条書きの項目を
増やしたり減らしたりしたときに文章の側が破綻するかどうかを見ている点は同じである。

## ほかのプリセットとの重なり

[`@textlint-ja/textlint-rule-preset-ai-writing`](https://github.com/textlint-ja/textlint-rule-preset-ai-writing)には `no-ai-colon-continuation` という規則がある。
これも kuromojin を使って、コロンの手前が述語で終端しているかを見ており、
`list-lead-in` の日本語判定に近い。判定が重なる場面でも目的は異なる。

あちらは AI が書いた日本語に特有の言い回しを検出し、
`list-lead-in`は項目数が変わると破綻する箇条書きの構造を検出する。
同じ行が両方から指摘されることがある。どちらか一方が他方を抑える仕組みは設けていない。
両方のプリセットを有効にした利用者は、
両方の説明文を読んだうえで自分の文書に合う方を選べる立場にある。

## テスト

```sh
npm test
```

テストは [`textlint-tester`](https://github.com/textlint/textlint-tester) ではなく、
Node 本体のテストランナー（`node --test`）で走る。textlint-tester は Mocha の
グローバルな `describe`・`it` を前提に作られている。それらが無い環境での代用は、
テストケースが返す Promise を待たないため、`node --test` にそのまま載せると、
判定が終端する前にテストが成功したことになってしまう箇所があった。
`test/support/lint.js` は `@textlint/kernel` を直に薄く包んでおり、
`node --test` の
非同期テストの下で素直に待ち合わせる。

`test/golden.test.js` は、このパッケージを
[`tests/golden/input/`](../../tests/golden/input/) の各ファイルにかけて、
ルールごとの
指摘の件数が同じファイルに対する `suikou check --format json` の件数と一致することを
確認する。この一致が、このパッケージの完了条件である。

## この文書を書くときの注意

この文書自身にも `suikou check` をかけてある。指摘はゼロである。箇条書きの導入文は
完全な文で終え、地の文に項目数を書かず、見出しに連番を付けず、`docs/content/ja/metrics.md`
にある M6 の語彙を避けてある。

## ライセンス

MIT または Apache-2.0 のどちらか好きな方。
