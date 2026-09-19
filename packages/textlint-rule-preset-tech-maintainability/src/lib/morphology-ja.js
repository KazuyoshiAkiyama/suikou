"use strict";

// M1・M5 の日本語判定。crates/suikou-core/src/rules/mod.rs の
// classify_lead_in_ja / item_ending_ja を、IPADIC ベースの kuromoji に
// 移し替えたもの。
//
// suikou-core は UniDic を使い、体言止めの判定を
// pos1 in {名詞, 接尾辞, 代名詞, 形状詞} で行う。IPADIC にはこの4つを
// 分ける品詞大分類がなく、代名詞・接尾・形容動詞語幹はいずれも
// pos（品詞細分類の前、UniDicのpos1に相当する層）が「名詞」の下位分類
// （pos_detail_1 が「代名詞」「接尾」「形容動詞語幹」）として現れる。
// したがって IPADIC では pos === "名詞" のひとつの条件に潰してよい。
// kuromojin 3.0.1 の実測で確認した（「これ」→名詞/代名詞、
// 「委員長」の「長」→名詞/接尾、「静か」→名詞/形容動詞語幹）。
// 詳しい経緯は docs/content/ja/decisions.md の D-23 にある。
//
// 連用形の判定も同様に実測で確認した。IPADIC の conjugated_form は
// UniDic の「連用形-一般」のような細分がなく、単に「連用形」という値を
// 返す（例:「使い」「し」)。 startsWith("連用形") のまま移植して問題ない。

const { tokenize } = require("kuromojin");

function isTaigen(token) {
    return token.pos === "名詞";
}

function lastChar(s) {
    const chars = Array.from(s);
    return chars.length > 0 ? chars[chars.length - 1] : "";
}

/**
 * @param {string} line
 * @returns {Promise<"Ok"|"Ng"|"Other">}
 */
async function classifyLeadInJa(line) {
    const s = line.trim();
    if (s === "") {
        return "Other";
    }
    const last = lastChar(s);
    if (last === "。" || last === "！" || last === "？") {
        return "Ok";
    }
    if (last === ":" || last === "：") {
        const core = Array.from(s).slice(0, -1).join("").trim();
        const toks = await tokenize(core);
        const t = toks[toks.length - 1];
        if (!t) {
            return "Other";
        }
        return isTaigen(t) ? "Ok" : "Ng";
    }
    const toks = await tokenize(s);
    const t = toks[toks.length - 1];
    if (!t) {
        return "Other";
    }
    if (t.pos === "助詞") {
        return "Ng";
    }
    if ((t.pos === "動詞" || t.pos === "助動詞") && (t.conjugated_form || "").startsWith("連用形")) {
        return "Ng";
    }
    return "Other";
}

/**
 * @param {string} text
 * @returns {Promise<"Kuten"|"Taigen"|"Yougen"|"Other">}
 */
async function itemEndingJa(text) {
    const t = text.trim();
    if (t === "") {
        return "Other";
    }
    const last = lastChar(t);
    if (last === "。" || last === "！" || last === "？") {
        return "Kuten";
    }
    const toks = await tokenize(t);
    const tok = toks[toks.length - 1];
    if (!tok) {
        return "Other";
    }
    return isTaigen(tok) ? "Taigen" : "Yougen";
}

module.exports = { classifyLeadInJa, itemEndingJa, isTaigen };
