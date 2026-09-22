"""公開されている ADR を集める。`decision` の型を外から確かめるために使う。

`decision` の必須の節は Michael Nygard の Architecture Decision Record から取った。
その様式に従って書かれた実物が、多くのリポジトリの `doc/adr/` や `docs/adr/` にある。

adr-tools が最初に作る `0001-record-architecture-decisions.md` を手掛かりにする。
このファイル自体は道具が生成する定型文であるため、集める対象から外す。
残りは人が書いた決定の記録である。

1つのリポジトリから取る数に上限を置く。
上限が無いと、ひとつのプロジェクトの書き方を様式の性質と取り違える。

使い方: python fetch_adrs.py <出力ディレクトリ> [総数] [1リポジトリあたり]
"""

import sys
import json
import pathlib
import subprocess

SEED = "0001-record-architecture-decisions.md"


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


def is_record(name: str) -> bool:
    """決定の記録そのものかどうか。

    索引と雛形は決定の記録ではない。混ぜると、型の当たり外れを測れなくなる。
    """
    low = name.lower()
    if low.startswith(".") or "template" in low:
        return False
    return low not in ("readme.md", "index.md", "contributing.md")


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/adrs")
    total = int(sys.argv[2]) if len(sys.argv) > 2 else 200
    per_repo = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    out.mkdir(parents=True, exist_ok=True)

    seen = 0
    repos = 0
    for page in range(1, 11):
        hits = gh(f"search/code?q=filename:{SEED}&per_page=20&page={page}")
        if not hits or not hits.get("items"):
            break
        for item in hits["items"]:
            if seen >= total:
                break
            repo = item["repository"]["full_name"]
            folder = str(pathlib.PurePosixPath(item["path"]).parent)
            listing = gh(f"repos/{repo}/contents/{folder}")
            if not isinstance(listing, list):
                continue
            taken = 0
            for entry in listing:
                if taken >= per_repo or seen >= total:
                    break
                name = entry.get("name", "")
                if entry.get("type") != "file" or not name.endswith(".md"):
                    continue
                if name == SEED or not is_record(name):
                    continue
                blob = gh(f"repos/{repo}/contents/{folder}/{name}")
                if not isinstance(blob, dict) or blob.get("encoding") != "base64":
                    continue
                import base64

                try:
                    src = base64.b64decode(blob["content"]).decode("utf-8", "replace")
                except (KeyError, ValueError):
                    continue
                stem = repo.replace("/", "-")
                (out / f"{stem}__{name}").write_text(src, encoding="utf-8")
                taken += 1
                seen += 1
            if taken:
                repos += 1
        if seen >= total:
            break
    print(f"adrs: {seen} 件 / {repos} リポジトリ")


if __name__ == "__main__":
    main()
