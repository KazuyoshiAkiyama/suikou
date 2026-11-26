//! 見出しに当てる規則。
//!
//! 見出しの文字列を対象とする判定をここに集める。ほかのブロックは見ない。
//! 規則ごとに対象ブロックを個別に絞る形では、どの規則が見出しに当たるかが
//! コードを読むまで分からない。その境界をこのモジュールで明示する。
//!
//! | 層 | 見出し |
//! |---|---|
//! | 文書指標（`ja.*`、`en.*`） | 見ない。`Document::body` が地の文とリスト項目だけを連結する |
//! | `register/rare-wago`、`register/prefer-katakana` | 見ない |
//! | `glossary/variant` | 見ない |
//! | `structure/heading-form` | 見る。章タイトルだけを見る |
//! | `maint/numbered-heading` | 見る。標題を除く見出しを見る |
//! | `maint/time-dependent` | 見る。標題を含むすべての見出しを見る |
//! | `glossary/defined-name` | 見る。本文と合わせて見る |
//!
//! 規則の識別子ごとに `LocalFinding` は1件だけ作る。`report.rs` の `to_markdown` は
//! `local` の要素ごとに節を出力するため、同じ識別子で2件返すと説明文が二重に出る。
//! 本文と見出しの双方に当たる規則は、ここが `Position` を返して本文側が束ねる。

use crate::markdown::Document;
use crate::report::{LocalFinding, Position, Severity};
use crate::tokenizer::Morphology;

pub const RULE_HEADING_FORM: &str = "structure/heading-form";

/// 日本語の文字を含むかどうか。
///
/// 含まない見出しは判定の対象にしない。`## ja.renyo_per_1k` や `## JSON` のような
/// 識別子の見出しは、形態素解析の結果で形を論じても意味を持たない。
fn has_japanese(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(c,
            '\u{3040}'..='\u{309f}' | '\u{30a0}'..='\u{30ff}' | '\u{4e00}'..='\u{9fff}')
    })
}

/// 章タイトルが用言で終わっているかどうか。
///
/// 対象は `structure::sections` が返す見出しに限る。先頭の第1階層の見出しは標題であって
/// 節ではなく、残りのうち最も浅い階層だけが章である。階層を2に固定しないのは、
/// `##` から始まる文書と `###` から始まる文書を同じ定義で扱うためである。
///
/// 参照コーパスでの章タイトルの用言率は、人間のネイティブ執筆 798件で 4.8%、
/// 公開 ADR 123件で 1.8% である。人間と AI では分離しない（AI の出力 150件で 6.6%）。
/// したがってこれは文体判定の指標ではなく、優勢な慣行からの逸脱を見る規則である。
/// 経緯は決定の記録の D-52 にある。
///
/// 呼び出し側は日本語の文書に限って呼ぶ。英語の経路の形態素解析器は
/// `unreachable!` であり、言語で絞らないと英語と判定された文書で落ちる。
pub fn check_form(doc: &Document, morph: &dyn Morphology) -> Option<LocalFinding> {
    let mut positions: Vec<Position> = Vec::new();
    for o in crate::structure::sections(doc) {
        let text = trim_tail(&o.block.text);
        if text.is_empty() || !has_japanese(text) {
            continue;
        }
        if morph.tokenize(text).last().is_some_and(|t| t.is_yougen()) {
            positions.push(Position {
                line: o.block.line,
                column: 1,
                text: text.chars().take(60).collect(),
            });
        }
    }
    if positions.is_empty() {
        return None;
    }
    let total = positions.len();
    let kept: Vec<Position> = positions
        .into_iter()
        .take(crate::report::Report::MAX_POSITIONS_PER_RULE)
        .collect();
    Some(LocalFinding {
        rule_id: RULE_HEADING_FORM.to_string(),
        severity: Severity::Warning,
        message: "章タイトルが用言で終わっている。体言止めの名詞句にする".to_string(),
        truncated: total - kept.len(),
        positions: kept,
    })
}

/// 見出しの末尾から、形の判定に関わらない部分を落とす。
///
/// `（完了）` のような注記と句読点を落とす。落とさないと最後の形態素が
/// 補助記号になり、`## 設定を変更する（任意）` の用言止めを取り逃がす。
///
/// 開き括弧の無い閉じ括弧では何も落とさない。`## 設定する（任意` のような
/// 閉じ忘れで、落とす範囲を決められないためである。
pub fn trim_tail(text: &str) -> &str {
    let mut s = text.trim();
    loop {
        let before = s;
        if let Some(rest) = s.strip_suffix('）') {
            if let Some(i) = rest.rfind('（') {
                s = &rest[..i];
            }
        } else if let Some(rest) = s.strip_suffix(')') {
            if let Some(i) = rest.rfind('(') {
                s = &rest[..i];
            }
        }
        s = s.trim_end_matches(|c: char| {
            matches!(c, '。' | '：' | ':' | '、' | '！' | '？' | '.' | ' ' | '　')
        });
        if s == before {
            return s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tail_of_a_heading_is_trimmed() {
        assert_eq!(trim_tail("設定を変更する（任意）"), "設定を変更する");
        assert_eq!(
            trim_tail("T14 ルールを導入する（済み）"),
            "T14 ルールを導入する"
        );
        assert_eq!(trim_tail("検証の方法。"), "検証の方法");
        assert_eq!(
            trim_tail("register/rare-wago: 文体にそぐわない和語"),
            "register/rare-wago: 文体にそぐわない和語"
        );
        assert_eq!(trim_tail("List lead-ins (M1)"), "List lead-ins");
        assert_eq!(trim_tail("設定する（任意"), "設定する（任意");
        assert_eq!(trim_tail("（）"), "");
        assert_eq!(trim_tail("   "), "");
    }
}

#[cfg(test)]
mod form_tests {
    use super::*;
    use crate::tokenizer::FakeMorphology;

    fn morph() -> FakeMorphology {
        FakeMorphology::from_spec(
            "構成|名詞|普通名詞||漢 を|助詞|格助詞||和 設計|名詞|普通名詞||漢 \
             する|動詞|非自立可能|終止形-一般|和 検証|名詞|普通名詞||漢 の|助詞|格助詞||和 \
             方法|名詞|普通名詞||漢 本文|名詞|普通名詞||漢 で|助動詞||連用形-一般|和 \
             ある|動詞|非自立可能|終止形-一般|和 ja|名詞|普通名詞||外 \
             renyo|名詞|普通名詞||外 per|名詞|普通名詞||外 k|名詞|普通名詞||外 \
             任意|名詞|普通名詞||漢",
        )
    }

    #[test]
    fn a_section_heading_ending_in_a_predicate_is_reported() {
        let d = Document::parse("# 標題\n\n## 構成を設計する\n\n本文である。\n");
        let f = check_form(&d, &morph()).expect("発火していない");
        assert_eq!(f.rule_id, RULE_HEADING_FORM);
        assert_eq!(f.severity, Severity::Warning);
        assert_eq!(f.positions.len(), 1);
        assert_eq!(f.positions[0].line, 3);
    }

    #[test]
    fn a_nominal_section_heading_is_not_reported() {
        let d = Document::parse("# 標題\n\n## 検証の方法\n\n本文である。\n");
        assert!(check_form(&d, &morph()).is_none());
    }

    #[test]
    fn the_title_is_not_a_section_heading() {
        let d = Document::parse("# 構成を設計する\n\n本文である。\n");
        assert!(check_form(&d, &morph()).is_none());
    }

    #[test]
    fn a_document_without_sections_is_not_reported() {
        let d = Document::parse("# 標題\n\n本文である。\n");
        assert!(check_form(&d, &morph()).is_none());
    }

    #[test]
    fn the_shallowest_level_is_the_section_even_when_it_is_the_third() {
        let d = Document::parse("### 構成を設計する\n\n本文である。\n");
        let f = check_form(&d, &morph()).expect("第3階層が章タイトルになっていない");
        assert_eq!(f.positions[0].line, 1);
    }

    #[test]
    fn a_nested_heading_is_not_a_section_heading() {
        let d = Document::parse("# 標題\n\n## 検証の方法\n\n### 構成を設計する\n\n本文である。\n");
        assert!(check_form(&d, &morph()).is_none());
    }

    #[test]
    fn a_heading_without_japanese_is_not_reported() {
        let d = Document::parse("# 標題\n\n## ja.renyo_per_1k\n\n本文である。\n");
        assert!(check_form(&d, &morph()).is_none());
    }

    #[test]
    fn a_parenthesised_suffix_does_not_hide_the_predicate() {
        let d = Document::parse("# 標題\n\n## 構成を設計する（任意）\n\n本文である。\n");
        assert!(check_form(&d, &morph()).is_some());
    }
}
