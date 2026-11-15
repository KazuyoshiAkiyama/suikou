"""Kubernetes Enhancement Proposal (KEP) を集める。`kep` の型を実物と照合するために使う。

KEP は kubernetes/enhancements の `keps/sig-<name>/<NNNN>-<slug>/README.md` にある。
テンプレートを手本にした文書の節が、実際の KEP にどれだけ現れるかを測る。
テンプレートの節の並びを正解とみなすと、書かれていない節まで必須と取り違える。

未記入の KEP は集めない。テンプレートの指示文がそのまま残っていると、
節が見出しとして存在するだけで、誰も書いていないものを数えることになる。
次の条件のどれかに当たるものは除く。

- `<!--` が 8 回を超えて現れる（テンプレートの指示コメントが大量に残っている）
- 雛形の文字列の出現が合計 2 回を超える。対象は `NNNN`、
  `Your short, descriptive title`、`[ ] (R) Enhancement issue in release milestone`、`TODO`
- HTML コメントとコードブロックを除いた本文が 2000 字未満

`keps/kep-template` を含むパスと `keps/NNNN-*` は対象外とする。

SIG ごとに採用する数に上限を置く（1 SIG 25 件まで）。
上限が無いと、大きな SIG（sig-node など）の書き方を KEP の形そのものと取り違える。
上限は採用数に掛ける。除外されたものは数えず、列挙順に次の候補を見る。
列挙順は slug の辞書順であり、選択に意図を含めない。

出力は `<sig>__<nnnn-slug>.md` の名前にする。元の場所を後から辿れるようにするためである。

使い方: python fetch_keps.py <出力ディレクトリ>
"""

import base64
import collections
import json
import pathlib
import re
import subprocess
import sys

REPO = "kubernetes/enhancements"
PER_SIG = 25
COMMENT_MAX = 8
PLACEHOLDERS = [
    "NNNN",
    "Your short, descriptive title",
    "[ ] (R) Enhancement issue in release milestone",
    "TODO",
]
PLACEHOLDER_MAX = 2
BODY_MIN = 2000

KEP_PATH = re.compile(r"^keps/(sig-[^/]+)/([^/]+)/README\.md$")


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


def list_keps() -> dict[str, list[tuple[str, str]]]:
    """木を一度だけ取り、SIG ごとの README のパスを返す。

    木が切り詰められていると一部の KEP が漏れるため、その場合は警告を出す。
    """
    tree = gh(f"repos/{REPO}/git/trees/master?recursive=1")
    if not isinstance(tree, dict) or "tree" not in tree:
        sys.exit("trees API の応答を取れなかった")
    if tree.get("truncated"):
        print("警告: trees API の応答は truncated である。一部の KEP が漏れている", file=sys.stderr)
    groups: dict[str, list[tuple[str, str]]] = collections.defaultdict(list)
    for entry in tree["tree"]:
        path = entry.get("path", "")
        if entry.get("type") != "blob" or "kep-template" in path:
            continue
        m = KEP_PATH.match(path)
        if not m or m.group(2).startswith("NNNN-"):
            continue
        groups[m.group(1)].append((path, m.group(2)))
    for sig in groups:
        groups[sig].sort(key=lambda pair: pair[1])
    return groups


def body_text(src: str) -> str:
    """HTML コメントとコードブロックを除いた本文。"""
    s = re.sub(r"<!--.*?-->", "", src, flags=re.S)
    s = re.sub(r"```.*?```", "", s, flags=re.S)
    s = re.sub(r"~~~.*?~~~", "", s, flags=re.S)
    return s


def judge(src: str) -> list[str]:
    """除外の理由を全部返す。空なら採用する。"""
    reasons = []
    if src.count("<!--") > COMMENT_MAX:
        reasons.append("comment_block")
    if sum(src.count(p) for p in PLACEHOLDERS) > PLACEHOLDER_MAX:
        reasons.append("placeholder")
    if len(body_text(src).strip()) < BODY_MIN:
        reasons.append("short_body")
    return reasons


def main() -> None:
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/keps")
    out.mkdir(parents=True, exist_ok=True)

    groups = list_keps()
    print(f"候補: {sum(len(v) for v in groups.values())} 件 / {len(groups)} SIG", file=sys.stderr)

    # 除外理由ごとの件数。1 件で複数の理由に当たることがあるため、合計は候補数と一致しない。
    rejected: collections.Counter[str] = collections.Counter()
    fetch_failed = 0
    evaluated = 0
    per_sig: dict[str, int] = {}
    total = 0

    for sig in sorted(groups):
        kept = 0
        for path, slug in groups[sig]:
            if kept >= PER_SIG:
                break
            blob = gh(f"repos/{REPO}/contents/{path}")
            if not isinstance(blob, dict) or blob.get("encoding") != "base64":
                fetch_failed += 1
                continue
            try:
                src = base64.b64decode(blob["content"]).decode("utf-8", "replace")
            except (KeyError, ValueError):
                fetch_failed += 1
                continue
            evaluated += 1
            reasons = judge(src)
            if reasons:
                rejected.update(reasons)
                continue
            (out / f"{sig}__{slug}.md").write_text(src, encoding="utf-8")
            kept += 1
        per_sig[sig] = kept
        total += kept

    for sig, n in sorted(per_sig.items(), key=lambda kv: -kv[1]):
        print(f"{sig}: {n}")
    print(f"評価: {evaluated} 件 / 取得失敗: {fetch_failed} 件")
    print("除外理由（重複あり）: " + ", ".join(f"{k}={v}" for k, v in sorted(rejected.items())))
    print(f"keps: {total} 件 / {sum(1 for n in per_sig.values() if n)} SIG")


if __name__ == "__main__":
    main()
