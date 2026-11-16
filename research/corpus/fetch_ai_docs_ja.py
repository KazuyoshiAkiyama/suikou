"""Claude Code の帰属が付いたコミットを持つリポジトリから、AI が書いた日本語の Markdown を集める。

前回の収集（fetch_ai_docs.py）は言語を指定せずに 200 件を集め、日本語は 1 件しか得られなかった。
そのため、このスクリプトは日本語の文書が見つかりやすい経路から候補を集める。

候補の集め方は三つある。
1. コミット検索で、帰属の文字列と日本語のコミットメッセージの語を組み合わせる。
   Claude Code は利用者の言語でコミットメッセージを書くため、日本語の利用者の
   リポジトリには日本語のコミットメッセージと帰属が並ぶ。
2. コード検索で、日本語の README（README.ja.md など）を探す。
3. コード検索で、docs/ の下の日本語の散文を探す。

AI が書いた文書かどうかは文体から推測せず、コミット履歴で確かめる。
リポジトリのコミットメッセージに Claude の文字列が含まれる割合が 60% 以上のものだけを採用する。
割合は直近 100 コミットで測る。リポジトリ全体の履歴ではない。

文書の条件は次のとおり。
- 日本語であること。仮名（U+3040–U+30FF）が 300 字以上あること。
  漢字（U+4E00–U+9FFF）が仮名の 10 倍を超えるものは中国語とみなして除く。
  ハングル（U+AC00–U+D7AF）かキリル文字（U+0400–U+04FF）が 200 字を超えるものは除く。
  前回の収集は仮名の有無だけを見たため、韓国語が混ざった。
- エージェントへの指示ファイルではないこと。
  CLAUDE.md、AGENTS.md、.memory/、orchestrator などの語を 3 種類以上含むものは除く。
  読み手ではなくエージェントに宛てた注記は、前回の収集の 21% を占めた。
- 文書の形をしていること。全体で 400 字以上、コードブロックを除いた地の文が 200 字以上。
  ほぼコードか表だけの文書は、文の構成を測る材料にならないためである。
- ファイル名に template、CHANGELOG、LICENSE、CODE_OF_CONDUCT を含むもの、
  node_modules の下のものは除く。

このリポジトリ自体（suikou）は除く。自分の文書を AI 側の材料にすると循環する。
1 リポジトリから取る数は 4 件まで、1 所有者から取る数は 8 件までとする。
上限が無いと、ひとつのプロジェクトの書き方を AI の書き方と取り違える。

二次の制限に当たったら 60 秒待って同じ呼び出しを繰り返す。
コミット検索は 6 秒以上、コード検索は 10 回/分の上限に合わせて 7 秒以上、
その他の呼び出しは 2 秒以上の間隔を置く。

出力は `<所有者>__<リポジトリ>__<パスの / を _ に置き換えたもの>` の名前で書く。
同じディレクトリに provenance.tsv を書き、採用したリポジトリごとに根拠を記録する。

使い方: python3 -I fetch_ai_docs_ja.py <出力ディレクトリ>
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

COMMIT_QUERIES = [
    '"Co-Authored-By: Claude" 追加',
    '"Co-Authored-By: Claude" 修正',
    '"Co-Authored-By: Claude" 実装',
    '"Co-Authored-By: Claude" 対応',
    '"Co-Authored-By: Claude" 更新',
    '"Generated with Claude Code" 更新',
    '"Generated with Claude Code" 追加',
]
CODE_QUERIES = [
    ("readme_ja_dot", "filename:README.ja.md"),
    ("readme_ja_underscore", "filename:README_ja.md"),
    ("docs_kudasai", '"ください" path:docs extension:md'),
    ("docs_jissou", "実装 path:docs extension:md"),
    ("docs_settei", "設定 path:docs extension:md"),
]
SEARCH_PAGES = 3
CANDIDATE_MAX = 400
COMMIT_SLEEP = 6
CODE_SLEEP = 7
API_SLEEP = 2
RATE_SLEEP = 60
RATE_RETRY = 10
SAMPLE = 100
SHARE_MIN = 0.6
PER_REPO = 4
PER_OWNER = 8
TOTAL_MAX = 150
BLOB_MAX_PER_REPO = 30
DOC_MIN = 400
PROSE_MIN = 200
KANA_MIN = 300
HANGUL_MAX = 200
CYRILLIC_MAX = 200
HAN_KANA_RATIO = 10
EXCLUDE_REPO = "suikou"
SKIP_WORDS = ["template", "changelog", "license", "code_of_conduct", "node_modules"]
DOC_DIRS = ("docs", "doc", "documentation")
AGENT_TERMS = ["claude.md", "agents.md", ".memory/", "orchestrator", "sub-agent",
               "subagent", "claude code", "mcp__"]
AGENT_TERM_MAX = 2  # これを超える（3 種類以上）で除く

MARK = re.compile(r"claude", re.I)
KANA = re.compile(r"[぀-ヿ]")
HANGUL = re.compile(r"[가-힯]")
CYRILLIC = re.compile(r"[Ѐ-ӿ]")
HAN = re.compile(r"[一-鿿]")

# 除外の理由。一つの文書が複数の理由に当たる場合は、すべて数える。
CHECKS = ["short_doc", "short_prose", "kana_low", "hangul_high", "cyrillic_high",
          "han_dominant", "agent_instructions"]


def gh(path: str, sleep: float) -> object:
    """gh api を呼ぶ。二次制限に当たったら待って同じ呼び出しを繰り返す。"""
    time.sleep(sleep)
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
        if "rate limit" in err or "403" in err:
            print(f"制限に当たった。{RATE_SLEEP} 秒待つ", file=sys.stderr)
            time.sleep(RATE_SLEEP)
            continue
        print(f"取得失敗: {path[:80]} {out.stderr[:120]}", file=sys.stderr)
        return None
    return None


def is_excluded(full: str) -> bool:
    """このプロジェクト自身を除く。"""
    repo = full.partition("/")[2]
    return repo.lower() == EXCLUDE_REPO or full.lower() == "kazuyoshiakiyama/suikou"


def collect_candidates() -> tuple[dict[str, str], dict[str, set[str]]]:
    """検索の経路ごとに候補を集める。

    戻り値は、候補のリポジトリ名から経路の名前への対応と、
    コード検索で当たったパスの集合である。どちらも後の並べ替えに使う。
    """
    source: dict[str, str] = {}
    hint_paths: dict[str, set[str]] = collections.defaultdict(set)
    for query in COMMIT_QUERIES:
        for page in range(1, SEARCH_PAGES + 1):
            q = urllib.parse.quote(query)
            res = gh(f"search/commits?q={q}&per_page=100&page={page}", COMMIT_SLEEP)
            if not isinstance(res, dict) or not res.get("items"):
                break
            for item in res["items"]:
                full = item["repository"]["full_name"]
                if not is_excluded(full):
                    source.setdefault(full, f"commit:{query}")
            print(f"コミット検索 {query} p{page}: 候補 {len(source)} 件", file=sys.stderr)
            if len(source) >= CANDIDATE_MAX:
                break
    for label, query in CODE_QUERIES:
        for page in range(1, SEARCH_PAGES + 1):
            q = urllib.parse.quote(query)
            res = gh(f"search/code?q={q}&per_page=100&page={page}", CODE_SLEEP)
            if not isinstance(res, dict) or not res.get("items"):
                break
            for item in res["items"]:
                full = item["repository"]["full_name"]
                if is_excluded(full):
                    continue
                source.setdefault(full, f"code:{label}")
                hint_paths[full].add(item["path"])
            print(f"コード検索 {label} p{page}: 候補 {len(source)} 件", file=sys.stderr)
    return source, hint_paths


def claude_share(full: str) -> tuple[int, int] | None:
    """直近 SAMPLE コミットのうち、メッセージに Claude を含むものの数と、取得した数。"""
    commits = gh(f"repos/{full}/commits?per_page={SAMPLE}", API_SLEEP)
    if not isinstance(commits, list) or not commits:
        return None
    hits = sum(1 for c in commits if MARK.search(c["commit"]["message"]))
    return hits, len(commits)


def is_doc_path(path: str) -> bool:
    """ルートの README*.md と、docs/、doc/、documentation/ の下の .md を対象にする。"""
    low = path.lower()
    if any(word in low for word in SKIP_WORDS):
        return False
    if not low.endswith(".md"):
        return False
    parts = path.split("/")
    if len(parts) == 1:
        return low.startswith("readme")
    return parts[0].lower() in DOC_DIRS


def prose_len(src: str) -> int:
    """コードブロックを除いた地の文の字数。"""
    s = re.sub(r"```.*?```", "", src, flags=re.S)
    s = re.sub(r"~~~.*?~~~", "", s, flags=re.S)
    return len(s.strip())


def judge(src: str) -> list[str]:
    """当たった除外の理由をすべて返す。空なら採用する。"""
    failed = []
    if len(src) < DOC_MIN:
        failed.append("short_doc")
    if prose_len(src) < PROSE_MIN:
        failed.append("short_prose")
    kana = len(KANA.findall(src))
    if kana < KANA_MIN:
        failed.append("kana_low")
    if len(HANGUL.findall(src)) > HANGUL_MAX:
        failed.append("hangul_high")
    if len(CYRILLIC.findall(src)) > CYRILLIC_MAX:
        failed.append("cyrillic_high")
    if len(HAN.findall(src)) > HAN_KANA_RATIO * kana:
        failed.append("han_dominant")
    low = src.lower()
    if sum(1 for term in AGENT_TERMS if term in low) > AGENT_TERM_MAX:
        failed.append("agent_instructions")
    return failed


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/ai_docs_ja")
    out.mkdir(parents=True, exist_ok=True)

    source, hint_paths = collect_candidates()
    candidates = list(source)[:CANDIDATE_MAX]
    print(f"候補: {len(candidates)} 件", file=sys.stderr)

    checked = 0
    no_commits = 0
    share_fail = 0
    passed_share: list[str] = []
    check_hits: collections.Counter[str] = collections.Counter()
    reject_first: collections.Counter[str] = collections.Counter()
    provenance: list[tuple[str, int, int, float, int]] = []
    owner_count: collections.Counter[str] = collections.Counter()
    shares_kept: list[float] = []
    sizes: list[int] = []
    total = 0
    kept_source: collections.Counter[str] = collections.Counter()

    for full in candidates:
        if total >= TOTAL_MAX:
            break
        checked += 1
        share = claude_share(full)
        if share is None:
            no_commits += 1
            continue
        hits, n = share
        if hits / n < SHARE_MIN:
            share_fail += 1
            continue
        passed_share.append(full)

        owner, _, repo = full.partition("/")
        info = gh(f"repos/{full}", API_SLEEP)
        branch = info.get("default_branch") if isinstance(info, dict) else None
        tree = gh(f"repos/{full}/git/trees/{branch}?recursive=1", API_SLEEP) if branch else None
        if not isinstance(tree, dict) or "tree" not in tree:
            reject_first["tree_failed"] += 1
            continue
        if tree.get("truncated"):
            print(f"警告: {full} の木は truncated である", file=sys.stderr)

        # 日本語の文書が先に当たるよう、パスに ja を含むものと検索で当たったものを先頭に置く。
        def priority(e: dict) -> tuple[int, str]:
            p = e["path"]
            hinted = p in hint_paths.get(full, set())
            ja_name = bool(re.search(r"(^|[._-])ja([._-]|$)", p.lower()))
            return (0 if (hinted or ja_name) else 1, p)

        docs = sorted(
            (e for e in tree["tree"] if e.get("type") == "blob" and is_doc_path(e["path"])),
            key=priority,
        )[:BLOB_MAX_PER_REPO]
        taken = 0
        for entry in docs:
            if taken >= PER_REPO or owner_count[owner.lower()] >= PER_OWNER or total >= TOTAL_MAX:
                break
            blob = gh(f"repos/{full}/git/blobs/{entry['sha']}", API_SLEEP)
            if not isinstance(blob, dict) or blob.get("encoding") != "base64":
                reject_first["fetch_failed"] += 1
                continue
            try:
                src = base64.b64decode(blob["content"]).decode("utf-8", "replace")
            except (KeyError, ValueError):
                reject_first["fetch_failed"] += 1
                continue
            failed = judge(src)
            for check in failed:
                check_hits[check] += 1
            if failed:
                reject_first[failed[0]] += 1
                continue
            name = f"{owner}__{repo}__{entry['path'].replace('/', '_')}"
            target = out / name
            if target.exists():
                reject_first["name_collision"] += 1
                continue
            target.write_text(src, encoding="utf-8")
            sizes.append(len(src))
            taken += 1
            total += 1
            owner_count[owner.lower()] += 1

        if taken:
            kept_source[source[full]] += 1
            shares_kept.append(hits / n)
            provenance.append((full, hits, n, hits / n, taken))
        print(
            f"{'採用' if taken else '文書なし'} {full}: 割合 {hits}/{n}, 文書 {taken} 件 (合計 {total})",
            file=sys.stderr,
        )

    with (out / "provenance.tsv").open("w", encoding="utf-8") as f:
        for full, hits, n, ratio, taken in provenance:
            f.write(f"{full}\t{hits}\t{n}\t{ratio:.2f}\t{taken}\n")

    print(f"確認した候補: {checked} 件（候補総数 {len(candidates)}）")
    print(f"コミットなし: {no_commits} / 60% 未満: {share_fail} / 60% を満たした: {len(passed_share)}")
    print("文書の除外（一つの文書が複数に当たれば、それぞれ数える）: "
          + ", ".join(f"{k}={check_hits[k]}" for k in CHECKS))
    print("最初に当たった除外の理由: " + ", ".join(f"{k}={v}" for k, v in sorted(reject_first.items())))
    print(f"採用リポジトリ: {len(provenance)} 件 / 文書: {total} 件")
    if shares_kept:
        print("採用リポジトリの割合: " + " ".join(f"{s:.2f}" for s in sorted(shares_kept)))
    print("経路別の採用リポジトリ: " + ", ".join(f"{k}={v}" for k, v in kept_source.most_common()))
    if sizes:
        print(f"文字数の合計: {sum(sizes)} / 中央値: {int(statistics.median(sizes))}")


if __name__ == "__main__":
    main()
