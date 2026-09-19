"use strict";

// M2: 地の文に項目数を書いている。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M2 部分を移植。
// 対象は Prose ブロックのみ（見出しと箇条書きの本文は含まない）。
// 1行に複数の一致があれば、それぞれを別の指摘として報告する（D-13）。

const { parseDocument } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const { RE_M2_JA, RE_M2_EN } = require("../lib/patterns");
const { makeLineCursor } = require("../lib/position");
const { messageFor } = require("../lib/messages");

module.exports = function itemCount(context) {
    const { Syntax, RuleError, report, getSource } = context;
    return {
        [Syntax.Document](node) {
            const source = getSource(node);
            const lang = detectLang(source);
            const { blocks } = parseDocument(source);
            const message = messageFor("item-count", lang);
            const re = lang === "ja" ? RE_M2_JA : RE_M2_EN;

            for (const b of blocks) {
                if (b.kind !== "Prose") {
                    continue;
                }
                re.lastIndex = 0;
                const cursor = makeLineCursor(source, b.line);
                let m;
                while ((m = re.exec(b.text)) !== null) {
                    const index = cursor(m[0]);
                    report(node, new RuleError(message, { index }));
                    if (m[0].length === 0) {
                        re.lastIndex += 1;
                    }
                }
            }
        },
    };
};
