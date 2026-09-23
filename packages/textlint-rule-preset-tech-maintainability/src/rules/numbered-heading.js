"use strict";

// M3: 見出しの手動連番。
// crates/suikou-core/src/rules/mod.rs の check_all() のうち M3 部分を移植。
// 判定は `#` を含む生の行（raw、インライン記法を剥がす前）に対して行う。

const { parseDocument } = require("../lib/blocks");
const { detectLang } = require("../lib/lang");
const { RE_M3 } = require("../lib/patterns");
const { lineStartIndex } = require("../lib/position");
const { messageFor } = require("../lib/messages");

module.exports = function numberedHeading(context) {
    const { Syntax, RuleError, report, getSource } = context;
    return {
        [Syntax.Document](node) {
            const source = getSource(node);
            const lang = detectLang(source);
            const { blocks } = parseDocument(source);
            const message = messageFor("numbered-heading", lang);

            // 標題の番号は識別子であって節の連番ではない。
            // ADR は `# 3. 題` を様式とし、その番号は振り直されない。
            // 実測では、公開されている ADR 190件のうち155件がこれで発火していた。
            const first = blocks.find((b) => b.kind === "Heading");
            let isTitle = Boolean(first && first.raw.trimStart().startsWith("# "));

            for (const b of blocks) {
                if (b.kind !== "Heading") {
                    continue;
                }
                if (!isTitle && RE_M3.test(b.raw.trim())) {
                    const index = lineStartIndex(source, b.line);
                    report(node, new RuleError(message, { index }));
                }
                isTitle = false;
            }
        },
    };
};
