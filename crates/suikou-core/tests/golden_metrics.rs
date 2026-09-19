//! T3。文書指標を `research/` の参照実装の出力と突き合わせる。
//!
//! 形態素解析器が lindera と fugashi で異なるため完全一致は期待できない。
//! `tests/golden/README.md` の方針に従い、比率は絶対誤差 0.02、
//! 1000文字あたりの密度は 0.5 を許容範囲とする。
//!
//! 参照実装は `_全` 付きのキーを地の文＋箇条書き本文で、無印を地の文だけで計算している。
//! `docs/METRICS.md` は語彙系を地の文＋箇条書き本文と定めているため、
//! 語彙系は `_全` 側と突き合わせる。

use serde_json::Value;
use std::collections::BTreeMap;
use suikou_core::markdown::Document;

const RATIO_TOL: f64 = 0.02;
const DENSITY_TOL: f64 = 0.5;
/// 語数と文数はトークナイザの定義に依存するため、絶対値では比べられない。
/// spaCy は `trade-off` を三つに割って記号を除き2語と数え、空白分割は1語と数える。
/// 系統的なずれだけを捕らえるために相対誤差で見る。
const COUNT_REL_TOL: f64 = 0.01;

fn read(name: &str) -> String {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/input/");
    std::fs::read_to_string(format!("{dir}{name}")).expect(name)
}

fn expected(file: &str, doc: &str) -> BTreeMap<String, Value> {
    let src = match file {
        "ja" => include_str!("../../../tests/golden/expected/ja_metrics.json"),
        _ => include_str!("../../../tests/golden/expected/en_metrics.json"),
    };
    let all: BTreeMap<String, BTreeMap<String, Value>> = serde_json::from_str(src).expect(file);
    all.get(doc)
        .unwrap_or_else(|| panic!("{doc} が {file}_metrics.json にない"))
        .clone()
}

/// 比較の結果を表にして返す。落ちたときに全項目が一度に見えるようにする。
struct Cmp {
    rows: Vec<(String, f64, f64, f64, bool)>,
}

impl Cmp {
    fn new() -> Self {
        Cmp { rows: Vec::new() }
    }

    fn add(&mut self, key: &str, got: f64, want: &BTreeMap<String, Value>, tol: f64) {
        let w = want
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("期待値に {key} がない"));
        let diff = (got - w).abs();
        self.rows.push((key.to_string(), got, w, diff, diff <= tol));
    }

    /// 語数のようにトークナイザの定義に依存する数を、相対誤差で比べる。
    fn add_count(&mut self, key: &str, got: f64, want: &BTreeMap<String, Value>) {
        let w = want
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("期待値に {key} がない"));
        let diff = (got - w).abs();
        let ok = diff <= w.abs() * COUNT_REL_TOL;
        self.rows.push((key.to_string(), got, w, diff, ok));
    }

    fn assert_all(&self, name: &str) {
        if self.rows.iter().all(|r| r.4) {
            return;
        }
        let mut msg = format!("\n{name} が許容範囲を超えた\n");
        msg.push_str(&format!(
            "{:<20} {:>10} {:>10} {:>10}  {}\n",
            "指標", "Rust", "参照", "差", "判定"
        ));
        for (k, got, want, diff, ok) in &self.rows {
            msg.push_str(&format!(
                "{k:<20} {got:>10.3} {want:>10.3} {diff:>10.3}  {}\n",
                if *ok { "ok" } else { "NG" }
            ));
        }
        panic!("{msg}");
    }
}

#[test]
fn en_prose_metrics_match_reference() {
    let name = "en_prose.md";
    let doc = Document::parse(&read(name));
    let m = suikou_core::metrics::en::analyze(&doc.prose(), &doc.body());
    let want = expected("en", name);

    let mut c = Cmp::new();
    c.add("MATTR100", m.mattr100, &want, RATIO_TOL);
    c.add(
        "simile_marker_1k",
        m.simile_marker_per_1k,
        &want,
        DENSITY_TOL,
    );
    c.add("list_ratio", doc.list_ratio(), &want, RATIO_TOL);
    c.add_count("words", m.words as f64, &want);
    c.assert_all(name);
}

#[cfg(feature = "lindera-unidic")]
mod ja {
    use super::*;
    use suikou_core::tokenizer::LinderaMorphology;

    fn check(name: &str) {
        let doc = Document::parse(&read(name));
        let morph = LinderaMorphology::new().expect("辞書");
        let m = suikou_core::metrics::ja::analyze(&doc.prose(), &doc.body(), &morph);
        let want = expected("ja", name);

        let mut c = Cmp::new();
        // 語彙系。参照実装の `_全` 側が地の文＋箇条書き本文にあたる。
        c.add("MATTR100_全", m.mattr100, &want, RATIO_TOL);
        c.add("漢字率_全", m.kanji_ratio, &want, RATIO_TOL);
        c.add("漢語率_全", m.kango_ratio, &want, RATIO_TOL);
        c.add("指示詞_全千字", m.demonstrative_per_1k, &want, DENSITY_TOL);
        // 文構造系。地の文だけを対象とする。
        c.add("連用中止_千字", m.renyo_per_1k, &want, DENSITY_TOL);
        c.add("テ形接続_千字", m.te_per_1k, &want, DENSITY_TOL);
        // 書式系。
        c.add("地の文比率", doc.prose_ratio(), &want, RATIO_TOL);
        c.add("箇条書き割合", doc.list_ratio(), &want, RATIO_TOL);
        c.add_count("文数", m.sentences as f64, &want);
        c.assert_all(name);
    }

    #[test]
    fn ja_prose_metrics_match_reference() {
        check("ja_prose.md");
    }

    #[test]
    fn ja_maintainability_metrics_match_reference() {
        check("ja_maintainability.md");
    }
}
