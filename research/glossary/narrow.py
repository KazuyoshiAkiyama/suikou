"""語の意味がどれだけ限定されるかを、対訳の散り方で測る。

日英の両方を扱うため、方向を2つ用意する。測るものが違う。

    --ja2en   その日本語が、いくつの英語に分かれるか。日本語の側の広さを見る。
    --en2ja   その英語が、いくつの日本語に分かれるか。揃える先を決めるときに見る。

広い語を使うと、読み手はどの意味かを文脈から埋めることになる。
埋め方を誤ると、そこで誤読が起きる。
置き換え先に別の広い語を選んでも、曖昧さは減らない。

コーパスの文をそのまま訳させると、話題の語が混ざって測れない。
最小限の枠に入れて訳させ、変わるのはその語だけにする。

使い方:
    plamo-translate server &
    python research/glossary/narrow.py --ja2en 置く 配置する 設定する
    python research/glossary/narrow.py --en2ja place configure deploy
"""

import re
import subprocess
import sys
from collections import Counter

CLI = "plamo-translate"

# 最小限の枠。変わるのは入れる語だけにする。
# 名詞は、技術文書でその動詞が受けそうなものを選ぶ。
JA_FRAMES = [
    "設定を{}。",
    "ファイルを{}。",
    "値を{}。",
    "一覧を{}。",
    "規則を{}。",
    "節を{}。",
]
EN_FRAMES = [
    "{} the configuration.",
    "{} the file.",
    "{} the value.",
    "{} the list.",
    "{} the rule.",
    "{} the section.",
]

# 枠に由来する語。これを除いた残りが、測る語の訳である。
JA_FRAME_WORDS = {
    "configuration", "config", "file", "value", "list", "rule", "section",
    "settings", "setting", "the", "a", "an", "to", "of", "in", "on", "and",
    "is", "are", "be", "will", "please", "you", "your", "it", "this", "that",
}
# 枠の名詞は、訳でカタカナになることがある。両方の形を並べる。
EN_FRAME_NOUNS = [
    "設定", "コンフィグ", "構成",
    "ファイル",
    "値", "バリュー",
    "一覧", "リスト",
    "規則", "ルール",
    "節", "セクション",
]
# 丁寧さの語尾。語そのものではないため取り除く。
EN_TAIL = ["してください", "してくださ", "ください", "します", "せよ", "する", "を", "の", "に", "は", "、", "。"]


def translate(text: str, src: str, dst: str) -> str:
    r = subprocess.run(
        [CLI, "--from", src, "--to", dst, "--input", text, "--no-stream"],
        capture_output=True,
        text=True,
        timeout=240,
    )
    lines = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    return lines[-1] if lines else ""


def ja2en(word: str) -> tuple[Counter, int]:
    seen: Counter[str] = Counter()
    n = 0
    for f in JA_FRAMES:
        en = translate(f.format(word), "Japanese", "English").lower()
        if not en:
            continue
        n += 1
        words = [w for w in re.findall(r"[a-z]+", en) if w not in JA_FRAME_WORDS and len(w) > 2]
        # 枠に無い語のうち最初のものを、その語の訳とみなす。
        if words:
            seen[words[0]] += 1
    return seen, n


def en2ja(word: str) -> tuple[Counter, int]:
    """英語の語が、いくつの日本語に分かれるか。

    測る語の訳が枠の名詞と重なると、取り除いた残りが消える。
    `configure` の訳は `設定` であり、枠の `設定を` と重なる。
    その場合は数を出さず、測れないと返す。誤った数を出すより気付ける形にする。
    """
    seen: Counter[str] = Counter()
    n = 0
    for f in EN_FRAMES:
        ja = translate(f.format(word.capitalize()), "English", "Japanese")
        if not ja:
            continue
        n += 1
        # 枠の名詞と丁寧さの語尾を取り除いた残りが、測る語の訳である。
        rest = ja
        for w in EN_FRAME_NOUNS:
            rest = rest.replace(w, "")
        for w in EN_TAIL:
            rest = rest.replace(w, "")
        m = re.findall(r"[ぁ-んァ-ヶ一-龥ー]{2,}", rest)
        if m:
            seen[max(m, key=len)] += 1
    # 半分の枠で訳が取れなければ、枠と重なっているとみなす。
    if sum(seen.values()) * 2 < len(EN_FRAMES):
        return Counter(), 0
    return seen, n


def main() -> None:
    if len(sys.argv) < 3 or sys.argv[1] not in ("--ja2en", "--en2ja"):
        print(__doc__)
        sys.exit(2)
    direction = sys.argv[1]
    words = sys.argv[2:]
    label = "英語" if direction == "--ja2en" else "日本語"
    print(f"{'語':<14}{'枠':>4}{'収束':>6}{'種類':>6}   よく現れた" + label)
    rows = []
    for w in words:
        seen, n = (ja2en if direction == "--ja2en" else en2ja)(w)
        if n == 0:
            print(f"{w:<14}  測れない（訳が枠の名詞と重なる）")
            continue
        top = seen.most_common()
        share = top[0][1] / n if top else 0.0
        rows.append((w, share, len(seen)))
        detail = " ".join(f"{k}×{v}" for k, v in top[:4])
        print(f"{w:<14}{n:>4}{share:>6.0%}{len(seen):>6}   {detail}")
    if rows:
        rows.sort(key=lambda x: -x[1])
        print("\n狭い順: " + " > ".join(f"{w}({s:.0%})" for w, s, _ in rows))


if __name__ == "__main__":
    main()
