# suikou の用語集

suikou の用語集は、日本語の技術文書における語彙の選定および表記の統一基準をまとめた一覧である。suikou 以外の環境でも利用できるよう、2種類の汎用フォーマットで配布している。

| ファイル | 対応ツール |
|---|---|
| `suikou-ja.prh.yml` | [textlint-rule-prh](https://github.com/prh/prh) |
| `suikou-ja.csv` | 表計算ソフト、翻訳支援ツール |

マスターデータは `crates/suikou-core/data/glossary_ja.toml` に格納されている。これら2ファイルは自動生成された成果物であり、手動で編集しない。再生成の処理は `research/glossary/build_glossary.py` に実装されている。

## textlint での利用

```yaml
# .textlintrc.yml
rules:
  prh:
    rulePaths: [glossary/suikou-ja.prh.yml]
```

`textlint --fix` を実行すると、定義に従って自動的に置換される。

## 選定と表記の根拠

本用語集では、判断根拠の性質が異なるため、語彙選定と表記統一の2種別で管理している。

| 種別 | 対象 | 判断根拠 |
|---|---|---|
| `term` | 語彙の選定 | 参照コーパスでの出現頻度および PLaMo Translate の訳語 |
| `orthography` | 表記の統一 | JTCA の規約を基準とした参照コーパスでの検証結果 |

表記に関して、JTCA「外来語（カタカナ）表記ガイドライン」第3版は、英語の語尾 -er、-or、-ar、-y に長音符号「ー」を付す方針を示している。これに対し、438 万字の参照コーパスで出現傾向を調査したところ、-er、-or、-ar は規約どおりの傾向であった。一方、-y については長音符号を省く表記の方が多数を占めた（`プロパティ` 870 / `プロパティー` 3）。また、語尾が -er に該当する `コンテナ` も、この分野では長音符号なしの表記が定着している（1783 / 95）。

このような実態を踏まえ、規約から一律に導出するのではなく、語ごとの実測結果に基づいて表記を決定する方針を採っている。規約と異なる表記を採用した語には `measured-against-jtca` を付記している。

## CSV の列構成

| 列名 | 説明 |
|---|---|
| `kind` | 種別（`term` または `orthography`） |
| `concept` | 対応する英語の専門用語（`term` のみ） |
| `preferred` | 推奨される語または表記 |
| `avoid` | 非推奨の語または表記（空白区切りで列挙） |
| `keep` | 文字どおりの意味で用いる場合に置換の対象外とする語 |
| `basis` | 根拠の分類 |
| `evidence` | 根拠となる実測値 |

## 出典と謝辞

語彙選定の方針は、[yomiyasu](https://github.com/nanaism/yomiyasu)（MIT License、Copyright (c) 2026 nanaism）における硬い文体向けの規範を参考にしている。同プロジェクトの表現は引用せず、判断基準の考え方のみを取り入れて独自に作成している。

訳語の検証には [PLaMo Translate](https://github.com/pfnet/plamo-translate-cli) を利用した。同モデルは PLaMo Community License のもとで配布されている。本用語集はモデルの出力を検証の根拠として参照した結果をまとめたものであり、出力内容自体は収録していない。
