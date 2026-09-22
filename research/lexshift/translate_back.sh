#!/bin/sh
# プロの日本語訳を英語へ訳し戻させる。元の英語の原文と比べると、
# 内容が同じまま語の選び方の差だけが残る。文体の指示は入れない。
# 使い方: sh translate_back.sh <キャッシュのディレクトリ> <model> [上限]
dir=$1; model=$2; limit=${3:-9999}; n=0
PROMPT='Translate the following Japanese technical documentation into English. Keep the Markdown structure exactly as it is: the same headings, lists, code blocks, tables, paragraph breaks, and front matter. Do not change the number or the order of paragraphs. Do not translate the contents of code blocks or the shortcodes written as {{< >}}. Output only the translation, with no preamble.'
for ja in "$dir"/*.ja.md; do
  [ "$n" -ge "$limit" ] && break
  out="${ja%.ja.md}.$model-en.md"
  [ -s "$out" ] && { n=$((n+1)); continue; }
  claude -p "$PROMPT" --model "$model" --output-format text < "$ja" > "$out.tmp" 2>/dev/null && mv "$out.tmp" "$out" && echo "訳した $(basename $out)"
  n=$((n+1))
done
