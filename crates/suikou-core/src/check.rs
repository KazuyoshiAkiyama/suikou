//! 文書を解析し、閾値を外れた指標と局所指摘を一つのレポートにまとめる。
//!
//! 閾値との突き合わせを CLI ではなくここに置くのは、
//! 出力の形（Markdown、JSON、MCP）によらず判定が同じであることを保つためである。
//!
//! 閾値の範囲内にある指標は載せない。トークンを使わずに済ませる。

use crate::lang::Lang;
use crate::markdown::Document;
use crate::metrics;
use crate::profile::Profile;
use crate::register::{self, Reference};
use crate::report::{Direction, DocumentMetric, Report};
use crate::rules;
use crate::tokenizer::Morphology;
use std::collections::{BTreeMap, HashSet};

/// 指標の名前から値へ。名前はプロファイルの鍵と一致させる。
pub fn measure(doc: &Document, lang: Lang, morph: &dyn Morphology) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    let prose = doc.prose();
    let body = doc.body();
    match lang {
        Lang::Ja => {
            let m = metrics::ja::analyze(&prose, &body, morph);
            out.insert("ja.mattr100".into(), m.mattr100);
            out.insert("ja.renyo_per_1k".into(), m.renyo_per_1k);
            out.insert("ja.te_per_1k".into(), m.te_per_1k);
            out.insert("ja.keishiki_meishi_per_1k".into(), m.keishiki_meishi_per_1k);
            out.insert("ja.demonstrative_per_1k".into(), m.demonstrative_per_1k);
            out.insert("ja.kango_ratio".into(), m.kango_ratio);
            out.insert("ja.kanji_ratio".into(), m.kanji_ratio);
            out.insert("ja.prose_ratio".into(), doc.prose_ratio());
            out.insert("ja.list_ratio".into(), doc.list_ratio());
            // 分母が小さすぎる文書では比を計算しない。載せなければ判定もされない。
            if let Some(r) = m.renyo_te_ratio {
                out.insert("ja.renyo_te_ratio".into(), r);
            }
        }
        Lang::En => {
            let m = metrics::en::analyze(&prose, &body);
            out.insert("en.mattr100".into(), m.mattr100);
            out.insert("en.simile_marker_per_1k".into(), m.simile_marker_per_1k);
            out.insert("en.prose_ratio".into(), doc.prose_ratio());
            out.insert("en.list_ratio".into(), doc.list_ratio());
        }
    }
    out
}

/// 閾値を外れたかどうか。
fn out_of_range(value: f64, threshold: f64, direction: Direction) -> bool {
    match direction {
        Direction::Above => value > threshold,
        Direction::Below => value < threshold,
    }
}

pub fn evaluate(doc: &Document, lang: Lang, morph: &dyn Morphology, profile: &Profile) -> Report {
    evaluate_with_glossary(doc, lang, morph, profile, &HashSet::new())
}

/// 分野で定まった語を渡して判定する。`suikou terms` の出力を渡せば、
/// 分野の語が文体の指摘に出てこなくなる。
pub fn evaluate_with_glossary(
    doc: &Document,
    lang: Lang,
    morph: &dyn Morphology,
    profile: &Profile,
    glossary: &HashSet<String>,
) -> Report {
    let values = measure(doc, lang, morph);
    let mut document = Vec::new();
    for (name, t) in &profile.thresholds {
        let Some(&value) = values.get(name) else {
            continue;
        };
        if !out_of_range(value, t.value, t.direction) {
            continue;
        }
        document.push(DocumentMetric {
            metric: name.clone(),
            value,
            threshold: t.value,
            direction: t.direction,
            severity: t.severity,
            guidance: t.guidance.clone(),
        });
    }
    let mut local = rules::check_all(doc, lang, morph);
    // 文体の規則は日本語だけに当てる。英語の参照コーパスはまだ作っていない。
    if lang == Lang::Ja {
        local.extend(register::check(doc, morph, Reference::builtin(), glossary));
    }
    Report { document, local }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::FakeMorphology;

    fn morph() -> FakeMorphology {
        FakeMorphology::from_spec(
            "設定|名詞|普通名詞||漢 を|助詞|格助詞||和 変更|名詞|普通名詞||漢 \
             し|動詞|非自立可能|連用形-一般|和 て|助詞|接続助詞||和 \
             こと|名詞|普通名詞||和 これ|代名詞|||和 する|動詞|非自立可能|終止形-一般|和",
        )
    }

    #[test]
    fn metrics_inside_the_range_are_not_reported() {
        let p = Profile::builtin("oss").unwrap();
        // 地の文だけの文書。箇条書き割合は 0 で閾値 0.25 の下にある。
        let d = Document::parse("設定を変更する。設定を変更する。\n");
        let r = evaluate(&d, Lang::Ja, &morph(), &p);
        assert!(
            !r.document.iter().any(|m| m.metric == "ja.list_ratio"),
            "{:?}",
            r.document
        );
    }

    #[test]
    fn metrics_outside_the_range_are_reported_with_guidance() {
        let p = Profile::builtin("oss").unwrap();
        let src = "短い導入。\n\n- ああああ\n- いいいい\n- うううう\n";
        let d = Document::parse(src);
        let r = evaluate(&d, Lang::Ja, &morph(), &p);
        let m = r
            .document
            .iter()
            .find(|m| m.metric == "ja.list_ratio")
            .expect("箇条書き割合が出ていない");
        assert_eq!(m.direction, Direction::Above);
        assert!(!m.guidance.is_empty());
    }

    #[test]
    fn local_findings_are_collected() {
        let d = Document::parse("現在の実装では反映されない。\n");
        let p = Profile::builtin("oss").unwrap();
        let r = evaluate(&d, Lang::Ja, &morph(), &p);
        assert!(r
            .local
            .iter()
            .any(|f| f.rule_id == rules::RULE_TIME_DEPENDENT));
    }

    #[test]
    fn ratio_is_skipped_when_the_denominator_is_too_small() {
        // テ形接続がない文書では比を出さない。
        let d = Document::parse("設定を変更し、設定を変更する。\n");
        let v = measure(&d, Lang::Ja, &morph());
        assert!(!v.contains_key("ja.renyo_te_ratio"));
    }
}
