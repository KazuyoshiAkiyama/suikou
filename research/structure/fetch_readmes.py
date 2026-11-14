"""著名な OSS の README を集める。`readme` の型を外から確かめるために使う。

星の数で選ぶ。星の数は人気であって質ではないが、
読み手の多い README は読み手の必要に合わせて直され続けている。

言語ごとに分けて取る。一つの生態系に偏ると、
その生態系の慣習を README の性質と取り違える。
npm の README は `npm install` を必ず持つが、それは README の性質ではない。

同じ組織から取る数に上限を置く。
上限が無いと、ひとつの組織の書き方を様式の性質と取り違える。

使い方: python fetch_readmes.py <出力ディレクトリ> [1言語あたり] [1組織あたり]
"""

import base64
import json
import pathlib
import subprocess
import sys

# 生態系の偏りを避けるために言語を分ける。
LANGS = ["Rust", "Go", "Python", "TypeScript", "C", "Java", "Ruby", "C++"]


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


def readme_of(repo: str) -> str | None:
    blob = gh(f"repos/{repo}/readme")
    if not isinstance(blob, dict) or blob.get("encoding") != "base64":
        return None
    name = blob.get("name", "")
    # README.md 以外は様式が違う。rst や adoc は見出しの書き方が別である。
    if not name.lower().endswith(".md"):
        return None
    try:
        return base64.b64decode(blob["content"]).decode("utf-8", "replace")
    except (KeyError, ValueError):
        return None


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/readmes")
    per_lang = int(sys.argv[2]) if len(sys.argv) > 2 else 30
    per_org = int(sys.argv[3]) if len(sys.argv) > 3 else 3
    out.mkdir(parents=True, exist_ok=True)

    org_count: dict[str, int] = {}
    total = 0
    for lang in LANGS:
        taken = 0
        for page in (1, 2, 3):
            if taken >= per_lang:
                break
            q = f"search/repositories?q=language:{lang}+stars:>2000&sort=stars&per_page=50&page={page}"
            hits = gh(q)
            if not isinstance(hits, dict) or not hits.get("items"):
                break
            for item in hits["items"]:
                if taken >= per_lang:
                    break
                repo = item["full_name"]
                org = repo.split("/")[0]
                if org_count.get(org, 0) >= per_org:
                    continue
                src = readme_of(repo)
                # 短すぎるものは README として機能していない。
                if not src or len(src) < 400:
                    continue
                stem = repo.replace("/", "__")
                (out / f"{lang}__{stem}.md").write_text(src, encoding="utf-8")
                org_count[org] = org_count.get(org, 0) + 1
                taken += 1
                total += 1
        print(f"{lang}: {taken} 件", file=sys.stderr)
    print(f"readmes: {total} 件 / {len(org_count)} 組織")


if __name__ == "__main__":
    main()
