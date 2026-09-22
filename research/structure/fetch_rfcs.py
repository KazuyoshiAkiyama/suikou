"""Rust RFC を集める。`design` の型を外から確かめるために使う。

`design` の必須の節は Rust RFC テンプレートから取った。
そのテンプレートに従って書かれた実物が `rust-lang/rfcs` の `text/` にある。
型の出どころと同じ場所で測ることになるため、最も厳しい確認になる。

使い方: python fetch_rfcs.py <出力ディレクトリ> [頁数]
"""
import sys
import json
import pathlib
import urllib.request
import urllib.error

TREE = "https://api.github.com/repos/rust-lang/rfcs/git/trees/master?recursive=1"
RAW = "https://raw.githubusercontent.com/rust-lang/rfcs/master/"


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/rfcs")
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 200
    out.mkdir(parents=True, exist_ok=True)

    req = urllib.request.Request(TREE, headers={"User-Agent": "suikou"})
    with urllib.request.urlopen(req) as r:
        tree = json.load(r)["tree"]
    # `text/` の直下だけを採る。その下の階層には各国語の訳が置いてある。
    # 新しいものから採る。テンプレートが定着したあとの書き方を見るためである。
    paths = sorted(
        (
            e["path"]
            for e in tree
            if e["type"] == "blob"
            and e["path"].startswith("text/")
            and e["path"].endswith(".md")
            and e["path"].count("/") == 1
        ),
        reverse=True,
    )[:limit]

    n = 0
    for p in paths:
        try:
            with urllib.request.urlopen(RAW + p) as r:
                src = r.read().decode("utf-8", "replace")
        except (urllib.error.HTTPError, urllib.error.URLError):
            continue
        (out / p.replace("/", "__")).write_text(src, encoding="utf-8")
        n += 1
    print(f"rfcs: {n}")


if __name__ == "__main__":
    main()
