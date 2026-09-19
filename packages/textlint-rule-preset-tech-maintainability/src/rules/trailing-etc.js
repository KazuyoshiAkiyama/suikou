"use strict";

// M7: 列挙が「など」「等」「etc.」で終わっている。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M7 部分を移植。
// 対象は Prose ブロックと ListItem ブロック（見出し・表・引用は対象外）。
// 判定は text（インライン記法を剥がした後）に対して行う。

const { parseDocument } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const { RE_M7_JA, RE_M7_EN } = require("../lib/patterns");
const { lineStartIndex } = require("../lib/position");
const { messageFor } = require("../lib/messages");

module.exports = function trailingEtc(context) {
    const { Syntax, RuleError, report, getSource } = context;
    return {
        [Syntax.Document](node) {
            const source = getSource(node);
            const lang = detectLang(source);
            const { blocks } = parseDocument(source);
            const message = messageFor("trailing-etc", lang);
            const re = lang === "ja" ? RE_M7_JA : RE_M7_EN;

            for (const b of blocks) {
                if (b.kind !== "Prose" && b.kind !== "ListItem") {
                    continue;
                }
                if (re.test(b.text.trim())) {
                    const index = lineStartIndex(source, b.line);
                    report(node, new RuleError(message, { index }));
                }
            }
        },
    };
};
