"use strict";

// tests/golden/input/*.md に対して、Rust 実装 (`suikou check --format json`)
// と同じ件数の指摘が出ることを確かめる。T7 の完了条件そのものにあたる。
//
// 期待値は `suikou check tests/golden/input/*.md --format json` を実際に
// 走らせて得た件数を書き写したものである
// (`suikou check` の local findings の positions.length + truncated の和)。
// tests/golden/expected/rules.json の m1_ng などの列とも一致する。
//
// tests/golden/input/*.md 自体は編集しない。

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { countsByRule } = require("./support/lint");

const GOLDEN_DIR = path.join(__dirname, "..", "..", "..", "tests", "golden", "input");

// suikou check tests/golden/input/*.md --format json で確かめた件数。
const EXPECTED = {
    "ja_maintainability.md": {
        "list-lead-in": 1,
        "item-count": 1,
        "numbered-heading": 1,
        "parallel-items": 1,
        "time-dependent": 2,
    },
    "en_maintainability.md": {
        "list-lead-in": 1,
        "numbered-heading": 1,
        "parallel-items": 1,
        "time-dependent": 2,
        "trailing-etc": 1,
    },
    "ja_prose.md": {
        "item-count": 1,
    },
    "en_prose.md": {},
    "en_escaped.md": {},
};

for (const [file, expected] of Object.entries(EXPECTED)) {
    test(`golden parity: ${file}`, async () => {
        const text = fs.readFileSync(path.join(GOLDEN_DIR, file), "utf-8");
        const counts = await countsByRule(text);
        const normalized = {};
        for (const [ruleId, n] of Object.entries(counts)) {
            normalized[ruleId.replace("maint/", "")] = n;
        }
        assert.deepEqual(normalized, expected, `mismatch for ${file}`);
    });
}
