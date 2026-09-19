"use strict";

// ルールごとの説明文。日本語と英語の両方を持つ。
//
// suikou-core（Rust側）は文書の言語にかかわらず常に日本語の説明文を出す
// （crates/suikou-core/src/rules/mod.rs の finding() 呼び出しを参照）。
// この textlint プリセットは英語文書の利用者にも読めることを優先し、
// 検出した文書の言語に合わせて日本語・英語の説明文を出し分ける。
// 判定そのもの（正規表現・分類ロジック）は Rust 側と揃えてあるが、
// 説明文の言語だけは意図して変えてある。経緯は docs/content/ja/decisions.md
// の D-23 にある。

const MESSAGES = {
    "list-lead-in": {
        ja: "箇条書きの導入文が項目と文法的に結合している。項目を増減すると壊れる",
        en: "the lead-in to this list is grammatically bound to the items; adding or removing an item breaks the sentence",
    },
    "item-count": {
        ja: "地の文に項目数を書いている。項目を増減すると本文が合わなくなる",
        en: "the prose states the number of items; adding or removing an item makes it wrong",
    },
    "numbered-heading": {
        ja: "見出しに節の連番がある。節を増減すると振り直しが要る",
        en: "this heading carries a manual section number; adding or removing a section forces renumbering",
    },
    "manual-number": {
        ja: "箇条書きの項目に手動で番号を書いている",
        en: "this list item has a hand-written number; use an ordered list instead",
    },
    "parallel-items": {
        ja: "同じ箇条書きの中で項目の末尾の形が揃っていない",
        en: "the items in this list do not share a common ending shape",
    },
    "time-dependent": {
        ja: "時点に依存する語を使っている。文書が古びる",
        en: "this word depends on a moment in time; the document will go stale",
    },
    "trailing-etc": {
        ja: "列挙が「など」で終わっている。網羅的でないことは導入文で示す",
        en: "this enumeration trails off with etc.; state non-exhaustiveness in the lead-in instead",
    },
};

/**
 * @param {string} ruleKey list-lead-in などプレフィックスなしのルール名
 * @param {"ja"|"en"} lang
 * @returns {string}
 */
function messageFor(ruleKey, lang) {
    const entry = MESSAGES[ruleKey];
    if (!entry) {
        throw new Error(`unknown rule key: ${ruleKey}`);
    }
    return lang === "ja" ? entry.ja : entry.en;
}

module.exports = { MESSAGES, messageFor };
