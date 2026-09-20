#!/bin/sh
# 英語の原文を LLM に訳させる。LLM の既定の癖を捉えるため、文体の指示は入れない。
# 使い方: sh translate.sh <キャッシュのディレクトリ> <model> [上限]
dir=$1; model=$2; limit=${3:-9999}; n=0
PROMPT='次の英語の技術文書を日本語に翻訳してください。Markdown の構造（見出し、リスト、コードブロック、表、段落の区切り、フロントマター）をそのまま保ち、段落の数と順序を変えないでください。コードブロックの中とショートコード {{< >}} は翻訳しないでください。訳文だけを出力し、前置きや説明は付けないでください。'
for en in "$dir"/*.en.md; do
  [ "$n" -ge "$limit" ] && break
  out="${en%.en.md}.$model.md"
  [ -s "$out" ] && { n=$((n+1)); continue; }
  claude -p "$PROMPT" --model "$model" --output-format text < "$en" > "$out.tmp" 2>/dev/null && mv "$out.tmp" "$out" && echo "訳した $(basename $out)"
  n=$((n+1))
done
