"""Kubernetes の英語の原文と日本語訳の対を取ってきて、段落の並びが揃う頁だけを残す。

使い方: python fetch_k8s.py <pairs.txt> <出力ディレクトリ> [上限]
pairs.txt は docs/ から始まる相対パスの一覧とする。
"""
import json, sys, pathlib, urllib.request
from chunks import signature

RAW = "https://raw.githubusercontent.com/kubernetes/website/main/content/{lang}/{path}"


def get(lang, path):
    with urllib.request.urlopen(RAW.format(lang=lang, path=path), timeout=30) as r:
        return r.read().decode("utf-8")


def main():
    pairs = pathlib.Path(sys.argv[1]).read_text().split()
    out = pathlib.Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else len(pairs)
    kept, seen = [], 0
    for p in pairs:
        if len(kept) >= limit:
            break
        seen += 1
        try:
            en, ja = get("en", p), get("ja", p)
        except Exception as e:
            print("取得失敗", p, e, file=sys.stderr); continue
        if signature(en) != signature(ja) or len(signature(en)) < 8:
            continue
        name = p.replace("/", "__")[:-3]
        (out / f"{name}.en.md").write_text(en)
        (out / f"{name}.ja.md").write_text(ja)
        kept.append({"path": p, "chunks": len(signature(en))})
    (out / "index.json").write_text(json.dumps(kept, ensure_ascii=False, indent=1))
    print(f"調べた頁 {seen} / 段落の並びが揃った頁 {len(kept)}")


if __name__ == "__main__":
    main()
