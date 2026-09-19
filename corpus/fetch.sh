#!/usr/bin/env bash
# ベースライン算出に使った人間コーパスを取得する。
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p cache && cd cache

get() { # repo branch subdir-pattern
  local repo=$1 branch=$2 pat=$3 name
  name=$(basename "$repo")
  [ -f "$name.tgz" ] || curl -sL -o "$name.tgz" \
    "https://codeload.github.com/$repo/tar.gz/refs/heads/$branch"
  tar xzf "$name.tgz" --wildcards --wildcards-match-slash "$pat" 2>/dev/null || true
}

# 英語
git clone --depth 1 --filter=blob:none --sparse https://github.com/torvalds/linux.git linux 2>/dev/null || true
(cd linux && git sparse-checkout set Documentation)
get rust-lang/book main '*/src/*.md'
get django/django main '*/docs/*.txt'
get google/eng-practices master '*.md'
get awsdocs/amazon-s3-userguide main '*doc_source/*.md'
get awsdocs/aws-cloudformation-user-guide main '*doc_source/*.md'

# 日本語（ネイティブ執筆）
get kaityo256/sevendayshpc main '*.md'
get kaityo256/python_zero main '*.md'
get future-architect/coding-standards master '*.md'

# 日本語（翻訳）
get kubernetes/website main '*/content/ja/docs/concepts/*.md'
get vuejs-translations/docs-ja main '*/src/guide/*.md'

echo "取得完了。日本語のネイティブ執筆は解説書に偏っている点に注意する。"
echo "手順書型のネイティブ日本語を足すまで、形式名詞と漢語率の閾値は info に置く。"
