"""日本語の手順書（how-to）を集める。`howto` の閾値を日本語側で実測するために使う。

既存の日本語コーパスは解説書に偏っていて、手順書が少ない。
手順書とは、読み手に順番どおりの作業を示す文書である。
番号付きの手順、〜してください / 〜します の指示、前提条件、
地の文に挟まったシェルのコマンドが目印になる。
API の参照、概念の解説、変更履歴は集めない。

最初に kubernetes/website の日本語訳（content/ja/docs/tasks/）を取る。
それらは content_type: task を持つ手順ページである。
次に GitHub の code search で、日本語のドキュメントディレクトリを探す。
code search の結果は手掛かりにすぎない。
非 Kubernetes のファイルは、手順の目印を 2 つ以上持つものだけを残す。
目印は次のとおり。
- 3 項目以上の番号付きリスト
- コードブロック
- 〜してください / 〜します が 2 回以上
- 手順・前提・準備・インストール・セットアップを含む見出し

質の条件:
- ひらがなとカタカナ（U+3040-U+30FF）が 300 字以上
- 全体が 400 字以上
- `_index.md`、テンプレート、CHANGELOG 系、content_type が concept か reference のものを除く

上限（owner は GitHub の組織またはユーザー）:
- 全体で 200 件まで。目標の範囲に収めるため。
- kubernetes 組織からは 40 件まで。kubernetes/website は単一プロジェクトの文書群が大きく、
  その書き方が手順書という型の性質と取り違えられるのを防ぐため。
  上限に達したら、パスの昇順で先頭から取る（再現性のため無作為にしない）。
- ほかの owner からは 5 件まで。一つのプロジェクトに偏らせないため。

使い方: python fetch_howto_ja.py <出力ディレクトリ>
"""

import base64
import json
import pathlib
import re
import subprocess
import sys
import time
from urllib.parse import quote, quote_plus

K8S_OWNER = "kubernetes"
K8S_REPO = "kubernetes/website"
K8S_PREFIX = "content/ja/docs/tasks/"
CAP_K8S = 40
CAP_OTHER = 5
# 全体の上限。目標の 100 から 200 件に収める。
CAP_TOTAL = 200
MIN_KANA = 300
MIN_TOTAL = 400
MIN_SIGNALS = 2

# code search の手掛かり。日本語の語は URL エンコードして渡す。
QUERIES = [
    "path:docs/ja 手順 extension:md",
    "path:ja 手順 extension:md",
    "path:docs/ja 前提条件 extension:md",
    "path:docs/ja してください extension:md",
    "path:docs/jp 手順 extension:md",
    "path:docs/ja/guide extension:md",
    "path:docs/ja/tutorial extension:md",
    "path:content/ja/docs extension:md",
    "filename:INSTALL.ja.md",
    "filename:setup.ja.md",
]

NUMBERED = re.compile(r"^\s*\d+[.)]\s", re.M)
FENCE = re.compile(r"^```", re.M)
POLITE = re.compile(r"してください|します")
PROC_HEAD = re.compile(r"^#{1,4}\s*.*(手順|前提|準備|インストール|セットアップ)", re.M)
KANA = re.compile(r"[぀-ヿ]")


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


def search(q: str, page: int) -> object:
    # code search は二次制限が厳しい。呼び出しの間を空ける。
    time.sleep(7)
    return gh(f"search/code?q={quote_plus(q)}&per_page=30&page={page}")


def fetch_text(repo: str, path: str) -> str | None:
    blob = gh(f"repos/{repo}/contents/{quote(path)}")
    if not isinstance(blob, dict) or blob.get("encoding") != "base64":
        return None
    try:
        return base64.b64decode(blob["content"]).decode("utf-8", "replace")
    except (KeyError, ValueError):
        return None


def skip_name(path: str) -> bool:
    """手順書ではないファイルを除く。索引、雛形、変更履歴が対象。"""
    name = path.rsplit("/", 1)[-1]
    low = path.lower()
    if name.startswith("_index") or "template" in low:
        return True
    return "changelog" in low or "release" in low or "history" in low


def content_type(src: str) -> str | None:
    m = re.match(r"---\n(.*?)\n---", src, re.S)
    if not m:
        return None
    t = re.search(r"^content_type:\s*(\S+)", m.group(1), re.M)
    return t.group(1) if t else None


def is_text_ok(src: str) -> bool:
    """日本語の地の文が十分にあるか。英語が主の文書を除く。"""
    return len(src) >= MIN_TOTAL and len(KANA.findall(src)) >= MIN_KANA


def procedure_signals(src: str) -> int:
    """手順の目印の数。code search の非 Kubernetes 側でだけ使う。"""
    s = 0
    if len(NUMBERED.findall(src)) >= 3:
        s += 1
    if FENCE.search(src):
        s += 1
    if len(POLITE.findall(src)) >= 2:
        s += 1
    if PROC_HEAD.search(src):
        s += 1
    return s


def save(out: pathlib.Path, owner: str, repo: str, path: str, src: str) -> None:
    # 命名は <owner>__<repo>__<path-with-underscores>.md
    stem = path[:-3] if path.endswith(".md") else path
    stem = re.sub(r"[\s/]+", "_", stem)
    (out / f"{owner}__{repo}__{stem}.md").write_text(src, encoding="utf-8")


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/howto_ja")
    out.mkdir(parents=True, exist_ok=True)

    owner_count: dict[str, int] = {}
    seen: set[tuple[str, str]] = set()
    sizes: list[int] = []
    by_source: dict[str, int] = {}

    def take(owner: str, repo: str, path: str, src: str) -> None:
        save(out, owner, repo, path, src)
        owner_count[owner] = owner_count.get(owner, 0) + 1
        sizes.append(len(src))
        by_source[f"{owner}/{repo}"] = by_source.get(f"{owner}/{repo}", 0) + 1

    # 1. kubernetes/website の日本語の手順ページ。content_type: task のみ。
    meta = gh(f"repos/{K8S_REPO}")
    branch = meta.get("default_branch", "main") if isinstance(meta, dict) else "main"
    tree = gh(f"repos/{K8S_REPO}/git/trees/{branch}?recursive=1")
    paths: list[str] = []
    if isinstance(tree, dict):
        for e in tree.get("tree", []):
            p = e.get("path", "")
            if (
                e.get("type") == "blob"
                and p.startswith(K8S_PREFIX)
                and p.endswith(".md")
                and not skip_name(p)
            ):
                paths.append(p)
    for p in sorted(paths):
        if owner_count.get(K8S_OWNER, 0) >= CAP_K8S or sum(owner_count.values()) >= CAP_TOTAL:
            break
        src = fetch_text(K8S_REPO, p)
        if src is None or content_type(src) != "task" or not is_text_ok(src):
            continue
        take(K8S_OWNER, "website", p, src)
        seen.add((K8S_REPO, p))
    print(f"kubernetes: {owner_count.get(K8S_OWNER, 0)} 件（tree 内 {len(paths)} 件を確認）",
          file=sys.stderr)

    # 2. それ以外の owner。code search の手掛かりから、手順の目印で絞る。
    rejected = {"concept": 0, "reference": 0, "text": 0, "signal": 0}
    for q in QUERIES:
        if sum(owner_count.values()) >= CAP_TOTAL:
            break
        for page in (1, 2):
            if sum(owner_count.values()) >= CAP_TOTAL:
                break
            hits = search(q, page)
            if not isinstance(hits, dict) or not hits.get("items"):
                break
            for item in hits["items"]:
                if sum(owner_count.values()) >= CAP_TOTAL:
                    break
                repo = item["repository"]["full_name"]
                path = item["path"]
                owner = repo.split("/")[0]
                if owner == K8S_OWNER or owner_count.get(owner, 0) >= CAP_OTHER:
                    continue
                if (repo, path) in seen or skip_name(path):
                    continue
                seen.add((repo, path))
                src = fetch_text(repo, path)
                if src is None:
                    continue
                ct = content_type(src)
                if ct in ("concept", "reference"):
                    rejected[ct] += 1
                    continue
                if not is_text_ok(src):
                    rejected["text"] += 1
                    continue
                if procedure_signals(src) < MIN_SIGNALS:
                    rejected["signal"] += 1
                    continue
                take(owner, repo.split("/", 1)[1], path, src)
        print(f"query done: {q} -> 計 {sum(owner_count.values())} 件", file=sys.stderr)

    total = sum(owner_count.values())
    sizes.sort()
    median = sizes[len(sizes) // 2] if sizes else 0
    print(f"howto_ja: {total} 件 / {len(owner_count)} 組織 / 総字数 {sum(sizes)} / 中央値 {median} 字")
    for k, v in sorted(by_source.items(), key=lambda kv: -kv[1]):
        print(f"  {k}: {v}")
    print(f"却下: {rejected}", file=sys.stderr)


if __name__ == "__main__":
    main()
