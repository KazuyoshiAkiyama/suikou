"use strict";

// M6: 時点に依存する語。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M6 部分を移植。
// 対象はブロックの種類を問わない（見出しや表、引用も含めてすべてのブロック）。
// 1行に複数の一致があれば、それぞれを別の指摘として報告する（D-13）。
// 英語の語彙は厳格版に絞ってある。new / now / future / existing は含めない
// （crates/suikou-core/src/rules/mod.rs のコメントと同じ理由。実測で
// 偽陽性が増えることが分かっている。CLAUDE.md により、ここへ語を足す
// ときは実測かスタイルガイドの根拠が要る）。

const { parseDocument } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const { RE_M6_JA, RE_M6_EN } = require("../lib/patterns");
const { makeLineCursor } = require("../lib/position");
const { messageFor } = require("../lib/messages");

module.exports = function timeDependent(context) {
    const { Syntax, RuleError, report, getSource } = context;
    return {
        [Syntax.Document](node) {
            const source = getSource(node);
            const lang = detectLang(source);
            const { blocks } = parseDocument(source);
            const message = messageFor("time-dependent", lang);
            const re = lang === "ja" ? RE_M6_JA : RE_M6_EN;

            for (const b of blocks) {
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
