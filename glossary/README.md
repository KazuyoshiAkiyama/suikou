# suikou の用語集

日本語の技術文書で、どの語を選び、その語をどう書くかを決めた一覧である。
suikou の外でも使えるように、2つの汎用のフォーマットで置いてある。

| ファイル | 読む道具 |
|---|---|
| `suikou-ja.prh.yml` | [textlint-rule-prh](https://github.com/prh/prh) |
| `suikou-ja.csv` | 表計算、翻訳支援ツール |

正本は `crates/suikou-core/data/glossary_ja.toml` にある。
この2つは生成物であり、手で直さない。
作り直す手順は `research/glossary/build_glossary.py` にある。

## textlint から使う

```yaml
# .textlintrc.yml
rules:
  prh:
    rulePaths: [glossary/suikou-ja.prh.yml]
```

`textlint --fix` で自動的に寄せられる。

## 何を根拠にしているか

2つのことを分けて持つ。根拠の出どころが違うためである。

| 種類 | 決めるもの | 根拠 |
|---|---|---|
| `term` | どの語を選ぶか | 参照コーパスの頻度と PLaMo Translate の訳語 |
| `orthography` | その語をどう書くか | JTCA の規約を出発点にして参照コーパスで確認 |

表記については、JTCA「外来語（カタカナ）表記ガイドライン」第3版が
英語の語尾 -er、-or、-ar、-y を長音符号「ー」で書くとしている。
参照コーパス 438 万字で確認すると、-er、-or、-ar はそのとおりであった。
一方で -y は逆で、長音を付けない書き方が優勢である（`プロパティ` 870 / `プロパティー` 3）。
`コンテナ` は -er だが、この分野では長音を付けない書き方が定着している（1783 / 95）。

規約から導くのではなく、語ごとに実測で決めることにした。
食い違う語には `measured-against-jtca` と書いてある。

## CSV の列

| 列 | 中身 |
|---|---|
| `kind` | `term` か `orthography` |
| `concept` | 英語の術語（`term` のみ） |
| `preferred` | 選ぶ書き方 |
| `avoid` | 避ける書き方。空白で区切る |
| `keep` | 文字どおりの意味で使う語。置き換えない |
| `basis` | 根拠の種類 |
| `evidence` | 実測した値 |

## 出どころと謝辞

語の選択の考え方は [yomiyasu](https://github.com/nanaism/yomiyasu)（MIT License、
Copyright (c) 2026 nanaism）の、硬い文章向けの規範から採った。
文言は引用せず、判断の立て方だけを参考にして書き直している。

訳語の確認には [PLaMo Translate](https://github.com/pfnet/plamo-translate-cli) を使った。
モデルは PLaMo Community License で配られている。
この用語集はモデルの出力を証拠として読んだ結果であり、出力そのものを載せていない。
