<!-- この文書も suikou の lint の対象になっている。
編集を始める前に、末尾の「この文書を書くときの注意」を読む。 -->
# textlint-rule-preset-tech-maintainability

textlint-rule-preset-tech-maintainability は、[suikou](https://github.com/KazuyoshiAkiyama/suikou) の M1 から M7 を
textlint の規則として実装したプリセットである。日本語と英語の両方に対応し、文書の言語は文書ごとに自動で判定する。

[English](README.md)

## 収録している規則

各規則の出典は、Google developer documentation style guide の Lists、
Headings、Timeless
documentation の節である。どの規則も生成AIが現れる前からある問題を扱っているので、モデルの世代が進んでも古くならない。
規則の厳密な定義は
[`docs/content/ja/metrics.md`](../../docs/content/ja/metrics.md) の「M 系の規則」の節に書いてある。

表の左の列は、このパッケージが内部で使うルール名である。この名前は、Rust 側のルールID
（`crates/suikou-core/src/rules/mod.rs` で定義した `maint/list-lead-in` など）から
接頭辞を除いた部分と一致させている。textlint が実際に表示するIDには、これとは別の
接頭辞が付く。詳細は後の「ルールID」の節で説明する。

| ルール名 | 検出する問題 |
|---|---|
| `list-lead-in` | 箇条書きの導入文が項目と文法的につながっていて、項目を増減すると文が成り立たなくなる |
| `item-count` | 地の文に項目の数を書いていて、項目を増減すると本文と食い違う |
| `numbered-heading` | 見出しに、手作業で付けた節番号がある |
| `manual-number` | 箇条書きの項目の本文が、手作業で付けた番号で始まる |
| `parallel-items` | 同じ箇条書きの中で、項目の末尾の形がそろっていない |
| `time-dependent` | 読む時点によって意味が変わる語を使っている |
| `trailing-etc` | 地の文または項目が「など」「etc.」で終わっている |

## 導入方法

```sh
npm install --save-dev textlint textlint-rule-preset-tech-maintainability
```

`.textlintrc` にこのパッケージを追加する。

```json
{
  "rules": {
    "preset-tech-maintainability": true
  }
}
```

個別のルールは、このパッケージの名前空間を通して無効にできる。

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

textlint は、プリセットの指摘に付ける接頭辞を、`.textlintrc` に書いた設定キーから作る。
設定キーの先頭にある `preset-` を除いた文字列が、その接頭辞になる。このパッケージを前述の設定で
読み込んだ場合、指摘のIDは `tech-maintainability/list-lead-in` のように、
どれも `tech-maintainability/` で始まる。この動作は textlint 自体の命名規則によるものであり、
このパッケージが何かを上書きしているわけではない。

設定キーが `preset-` で始まらないと、textlint はそのキーをプリセットとして認識しない。
認識した場合は、必ずその `preset-` を取り除く。
そのため、標準の読み込み方法を使う限り、どの設定キーを選んでもIDは `maint/list-lead-in` という形に
ならない。

Rust 側のバイナリが使うID（`maint/list-lead-in`、`maint/item-count` など）は、
表に挙げた個々のルール名と、判定ロジックそのものの正本であり続ける。`maint/` という
接頭辞は `suikou check` の出力に属するものであり、このパッケージが
表示するものではない。このパッケージのテスト（`test/support/lint.js`）は、
`@textlint/kernel` を直接呼び出し、各ルールを `maint/` というIDで手作業で登録している。この方法は
標準のプリセット読み込みを通らない。この方法を使うのは、`suikou check --format json` との
ゴールデンテストで、ルールIDを一対一で対応させるときだけである。
経緯は [`docs/content/ja/decisions.md`](../../docs/content/ja/decisions.md) の D-23 に記録している。

## Rust 側の実装との関係

suikou の解析処理の中核は、Rust の crate（[`crates/suikou-core`](../../crates/suikou-core)）が担い、
`suikou` バイナリとして配布している。
このパッケージは、textlint を使っている人に向けて、同じ規則を別の言語で
実装し直したものである。判定ロジックと正規表現は、
`crates/suikou-core/src/rules/mod.rs` から一字一句そのまま移植した。
思いつきで追加した処理はない。

二つの実装には、構造上の違いが一つだけある。
`suikou check` は原文を直接受け取り、
物理行一つを1ブロックとして扱う行単位の手順（`markdown.rs`）でブロックに分割する。

このパッケージも同じ処理をする。
各ルールは、ファイル全体の原文を `context.getSource()` で取得し、
その手順を JavaScript に移植した処理にかけてから判定する。
textlint 自体が持つ Markdown の AST はたどらない。

ソフトラップで複数行にわたる段落は、textlint の中では一つに結合したノードになる。
結合後の文字列で判定すると、
Rust 側では結び付けない語どうしが、正規表現の一致範囲に入ってしまう。
行単位の手順を移植したので、二つのツールの指摘件数が一致する。
一致しているかどうかは、このリポジトリの[ゴールデンテスト](../../tests/golden/)がファイルごとに確認している。

前処理も、段ごとに移植している。中身がコードになる Hugo のショートコードの扱いも同じである。
`{{< highlight yaml >}}` の中には YAML のコメントが入っており、
先に除去しないと行頭の `#` が見出しとして解釈される。
どちらの実装も、`highlight` と `mermaid`、および `codelang` を宣言した `tab` を除去し、
閉じタグが見つからない開きタグは処理しない。

日本語の形態素解析も、二つの実装で異なる点である。suikou-core は lindera を通して UniDic を読み込む。

このパッケージは [kuromojin](https://github.com/azu/kuromojin) を通して IPADIC を読み込む。
kuromojin は、textlint の周辺で標準的に使われている。IPADIC には、
UniDic にある「接尾辞」「代名詞」「形状詞」に相当する独立した品詞大分類がない。
この3つは、どれも品詞細分類付きの「名詞」として出力される。
その結果、`list-lead-in` と `parallel-items` が使う体言止めの判定は、
IPADIC では一つの条件にまとまる。この理由と、
それを確かめた実測は [`docs/content/ja/decisions.md`](../../docs/content/ja/decisions.md) の D-23 に書いてある。

構造の規則は、このパッケージには含めない。
`structure/missing-section` とその関連の規則は、文書にある節が無いことを判定する。
無い節には、指摘を結び付けるノードが存在しない。
文書全体で測る指標を textlint に入れない理由と同じであり、経緯は D-01 に記録している。
これらの規則は `suikou` 側で扱う。

もう一つの違いは、構造上の都合で生じたものではなく、意図して設けたものである。`suikou check` は、文書が
英語でも、説明文を常に日本語で出力する。このパッケージは、検出した文書の言語に
説明文の言語を合わせる。その場の誰も読めない説明文は、利用者の役に立たない
からである。

## 指摘の例

導入文が項目と文法的につながっていると、下の箇条書きを修正したときに文が成り立たなくなる。

```markdown
対象となるファイルは:

- `/etc/app/main.conf`
- `/etc/app/conf.d/*.conf`
```

`list-lead-in` がこの導入文を指摘するのは、導入文が助詞で終わっていて、完全な文でも体言止めの
コロンでもないからである。導入文を「対象となるファイルは次のとおり。」
のような完全な文に書き直せば、箇条書きを変えなくても指摘は出なくなる。

項目の末尾の形がそろっていない箇条書きも、同じ理由で壊れやすい。

```markdown
- 設定ファイルの構文を検証する。
- サービスを再読み込み
```

`parallel-items` がこの箇条書きを指摘するのは、一方の項目が句点で終わり、もう一方が
終わっていないからである。すべての項目を完全な文で終えるか、すべての項目を
言い切らない形にそろえれば、指摘は出なくなる。どちらの指摘も、項目を
増減したときに文章の側が成り立たなくなるかどうかを判定している点で共通している。

## 他のプリセットと重なる判定

[`@textlint-ja/textlint-rule-preset-ai-writing`](https://github.com/textlint-ja/textlint-rule-preset-ai-writing)には `no-ai-colon-continuation` という規則がある。
この規則も kuromojin を使い、コロンの直前が述語で終わっているかを判定するので、
`list-lead-in` の日本語の判定に近い。判定が重なる場合でも、両者の目的は異なる。

ai-writing 側の規則は、AI が書いた日本語に特有の言い回しを検出する。
`list-lead-in` は、項目数が変わると成り立たなくなる箇条書きの構造を検出する。
同じ行が両方の規則から指摘されることがある。一方の指摘で他方を抑える仕組みは設けていない。
両方のプリセットを有効にした利用者は、
両方の説明文を読んで、自分の文書に合う方を選べる。

## テスト

```sh
npm test
```

テストは [`textlint-tester`](https://github.com/textlint/textlint-tester) を使わず、
Node 本体のテストランナー（`node --test`）で実行する。textlint-tester は、Mocha の
グローバルな `describe`・`it` があることを前提に作られている。これらが無い環境で使われる代替の実装は、
テストケースが返す Promise の完了を待たない。そのため `node --test` でそのまま動かすと、
判定が終わる前にテストが成功扱いになる箇所があった。
`test/support/lint.js` は `@textlint/kernel` を直接呼び出す薄いラッパーであり、
`node --test` の
非同期テストの中で正しく完了を待つ。

`test/golden.test.js` では、このパッケージを
[`tests/golden/input/`](../../tests/golden/input/) の各ファイルに適用する。そして、ルールごとの
指摘件数が、同じファイルに対する `suikou check --format json` の件数と一致するかを
確かめる。この一致を、このパッケージの完了条件とする。

## この文書を書くときの注意

この文書にも `suikou check` を適用しており、指摘は一件も出ていない。箇条書きの導入文は
完全な文で終え、地の文には項目の数を書かず、見出しには連番を付けず、`docs/content/ja/metrics.md`
にある M6 の語彙を避けている。

## ライセンス

MIT と Apache-2.0 のどちらでも、好きな方を選んで利用できる。
