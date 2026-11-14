"""日本語で書かれた ADR と設計文書を集める。日本語の様式の型を実測するために使う。

英語の ADR は `fetch_adrs.py` が集める。ここでは日本語の地の文を持つものだけを取る。
見出しの語（`## 決定` や `## コンテキスト` など）と、置かれているパス（`docs/adr` など）で
GitHub のコード検索を行い、候補を得る。

候補は次の条件で絞る。
- 平仮名とカタカナ（U+3040 から U+30FF）が 300 字以上ある。
- 全体が 400 字以上ある。
英語が主で題名だけ日本語のものは、ここで落ちる。

索引と雛形は取らない。ファイル名に template を含むもの、README.md、index.md、
_index.md、CONTRIBUTING.md が対象になる。

1つの組織（GitHub の owner）から取る数に上限を置く。
上限は組織ごとに 3 件とする。リポジトリごとではない。
同じ組織の複数のリポジトリは、同じ執筆者と同じ規約を共有しやすい。
リポジトリごとに上限を置くと、ひとつの組織の書き方が様式の性質に見えてしまう。

出力のファイル名は `<owner>__<repo>__<元のファイル名>.md` とする。
どのリポジトリから来たかを後から辿れるようにするためである。

使い方: python fetch_adrs_ja.py <出力ディレクトリ>
"""

import base64
import json
import pathlib
import subprocess
import sys
import time
import urllib.parse

# コード検索の上限に当たらないように、検索のあいだを空ける。
SEARCH_INTERVAL = 2.5
PER_PAGE = 30
MAX_PAGES = 3
TARGET = 150
PER_ORG = 3
MIN_KANA = 300
MIN_CHARS = 400

# 見出しの語と置かれているパスで候補を探す。
QUERIES = [
    '"## 決定" path:docs/adr',
    '"## 決定" path:doc/adr',
    '"## コンテキスト" "## 決定"',
    '"## 状況" "## 決定"',
    '"## 背景" "## 決定" "## 結果"',
    '"## 検討した選択肢"',
    '"## 決定事項" path:docs',
    '"## 代替案" "## 決定"',
    '"## 理由" "## 決定" path:docs/adr',
    '"## 設計方針" path:docs',
    '"## 背景" "## 設計" path:docs/design',
    '"## 設計判断" path:docs',
]

# 索引と雛形は決定の記録ではない。
SKIP_NAMES = {"readme.md", "index.md", "_index.md", "contributing.md"}


def gh(path: str) -> object:
    out = subprocess.run(
        ["gh", "api", "-X", "GET", path],
        capture_output=True,
        text=True,
        check=False,
    )
    if out.returncode != 0:
        return None
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError:
        return None


def search(q: str, page: int) -> list[dict]:
    """コード検索を1ページ分行う。検索の上限を越えないように待つ。"""
    time.sleep(SEARCH_INTERVAL)
    qs = urllib.parse.urlencode({"q": q, "per_page": PER_PAGE, "page": page})
    hits = gh(f"search/code?{qs}")
    if not isinstance(hits, dict):
        return []
    return hits.get("items", []) or []


def is_record(name: str) -> bool:
    low = name.lower()
    if not low.endswith(".md") or "template" in low:
        return False
    return low not in SKIP_NAMES


def fetch_text(repo: str, path: str) -> str | None:
    blob = gh(f"repos/{repo}/contents/{urllib.parse.quote(path)}")
    if not isinstance(blob, dict) or blob.get("encoding") != "base64":
        return None
    try:
        return base64.b64decode(blob["content"]).decode("utf-8", "replace")
    except (KeyError, ValueError):
        return None


def is_japanese_prose(src: str) -> bool:
    """平仮名とカタカナの量で、日本語の地の文かどうかを決める。"""
    kana = sum(1 for c in src if "぀" <= c <= "ヿ")
    return kana >= MIN_KANA and len(src) >= MIN_CHARS


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/adrs_ja")
    out.mkdir(parents=True, exist_ok=True)

    org_count: dict[str, int] = {}
    seen_files: set[str] = set()
    total = 0
    productive: dict[str, int] = {}

    def take(item: dict) -> bool:
        """候補を一件検査し、条件を満たせば保存する。保存したら True を返す。"""
        nonlocal total
        repo = item["repository"]["full_name"]
        path = item["path"]
        name = pathlib.PurePosixPath(path).name
        key = f"{repo}:{path}"
        if key in seen_files or not is_record(name):
            return False
        seen_files.add(key)
        owner, rname = repo.split("/", 1)
        if org_count.get(owner, 0) >= PER_ORG:
            return False
        src = fetch_text(repo, path)
        if not src or not is_japanese_prose(src):
            return False
        fname = f"{owner}__{rname}__{name}"
        (out / fname).write_text(src, encoding="utf-8")
        org_count[owner] = org_count.get(owner, 0) + 1
        total += 1
        return True

    for q in QUERIES:
        if total >= TARGET:
            break
        before = total
        for page in range(1, MAX_PAGES + 1):
            if total >= TARGET:
                break
            items = search(q, page)
            if not items:
                break
            for item in items:
                if total >= TARGET:
                    break
                take(item)
        productive[q] = total - before
        print(f"{q}: {productive[q]} 件", file=sys.stderr)

    print(
        f"adrs_ja: {total} 件 / {len(org_count)} 組織",
    )


if __name__ == "__main__":
    main()
