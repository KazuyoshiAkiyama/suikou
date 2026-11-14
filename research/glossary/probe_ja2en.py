"""PLaMo Translate の日本語から英語への訳を評価する。

D-28 で英語側の文体の規則は実装しないと決めてある。
この評価は規則を足すためではなく、訳の癖を記録するために行う。
日本語の冗長な形が英語でどう処理されるかを見ると、
日本語側の規則に使える手掛かりが出ることがある。

入力は、AI が書いた日本語に現れる冗長な形を並べたものである。
出典は `corpus/baselines.toml` と M 系の規則の説明にある。

使い方:
    plamo-translate server &
    python research/glossary/probe_ja2en.py
"""

import subprocess

# 冗長な形と、その形が何を含んでいるか。
CASES: list[tuple[str, str]] = [
    ("設定の変更を行う。", "機能動詞。行う が意味を担っていない"),
    ("この指標については、しきい値を下げる。", "については。主題の提示が空である"),
    ("本節では、前処理の順序を説明を行います。", "本節では と 説明を行う の重複"),
    ("エラーが発生した場合には、処理を中断するようにする。", "ようにする。指示が間接になっている"),
    ("この値は変更することが可能です。", "ことが可能。できる で足りる"),
    ("当該の設定を適用することにより、動作が変わります。", "当該 と することにより"),
    ("以下の表に示すとおり、3つの方式が存在します。", "存在します。ある で足りる"),
    ("実装を行った結果、性能の改善が見られました。", "行った と 見られました"),
    ("なお、この制約については例外が存在します。", "なお と については の重複"),
    ("パラメータの指定を実施してください。", "実施。指定する で足りる"),
]


def translate(text: str) -> str:
    r = subprocess.run(
        ["plamo-translate", "--from", "Japanese", "--to", "English",
         "--input", text, "--no-stream"],
        capture_output=True, text=True, timeout=240,
    )
    lines = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    return lines[-1] if lines else ""


def main() -> None:
    print(f"{'日本語':<34}{'英語':<52}語数")
    print("-" * 100)
    for ja, note in CASES:
        en = translate(ja)
        print(f"{ja:<34}{en:<52}{len(en.split()):>4}")
        print(f"  {note}")


if __name__ == "__main__":
    main()
