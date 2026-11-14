"""M2（地の文の項目数）の誤検出を測る。

M2 は、箇条書きの導入文が項目の数を述べる形を検出する。
項目を増減すると本文が合わなくなるためである。

現在の実装は地の文に当てているだけで、箇条書きが続くかを見ていない。
箇条書きが続かない説明文で数を述べる場合も検出する。
`tests/golden/input/ja_prose.md` がその例である。

ここでは、発火した箇所の直後に箇条書きが続くかを数える。
続かない箇所は誤検出の候補である。

汚染されていない群で測る。D-44 の測定により、
GitHub から収集した日本語の文書は人間の参照コーパスにならない。
k8s の手順頁と参照コーパスを使う。

使い方: python research/structure/measure_m2.py
"""

import pathlib
import re
import sys

# `rules/mod.rs` の `re_m2_ja` と同じ。写しであり、変更したら両方を直す。
M2_JA = re.compile(
    r"[0-9０-９一二三四五六七八九十]+\s*[つ個点種類]の\s*"
    r"(?:理由|方法|ポイント|点|要素|ステップ|手順|方針|特徴|観点|側面|利点|欠点|課題|原則|要因|種類)"
    r"|(?:以下|次)の\s*[0-9０-９一二三四五六七八九十]+\s*[つ個点]"
    r"|[0-9０-９一二三四五六七八九十]+\s*[つ個]あ(?:る|ります)"
)

LIST = re.compile(r"^\s*(?:[-*+]\s|[0-9０-９]+[.)．）]\s)")
HEADING = re.compile(r"^\s*#{1,6}\s")


def strip_code(lines: list[str]) -> list[str]:
    """コード塊の中は対象にしない。塊の行は空行に替えて行番号を保つ。"""
    out = []
    inside = False
    for ln in lines:
        if ln.lstrip().startswith("```"):
            inside = not inside
            out.append("")
            continue
        out.append("" if inside else ln)
    return out


def followed_by_list(lines: list[str], i: int) -> bool:
    """i 行目の後に、空行をはさんで箇条書きが続くかどうか。

    見出しか別の地の文が先に来た場合は、続いていないとみなす。
    """
    for ln in lines[i + 1 : i + 5]:
        if not ln.strip():
            continue
        if LIST.match(ln):
            return True
        if HEADING.match(ln):
            return False
        # 地の文が続く場合は、導入文ではない。
        return False
    return False


def measure(files: list[pathlib.Path]) -> tuple[int, int, list[str]]:
    with_list = 0
    without = 0
    examples: list[str] = []
    for p in files:
        lines = strip_code(p.read_text(encoding="utf-8", errors="replace").splitlines())
        for i, ln in enumerate(lines):
            if not M2_JA.search(ln):
                continue
            if followed_by_list(lines, i):
                with_list += 1
            else:
                without += 1
                if len(examples) < 8:
                    examples.append(f"{p.name[:28]}:{i + 1}  {ln.strip()[:64]}")
    return with_list, without, examples


def main() -> None:
    groups = {
        "k8s 手順頁（人間訳）": [
            p for p in pathlib.Path("corpus/cache/k8s").glob("*.ja.md")
            if "content_type: task" in p.read_text(encoding="utf-8", errors="replace")
        ],
        "参照コーパス": sorted(pathlib.Path("corpus/cache/pro_ja").glob("*.md")),
        "自リポジトリ": sorted(pathlib.Path("docs/content/ja").glob("*.md")),
    }
    print(f"{'群':<24}{'文書':>6}{'箇条書きが続く':>14}{'続かない':>10}{'誤検出の率':>12}")
    all_examples: list[str] = []
    for name, files in groups.items():
        if not files:
            print(f"{name:<24} 見つからない")
            continue
        w, wo, ex = measure(files)
        total = w + wo
        rate = f"{wo / total:.0%}" if total else "—"
        print(f"{name:<24}{len(files):>6}{w:>14}{wo:>10}{rate:>12}")
        all_examples.extend(ex)
    if all_examples:
        print("\n箇条書きが続かなかった例")
        for e in all_examples[:10]:
            print(f"  {e}")


if __name__ == "__main__":
    main()
