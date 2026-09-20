"""複数のモデルの訳で共通して偏る語だけを残す。

設計の方針では、モデルごとの癖に閾値を持つ規則を置いてはならない。
一つのモデルにしか出ない偏りは、そのモデルの癖である可能性がある。
使い方: python cross.py <キャッシュのディレクトリ> <tag1> <tag2> ...
"""
import subprocess, sys, pathlib, collections, re

ROW = re.compile(r"^\| ([^|]+) \| ([^|]*) \| (\d+) \| (\d+) \| ([\d.]+) \| ([+-][\d.]+) \| (\d+) \| ([^|]*) \|")


def run(d, tag):
    out = subprocess.run([sys.executable, "analyze.py", d, tag], capture_output=True, text=True,
                         cwd=pathlib.Path(__file__).parent).stdout
    rows, on = {}, False
    for line in out.splitlines():
        if line.startswith("## LLM が偏って使う語"): on = True; continue
        if line.startswith("## LLM が避ける"): on = False
        m = ROW.match(line)
        if on and m:
            u = m.group(1).strip()
            rows[u] = dict(pos=m.group(2).strip(), llm=int(m.group(3)), pro=int(m.group(4)),
                           g2=float(m.group(5)), lr=float(m.group(6)), n=int(m.group(7)),
                           sub=m.group(8).strip())
    return rows


def main():
    d, tags = sys.argv[1], sys.argv[2:]
    per = {t: run(d, t) for t in tags}
    common = set.intersection(*(set(v) for v in per.values()))
    print(f"# モデルをまたいで共通する偏り（{', '.join(tags)}）\n")
    print(f"各モデルで検出した語: " + "、".join(f"{t} {len(per[t])}" for t in tags) + f"。共通 {len(common)}\n")
    print("| 語 | 品詞 | " + " | ".join(f"{t} 回数" for t in tags) + " | プロ | プロが代わりに使う語 |")
    print("|---|---|" + "---|" * (len(tags) + 2))
    for u in sorted(common, key=lambda u: -min(per[t][u]["g2"] for t in tags)):
        a = per[tags[0]][u]
        print(f"| {u} | {a['pos']} | " + " | ".join(str(per[t][u]["llm"]) for t in tags) +
              f" | {a['pro']} | {a['sub']} |")


if __name__ == "__main__":
    main()
