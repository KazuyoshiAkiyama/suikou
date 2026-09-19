"use strict";

// M1: 箇条書きの導入文。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M1 部分を移植。
// 判定の対象は「リストのまとまりの直前にある、空行を挟まない最後の非リスト
// ブロック」。それが地の文（Prose）でなければ判定しない。

const { parseDocument, listBlocks, blockBefore } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const { classifyLeadInEn } = require("../lib/lead-in-en");
const { classifyLeadInJa } = require("../lib/morphology-ja");
const { lineStartIndex } = require("../lib/position");
const { messageFor } = require("../lib/messages");

module.exports = function listLeadIn(context) {
    const { Syntax, RuleError, report, getSource } = context;
    return {
        async [Syntax.Document](node) {
            const source = getSource(node);
            const lang = detectLang(source);
            const { blocks } = parseDocument(source);
            const message = messageFor("list-lead-in", lang);

            for (const group of listBlocks(blocks)) {
                const first = group[0];
                const before = blockBefore(blocks, first.line);
                if (!before || before.kind !== "Prose") {
                    continue;
                }
                const kind =
                    lang === "ja" ? await classifyLeadInJa(before.text) : classifyLeadInEn(before.text);
                if (kind === "Ng") {
                    const index = lineStartIndex(source, before.line);
                    report(node, new RuleError(message, { index }));
                }
            }
        },
    };
};
