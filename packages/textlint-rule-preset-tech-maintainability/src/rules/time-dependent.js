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
const {
    RE_M6_JA,
    RE_M6_EN,
    RE_ANCHOR,
    RE_RELATIVE_PERIOD_JA,
    RE_RELATIVE_PERIOD_EN,
} = require("../lib/patterns");
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
            const words = lang === "ja" ? RE_M6_JA : RE_M6_EN;
            const period = lang === "ja" ? RE_RELATIVE_PERIOD_JA : RE_RELATIVE_PERIOD_EN;
            const enders = lang === "ja" ? /[。！？]/g : /[.!?]/g;

            // 係留は文ごとに見る。同じ段落の別の文が年号を持っていても、
            // この文が時点に依存することは変わらない。
            const sentenceOf = (text, at) => {
                enders.lastIndex = 0;
                let from = 0;
                let e;
                while ((e = enders.exec(text)) !== null) {
                    const end = e.index + e[0].length;
                    if (end > at) {
                        return text.slice(from, end);
                    }
                    from = end;
                }
                return text.slice(from);
            };

            for (const b of blocks) {
                const cursor = makeLineCursor(source, b.line);
                const hits = [];
                for (const re of [words, period]) {
                    re.lastIndex = 0;
                    let m;
                    while ((m = re.exec(b.text)) !== null) {
                        if (m[0].length === 0) {
                            re.lastIndex += 1;
                            continue;
                        }
                        // 絶対の係留がある文は見送る。
                        if (RE_ANCHOR.test(sentenceOf(b.text, m.index))) {
                            continue;
                        }
                        hits.push({ at: m.index, text: m[0] });
                    }
                }
                // 「直近3ヶ月」は語の一覧と期間の型の両方に当たる。
                // 同じ位置を二度示すと、直す側は同じ箇所を二度読むことになる。
                hits.sort((x, y) => x.at - y.at);
                let prev = -1;
                for (const h of hits) {
                    if (h.at === prev) {
                        continue;
                    }
                    prev = h.at;
                    const index = cursor(h.text);
                    report(node, new RuleError(message, { index }));
                }
            }
        },
    };
};
