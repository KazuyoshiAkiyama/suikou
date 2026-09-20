"""Markdown を段落の単位に割り、段落の種類の並びを署名として返す。

英語の原文と日本語訳は、段落の種類の並びが一致するときに限って、
段落どうしを位置で対応づける。訳が原文の古い版から作られている場合など、
並びが食い違う頁は対応づけに使わない。
"""
import re

FRONT = re.compile(r"\A---\n.*?\n---\n", re.S)
COMMENT = re.compile(r"<!--.*?-->", re.S)


def chunks(src: str):
    src = FRONT.sub("", src)
    src = COMMENT.sub("", src)
    out, cur, in_code = [], [], False
    for line in src.split("\n"):
        if line.lstrip().startswith("```"):
            if in_code:
                cur.append(line)
                out.append("\n".join(cur))
                cur, in_code = [], False
                continue
            if cur:
                out.append("\n".join(cur))
            cur, in_code = [line], True
            continue
        if in_code:
            cur.append(line)
            continue
        if not line.strip():
            if cur:
                out.append("\n".join(cur))
                cur = []
            continue
        cur.append(line)
    if cur:
        out.append("\n".join(cur))
    return [c for c in out if c.strip()]


def kind(chunk: str) -> str:
    s = chunk.lstrip()
    if s.startswith("```"):
        return "code"
    if s.startswith("#"):
        return "heading"
    if s.startswith("{{"):
        return "shortcode"
    if s.startswith("|"):
        return "table"
    if re.match(r"(\s*)([-*+]|\d+[.)])\s", s):
        return "list"
    return "prose"


def signature(src: str):
    """ショートコードは訳で出し入れされやすいため、署名から外す。"""
    return [kind(c) for c in chunks(src) if kind(c) != "shortcode"]


def aligned(src: str):
    return [c for c in chunks(src) if kind(c) != "shortcode"]
