"""対の一覧が分野をまたいで通用するかを、プロの日本語訳の頻度で確かめる。

使い方: python validate_pairs.py <プロ訳のディレクトリ> <対の一覧の JSON>
翻訳を伴わないため、LLM を呼ばずに確かめられる。
"""
import json, sys, pathlib, collections
from chunks import aligned, kind
from analyze import units


def counts(paths):
    c = collections.Counter(); chars = 0
    for p in paths:
        src = p.read_text(errors="ignore")
        for ch in aligned(src):
            if kind(ch) == "code":
                continue
            u, _ = units(ch)
            c.update(u); chars += len(ch)
    return c, chars


def main():
    d = pathlib.Path(sys.argv[1])
    pairs = json.loads(pathlib.Path(sys.argv[2]).read_text())
    groups = collections.defaultdict(list)
    for p in sorted(d.glob("*.md")):
        groups[p.name.split("__")[0]].append(p)
    per = {g: counts(ps) for g, ps in groups.items()}
    names = sorted(per)
    print("| LLM 側の語 | プロ側の語 | " + " | ".join(f"{n}（LLM/プロ）" for n in names) + " |")
    print("|---|---|" + "---|" * len(names))
    for llm, pro in pairs:
        cells = []
        for n in names:
            c, _ = per[n]
            cells.append(f"{c[llm]} / {c[pro]}")
        print(f"| {llm} | {pro} | " + " | ".join(cells) + " |")
    print()
    for n in names:
        c, ch = per[n]
        print(f"{n}: {len(groups[n])} 頁、内容語 {sum(c.values())}")


if __name__ == "__main__":
    main()
