# suikou 用語集

この用語集は、日本語の技術文書で使う語の選択と、その語の表記を定めた一覧である。
suikou 以外の環境でも使えるよう、2つの汎用フォーマットで配布している。

| ファイル | 対応するツール |
|---|---|
| `suikou-ja.prh.yml` | [textlint-rule-prh](https://github.com/prh/prh) |
| `suikou-ja.csv` | 表計算ソフト、翻訳支援ツール |

元になるデータは `crates/suikou-core/data/glossary_ja.toml` である。
配布用の2つのファイルはそこから生成したものなので、手作業では編集しない。
生成し直す手順は `research/glossary/build_glossary.py` に書いてある。

## textlint で使う方法

```yaml
# .textlintrc.yml
rules:
  prh:
    rulePaths: [glossary/suikou-ja.prh.yml]
```

`textlint --fix` を実行すると、文書の表記が用語集のとおりに自動で直る。

## 各項目の根拠

用語集は、項目を2つの種類に分けて持っている。根拠の出どころが種類ごとに異なるからである。

| 種類 | 定める内容 | 根拠 |
|---|---|---|
| `term` | 使う語の選択 | 参照コーパスの頻度と PLaMo Translate の訳語 |
| `orthography` | 語の表記 | JTCA の規約を起点とし、参照コーパスで確認 |

表記について、JTCA「外来語（カタカナ）表記ガイドライン」第3版は、
英語の語尾 -er、-or、-ar、-y を長音符号「ー」で表すと定めている。
438 万字の参照コーパスで確かめたところ、-er、-or、-ar は規約と一致した。
-y は規約と逆で、長音符号を付けない表記が多数を占めた（`プロパティ` 870 / `プロパティー` 3）。
`コンテナ` は語尾が -er に当たるが、この分野では長音符号を付けない表記が定着している（1783 / 95）。

そこで、表記は規約から一律に導かず、語ごとに実測で決めた。
規約と実測の結果が食い違う語には `measured-against-jtca` を付けている。

## CSV の列の定義

| 列 | 内容 |
|---|---|
| `kind` | `term` か `orthography` |
| `concept` | 英語の術語（`term` の場合のみ） |
| `preferred` | 採用する表記 |
| `avoid` | 避ける表記。空白区切り |
| `keep` | 文字どおりの意味で使うため置き換えない語 |
| `basis` | 根拠の種類 |
| `evidence` | 実測値 |

## 出典と謝辞

語の選び方の方針は、[yomiyasu](https://github.com/nanaism/yomiyasu)（MIT License、
Copyright (c) 2026 nanaism）が硬い文章向けに定めた規範を参考にした。
規範の文言は引用していない。判断の組み立て方だけを参考にして、文は独自に書いている。

訳語の確認には [PLaMo Translate](https://github.com/pfnet/plamo-translate-cli) を使った。
このモデルは PLaMo Community License のもとで配布されている。
用語集には、モデルの出力を証拠として読み取った結果だけを収め、出力そのものは収めていない。
