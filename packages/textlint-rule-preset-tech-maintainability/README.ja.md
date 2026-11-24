<!-- この文書自体も suikou の lint 対象である。
編集する前に末尾の「この文書を編集する際の注意」を確認する。 -->
# textlint-rule-preset-tech-maintainability

textlint-rule-preset-tech-maintainability は、[suikou](https://github.com/KazuyoshiAkiyama/suikou) の M1 から M7 に相当するルールを実装した textlint プリセットである。日本語と英語の両方に対応しており、文書ごとに言語を自動で判定する。

[English](README.md)

## 収録ルール

各ルールの出典は、Google developer documentation style guide の Lists、Headings、Timeless documentation の各節である。生成AIの登場前から存在する執筆上の問題を対象としているため、モデルの世代が進んでも陳腐化しない。ルールの厳密な定義は [`docs/content/ja/metrics.md`](../../docs/content/ja/metrics.md) の「M 系の規則」に記載されている。

以下の表の左列は、本パッケージが内部的に使用するルール名である。これらは Rust 側の `crates/suikou-core/src/rules/mod.rs` で定義されているルールID（`maint/list-lead-in` など）から接頭辞を除いた名称と一致する。textlint の実行時に実際に出力されるルールIDには別の接頭辞が付くため、詳細は後述の「ルールID」で説明する。

| ルール名 | 検出対象 |
|---|---|
| `list-lead-in` | 箇条書きの導入文が項目と文法的に連結しており、項目の増減によって破綻する記述 |
| `item-count` | 地の文に項目数が書かれており、項目の増減によって本文との整合が崩れる記述 |
| `numbered-heading` | 見出しに手動で節番号が付いている記述 |
| `manual-number` | 箇条書きの項目本文が手動の番号で始まっている記述 |
| `parallel-items` | 同一の箇条書き内で項目の文末表現が揃っていない記述 |
| `time-dependent` | 読む時点によって意味が変化する語句を含む記述 |
| `trailing-etc` | 地の文または箇条書き項目が「など」「etc.」で終わっている記述 |

## インストールと設定

```sh
npm install --save-dev textlint textlint-rule-preset-tech-maintainability
```

`.textlintrc` に本パッケージを追加する。

```json
{
  "rules": {
    "preset-tech-maintainability": true
  }
}
```

個別のルールは、本パッケージの名前空間を通じて無効化できる。

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

textlint は、プリセットによる指摘に付与する接頭辞を `.textlintrc` の設定キーから生成し、先頭の `preset-` を除いた文字列を接頭辞として使う。前述の設定どおりに本パッケージを読み込むと、指摘のルールIDは `tech-maintainability/list-lead-in` のようにすべて `tech-maintainability/` で始まる。これは textlint 本体の命名規則による挙動であり、本パッケージ側で接頭辞を上書きしているわけではない。設定キーは `preset-` で始まらなければプリセットとして認識されず、認識された場合は必ずその `preset-` が除去されるため、標準の読み込み経路ではどの設定キーを指定しても `maint/list-lead-in` という形式にはならない。

Rust 側バイナリが用いるルールID（`maint/list-lead-in`、`maint/item-count` など）は、前掲の表に示した各ルール名および判定ロジック自体の正規の基準である。`maint/` という接頭辞自体は `suikou check` の出力仕様に属するものであり、本パッケージが通常出力するものではない。本パッケージのテスト補助コード（`test/support/lint.js`）では `@textlint/kernel` を直接呼び出して各ルールを `maint/` 付きのIDで手動登録しているが、これは標準のプリセット読み込みを経由しない処理であり、`suikou check --format json` とのゴールデンテスト比較でルールIDを一対一に対応させる目的だけに限定している。この設計判断の経緯は、[`docs/content/ja/decisions.md`](../../docs/content/ja/decisions.md) の D-23 に記載されている。

## Rust 実装との関係

suikou の解析中核は Rust クレート（[`crates/suikou-core`](../../crates/suikou-core)）にあり、`suikou` バイナリとして配布される。本パッケージは、textlint を利用しているユーザー向けに同じルールを別言語で再実装したものである。判定ロジックと正規表現は `crates/suikou-core/src/rules/mod.rs` から一字一句違わず移植しており、独自の追加は行っていない。

両実装には構造上、異なる点が一つだけある。`suikou check` は原文を直接扱い、物理行ごとに1ブロックとして分割する行ベースの手順（`markdown.rs`）を用いる。本パッケージでも同様の手法を採用しており、各ルールが `context.getSource()` でファイル全体の原文を取得し、JavaScript へ移植した行ベースの分割手順にかけてから判定する。textlint が生成する Markdown の AST は走査しない。

ソフトラップによって複数行に分かれた段落は、textlint 内部では結合されたひとつのノードとして扱われる。結合後の文字列を対象に判定すると、Rust 側では決して結び付かない語同士が正規表現の一致範囲に含まれてしまう。行ベースの分割手順をそのまま移植することで、両ツール間の指摘件数が一致する。ファイル単位での一致は、このリポジトリの[ゴールデンテスト](../../tests/golden/)で検証している。

前処理も各段階をそのまま移植しており、本文にコードを含む Hugo のショートコードも対象である。`{{< highlight yaml >}}` の内部には YAML のコメントが含まれるため、あらかじめ除去しなければ行頭の `#` が見出しとして解釈される。両実装とも、`highlight`、`mermaid`、および `codelang` を指定した `tab` を除去対象とし、対応する閉じタグが見つからない開始タグはそのまま残す。

日本語の形態素解析も両実装の相違点である。suikou-core は lindera を介して UniDic を利用するのに対し、本パッケージは textlint エコシステムの標準である [kuromojin](https://github.com/azu/kuromojin) を介して IPADIC を利用する。IPADIC には UniDic の「接尾辞」「代名詞」「形状詞」に相当する独立した品詞大分類がなく、この3つはすべて品詞細分類付きの「名詞」として出力される。そのため、`list-lead-in` と `parallel-items` で用いる体言止め判定は、IPADIC 環境ではひとつの条件に統合される。この設計上の理由と検証時の実測結果は、[`docs/content/ja/decisions.md`](../../docs/content/ja/decisions.md) の D-23 に記録されている。

文書構造に関するルールは本パッケージに含めない。`structure/missing-section` をはじめとする構造ルールは文書内に特定の節が存在しないことを検証するが、存在しない節には指摘を対応付けるノードがない。文書全体を対象に算出する指標を textlint 側へ収めない理由と同じであり、経緯は D-01 に記載されている。これらのルールは `suikou` 側が担う。

もうひとつの違いは実装上の制約ではなく、意図的な仕様である。`suikou check` は英語文書に対しても説明文を常に日本語で出力するが、本パッケージは判定した文書の言語に合わせて説明文の言語を切り替える。その場にいる誰も読めない説明文は役に立たないためである。

## 検出例

導入文が項目と文法的に連結していると、後続の箇条書きを編集した際に文章が破綻する。

```markdown
対象となるファイルは:

- `/etc/app/main.conf`
- `/etc/app/conf.d/*.conf`
```

この例では、導入文が助詞で終わっており完全な文でも体言止めのコロンでもないため、`list-lead-in` が指摘を出す。導入文を「対象となるファイルは次のとおり。」のような完全な文へ書き換えれば、箇条書き本体を変更せずに指摘を解消できる。

項目の文末表現が不統一な箇条書きも、同じ理由で破綻しやすい。

```markdown
- 設定ファイルの構文を検証する。
- サービスを再読み込み
```

ここでは一方が句点で終わり、もう一方が句点で終わっていないため、`parallel-items` が指摘を出す。すべての項目を完全な文で統一するか、言い切らない形式で揃えれば指摘は解消する。いずれのルールも、箇条書きの項目を増減したときに文章側が破綻しないかという観点は共通している。

## ほかのプリセットとの重複

[`@textlint-ja/textlint-rule-preset-ai-writing`](https://github.com/textlint-ja/textlint-rule-preset-ai-writing) には `no-ai-colon-continuation` というルールが含まれる。このルールも kuromojin を用いてコロン直前が述語で終わっているかを判定しており、`list-lead-in` の日本語判定と近い挙動を示す。ただし、検出箇所が重なる場合でも両者の目的は異なる。

あちらは AI が生成した日本語に特有の言い回しを検出するのに対し、`list-lead-in` は項目数の増減によって破綻する箇条書き構造を検出する。同一行が双方から指摘されるケースはあるが、一方の指摘で他方を抑制する仕組みは設けていない。両方のプリセットを有効にした利用者は、双方の説明文を比較したうえで自身の文書に合う判断を選べる。

## テスト

```sh
npm test
```

テストの実行には [`textlint-tester`](https://github.com/textlint/textlint-tester) ではなく、Node.js 標準のテストランナー（`node --test`）を使用する。textlint-tester は Mocha のグローバル関数である `describe` と `it` の存在を前提としている。これらが存在しない環境向けの代替処理はテストケースが返す Promise を待機しないため、`node --test` 上でそのまま実行すると、判定完了前にテスト成功とみなされてしまう箇所があった。そこで `test/support/lint.js` に `@textlint/kernel` を直接呼び出す単純なラッパーを用意し、`node --test` の非同期テストとして確実に完了を待機する構成にしている。

`test/golden.test.js` は、[`tests/golden/input/`](../../tests/golden/input/) 内の各ファイルに対して本パッケージを実行し、ルール別の指摘件数が同一ファイルに対する `suikou check --format json` の結果と一致することを検証する。この件数の一致が、本パッケージの完了条件である。

## この文書を編集する際の注意

この文書自体にも `suikou check` を実行しており、指摘はゼロである。箇条書きの導入文は完全な文で終え、地の文に項目数を書かず、見出しに連番を付けず、`docs/content/ja/metrics.md` に定義された M6 の語彙を使わないようにしている。

## ライセンス

MIT または Apache-2.0 のいずれかを選択できる。
