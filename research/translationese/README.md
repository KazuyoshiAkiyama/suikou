# 翻訳調と手癖の候補を計測する

`coji/natural-japanese`（MIT License、Copyright (c) 2026 coji）の規則を候補として、
このリポジトリの基準で計測した記録である。

規則を追加する条件は、実測した値で人間と AI の幅が重ならないか、
明文のスタイルガイドの規約を出典とともに示すかのいずれかである。
候補は人が挙げてよいが、採否は計測した値が決める。

```sh
python measure_patterns.py corpus/cache/k8s   # 翻訳調のパターンと語の候補
python measure_flow.py                        # 段落頭の接続詞と文末の同形反復
```

`corpus/cache/k8s/` は同じ文書の人間訳と AI 訳を対で持つ。
同じ原文に対する訳を比べるため、分野や題材の違いが差に紛れ込まない。

計測の結果、採用した規則はない。
理由は `corpus/baselines.toml` の `[rejected]` に、経緯は決定の記録の D-34 にある。
