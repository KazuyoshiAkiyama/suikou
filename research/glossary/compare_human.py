"""人間の翻訳家の訳と PLaMo Translate の訳を、同じ原文に対して比べる。

人間の訳を正とする。PLaMo を用語の権威として使えるかを決めるために測る。

`corpus/cache/k8s/` は、同じ頁の英語原文（`.en.md`）と
Kubernetes の翻訳チームによる日本語訳（`.ja.md`）を対で持つ。
見出しが1対1で並ぶため、見出しで区切って段落の順で突き合わせられる。

突き合わせるのは語である。文の似ている度合いは測らない。
訳文の良し悪しは一意に決まらないが、「どの語を当てたか」は一致を数えられる。

使い方:
    plamo-translate server &
    python research/glossary/compare_human.py [対の数]
"""

import pathlib
import re
import subprocess
import sys
from collections import Counter

CLI = "plamo-translate"
ROOT = pathlib.Path("corpus/cache/k8s")

# 判定する語。英語の術語と、日本語で取りうる書き方を並べる。
# どれが出たかを数えるため、取りうる形をすべて挙げる。
TERMS: dict[str, list[str]] = {
    "version": ["バージョン", "版"],
    "branch": ["ブランチ", "枝"],
    "baseline": ["ベースライン", "基準値", "基準線"],
    "container": ["コンテナ", "コンテナー"],
    "cluster": ["クラスター", "クラスタ"],
    "node": ["ノード", "節点"],
    "user": ["ユーザー", "ユーザ", "利用者"],
    "server": ["サーバー", "サーバ"],
    "parameter": ["パラメーター", "パラメータ", "媒介変数"],
    "memory": ["メモリ", "メモリー", "記憶"],
    "directory": ["ディレクトリ", "ディレクトリー"],
    "property": ["プロパティ", "プロパティー", "属性"],
    "resource": ["リソース", "資源"],
    "label": ["ラベル", "札"],
    "log": ["ログ", "記録"],
    "scale": ["スケール", "規模"],
}


def strip(md: str) -> str:
    md = re.sub(r"(?s)\A---\n.*?\n---\n", "", md)
    md = re.sub(r"(?s)```.*?```", "", md)
    md = re.sub(r"\{\{[<%].*?[%>]\}\}", "", md)
    md = re.sub(r"\{#[^}]*\}", "", md)
    return md


def sections(md: str) -> list[tuple[str, list[str]]]:
    """見出しごとに、その下の段落を返す。"""
    out: list[tuple[str, list[str]]] = []
    head = ""
    buf: list[str] = []
    for block in re.split(r"\n\s*\n", strip(md)):
        b = block.strip()
        if not b:
            continue
        if b.startswith("#"):
            out.append((head, buf))
            head = b.lstrip("# ").strip()
            buf = []
        elif not b.startswith(("-", "*", "|", ">")):
            buf.append(" ".join(b.split()))
    out.append((head, buf))
    return [(h, p) for h, p in out if p]


def pairs(limit: int) -> list[tuple[str, str]]:
    """原文と人間訳の段落の対。見出しと段落の順が一致する箇所だけを採る。"""
    out = []
    for en_path in sorted(ROOT.glob("*.en.md")):
        ja_path = en_path.with_name(en_path.name.replace(".en.md", ".ja.md"))
        if not ja_path.exists():
            continue
        es = sections(en_path.read_text(encoding="utf-8", errors="replace"))
        js = sections(ja_path.read_text(encoding="utf-8", errors="replace"))
        if len(es) != len(js):
            continue
        for (_, ep), (_, jp) in zip(es, js):
            if len(ep) != len(jp):
                continue
            for e, j in zip(ep, jp):
                # 長すぎる段落は訳が分かれるため採らない
                if 40 <= len(e) <= 300:
                    out.append((e, j))
        if len(out) >= limit:
            break
    return out[:limit]


def translate(text: str) -> str:
    r = subprocess.run(
        [CLI, "--from", "English", "--to", "Japanese", "--input", text, "--no-stream"],
        capture_output=True,
        text=True,
        timeout=240,
    )
    lines = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    return lines[-1] if lines else ""


def picked(ja: str, forms: list[str]) -> str | None:
    """その日本語に現れた形。長い形から見て、部分一致の取り違えを避ける。"""
    for f in sorted(forms, key=len, reverse=True):
        if f in ja:
            return f
    return None


def main() -> None:
    limit = int(sys.argv[1]) if len(sys.argv) > 1 else 60
    ps = pairs(limit)
    print(f"対訳の段落 {len(ps)} 組を比べる", file=sys.stderr)

    agree: Counter[str] = Counter()
    differ: Counter[str] = Counter()
    examples: list[str] = []
    for i, (en, human) in enumerate(ps, 1):
        hits = [(t, f) for t, f in TERMS.items() if any(x in human for x in f)]
        if not hits:
            continue
        plamo = translate(en)
        for term, forms in hits:
            h = picked(human, forms)
            p = picked(plamo, forms)
            if p is None:
                differ[f"{term}: 人間={h} / PLaMo=出さない"] += 1
            elif h == p:
                agree[term] += 1
            else:
                differ[f"{term}: 人間={h} / PLaMo={p}"] += 1
                if len(examples) < 6:
                    examples.append(f"  EN   {en[:90]}\n  人間 {human[:70]}\n  PLaMo {plamo[:70]}")
        if i % 10 == 0:
            print(f"  {i}/{len(ps)}", file=sys.stderr)

    a, d = sum(agree.values()), sum(differ.values())
    print(f"\n一致 {a} / 不一致 {d}（一致率 {a * 100 / max(a + d, 1):.1f}%）\n")
    print("一致した語:", ", ".join(f"{k}×{v}" for k, v in agree.most_common()))
    print("\n不一致:")
    for k, v in differ.most_common(14):
        print(f"  {v:>3}  {k}")
    if examples:
        print("\n例:")
        print("\n\n".join(examples))


if __name__ == "__main__":
    main()
