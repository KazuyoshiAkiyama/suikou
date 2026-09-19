"use strict";

// Ported from crates/suikou-core/src/lang.rs (detect_lang).
// Do not change the threshold without updating the Rust side too; the two
// must agree so that `suikou check` and this preset classify the same
// document the same way.

const JA_RANGES = [
    [0x3040, 0x30ff],
    [0x4e00, 0x9fff],
];

function isJapaneseChar(codePoint) {
    return JA_RANGES.some(([lo, hi]) => codePoint >= lo && codePoint <= hi);
}

/**
 * @param {string} text
 * @returns {"ja" | "en"}
 */
function detectLang(text) {
    let ja = 0;
    let total = 0;
    for (const ch of text) {
        total += 1;
        if (isJapaneseChar(ch.codePointAt(0))) {
            ja += 1;
        }
    }
    if (total === 0) {
        return "en";
    }
    return ja * 20 > total ? "ja" : "en";
}

module.exports = { detectLang, isJapaneseChar };
