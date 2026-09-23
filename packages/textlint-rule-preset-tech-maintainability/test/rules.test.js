"use strict";

// ルールごとの valid/invalid の最小例。
// tests/golden/input/*.md は M4 (manual-number) を一度も踏まないので、
// ここでその穴を埋める。

const test = require("node:test");
const assert = require("node:assert/strict");
const { countsByRule } = require("./support/lint");

async function expectCount(text, ruleKey, expected) {
    const counts = await countsByRule(text, [ruleKey]);
    const actual = counts[`maint/${ruleKey}`] || 0;
    assert.equal(actual, expected, `text=${JSON.stringify(text)} counts=${JSON.stringify(counts)}`);
}

test("list-lead-in: ja bound lead-in (particle before colon) is ng", async () => {
    await expectCount("対象となるファイルは:\n\n- a\n- b\n", "list-lead-in", 1);
});

test("list-lead-in: ja complete sentence lead-in is ok", async () => {
    await expectCount("手順は次のとおり。\n\n- a\n- b\n", "list-lead-in", 0);
});

test("list-lead-in: en bare colon with no marker is ng", async () => {
    await expectCount("Use the reload command to:\n\n- a\n- b\n", "list-lead-in", 1);
});

test("list-lead-in: en colon with 'the following' is ok", async () => {
    await expectCount("Apply the change by doing the following:\n\n- a\n- b\n", "list-lead-in", 0);
});

test("item-count: ja states the number of items in prose", async () => {
    await expectCount("よくある失敗は3つある。\n", "item-count", 1);
});

test("item-count: en states the number of items in prose", async () => {
    await expectCount("There are three reasons to do this.\n", "item-count", 1);
});

test("item-count: no count when no shape matches", async () => {
    await expectCount("よくある失敗がある。\n", "item-count", 0);
});

test("numbered-heading: ja heading with a manual section number", async () => {
    await expectCount("## 2. 検証の方法\n\n本文である。\n", "numbered-heading", 1);
});

test("numbered-heading: en heading with a manual section number", async () => {
    await expectCount("## 2. Validation\n\nBody text here.\n", "numbered-heading", 1);
});

test("numbered-heading: a heading without a number is fine", async () => {
    await expectCount("## Validation\n\nBody text here.\n", "numbered-heading", 0);
});

test("manual-number: hand-written numbers inside bullet markers", async () => {
    await expectCount("Intro text.\n\n- 1. first\n- 2. second\n", "manual-number", 2);
});

test("manual-number: a real ordered list is not manual numbering", async () => {
    await expectCount("Intro text.\n\n1. first\n2. second\n", "manual-number", 0);
});

test("parallel-items: ja mixed endings in one list", async () => {
    await expectCount("導入である。\n\n- 構文を検証する。\n- サービスを再読み込み\n", "parallel-items", 1);
});

test("parallel-items: ja uniform endings in one list", async () => {
    await expectCount("導入である。\n\n- 構文を検証する。\n- ログを確認する。\n", "parallel-items", 0);
});

test("parallel-items: en mixed endings in one list", async () => {
    await expectCount(
        "Intro text here.\n\n- Validate the syntax of the file.\n- Reload the service\n",
        "parallel-items",
        1,
    );
});

test("parallel-items: en uniform endings in one list", async () => {
    await expectCount(
        "Intro text here.\n\n- Validate the syntax of the file.\n- Reload the service.\n",
        "parallel-items",
        0,
    );
});

test("parallel-items: a single-item list is never mixed", async () => {
    await expectCount("導入である。\n\n- 構文を検証する。\n", "parallel-items", 0);
});

test("time-dependent: ja single occurrence", async () => {
    await expectCount("現在の実装では反映されない。\n", "time-dependent", 1);
});

test("time-dependent: ja two occurrences on one line count as two", async () => {
    await expectCount("現在の実装は、今後の版で変わる。\n", "time-dependent", 2);
});

test("time-dependent: en single occurrence", async () => {
    await expectCount("The validator currently reports errors.\n", "time-dependent", 1);
});

test("time-dependent: en strict vocabulary excludes 'new' and 'existing'", async () => {
    await expectCount("This is a new feature for the existing system.\n", "time-dependent", 0);
});

test("trailing-etc: ja line ending in など", async () => {
    await expectCount("エラーの種類は不一致など。\n", "trailing-etc", 1);
});

test("trailing-etc: en line ending in etc.", async () => {
    await expectCount("Errors include braces, unknown keys, etc.\n", "trailing-etc", 1);
});

test("trailing-etc: a line that does not trail off is fine", async () => {
    await expectCount("Errors include braces and unknown keys.\n", "trailing-etc", 0);
});

test("clean document produces no findings from any rule", async () => {
    const text = "手順は次のとおり。\n\n- 構文を検証する。\n- ログを確認する。\n";
    const counts = await countsByRule(text);
    assert.deepEqual(counts, {});
});

// 以下は crates/suikou-core/src/rules/mod.rs の同名のテストと対になる。
// 両実装が同じ件数を返すという約束を保つため、片方だけに足さない。
test("time-dependent: ja relative period", async () => {
    await expectCount("直近3ヶ月でリクエスト数が倍になった。\n", "time-dependent", 1);
    await expectCount("過去2年の運用で問題は出ていない。\n", "time-dependent", 1);
    await expectCount("先週の計測では収まっていた。\n", "time-dependent", 1);
});

test("time-dependent: an absolute anchor suppresses the finding", async () => {
    await expectCount("lindera 6.0.0 で現在の素性の順序を確認した。\n", "time-dependent", 0);
    await expectCount("2026年3月の計測では、直近3ヶ月で倍になっていた。\n", "time-dependent", 0);
    await expectCount("RFC 7322 は現在も必須の節を定めている。\n", "time-dependent", 0);
});

test("time-dependent: an anchor does not carry to another sentence", async () => {
    await expectCount("2026年3月に計測した。現在の実装では反映されない。\n", "time-dependent", 1);
});

test("time-dependent: a decimal is not an anchor", async () => {
    await expectCount("直近3ヶ月で約2.5倍に増加した。\n", "time-dependent", 1);
});

test("time-dependent: an overlapping match is reported once", async () => {
    await expectCount("直近3ヶ月で負荷が上がった。\n", "time-dependent", 1);
});

test("time-dependent: en relative period", async () => {
    await expectCount("Requests doubled over the past three months.\n", "time-dependent", 1);
});

test("time-dependent: en anchor suppresses the finding", async () => {
    await expectCount("Measured on 2026-03-01, requests doubled recently.\n", "time-dependent", 0);
});
