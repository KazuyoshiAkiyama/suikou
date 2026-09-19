"use strict";

// テキスト中の行番号から textlint へ渡す文字インデックスを作る。
//
// suikou-core の Position は前処理後の行・列で位置を持つが、その値を厳密に
// 再現しても利用者には意味がない（見ているファイルは前処理前の原文である）。
// ここでは常に原文の文字インデックスを計算する。列は「一致した文字列を
// その行の中で何番目に見つけたか」を目印に近似する。エスケープ解除や
// インライン記法の除去で前処理後の文字列が原文と食い違う行では、近似が
// ずれることがある。位置の見た目の精度は正本条件ではない
// （tests/golden との突き合わせは件数だけを見る）。

/** 1始まりの行番号から、原文中でその行が始まる0始まりの文字インデックスを返す。 */
function lineStartIndex(source, lineNo) {
    let idx = 0;
    let current = 1;
    while (current < lineNo) {
        const next = source.indexOf("\n", idx);
        if (next === -1) {
            return source.length;
        }
        idx = next + 1;
        current += 1;
    }
    return idx;
}

function lineEndIndex(source, lineNo) {
    const start = lineStartIndex(source, lineNo);
    const next = source.indexOf("\n", start);
    return next === -1 ? source.length : next;
}

/**
 * 指定した行の中で、与えた文字列の出現を左から順に探すカーソルを作る。
 * 同じ語が複数回出てくる場合も、呼ぶたびに次の出現へ進む。
 * 見つからない場合は行頭を返す（近似として、指摘そのものは落とさない）。
 */
function makeLineCursor(source, lineNo) {
    const start = lineStartIndex(source, lineNo);
    const end = lineEndIndex(source, lineNo);
    let cursor = start;
    return function indexOfNext(needle) {
        if (needle.length > 0) {
            const idx = source.indexOf(needle, cursor);
            if (idx !== -1 && idx < end) {
                cursor = idx + needle.length;
                return idx;
            }
        }
        return start;
    };
}

module.exports = { lineStartIndex, lineEndIndex, makeLineCursor };
