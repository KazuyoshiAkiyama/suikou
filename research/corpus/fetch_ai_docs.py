"""Claude Code の帰属が付いたコミットを持つリポジトリから、AI が書いた Markdown を集める。

AI が書いた文書かどうかは文体から推測せず、コミット履歴で確かめる。
リポジトリのコミットメッセージに `Co-Authored-By: Claude` や
`Generated with Claude Code` が付いている割合が 60% 以上のものだけを採用する。

割合は直近 100 コミットで測る。リポジトリ全体の履歴ではない。
直近の作業は慣れた書き方に寄るため、それでも全体の傾向を十分に代表するとは言えない。

文書は README.md と docs/、doc/、documentation/ の下の `.md` から取る。
短すぎるもの（400 字未満）と、コードブロックを除くと地の文が 200 字未満のものは除く。
ほぼコードか表だけの文書は、文の構成を測る材料にならないためである。

1 リポジトリから取る数と、1 所有者から取る数に上限を置く。
上限が無いと、ひとつのプロジェクトの書き方を AI の書き方と取り違える。

このリポジトリ自体（suikou）は除く。自分の文書を AI 側の材料にすると循環する。

二次の制限に当たったら 60 秒待って同じ呼び出しを繰り返す。
コミット検索は他の呼び出しより強く制限されるため、呼び出し間隔も分けてある。

出力は `<所有者>__<リポジトリ>__<パスの / を _ に置き換えたもの>.md` とする。
あわせて provenance.tsv に、採用したリポジトリごとの根拠を書く。

使い方: python fetch_ai_docs.py <出力ディレクトリ>
"""

import base64
import collections
import json
import pathlib
import re
import statistics
import subprocess
import sys
import time
import urllib.parse

QUERIES = ['"Co-Authored-By: Claude"', '"Generated with Claude Code"']
SEARCH_PAGES = 10
CANDIDATE_MAX = 300
SEARCH_SLEEP = 6
API_SLEEP = 2
RATE_SLEEP = 60
RATE_RETRY = 5
SAMPLE = 100
SHARE_MIN = 0.6
PER_REPO = 4
PER_OWNER = 8
TOTAL_MAX = 200
DOC_MIN = 400
PROSE_MIN = 200
KANA_MIN = 100
EXCLUDE_REPO = "suikou"
SKIP_WORDS = ["template", "changelog", "license", "code_of_conduct", "node_modules"]
DOC_DIRS = ("docs", "doc", "documentation")

KANA = re.compile(r"[぀-ヿ]")
MARK = re.compile(r"claude", re.I)


def gh(path: str, kind: str) -> object:
    """gh api を呼ぶ。検索は間隔を長く取り、二次制限に当たったら待って繰り返す。"""
    time.sleep(SEARCH_SLEEP if kind == "search" else API_SLEEP)
    for _ in range(RATE_RETRY + 1):
        out = subprocess.run(
            ["gh", "api", "-X", "GET", path],
            capture_output=True,
            text=True,
            check=False,
        )
        if out.returncode == 0:
            try:
                return json.loads(out.stdout)
            except json.JSONDecodeError:
                return None
        err = (out.stderr + out.stdout).lower()
        if "rate limit" in err:
            print(f"制限に当たった。{RATE_SLEEP} 秒待つ", file=sys.stderr)
            time.sleep(RATE_SLEEP)
            continue
        return None
    return None


def is_excluded(full: str) -> bool:
    """このプロジェクト自身を除く。"""
    owner, _, repo = full.partition("/")
    return repo.lower() == EXCLUDE_REPO or full.lower() == "kazuyoshiakiyama/suikou"


def collect_candidates() -> list[str]:
    """検索で見つかったリポジトリを、出現順に重複なく返す。"""
    seen: dict[str, None] = {}
    for query in QUERIES:
        for page in range(1, SEARCH_PAGES + 1):
            q = urllib.parse.quote(query)
            res = gh(f"search/commits?q={q}&per_page=100&page={page}", "search")
            if not isinstance(res, dict) or not res.get("items"):
                break
            for item in res["items"]:
                full = item["repository"]["full_name"]
                if not is_excluded(full):
                    seen.setdefault(full, None)
            print(f"検索 {query} p{page}: 候補 {len(seen)} 件", file=sys.stderr)
            if len(seen) >= CANDIDATE_MAX:
                return list(seen)[:CANDIDATE_MAX]
    return list(seen)[:CANDIDATE_MAX]


def claude_share(full: str) -> tuple[int, int] | None:
    """直近 SAMPLE コミットのうち、メッセージに Claude を含むものの数と、取得した数。"""
    commits = gh(f"repos/{full}/commits?per_page={SAMPLE}", "api")
    if not isinstance(commits, list) or not commits:
        return None
    hits = sum(1 for c in commits if MARK.search(c["commit"]["message"]))
    return hits, len(commits)


def is_doc_path(path: str) -> bool:
    """README.md と、docs/、doc/、documentation/ の下の .md を対象にする。"""
    low = path.lower()
    if any(word in low for word in SKIP_WORDS):
        return False
    if path == "README.md":
        return True
    parts = path.split("/")
    return low.endswith(".md") and len(parts) > 1 and parts[0].lower() in DOC_DIRS


def prose_len(src: str) -> int:
    """コードブロックを除いた地の文の字数。"""
    s = re.sub(r"```.*?```", "", src, flags=re.S)
    s = re.sub(r"~~~.*?~~~", "", s, flags=re.S)
    return len(s.strip())


def judge(src: str) -> str | None:
    """除外の理由を返す。採用するなら None を返す。"""
    if len(src) < DOC_MIN:
        return "short_doc"
    if prose_len(src) < PROSE_MIN:
        return "short_prose"
    return None


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/ai_docs")
    out.mkdir(parents=True, exist_ok=True)

    candidates = collect_candidates()
    print(f"候補: {len(candidates)} 件", file=sys.stderr)

    checked = 0
    skipped_no_commits = 0
    rejected_share = 0
    kept_repos: list[tuple[str, int, int]] = []
    provenance: list[tuple[str, int, int, float, int]] = []
    reasons: collections.Counter[str] = collections.Counter()
    owner_count: collections.Counter[str] = collections.Counter()
    total = 0
    lang_count: collections.Counter[str] = collections.Counter()
    sizes: list[int] = []

    for full in candidates:
        if total >= TOTAL_MAX:
            break
        checked += 1
        share = claude_share(full)
        if share is None:
            skipped_no_commits += 1
            continue
        hits, n = share
        if hits / n < SHARE_MIN:
            rejected_share += 1
            continue

        owner, _, repo = full.partition("/")
        taken = 0
        info = gh(f"repos/{full}", "api")
        branch = info.get("default_branch") if isinstance(info, dict) else None
        tree = gh(f"repos/{full}/git/trees/{branch}?recursive=1", "api") if branch else None
        if not isinstance(tree, dict) or "tree" not in tree:
            provenance.append((full, hits, n, hits / n, 0))
            kept_repos.append((full, hits, n))
            continue
        if tree.get("truncated"):
            print(f"警告: {full} の木は truncated である", file=sys.stderr)

        docs = sorted(
            (e for e in tree["tree"] if e.get("type") == "blob" and is_doc_path(e["path"])),
            key=lambda e: e["path"],
        )
        for entry in docs:
            if taken >= PER_REPO or owner_count[owner.lower()] >= PER_OWNER or total >= TOTAL_MAX:
                break
            blob = gh(f"repos/{full}/git/blobs/{entry['sha']}", "api")
            if not isinstance(blob, dict) or blob.get("encoding") != "base64":
                reasons["fetch_failed"] += 1
                continue
            try:
                src = base64.b64decode(blob["content"]).decode("utf-8", "replace")
            except (KeyError, ValueError):
                reasons["fetch_failed"] += 1
                continue
            reason = judge(src)
            if reason:
                reasons[reason] += 1
                continue
            name = f"{owner}__{repo}__{entry['path'].replace('/', '_')}"
            target = out / name
            if target.exists():
                reasons["name_collision"] += 1
                continue
            target.write_text(src, encoding="utf-8")
            lang = "ja" if len(KANA.findall(src)) >= KANA_MIN else "en"
            lang_count[lang] += 1
            sizes.append(len(src))
            taken += 1
            total += 1
            owner_count[owner.lower()] += 1

        kept_repos.append((full, hits, n))
        provenance.append((full, hits, n, hits / n, taken))
        print(
            f"採用 {full}: 割合 {hits}/{n}, 文書 {taken} 件 (合計 {total})",
            file=sys.stderr,
        )

    with (out / "provenance.tsv").open("w", encoding="utf-8") as f:
        for full, hits, n, ratio, taken in provenance:
            f.write(f"{full}\t{hits}\t{n}\t{ratio:.2f}\t{taken}\n")

    print(f"確認した候補: {checked} 件 / 60% を満たした: {len(kept_repos)} 件")
    print(f"取得できなかった（コミットなし）: {skipped_no_commits} / 割合不足: {rejected_share}")
    shares = sorted(r[1] / r[2] for r in kept_repos)
    if shares:
        print(f"採用リポジトリの割合: 最小 {shares[0]:.2f} / 中央 {statistics.median(shares):.2f} / 最大 {shares[-1]:.2f}")
    print(f"文書: {total} 件 (ja {lang_count['ja']} / en {lang_count['en']})")
    if sizes:
        print(f"文字数の合計: {sum(sizes)} / 中央値: {int(statistics.median(sizes))}")
    print("除外理由（重複なし）: " + ", ".join(f"{k}={v}" for k, v in sorted(reasons.items())))


if __name__ == "__main__":
    main()
