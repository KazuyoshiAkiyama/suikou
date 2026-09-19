// M1〜M7。AI臭さとは独立した価値を持つので、単体で使える形にしてある。
// 規範の出典は Google developer documentation style guide の Lists、Headings、
// Timeless documentation の各節。
//
// Rust 側の実装は crates/suikou-core/src/rules/mod.rs にある。判定ロジックは
// そこから移植した。日本語と英語の両方を、文書ごとに自動判定して扱う
// （lang.rs の detect_lang と同じしきい値）。個々のルールを textlint の
// 設定で有効・無効にできるよう、プリセットとしてまとめてある。
"use strict";

module.exports = {
    rules: {
        "list-lead-in": require("./rules/list-lead-in"),
        "item-count": require("./rules/item-count"),
        "numbered-heading": require("./rules/numbered-heading"),
        "manual-number": require("./rules/manual-number"),
        "parallel-items": require("./rules/parallel-items"),
        "time-dependent": require("./rules/time-dependent"),
        "trailing-etc": require("./rules/trailing-etc"),
    },
    rulesConfig: {
        "list-lead-in": true,
        "item-count": true,
        "numbered-heading": true,
        "manual-number": true,
        "parallel-items": true,
        "time-dependent": true,
        "trailing-etc": true,
    },
};
