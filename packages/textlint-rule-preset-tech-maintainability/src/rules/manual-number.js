"use strict";

// M4: 箇条書きの項目に手動で書いた番号。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M4 部分を移植。
// 対象はマーカーが `-` `*` `+` の項目のみ（`1.` のような順序付きリストの
// マーカー自体は対象にしない）。判定は raw（インライン記法を剥がす前の行）
// に対して行う。単独のリスト項目でも判定する。グループの大きさは問わない。

const { parseDocument } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const { RE_M4 } = require("../lib/patterns");
const { lineStartIndex } = require("../lib/position");
const { messageFor } = require("../lib/messages");

module.exports = function manualNumber(context) {
    const { Syntax, RuleError, report, getSource } = context;
    return {
        [Syntax.Document](node) {
            const source = getSource(node);
            const lang = detectLang(source);
            const { blocks } = parseDocument(source);
            const message = messageFor("manual-number", lang);

            for (const b of blocks) {
                if (b.kind !== "ListItem") {
                    continue;
                }
                if (RE_M4.test(b.raw)) {
                    const index = lineStartIndex(source, b.line);
                    report(node, new RuleError(message, { index }));
                }
            }
        },
    };
};
