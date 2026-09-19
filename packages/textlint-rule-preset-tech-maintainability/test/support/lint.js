"use strict";

// テスト用の最小限の textlint 実行器。
//
// textlint-tester は Mocha 前提の describe/it を期待する作りになっている
// （フォールバックはあるが、返り値の Promise を待たない）。
// このパッケージのテストは `node --test` で走らせる方針にしたため、
// @textlint/kernel を直に使う薄い層をここに用意した。判断の経緯は
// README.md の「テスト」節と docs/content/ja/decisions.md の D-23 にある。

const { TextlintKernel } = require("@textlint/kernel");
const markdownPlugin = require("@textlint/textlint-plugin-markdown").default;
const preset = require("../../src/index.js");

const kernel = new TextlintKernel();

const allRules = Object.entries(preset.rules).map(([key, rule]) => ({
    ruleId: `maint/${key}`,
    rule,
}));

/**
 * @param {string} text
 * @param {string[]} [ruleKeys] 絞り込むルール名（プレフィックスなし）。省略時は全ルール。
 */
async function lint(text, ruleKeys) {
    const rules = ruleKeys ? allRules.filter((r) => ruleKeys.includes(r.ruleId.replace("maint/", ""))) : allRules;
    const result = await kernel.lintText(text, {
        ext: ".md",
        plugins: [{ pluginId: "markdown", plugin: markdownPlugin }],
        rules,
    });
    return result.messages;
}

/** ルールIDごとの件数の対応表を返す。 */
async function countsByRule(text, ruleKeys) {
    const messages = await lint(text, ruleKeys);
    const counts = {};
    for (const m of messages) {
        counts[m.ruleId] = (counts[m.ruleId] || 0) + 1;
    }
    return counts;
}

module.exports = { lint, countsByRule };
