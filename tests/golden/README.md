# ゴールデンテスト

Rust 実装が `research/` の Python 実装と同じ値を返すことを確認する。
指標の定義は `docs/METRICS.md` にある。

## 中身

| 入力 | 目的 |
|---|---|
| `input/ja_maintainability.md` | M1 から M7 を日本語で発火させる |
| `input/en_maintainability.md` | 同じく英語で発火させる |
| `input/ja_prose.md` | 地の文中心の文書で日本語の文書指標を確認する |
| `input/en_prose.md` | 同じく英語で確認する |
| `input/en_escaped.md` | バックスラッシュ解除の有無で文分割が変わることを確認する |

期待値は `expected/` にある。

| ファイル | 生成元 |
|---|---|
| `ja_metrics.json` | `research/jametrics.py` |
| `en_metrics.json` | `research/enmetrics.py` |
| `rules.json` | `research/mcheck.py` |
| `unescape.json` | spaCy による文分割の比較 |

## 期待値の再生成

```sh
cd tests/golden
python ../../research/jametrics.py 'input/ja_*.md' > expected/ja_metrics.json
python ../../research/enmetrics.py 'input/en_*.md' > expected/en_metrics.json
python ../../research/mcheck.py   'input/*.md'    > expected/rules.json
```

出力の形は Python 実装に合わせてあるため、整形は手で行う。

## 比較の方針

形態素解析器が lindera と fugashi で異なるため、完全一致は期待できない。
比率の指標は絶対誤差 0.02、1000文字あたりの密度は 0.5 を許容範囲とする。
M 系の規則は整数の一致を求める。
語数と文数はトークナイザの定義に依存するため、相対誤差 1% で突き合わせる。
根拠は `docs/DECISIONS.md` の D-14 にある。

許容範囲を超える指標が出た場合、どちらの実装が正しいかを個別に判断する。
Python 側が常に正しいとは限らない。

## Rust 側の突き合わせ

```sh
cargo test --all                           # 英語だけ。辞書が要らない
cargo test --all --features lindera-unidic # 日本語を含む
```

`crates/suikou-core/tests/golden_rules.rs` が M 系の件数を、
`golden_metrics.rs` が文書指標を突き合わせる。

`unescape.json` はまだ突き合わせていない。
期待値が spaCy の文分割に基づいており、Rust に英語の文分割がないためである。
バックスラッシュ解除そのものの挙動は `markdown.rs` の単体テストが固定している。

## 実際のコーパスに対する回帰

`corpus/fetch.sh` で取得したコーパスに対して、
`corpus/baselines.toml` の範囲に収まることを確認する。
コーパス本体はライセンスの都合でリポジトリに入れない。
