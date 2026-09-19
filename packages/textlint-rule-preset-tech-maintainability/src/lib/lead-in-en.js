"use strict";

// M1・M5 の英語判定。crates/suikou-core/src/rules/mod.rs の
// classify_lead_in_en / item_ending_en をそのまま移植したもの。
// 形態素解析は要らない。

const { EN_LEAD_IN_MARKERS } = require("./patterns");

/**
 * @param {string} line
 * @returns {"Ok"|"Ng"|"Other"}
 */
function classifyLeadInEn(line) {
    const s = line.trim();
    if (s === "") {
        return "Other";
    }
    if (s.endsWith(":")) {
        const lower = s.toLowerCase();
        if (EN_LEAD_IN_MARKERS.some((m) => lower.includes(m))) {
            return "Ok";
        }
        return "Ng";
    }
    if (s.endsWith(".") || s.endsWith("!") || s.endsWith("?")) {
        return "Ok";
    }
    return "Other";
}

/**
 * @param {string} text
 * @returns {"Period"|"None"}
 */
function itemEndingEn(text) {
    const t = text.trim();
    if (t.endsWith(".") || t.endsWith("!") || t.endsWith("?")) {
        return "Period";
    }
    return "None";
}

module.exports = { classifyLeadInEn, itemEndingEn };
