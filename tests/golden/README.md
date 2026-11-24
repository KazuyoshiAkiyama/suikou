# ゴールデンテスト

ゴールデンテストは、Rust の実装が `research/` の Python と同じ値を返すかどうかを確認するテストである。
指標の定義は `docs/content/ja/metrics.md` に書いてある。

## 入力と期待値

| 入力 | 目的 |
|---|---|
| `input/ja_maintainability.md` | 日本語の文書で M1 から M7 を検出させる |
| `input/en_maintainability.md` | 英語の文書で同じ規則を検出させる |
| `input/ja_prose.md` | 地の文が主体の文書で、日本語の文書指標を確認する |
| `input/en_prose.md` | 地の文が主体の文書で、英語の文書指標を確認する |
| `input/en_escaped.md` | バックスラッシュのエスケープを戻すか否かで、文の分割結果が変わることを確認する |

期待値は `expected/` に置いてある。

| ファイル | 生成元 |
|---|---|
| `ja_metrics.json` | `research/jametrics.py` |
| `en_metrics.json` | `research/enmetrics.py` |
| `rules.json` | `research/mcheck.py` |
| `unescape.json` | spaCy による文分割の比較 |

入力ファイルは、値を固定するために置いている。したがって、入力ファイルの内容を変更してはならない。

## 期待値の再生成

```sh
cd tests/golden
python ../../research/jametrics.py 'input/ja_*.md' > expected/ja_metrics.json
python ../../research/enmetrics.py 'input/en_*.md' > expected/en_metrics.json
python ../../research/mcheck.py   'input/*.md'    > expected/rules.json
```

出力の形式は Python の出力に合わせているので、整形は手作業で行う。

## 値の照合基準

lindera と fugashi はトークナイザが異なるので、値の完全な一致は求めない。
比率は絶対誤差 0.02 まで、1000文字あたりの密度は 0.5 までを許容する。
M 系の規則には完全な一致を求める。
語数と文数はトークナイザの定義に左右されるため、相対誤差 1% までを許容する。
この基準を決めた根拠は、`docs/content/ja/decisions.md` の D-14 に記録している。

許容範囲を超える指標が出たときは、どちらの実装が正しいかを個別に判断する。
Python 側の値が正しいとは限らない。

## Rust 側での照合

```sh
cargo test --all                           # 英語だけ。辞書が要らない
cargo test --all --features lindera-unidic # 日本語を含む
```

M 系の件数は `crates/suikou-core/tests/golden_rules.rs` が照合し、
文書指標は `golden_metrics.rs` が照合する。

`unescape.json` は照合の対象に入っていない。
この期待値は spaCy の文分割に依存しており、Rust 側には英語の文分割の機能がないためである。
バックスラッシュのエスケープを戻す処理そのものは、`markdown.rs` の単体テストで固定している。

## 実コーパスでの回帰確認

`corpus/fetch.sh` で取得したコーパスについて、指標が `corpus/baselines.toml` に定めた範囲に収まるかを確認する。
コーパス本体は、ライセンス上の理由でリポジトリに含めない。
