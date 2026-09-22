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
// フェンス → コードを持つ Hugo のショートコード → バックスラッシュの
// エスケープ解除 → HTML タグの順。

const RE_FRONTMATTER = /^---\n[\s\S]*?\n---\n/;
const RE_ESCAPE = /\\([.\-+*_#`[\]()<>|])/g;
const RE_HTML = /<[^>]+>/g;

const RE_LINK = /!?\[([^\]]*)\]\([^)]*\)/g;
const RE_CODE_SPAN = /`[^`]*`/g;
const RE_EMPHASIS = /\*\*([^*]*)\*\*|\*([^*]*)\*/g;

const RE_LIST = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/;
// 見出しの字下げは3桁までとする。CommonMark がそう定めている。
// 4桁以上の字下げはコードであり、その中の `#` を見出しとして読んではならない。
const RE_HEADING = /^ {0,3}(#{1,6})\s+(.*)$/;

/** 取り除いた範囲を、同じ数の改行に置き換える。行番号を保つため。 */
function blankOut(match) {
    const n = (match.match(/\n/g) || []).length;
    return "\n".repeat(n);
}

/** 指定した行（1 始まり）を、同じ数の改行に置き換える。 */
function blankLines(source, drop) {
    const set = new Set(drop);
    return source
        .split("\n")
        .map((l, i) => (set.has(i + 1) ? "" : l))
        .join("\n");
}

/** 囲みのコードブロックの開きなら、その記号と長さを返す。 */
function fenceOpen(line) {
    const t = line.replace(/^\s+/, "");
    if (line.length - t.length > 3) {
        return null;
    }
    const c = t[0];
    if (c !== "`" && c !== "~") {
        return null;
    }
    let n = 0;
    while (t[n] === c) {
        n += 1;
    }
    return n >= 3 ? { c, n } : null;
}

/**
 * 囲みのコードブロックが占める行を、1 始まりで返す。
 *
 * 閉じは、開きと同じ記号で、開き以上の長さで、そのあとに何も無い行だけとする。
 * 短い囲みを長い囲みの中に入れる書き方が実際にあり、
 * 内側の囲みで閉じたことにすると、そこから先のコードが本文として読まれる。
 */
function codeFenceLines(source) {
    const lines = source.split("\n");
    const out = [];
    let open = null;
    for (let i = 0; i < lines.length; i += 1) {
        if (open) {
            out.push(i + 1);
            const t = lines[i].trim();
            if (t.length >= open.n && t.split("").every((x) => x === open.c)) {
                open = null;
            }
        } else {
            const f = fenceOpen(lines[i]);
            if (f) {
                open = f;
                out.push(i + 1);
            }
        }
    }
    return out;
}

// 中身がコードである Hugo のショートコード。
// note や caution は地の文を包むため対象にしない。取り除くと本文が消える。
const CODE_SHORTCODES = ["highlight", "mermaid"];

/** コードを持つショートコードの開きなら、その名前を返す。 */
function shortcodeOpen(line) {
    const t = line.replace(/^\s+/, "");
    if (!t.startsWith("{{<")) {
        return null;
    }
    const rest = t.slice(3).replace(/^\s+/, "");
    const named = CODE_SHORTCODES.find((n) => rest.startsWith(n));
    if (named) {
        return named;
    }
    // tab は名前では決まらない。codelang を宣言したものだけがコードを持つ。
    return rest.startsWith("tab ") && rest.includes("codelang") ? "tab" : null;
}

/** 閉じは行の途中にも現れる。行の先頭に限定しない。 */
function hasShortcodeClose(line, name) {
    return line
        .split("{{<")
        .slice(1)
        .some((t) => {
            const r = t.replace(/^\s+/, "");
            return r.startsWith("/") && r.slice(1).replace(/^\s+/, "").startsWith(name);
        });
}

/**
 * コードを持つショートコードの中身を、同じ数の改行に置き換える。
 * 閉じが見つからない開きは対象にしない。対応が崩れている文書で、
 * そこから先の本文をすべて落とすことになるため。
 */
function blankCodeShortcodes(source) {
    if (!source.includes("{{<")) {
        return source;
    }
    const lines = source.split("\n");
    const drop = new Set();
    for (let i = 0; i < lines.length; i += 1) {
        const name = shortcodeOpen(lines[i]);
        if (name === null) {
            continue;
        }
        let end = -1;
        for (let j = i + 1; j < lines.length; j += 1) {
            if (hasShortcodeClose(lines[j], name)) {
                end = j;
                break;
            }
        }
        if (end === -1) {
            continue;
        }
        for (let k = i; k <= end; k += 1) {
            drop.add(k);
        }
        i = end;
    }
    return lines.map((l, i) => (drop.has(i) ? "" : l)).join("\n");
}

/** 前処理。順序を変えてはならない。 */
function preprocess(source) {
    let s = source.replace(RE_FRONTMATTER, blankOut);
    s = blankLines(s, codeFenceLines(s));
    s = blankCodeShortcodes(s);
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

        const heading = RE_HEADING.exec(rawLine);
        if (heading) {
            blocks.push({
                kind: "Heading",
                line: lineNo,
                text: stripInline(heading[2]),
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
