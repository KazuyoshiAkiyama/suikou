# ゴールデンテスト

Rust の実装が `research/` の Python と同じ値を返すことを確かめるためのものである。
指標の定めは `docs/content/ja/metrics.md` にある。

## 中身

| 入力 | 何のためか |
|---|---|
| `input/ja_maintainability.md` | M1 から M7 を日本語で出させる |
| `input/en_maintainability.md` | 同じく英語で出させる |
| `input/ja_prose.md` | 地の文が中心の文書で、日本語の文書指標を確かめる |
| `input/en_prose.md` | 同じく英語で確かめる |
| `input/en_escaped.md` | バックスラッシュを戻すかどうかで文の切り分けが変わることを確かめる |

期待の値は `expected/` にある。

| ファイル | どこから作ったか |
|---|---|
| `ja_metrics.json` | `research/jametrics.py` |
| `en_metrics.json` | `research/enmetrics.py` |
| `rules.json` | `research/mcheck.py` |
| `unescape.json` | spaCy による文の切り分けの比べ |

これらの入力は、値を固めるためのものである。そのため中身を書き換えてはならない。

## 期待の値を作り直す

```sh
cd tests/golden
python ../../research/jametrics.py 'input/ja_*.md' > expected/ja_metrics.json
python ../../research/enmetrics.py 'input/en_*.md' > expected/en_metrics.json
python ../../research/mcheck.py   'input/*.md'    > expected/rules.json
```

出力の形は Python に合わせてあるため、整形は手で行う。

## 突き合わせの方針

lindera と fugashi は別のトークナイザであるため、値がぴたり合うことは求めない。
比率は絶対の誤差 0.02、1000文字あたりの密度は 0.5 を許容範囲とする。
M 系の規則はぴたり一致を求める。
語数と文数はトークナイザの定めに寄りかかるため、相対の誤差 1% とする。
その根拠は `docs/content/ja/decisions.md` の D-14 にある。

許容範囲を超えた指標が出た場合は、どちらの実装が正しいかをそのつど決める。
Python の側が常に正しいとは限らない。

## Rust の側の突き合わせ

```sh
cargo test --all                           # 英語だけ。辞書が要らない
cargo test --all --features lindera-unidic # 日本語を含む
```

`crates/suikou-core/tests/golden_rules.rs` が M 系の件数を、
`golden_metrics.rs` が文書指標を突き合わせる。

`unescape.json` はまだ突き合わせていない。
期待の値が spaCy の文の切り分けによっていて、Rust に英語の文の切り分けがないためである。
バックスラッシュを戻す振る舞いそのものは、`markdown.rs` の単体テストが固めている。

## 実際のコーパスに対する回帰

`corpus/fetch.sh` で取ってきたコーパスに対して、
`corpus/baselines.toml` の幅に収まることを確かめる。
コーパスそのものは、ライセンスの都合でリポジトリに入れない。
