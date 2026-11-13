"""英語の術語を PLaMo Translate に訳させて、定着した日本語の言い方を集める。

用語集に語を載せる前の証拠を取るために使う。
人が候補を挙げ、採否はここで取った訳語と参照コーパスの頻度が決める。
D-26 と同じ手順である。

語を単独で訳させると文脈が足りず当てにならないため、
中立な短い文に埋めて訳させ、複数の文で同じ語が出るかを見る。

常駐を先に立てておくと、モデルの読み込みが一度で済む。

    export PLAMO_TRANSLATE_CLI_LLAMA_SERVER=<llama-server のパス>
    plamo-translate server &
    python research/glossary/probe_plamo.py

PLaMo Translate のモデルは PLaMo Community License で配られている。
この手順はモデルの出力を証拠として読むだけで、出力そのものを文書に貼らない。
"""

import json
import re
import subprocess
import sys

CLI = "plamo-translate"

# 候補の英語の術語と、それを埋める中立な文。
# 文は、その語が技術文書でふつうに現れる形に近いものを選ぶ。
PROBES: dict[str, list[str]] = {
    "version": [
        "Pin the dependency to a known version.",
        "The version is recorded in the lockfile.",
    ],
    "branch": [
        "Create a branch before changing the schema.",
        "Merge the branch after the tests pass.",
    ],
    "baseline": [
        "Compare the result against the baseline.",
        "The baseline was measured on the reference corpus.",
    ],
    "root cause": [
        "The root cause was a missing index.",
        "Record the root cause in the incident report.",
    ],
    "background job": [
        "The report runs as a background job.",
        "Queue the export as a background job.",
    ],
    "time window": [
        "The metric is averaged over a time window.",
        "Widen the time window to reduce noise.",
    ],
    "watcher": [
        "A watcher restarts the process when the file changes.",
        "The watcher reports the new state.",
    ],
    "title": [
        "Write the title of the pull request first.",
        "The title of the section names its subject.",
    ],
    "head": [
        "The head of the list holds the newest entry.",
        "Reset the working tree to the head of the branch.",
    ],
    "mark": [
        "The lint leaves a mark on every finding.",
        "Each position carries a mark in the report.",
    ],
    "script": [
        "Run the migration script once.",
        "The script reads the configuration from disk.",
    ],
    "server": [
        "Restart the server after changing the configuration.",
        "The server answers on the loopback address.",
    ],
    "user": [
        "The user supplies the API key.",
        "Grant the user read access to the bucket.",
    ],
    "parameter": [
        "Pass the timeout as a parameter.",
        "Each parameter has a default value.",
    ],
    "folder": [
        "Put the archive in the download folder.",
        "The folder holds one file per target.",
    ],
    "cluster": [
        "The cluster runs three replicas.",
        "Drain the node before removing it from the cluster.",
    ],
    "browser": [
        "Open the report in a browser.",
        "The browser caches the stylesheet.",
    ],
    "computer": [
        "Install the binary on the build computer.",
        "The computer has eight cores.",
    ],
}


def translate(text: str) -> str:
    out = subprocess.run(
        [CLI, "--from", "English", "--to", "Japanese", "--input", text, "--no-stream"],
        capture_output=True,
        text=True,
        timeout=180,
    )
    lines = [l.strip() for l in out.stdout.splitlines() if l.strip()]
    return lines[-1] if lines else ""


def katakana_runs(text: str) -> list[str]:
    """訳文に出たカタカナの連なり。語の選び方を読むために拾う。"""
    return re.findall(r"[ァ-ヶー]{3,}", text)


def main() -> None:
    result = {}
    for term, carriers in PROBES.items():
        rows = []
        for c in carriers:
            ja = translate(c)
            rows.append({"en": c, "ja": ja, "katakana": katakana_runs(ja)})
            print(f"{term:<16} {ja}", flush=True)
        result[term] = rows
    out = "research/glossary/plamo_probe.json"
    with open(out, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2)
    print(f"\n{out} に書いた", file=sys.stderr)


if __name__ == "__main__":
    main()
