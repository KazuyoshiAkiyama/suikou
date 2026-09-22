"""文書の型を自分で宣言しているコーパスを集める。

Kubernetes の文書はフロントマターに `content_type:` を持つ。
この宣言は文書を書いた側が付けたものであり、こちらの見立てではない。
構造の規則の発火率を測るには、型の宣言を持つコーパスが要る。

使い方: python fetch_typed.py <出力ディレクトリ> [1種あたりの頁数]

出力は `<出力ディレクトリ>/<言語>/<content_type>/` に置く。
"""
import sys
import json
import pathlib
import urllib.request
import urllib.error

TREE = "https://api.github.com/repos/kubernetes/website/git/trees/main?recursive=1"
RAW = "https://raw.githubusercontent.com/kubernetes/website/main/"

# 採取の偏りを避けるため、宣言ごとに満遍なく集める。
# 分野で偏ると、その分野の書き方を型の性質と取り違える。
#
# どの頁から当たるかだけをパスで絞り、型そのものは必ず宣言から読む。
# 木の順に総当たりすると `concepts/` で頁数を使い切って `tasks/` に届かない。
WANT = {
    "task": "tasks/",
    "concept": "concepts/",
    "reference": "reference/",
    "tutorial": "tutorials/",
}


def tree(url: str) -> list[str]:
    req = urllib.request.Request(url, headers={"User-Agent": "suikou"})
    with urllib.request.urlopen(req) as r:
        data = json.load(r)
    return [e["path"] for e in data["tree"] if e["type"] == "blob"]


def content_type(src: str) -> str | None:
    if not src.startswith("---"):
        return None
    head = src.split("\n---", 1)[0]
    for line in head.splitlines():
        if line.startswith("content_type:"):
            return line.split(":", 1)[1].strip().strip('"')
    return None


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/typed")
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 120

    paths = tree(TREE)
    for lang in ("en", "ja"):
        prefix = f"content/{lang}/docs/"
        counts = dict.fromkeys(WANT, 0)
        for want, sub in WANT.items():
            pages = [
                p
                for p in paths
                if p.startswith(prefix + sub) and p.endswith(".md")
            ]
            for p in pages:
                if counts[want] >= limit:
                    break
                try:
                    with urllib.request.urlopen(RAW + p) as r:
                        src = r.read().decode("utf-8", "replace")
                except (urllib.error.HTTPError, urllib.error.URLError):
                    continue
                # 型は宣言から読む。パスは当たりを付けるためだけに使う。
                if content_type(src) != want:
                    continue
                counts[want] += 1
                dest = out / lang / want / p.replace("/", "__")
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text(src, encoding="utf-8")
        print(f"{lang}: " + ", ".join(f"{k}={v}" for k, v in counts.items()))


if __name__ == "__main__":
    main()
