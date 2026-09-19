"use strict";

// crates/suikou-core/src/markdown.rs の同名のテストの一部を移植して、
// JS 側のブロック抽出が同じ挙動をすることを確かめる。

const test = require("node:test");
const assert = require("node:assert/strict");
const { preprocess, parseDocument, listBlocks, blockBefore } = require("../src/lib/blocks");

test("unescapes before splitting into lines (order matters)", () => {
    const src = "The validator reports each error\\. It also reports a line number\\.";
    const out = preprocess(src);
    assert.ok(out.includes("error. It"), `escape was not undone: ${out}`);
});

test("keeps line numbers across removed blocks", () => {
    const src =
        "---\ntitle: x\n---\n\nBody line.\n\n```\ncode\ncode\n```\n\nLast line.\n";
    const { blocks } = parseDocument(src);
    const last = blocks[blocks.length - 1];
    assert.equal(last.text, "Last line.");
    assert.equal(last.line, 12, JSON.stringify(blocks));
});

test("removes code fences and frontmatter", () => {
    const src = "---\ntitle: x\n---\nhello\n```\ncode. code.\n```\nworld\n";
    const out = preprocess(src);
    assert.ok(!out.includes("title"));
    assert.ok(!out.includes("code."));
    assert.ok(out.includes("hello") && out.includes("world"));
});

test("classifies blocks by kind", () => {
    const src = "# Heading\n\nBody text.\n\n- item1\n- item2\n\n| a | b |\n> quote\n";
    const { blocks } = parseDocument(src);
    const kinds = blocks.map((b) => b.kind);
    assert.deepEqual(kinds, ["Heading", "Prose", "ListItem", "ListItem", "Table", "Quote"]);
});

test("ordered flag and depth", () => {
    const src = "1. first\n2. second\n  - nested\n";
    const { blocks } = parseDocument(src);
    assert.equal(blocks[0].ordered, true);
    assert.equal(blocks[2].ordered, false);
    assert.equal(blocks[2].depth, 1);
});

test("strips inline markup", () => {
    // Removing a code span leaves its surrounding spaces behind (as in
    // markdown.rs strip_inline, which does not collapse whitespace).
    const src = "**bold** and `code` and [link](http://example.com).\n";
    const { blocks } = parseDocument(src);
    assert.equal(blocks[0].text, "bold and  and link.");
});

test("groups consecutive list items", () => {
    const src = "Intro.\n\n- a\n- b\n\nMore prose.\n\n- c\n";
    const { blocks } = parseDocument(src);
    const groups = listBlocks(blocks);
    assert.equal(groups.length, 2);
    assert.equal(groups[0].length, 2);
    assert.equal(groups[1].length, 1);
});

test("finds the block right before a list", () => {
    const src = "Intro.\n\n- a\n";
    const { blocks } = parseDocument(src);
    const group = listBlocks(blocks)[0];
    const before = blockBefore(blocks, group[0].line);
    assert.equal(before.kind, "Prose");
    assert.equal(before.text, "Intro.");
});

test("a heading or list item right before a list is not treated as a lead-in", () => {
    const src = "## Heading\n\n- a\n- b\n";
    const { blocks } = parseDocument(src);
    const group = listBlocks(blocks)[0];
    const before = blockBefore(blocks, group[0].line);
    assert.equal(before.kind, "Heading");
});
