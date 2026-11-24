# ゴールデンテスト

ゴールデンテストは、Rust 実装が `research/` 配下の Python 実装と同じ値を返すことを検証するためのテストである。各指標の定義は `docs/content/ja/metrics.md` に記載されている。

## テストデータの構成

| 入力ファイル | 用途 |
|---|---|
| `input/ja_maintainability.md` | 日本語文書で M1 から M7 の規則を検出させる |
| `input/en_maintainability.md` | 英語文書で同じ規則群を検出させる |
| `input/ja_prose.md` | 地の文が主体の日本語文書で文書指標を検証する |
| `input/en_prose.md` | 地の文が主体の英語文書で文書指標を検証する |
| `input/en_escaped.md` | バックスラッシュのエスケープ解除の有無で文分割が変わることを検証する |

期待値は `expected/` ディレクトリに配置されている。

| ファイル | 生成元 |
|---|---|
| `ja_metrics.json` | `research/jametrics.py` |
| `en_metrics.json` | `research/enmetrics.py` |
| `rules.json` | `research/mcheck.py` |
| `unescape.json` | spaCy による文分割の比較結果 |

各入力ファイルは基準値を固定するためのデータであるため、内容を編集してはならない。

## 期待値の再生成

```sh
cd tests/golden
python ../../research/jametrics.py 'input/ja_*.md' > expected/ja_metrics.json
python ../../research/enmetrics.py 'input/en_*.md' > expected/en_metrics.json
python ../../research/mcheck.py   'input/*.md'    > expected/rules.json
```

出力形式は Python 側に合わせているため、整形は手動で行う。

## 比較方針

lindera と fugashi は異なるトークナイザであるため、数値の完全一致は要求しない。比率は絶対誤差 0.02、1000文字あたりの密度は 0.5 を許容範囲とする。M 系規則は完全一致を要求する。語数と文数はトークナイザの仕様に依存するため、相対誤差 1% を許容範囲とする。これらの基準の根拠は、`docs/content/ja/decisions.md` の D-14 に記載されている。

許容範囲を超える指標が生じた場合は、どちらの実装が正しいかを個別に判断する。Python 実装が常に正解とは限らない。

## Rust 側での検証

```sh
cargo test --all                           # 英語だけ。辞書が要らない
cargo test --all --features lindera-unidic # 日本語を含む
```

`crates/suikou-core/tests/golden_rules.rs` は M 系規則の検出件数を検証し、`golden_metrics.rs` は文書指標を検証する。

`unescape.json` の照合テストは未実装である。期待値が spaCy の文分割結果に基づいているのに対し、Rust 実装には英語の文分割機能が存在しないためである。バックスラッシュのエスケープ解除処理自体は、`markdown.rs` の単体テストで検証している。

## 実コーパスを用いた回帰テスト

`corpus/fetch.sh` で取得したコーパスを処理し、結果が `corpus/baselines.toml` に設定された範囲内に収まることを検証する。コーパス本体はライセンス上の制約によりリポジトリへ含めない。
