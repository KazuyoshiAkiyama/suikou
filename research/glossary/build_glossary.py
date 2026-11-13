"""用語集を、suikou の外でも使える形に書き出す。

出す形は2つである。

  glossary/suikou-ja.prh.yml   textlint-rule-prh の辞書。表記ゆれを直す道具が読む。
  glossary/suikou-ja.csv       表計算と翻訳支援ツールが読む。

prh を選んだのは、日本語の表記ゆれを直す道具として広く使われており、
その辞書の形がそのまま textlint で動くためである。
CSV は、prh が読めない道具のために置く。

使い方: python research/glossary/build_glossary.py
"""

import csv
import pathlib
import sys

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.10
    import tomli as tomllib  # type: ignore

SRC = pathlib.Path("crates/suikou-core/data/glossary_ja.toml")
OUT = pathlib.Path("glossary")


def load() -> dict:
    return tomllib.loads(SRC.read_text(encoding="utf-8"))


def write_prh(d: dict) -> None:
    """prh の辞書。1つの `expected` に、置き換える `patterns` を並べる。"""
    meta = d["meta"]
    lines = [
        "# suikou の用語集を textlint-rule-prh の辞書にしたもの。",
        "#",
        "# このファイルは生成物である。手で直さず、",
        "# crates/suikou-core/data/glossary_ja.toml を直して",
        "# research/glossary/build_glossary.py を走らせる。",
        "#",
        f"# 計測: {meta['measured']}  参照: {meta['corpus']}（{meta['corpus_chars']:,} 字）",
        f"# 表記: {meta['orthography_rule']}",
        f"# 参考: {meta['orthography_reference']}",
        "#",
        "# 使い方（.textlintrc.yml）:",
        "#   rules:",
        "#     prh:",
        "#       rulePaths: [glossary/suikou-ja.prh.yml]",
        "",
        "version: 1",
        "rules:",
    ]
    for e in d.get("orthography", []):
        lines.append(f"  # {e['rule']}: {e['evidence']}")
        lines.append(f"  - expected: {e['preferred']}")
        lines.append("    patterns:")
        for v in e["variants"]:
            lines.append(f"      - {v}")
    for t in d.get("term", []):
        if not t.get("rejected"):
            continue
        lines.append(f"  # {t['concept']}: {t['evidence']}")
        lines.append(f"  - expected: {t['preferred']}")
        lines.append("    patterns:")
        for v in t["rejected"]:
            lines.append(f"      - {v}")
        if t.get("keep"):
            # prh は除外を書けないため、残す形を注記として残す。
            lines.append(f"    # 文字どおりの意味は残す: {' '.join(t['keep'])}")
    (OUT / "suikou-ja.prh.yml").write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_csv(d: dict) -> None:
    rows = []
    for e in d.get("orthography", []):
        rows.append(
            {
                "kind": "orthography",
                "concept": "",
                "preferred": e["preferred"],
                "avoid": " ".join(e["variants"]),
                "keep": "",
                "basis": e["rule"],
                "evidence": e["evidence"],
            }
        )
    for t in d.get("term", []):
        rows.append(
            {
                "kind": "term",
                "concept": t["concept"],
                "preferred": t["preferred"],
                "avoid": " ".join(t.get("rejected", [])),
                "keep": " ".join(t.get("keep", [])),
                "basis": "measured",
                "evidence": t["evidence"],
            }
        )
    with (OUT / "suikou-ja.csv").open("w", encoding="utf-8", newline="\n") as f:
        # csv の既定は CRLF である。行末を LF にそろえる。
        w = csv.DictWriter(
            f,
            fieldnames=["kind", "concept", "preferred", "avoid", "keep", "basis", "evidence"],
            lineterminator="\n",
        )
        w.writeheader()
        w.writerows(rows)


def main() -> None:
    d = load()
    OUT.mkdir(exist_ok=True)
    write_prh(d)
    write_csv(d)
    n = len(d.get("orthography", [])) + len(d.get("term", []))
    print(f"{n} 件を glossary/ に書き出した", file=sys.stderr)


if __name__ == "__main__":
    main()
