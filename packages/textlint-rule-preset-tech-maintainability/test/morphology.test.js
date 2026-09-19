"use strict";

// crates/suikou-core/src/rules/mod.rs の日本語の単体テストと同じ例文で、
// kuromojin (IPADIC) 版の分類が同じ結果になることを確かめる。
// IPADIC には UniDic の品詞大分類「接尾辞」「代名詞」「形状詞」がなく、
// いずれも「名詞」の下位分類として現れるため、is_taigen の判定を
// pos === "名詞" のひとつにまとめてある。詳しくは
// src/lib/morphology-ja.js の先頭コメントと docs/content/ja/decisions.md
// の D-23 を見る。

const test = require("node:test");
const assert = require("node:assert/strict");
const { classifyLeadInJa, itemEndingJa } = require("../src/lib/morphology-ja");

test("lead-in ending in a complete sentence is ok", async () => {
    assert.equal(await classifyLeadInJa("手順は次のとおり使います。"), "Ok");
});

test("lead-in ending in a taigen + colon is ok", async () => {
    assert.equal(await classifyLeadInJa("手順:"), "Ok");
});

test("lead-in ending in a particle is ng", async () => {
    assert.equal(await classifyLeadInJa("以下の目的で"), "Ng");
});

test("lead-in ending in a predicate + colon is ng", async () => {
    assert.equal(await classifyLeadInJa("対象となるファイルは:"), "Ng");
});

test("lead-in ending in a renyou-form verb is ng", async () => {
    assert.equal(await classifyLeadInJa("設定ファイルの構文を検証し"), "Ng");
});

test("item ending in a kuten is Kuten", async () => {
    assert.equal(await itemEndingJa("設定ファイルの構文を検証する。"), "Kuten");
});

test("item ending in a taigen (noun) is Taigen", async () => {
    assert.equal(await itemEndingJa("サービスの再読み込み"), "Taigen");
});

test("item ending in a predicate with no kuten is Yougen", async () => {
    assert.equal(await itemEndingJa("サービスを再読み込みする"), "Yougen");
});
