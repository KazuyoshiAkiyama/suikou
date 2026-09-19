"use strict";

// M5: 箇条書きの項目末尾の形が揃っていない。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M5 部分を移植。
// 2項目以上のリストのまとまりごとに、末尾の形の集合が2種類以上あれば
// まとまりにつき1件を、先頭の項目の位置で報告する。

const { parseDocument, listBlocks } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const { itemEndingEn } = require("../lib/lead-in-en");
const { itemEndingJa } = require("../lib/morphology-ja");
const { lineStartIndex } = require("../lib/position");
const { messageFor } = require("../lib/messages");

module.exports = function parallelItems(context) {
    const { Syntax, RuleError, report, getSource } = context;
    return {
        async [Syntax.Document](node) {
            const source = getSource(node);
            const lang = detectLang(source);
            const { blocks } = parseDocument(source);
            const message = messageFor("parallel-items", lang);

            for (const group of listBlocks(blocks)) {
                if (group.length < 2) {
                    continue;
                }
                const kinds = new Set();
                for (const b of group) {
                    kinds.add(lang === "ja" ? await itemEndingJa(b.text) : itemEndingEn(b.text));
                }
                if (kinds.size > 1) {
                    const index = lineStartIndex(source, group[0].line);
                    report(node, new RuleError(message, { index }));
                }
            }
        },
    };
};
