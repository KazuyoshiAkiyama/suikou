"""README の節の構成を測る。`readme` の型の必須の節を外から決めるために使う。

候補の節は standard-readme の様式から採った。採否はここでは決めず、実測で決める。
D-31 と D-32 と同じ手順である。
Rust RFC で必須から外した判断と同じく、ほぼ全件に現れる節だけを必須とする。

見出しの文字列は揺れる。同義語をまとめて数える。
同義語は集めた README の実物から拾った。

使い方: python measure_readmes.py [corpus/cache/readmes]
"""

import re
import sys
import pathlib
from collections import Counter

# 候補の節と、その同義語。standard-readme の節の名前を出発点にした。
SECTIONS: dict[str, list[str]] = {
    "説明": ["about", "overview", "introduction", "what is", "description", "synopsis"],
    "入れ方": ["install", "installation", "setup", "getting started", "quick start",
               "quickstart", "get started", "building", "build"],
    "使い方": ["usage", "using", "example", "examples", "how to use", "basic usage",
               "quick example", "demo"],
    "機能": ["features", "feature", "why", "highlights", "capabilities", "benefits"],
    "文書": ["documentation", "docs", "guide", "guides", "reference", "manual",
             "learn more", "resources"],
    "寄与": ["contributing", "contribute", "development", "developing", "contributors"],
    "許諾": ["license", "licence", "licensing", "copyright"],
    "対応": ["support", "community", "help", "getting help", "discussion", "contact"],
    "目次": ["table of contents", "contents", "toc", "index"],
    "謝辞": ["acknowledgements", "acknowledgments", "thanks", "credits", "sponsors",
             "backers", "related"],
    "状態": ["status", "project status", "roadmap", "changelog", "news", "release notes",
             "what's new", "history", "current status"],
    "試験": ["testing", "tests", "benchmarks", "benchmark", "performance"],
    "安全": ["security", "security policy", "reporting vulnerabilities"],
}


def headings(src: str) -> list[str]:
    """見出しの文字列。コード塊の中は数えない。"""
    src = re.sub(r"(?s)```.*?```", "", src)
    src = re.sub(r"(?s)<!--.*?-->", "", src)
    out = []
    for m in re.finditer(r"(?m)^(#{1,4})\s+(.+?)\s*#*$", src):
        h = m.group(2)
        h = re.sub(r"!\[[^\]]*\]\([^)]*\)", "", h)       # 画像
        h = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", h)   # 連結
        h = re.sub(r"[*_`~:]", "", h)
        h = re.sub(r"<[^>]+>", "", h)
        h = re.sub(r"^[\W\d_]+|[\W_]+$", "", h.strip())
        if h:
            out.append((len(m.group(1)), h.strip().lower()))
    return out


def classify(h: str) -> str | None:
    for name, syns in SECTIONS.items():
        for s in syns:
            # 見出しがその語そのものか、語から始まるか。
            if h == s or h.startswith(s + " ") or h.endswith(" " + s):
                return name
    return None


def main() -> None:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "corpus/cache/readmes")
    files = sorted(root.glob("*.md"))
    if not files:
        print("README が見つからない", file=sys.stderr)
        sys.exit(1)

    present = Counter()
    first = Counter()
    depth1 = Counter()
    order: dict[str, list[int]] = {k: [] for k in SECTIONS}
    hs_total = 0
    unmatched = Counter()

    for p in files:
        hs = headings(p.read_text(encoding="utf-8", errors="replace"))
        hs_total += len(hs)
        seen = []
        for lvl, h in hs:
            name = classify(h)
            if name is None:
                unmatched[h] += 1
                continue
            if name not in seen:
                seen.append(name)
        for name in seen:
            present[name] += 1
            order[name].append(seen.index(name))
        if seen:
            first[seen[0]] += 1

    n = len(files)
    print(f"README {n} 件 / 見出し {hs_total} 個\n")
    print(f"{'節':<8}{'出現':>7}{'率':>7}{'平均の位置':>11}   最初に来た回数")
    rows = sorted(present.items(), key=lambda x: -x[1])
    for name, c in rows:
        pos = sum(order[name]) / len(order[name])
        print(f"{name:<8}{c:>7}{c / n:>7.0%}{pos:>11.1f}   {first.get(name, 0)}")

    print("\n分類できなかった見出しの上位（同義語に足す候補）")
    for h, c in unmatched.most_common(12):
        print(f"  {c:>4}  {h[:50]}")


if __name__ == "__main__":
    main()
