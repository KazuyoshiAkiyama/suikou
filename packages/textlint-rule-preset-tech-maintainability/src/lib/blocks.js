"use strict";

// crates/suikou-core/src/markdown.rs の行ベースの Block 抽出を JS に写した
// もの。
//
// textlint は Markdown を AST（段落・リストなどのノード）として渡すが、
// suikou-core の M1〜M7 は行単位で判定するよう書かれている。同じ行が
// ソフトラップで複数行にまたがる段落を textlint は 1 つの Paragraph
// ノードにまとめるため、AST 側の粒度で判定し直すと Rust 側と件数が
// 一致しない箇所が出る。これを避けるため、このモジュールは textlint の
// AST を経由せず、ソース文字列を直接 Rust と同じ行ベースの手順で
// ブロックへ分ける。判定を Rust 側と一致させることを最優先し、
// textlint のノード情報は使わない。
//
// 前処理の順序は変えてはならない（CLAUDE.md）。フロントマター → コード
// フェンス → バックスラッシュのエスケープ解除 → HTML タグの順。

const RE_FRONTMATTER = /^---\n[\s\S]*?\n---\n/;
const RE_FENCE = /```[\s\S]*?```|~~~[\s\S]*?~~~/g;
const RE_ESCAPE = /\\([.\-+*_#`[\]()<>|])/g;
const RE_HTML = /<[^>]+>/g;

const RE_LINK = /!?\[([^\]]*)\]\([^)]*\)/g;
const RE_CODE_SPAN = /`[^`]*`/g;
const RE_EMPHASIS = /\*\*([^*]*)\*\*|\*([^*]*)\*/g;

const RE_LIST = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/;
const RE_HEADING = /^#{1,6}\s+(.*)$/;

/** 取り除いた範囲を、同じ数の改行に置き換える。行番号を保つため。 */
function blankOut(match) {
    const n = (match.match(/\n/g) || []).length;
    return "\n".repeat(n);
}

/** 前処理。順序を変えてはならない。 */
function preprocess(source) {
    let s = source.replace(RE_FRONTMATTER, blankOut);
    s = s.replace(RE_FENCE, blankOut);
    s = s.replace(RE_ESCAPE, "$1");
    s = s.replace(RE_HTML, blankOut);
    return s;
}

function stripInline(line) {
    let s = line.replace(RE_LINK, "$1");
    s = s.replace(RE_CODE_SPAN, "");
    s = s.replace(RE_EMPHASIS, "$1$2");
    return s.trim();
}

/** Rust の str::lines() と同じ挙動で分割する。末尾の改行1つは空行を生まない。 */
function splitLines(text) {
    if (text === "") {
        return [];
    }
    const lines = text.split("\n");
    if (lines[lines.length - 1] === "") {
        lines.pop();
    }
    return lines;
}

/**
 * @param {string} source
 * @returns {{blocks: Array<object>}}
 */
function parseDocument(source) {
    const text = preprocess(source);
    const lines = splitLines(text);
    const blocks = [];

    for (let i = 0; i < lines.length; i++) {
        const lineNo = i + 1;
        const rawLine = lines[i];
        const trimmed = rawLine.trim();
        if (trimmed === "") {
            continue;
        }

        const heading = RE_HEADING.exec(trimmed);
        if (heading) {
            blocks.push({
                kind: "Heading",
                line: lineNo,
                text: stripInline(heading[1]),
                depth: 0,
                ordered: false,
                raw: rawLine,
            });
            continue;
        }

        const list = RE_LIST.exec(rawLine);
        if (list) {
            const indentSrc = list[1];
            const spaces = (indentSrc.match(/ /g) || []).length;
            const tabs = (indentSrc.match(/\t/g) || []).length;
            const indent = spaces + tabs * 2;
            const marker = list[2];
            blocks.push({
                kind: "ListItem",
                line: lineNo,
                text: stripInline(list[3]),
                depth: Math.floor(indent / 2),
                ordered: !(marker === "-" || marker === "*" || marker === "+"),
                raw: rawLine,
            });
            continue;
        }

        let kind;
        if (trimmed.startsWith("|")) {
            kind = "Table";
        } else if (trimmed.startsWith(">")) {
            kind = "Quote";
        } else {
            kind = "Prose";
        }
        blocks.push({
            kind,
            line: lineNo,
            text: stripInline(trimmed),
            depth: 0,
            ordered: false,
            raw: rawLine,
        });
    }

    return { blocks };
}

/** 連続するリスト項目のまとまりを返す。M1 と M5 が使う。 */
function listBlocks(blocks) {
    const out = [];
    let cur = [];
    for (const b of blocks) {
        if (b.kind === "ListItem") {
            cur.push(b);
        } else if (cur.length > 0) {
            out.push(cur);
            cur = [];
        }
    }
    if (cur.length > 0) {
        out.push(cur);
    }
    return out;
}

/** 指定した行より前にある最後のブロックを返す。M1 の導入文判定が使う。 */
function blockBefore(blocks, line) {
    for (let i = blocks.length - 1; i >= 0; i--) {
        if (blocks[i].line < line) {
            return blocks[i];
        }
    }
    return null;
}

module.exports = {
    preprocess,
    stripInline,
    parseDocument,
    listBlocks,
    blockBefore,
};
