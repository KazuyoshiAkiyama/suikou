//! T3。`research/mcheck.py` の出力を固定した `tests/golden/expected/rules.json` と
//! Rust 実装の件数を突き合わせる。M 系は整数の一致を求める。
//!
//! 日本語の判定は形態素解析を要するため `lindera-unidic` を有効にしたときだけ走る。
//! 英語は辞書なしで突き合わせられる。

use std::collections::BTreeMap;
use suikou_core::lang::Lang;
use suikou_core::markdown::{BlockKind, Document};
use suikou_core::rules::{
    self, LeadIn, RULE_ITEM_COUNT, RULE_LIST_LEAD_IN, RULE_MANUAL_NUMBER, RULE_NUMBERED_HEADING,
    RULE_PARALLEL_ITEMS, RULE_TIME_DEPENDENT, RULE_TRAILING_ETC,
};
use suikou_core::tokenizer::Morphology;

#[derive(Debug, Default, PartialEq, Eq)]
struct Counts {
    lines: usize,
    m1_ok: usize,
    m1_ng: usize,
    m2: usize,
    m3: usize,
    m4: usize,
    m5_blocks: usize,
    m5_mixed: usize,
    m6: usize,
    m7: usize,
}

fn count(doc: &Document, lang: Lang, morph: &dyn Morphology) -> Counts {
    let findings = rules::check_all(doc, lang, morph);
    let n = |rule: &str| {
        findings
            .iter()
            .find(|f| f.rule_id == rule)
            .map_or(0, |f| f.positions.len() + f.truncated)
    };

    // m1_ok と m5_blocks は違反ではないため findings に載らない。分母として別に数える。
    let mut m1_ok = 0usize;
    let mut m5_blocks = 0usize;
    for group in doc.list_blocks() {
        if group.len() >= 2 {
            m5_blocks += 1;
        }
        if let Some(before) = doc.block_before(group[0].line) {
            if before.kind == BlockKind::Prose {
                let kind = match lang {
                    Lang::Ja => rules::classify_lead_in_ja(&before.text, morph),
                    Lang::En => rules::classify_lead_in_en(&before.text),
                };
                if kind == LeadIn::Ok {
                    m1_ok += 1;
                }
            }
        }
    }

    Counts {
        lines: doc.non_empty_lines,
        m1_ok,
        m1_ng: n(RULE_LIST_LEAD_IN),
        m2: n(RULE_ITEM_COUNT),
        m3: n(RULE_NUMBERED_HEADING),
        m4: n(RULE_MANUAL_NUMBER),
        m5_blocks,
        m5_mixed: n(RULE_PARALLEL_ITEMS),
        m6: n(RULE_TIME_DEPENDENT),
        m7: n(RULE_TRAILING_ETC),
    }
}

fn want(name: &str) -> Counts {
    let src = include_str!("../../../tests/golden/expected/rules.json");
    let all: BTreeMap<String, BTreeMap<String, serde_json::Value>> =
        serde_json::from_str(src).expect("rules.json");
    let e = all
        .get(name)
        .unwrap_or_else(|| panic!("{name} が rules.json にない"));
    let g = |k: &str| e.get(k).and_then(|v| v.as_u64()).unwrap() as usize;
    Counts {
        lines: g("lines"),
        m1_ok: g("m1_ok"),
        m1_ng: g("m1_ng"),
        m2: g("m2"),
        m3: g("m3"),
        m4: g("m4"),
        m5_blocks: g("m5_blocks"),
        m5_mixed: g("m5_mixed"),
        m6: g("m6"),
        m7: g("m7"),
    }
}

fn read(name: &str) -> String {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/input/");
    std::fs::read_to_string(format!("{dir}{name}")).expect(name)
}

fn check_en(name: &str) {
    let doc = Document::parse(&read(name));
    let morph = suikou_core::tokenizer::FakeMorphology::default();
    assert_eq!(count(&doc, Lang::En, &morph), want(name), "{name}");
}

#[test]
fn en_maintainability_matches_reference() {
    check_en("en_maintainability.md");
}

#[test]
fn en_prose_matches_reference() {
    check_en("en_prose.md");
}

#[test]
fn en_escaped_matches_reference() {
    check_en("en_escaped.md");
}

#[cfg(feature = "lindera-unidic")]
mod ja {
    use super::*;
    use suikou_core::tokenizer::LinderaMorphology;

    fn check_ja(name: &str) {
        let doc = Document::parse(&read(name));
        let morph = LinderaMorphology::new().expect("辞書");
        assert_eq!(count(&doc, Lang::Ja, &morph), want(name), "{name}");
    }

    #[test]
    fn ja_maintainability_matches_reference() {
        check_ja("ja_maintainability.md");
    }

    #[test]
    fn ja_prose_matches_reference() {
        check_ja("ja_prose.md");
    }
}
