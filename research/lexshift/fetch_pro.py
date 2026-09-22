"""プロの日本語訳を分野ごとに集める。対の一覧が分野をまたいで通用するかを確かめるために使う。

使い方: python fetch_pro.py <出力ディレクトリ> [1分野あたりの頁数]
"""
import sys, pathlib, json, urllib.request, urllib.error

SOURCES_EN = {
    "k8s": ("https://api.github.com/repos/kubernetes/website/git/trees/main?recursive=1",
            "content/en/docs/", "https://raw.githubusercontent.com/kubernetes/website/main/"),
    "mdn": ("https://api.github.com/repos/mdn/content/git/trees/main?recursive=1",
            "files/en-us/web/", "https://raw.githubusercontent.com/mdn/content/main/"),
    "vue": ("https://api.github.com/repos/vuejs/docs/git/trees/main?recursive=1",
            "src/guide/", "https://raw.githubusercontent.com/vuejs/docs/main/"),
}

SOURCES = {
    # 分野: (木を引く API, 日本語の頁を選ぶ前置き, 生の中身を引く前置き)
    "k8s": ("https://api.github.com/repos/kubernetes/website/git/trees/main?recursive=1",
            "content/ja/docs/", "https://raw.githubusercontent.com/kubernetes/website/main/"),
    "mdn": ("https://api.github.com/repos/mdn/translated-content/git/trees/main?recursive=1",
            "files/ja/web/", "https://raw.githubusercontent.com/mdn/translated-content/main/"),
    "vue": ("https://api.github.com/repos/vuejs-translations/docs-ja/git/trees/main?recursive=1",
            "src/guide/", "https://raw.githubusercontent.com/vuejs-translations/docs-ja/main/"),
    # 計算や手順の語（数える、割る、求める）を含む分野を足す。
    # Web の文書だけでは、これらが参照に入らず誤って拾われる。
    "rustbook": ("https://api.github.com/repos/rust-lang-ja/book-ja/git/trees/master-ja?recursive=1",
                 "src/ch", "https://raw.githubusercontent.com/rust-lang-ja/book-ja/master-ja/"),
    "react": ("https://api.github.com/repos/reactjs/ja.react.dev/git/trees/main?recursive=1",
              "src/content/learn/", "https://raw.githubusercontent.com/reactjs/ja.react.dev/main/"),
}


def tree(url):
    req = urllib.request.Request(url, headers={"User-Agent": "suikou-research"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return [e["path"] for e in json.load(r)["tree"]]


def main():
    out = pathlib.Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 60
    sources = SOURCES_EN if len(sys.argv) > 3 and sys.argv[3] == "en" else SOURCES
    for name, (api, prefix, raw) in sources.items():
        paths = [p for p in tree(api) if p.startswith(prefix) and p.endswith(".md")][:limit]
        got = 0
        for p in paths:
            dest = out / f"{name}__{p.replace('/', '__')}"
            if dest.exists():
                got += 1; continue
            try:
                with urllib.request.urlopen(raw + urllib.parse.quote(p), timeout=30) as r:
                    dest.write_bytes(r.read())
                got += 1
            except Exception:
                pass
        print(f"{name}: {got} 頁")


if __name__ == "__main__":
    import urllib.parse
    main()
