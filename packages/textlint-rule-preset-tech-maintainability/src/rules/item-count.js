"use strict";

// M2: 地の文に項目数を書いている。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M2 部分を移植。
// 対象は Prose ブロックのみ（見出しと箇条書きの本文は含まない）。
// 1行に複数の一致があれば、それぞれを別の指摘として報告する（D-13）。
// 数が前方参照しているときだけ当てる（D-45）。

const { parseDocument } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const {
    RE_M2_JA,
    RE_M2_EN,
    RE_M2_FORWARD_JA,
    RE_M2_FORWARD_EN,
    RE_M2_ENUM_JA,
    RE_M2_ENUM_EN,
} = require("../lib/patterns");
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

            const forward = lang === "ja" ? RE_M2_FORWARD_JA : RE_M2_FORWARD_EN;
            const enumerate = lang === "ja" ? RE_M2_ENUM_JA : RE_M2_ENUM_EN;

            // 数が別の場所を指しているかどうか。Rust 側の m2_forward_reference と同じ条件。
            const pointsForward = (b, i) => {
                if (forward.test(b.text)) {
                    return true;
                }
                const next = blocks[i + 1];
                if (!next) {
                    return false;
                }
                if (next.kind === "ListItem" || next.kind === "Table") {
                    return true;
                }
                // 見出しは、今いる節の内側にあるものだけを数える（D-48）。
                // 兄弟の見出しが続くのは、その節が終わっただけである。
                if (next.kind === "Heading") {
                    const level = (raw) => (raw.match(/^\s*(#{1,6})\s/) || [, ""])[1].length;
                    let here = null;
                    for (let k = i - 1; k >= 0; k -= 1) {
                        if (blocks[k].kind === "Heading") {
                            here = level(blocks[k].raw ?? blocks[k].text);
                            break;
                        }
                    }
                    const there = level(next.raw ?? next.text);
                    return here === null || there > here;
                }
                return next.kind === "Prose" && enumerate.test(next.text);
            };

            for (let bi = 0; bi < blocks.length; bi += 1) {
                const b = blocks[bi];
                if (b.kind !== "Prose" || !pointsForward(b, bi)) {
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
