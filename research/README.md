# research

Tier 2。較正と探索に使う Python 実装。本番の実行経路には入らない。

ここにある三つのスクリプトが全指標の参照実装である。
Rust 実装はこれらと同じ値を返すことをゴールデンテストで確認する。

| ファイル | 対象 |
|---|---|
| `jametrics.py` | 日本語の文書指標。fugashi と unidic-lite を使う |
| `enmetrics.py` | 英語の文書指標。spaCy を使う |
| `mcheck.py` | M1〜M7。日英対応 |

```sh
pip install fugashi unidic-lite spacy --break-system-packages
python -m spacy download en_core_web_sm
```

`enmetrics.py` には棄却した指標（現在分詞節、名詞化、modifier stack、依存距離）も
残してある。モデル依存の参考値として使うことがあるため。
これらを本番で使う場合は、常駐サイドカー方式を採る。
