//! 文書の構造を見る規則。
//!
//! 文体の規則と違い、構造の規則は言語を問わない。
//! 節の構成、順序、段落の単位は文書の論理の性質であって、語彙の体系の外にあるためである。
//! 実際に測ると、7文以上の段落はプロの英語で 0.27%、プロの日本語で 0.59% と、
//! 言語をまたいで同じ桁に収まる。言語ごとに持つのは節の名前と文の切り方だけとする。
//!
//! 出典は次のとおり。
//!
//! - 必須の節と推奨する順序という枠組みは RFC 7322 による。
//!   節が下位の節だけを含んでよいことも、そこに明記がある。
//! - 型ごとの節は Rust RFC テンプレート、Kubernetes Enhancement Proposal、
//!   Michael Nygard の Architecture Decision Record、Diátaxis による。
//! - 段落を一つの考えの単位とすることと、5〜6文を超えたら分割の目安とすることは
//!   Google developer documentation style guide による。
//!   段落を構成の単位とすることは Strunk の The Elements of Style にもある。
//! - 標題が主題と文書の性格を示すこと、見出しを追えば全体がつかめることは
//!   文化審議会の「公用文作成の考え方」による。

use crate::lang::Lang;
use crate::markdown::{Block, BlockKind, Document};
use crate::report::{LocalFinding, Position, Severity};
use serde::Deserialize;
use std::sync::OnceLock;

pub const RULE_MISSING_SECTION: &str = "structure/missing-section";
pub const RULE_SECTION_ORDER: &str = "structure/section-order";
pub const RULE_LONG_PARAGRAPH: &str = "structure/long-paragraph";
pub const RULE_EMPTY_SECTION: &str = "structure/empty-section";
pub const RULE_PLOT_MISMATCH: &str = "structure/plot-mismatch";

/// 段落の文の数の上限。これを超えたら分割の目安とする。
/// Google のスタイルガイドが「5〜6文を超えたら多くを詰め込みすぎている目安」と定める。
pub const MAX_SENTENCES_PER_PARAGRAPH: usize = 6;

#[derive(Debug, Deserialize)]
struct RawStructure {
    doctype: std::collections::HashMap<String, DocType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocType {
    description: String,
    #[serde(default)]
    description_en: String,
    pub sections: Vec<Section>,
}

impl DocType {
    /// 型の説明。型枠を出すときに見出しの下に置く。
    pub fn description(&self, lang: Lang) -> &str {
        match lang {
            Lang::Ja => &self.description,
            Lang::En => &self.description_en,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub id: String,
    #[serde(default)]
    pub required: bool,
    /// その節が答える問い。書く前に示すために持つ。
    #[serde(default)]
    question: String,
    #[serde(default)]
    question_en: String,
    /// その節に何を書くか。検出ではなく誘導のために持つ。
    #[serde(default)]
    writes: String,
    #[serde(default)]
    writes_en: String,
    #[serde(default)]
    pub ja: Vec<String>,
    #[serde(default)]
    pub en: Vec<String>,
}

impl Section {
    pub fn names(&self, lang: Lang) -> &[String] {
        match lang {
            Lang::Ja => &self.ja,
            Lang::En => &self.en,
        }
    }

    /// その節が答える問い。
    pub fn question(&self, lang: Lang) -> &str {
        match lang {
            Lang::Ja => &self.question,
            Lang::En => &self.question_en,
        }
    }

    /// その節に何を書くか。
    pub fn writes(&self, lang: Lang) -> &str {
        match lang {
            Lang::Ja => &self.writes,
            Lang::En => &self.writes_en,
        }
    }

    /// 見出しがこの節にあたるかどうか。表記の揺れを同義語で吸収する。
    pub fn matches(&self, heading: &str, lang: Lang) -> bool {
        let h = heading.to_lowercase();
        self.names(lang)
            .iter()
            .any(|n| h.contains(&n.to_lowercase()))
    }
}

pub fn doctypes() -> &'static std::collections::HashMap<String, DocType> {
    static D: OnceLock<std::collections::HashMap<String, DocType>> = OnceLock::new();
    D.get_or_init(|| {
        let raw: RawStructure = toml::from_str(include_str!("../data/structure.toml"))
            .expect("同梱の structure.toml を読めない");
        raw.doctype
    })
}

/// 書き方の指針。違反の検出ではなく、書く前に示すために持つ。
///
/// `suikou plot` と `suikou brief --doctype` がこれを出す。
/// 出どころは段落の単位が Google developer documentation style guide と Strunk、
/// 結論を先に置くことと見出しで筋が分かることが「公用文作成の考え方」である。
/// 文書の型に依存しないため、ここに一か所だけ置く。
pub fn writing_rules(lang: Lang) -> String {
    let n = MAX_SENTENCES_PER_PARAGRAPH;
    match lang {
        Lang::Ja => format!(
            "- 一段落に一つの主張だけを置く。{n}文を超えたら分割する。\n\
             - 段落の最初の文でその段落の主張を述べ、理由と詳細を後に続ける。\n\
             - 節は見出しが立てた問いに答える形で書く。答えていない段落はその節に置かない。\n\
             - 見出しだけを追って全体の筋が分かるようにする。\n\
             - 扱わない範囲を書く。境界を書かないと、読み手が期待を広げる。\n"
        ),
        Lang::En => format!(
            "- Make one claim per paragraph. Split the paragraph once it passes {n} sentences.\n\
             - Put the claim in the first sentence, then the reasons and the detail.\n\
             - Write each section as an answer to the question its heading raises. A paragraph \
             that answers something else belongs in another section.\n\
             - Make the headings alone carry the line of the argument.\n\
             - State what is out of scope. Without the boundary the reader will expand the \
             expectation.\n"
        ),
    }
}

fn heading_level(raw: &str) -> usize {
    raw.trim_start().chars().take_while(|c| *c == '#').count()
}

/// 見出しと、その節が本文を持つかどうか。
pub struct Outline<'a> {
    pub block: &'a Block,
    pub level: usize,
    pub has_body: bool,
    pub has_child: bool,
}

/// 節として扱う見出し。最上位の階層だけを返す。
///
/// 先頭の第1階層の見出しは標題であって節ではないため外す。
/// 「公用文作成の考え方」が標題に主題と文書の性格を示す役割を与えている。
/// 外さないと、標題が同名の節に当たってしまい、節の有無と順序の判定が狂う。
///
/// 残りのうち最も浅い階層だけを節とする。文書の型が定めるのは最上位の並びであり、
/// 下位の見出しまで見ると、節の中の小見出しが別の節に当たって順序の判定が狂う。
pub fn sections(doc: &Document) -> Vec<Outline<'_>> {
    let mut all = outline(doc);
    if all.first().is_some_and(|o| o.level == 1) {
        all.remove(0);
    }
    let Some(top) = all.iter().map(|o| o.level).min() else {
        return all;
    };
    all.retain(|o| o.level == top);
    all
}

pub fn outline(doc: &Document) -> Vec<Outline<'_>> {
    let mut out: Vec<Outline> = Vec::new();
    // 次の見出しまでの間にコードがあれば、その節は本文を持つ。
    let mut heading_lines: Vec<usize> = Vec::new();
    for b in &doc.blocks {
        if b.kind == BlockKind::Heading {
            heading_lines.push(b.line);
        }
    }
    for b in &doc.blocks {
        if b.kind == BlockKind::Heading {
            let level = heading_level(&b.raw);
            for prev in out.iter_mut().rev() {
                if prev.level < level {
                    prev.has_child = true;
                    break;
                }
            }
            let next = heading_lines.iter().copied().find(|l| *l > b.line);
            let has_code = doc
                .code_lines
                .iter()
                .any(|l| *l > b.line && next.is_none_or(|n| *l < n));
            out.push(Outline {
                block: b,
                level,
                has_body: has_code,
                has_child: false,
            });
        } else if let Some(last) = out.last_mut() {
            last.has_body = true;
        }
    }
    out
}

/// 段落の文の数。
///
/// 公開しているのは、コーパスに対する発火率をこの定義のまま測るためである。
/// 測る側で数え方を写すと、実装とずれたまま気付かない。
pub fn sentence_count(text: &str, lang: Lang) -> usize {
    match lang {
        Lang::Ja => crate::markdown::split_sentences_ja(text).len(),
        // 英語の文の切り分けは spaCy に及ばない。数えるだけの用途であり、
        // 閾値を超えるかどうかを見るには足りる。
        //
        // 終止符の直後に空白が続く場合だけを切れ目とする。
        // そうしないと `11.095` や `8.5` の小数点で切れてしまう。
        Lang::En => {
            let b: Vec<char> = text.chars().collect();
            let mut n = 0;
            let mut words = 0;
            for (i, c) in b.iter().enumerate() {
                if c.is_whitespace() {
                    words += 1;
                }
                if !matches!(c, '.' | '!' | '?') {
                    continue;
                }
                let next_is_break = b.get(i + 1).is_none_or(|x| x.is_whitespace());
                if next_is_break && words >= 2 {
                    n += 1;
                    words = 0;
                }
            }
            if words >= 2 {
                n += 1;
            }
            n
        }
    }
}

fn finding(rule: &str, sev: Severity, msg: &str, positions: Vec<Position>) -> Option<LocalFinding> {
    if positions.is_empty() {
        return None;
    }
    let total = positions.len();
    let kept: Vec<Position> = positions
        .into_iter()
        .take(crate::report::Report::MAX_POSITIONS_PER_RULE)
        .collect();
    Some(LocalFinding {
        rule_id: rule.to_string(),
        severity: sev,
        message: msg.to_string(),
        truncated: total - kept.len(),
        positions: kept,
    })
}

fn pos(line: usize, text: String) -> Position {
    Position {
        line,
        column: 1,
        text,
    }
}

/// 型を宣言していない文書にも当てられる規則。言語と型によらない。
pub fn check_universal(doc: &Document, lang: Lang) -> Vec<LocalFinding> {
    let mut out = Vec::new();

    let long: Vec<Position> = doc
        .paragraphs()
        .into_iter()
        .filter_map(|(line, text)| {
            let n = sentence_count(&text, lang);
            (n > MAX_SENTENCES_PER_PARAGRAPH).then(|| {
                pos(
                    line,
                    format!("{n}文｜{}", text.chars().take(40).collect::<String>()),
                )
            })
        })
        .collect();
    out.extend(finding(
        RULE_LONG_PARAGRAPH,
        Severity::Warning,
        "段落が長い。一段落に一つの主張だけを置く",
        long,
    ));

    // 下位の節を持つ節は、本文が無くてもよい。RFC 7322 がそう定めている。
    let empty: Vec<Position> = outline(doc)
        .iter()
        .filter(|o| !o.has_body && !o.has_child)
        .map(|o| pos(o.block.line, o.block.text.clone()))
        .collect();
    out.extend(finding(
        RULE_EMPTY_SECTION,
        Severity::Warning,
        "末端の節に本文が無い。節を書くか、見出しを消す",
        empty,
    ));
    out
}

/// 標題の直後に、最初の節の見出しより前の地の文があるかどうか。
///
/// ここに置いた段落は、見出しを持たなくても最初の節の役目を果たす。
/// RFC 7322 の abstract が標題と目次の間に見出し無しで入る前例であり、
/// README も同じ形を取る。見出しを強いると、かえって読みにくくなる。
fn has_lead(doc: &Document) -> bool {
    let first_section = sections(doc).first().map(|o| o.block.line);
    doc.paragraphs()
        .iter()
        .any(|(line, _)| first_section.is_none_or(|h| *line < h))
}

/// 型を宣言した文書に当てる規則。必須の節と、その順序を見る。
pub fn check_doctype(doc: &Document, lang: Lang, dt: &DocType) -> Vec<LocalFinding> {
    let heads = sections(doc);
    let lead = has_lead(doc);
    let mut out = Vec::new();
    let mut missing = Vec::new();
    let mut found: Vec<(usize, usize)> = Vec::new(); // (定義の順, 見出しの位置)
    for (i, sec) in dt.sections.iter().enumerate() {
        match heads.iter().position(|h| sec.matches(&h.block.text, lang)) {
            Some(at) => found.push((i, at)),
            // 最初の節だけは、標題の直下の導入で代えられる。
            None if i == 0 && lead => {}
            None if sec.required => {
                // 指摘は欠落を告げるだけにせず、何を書くかまで示す。
                let name = sec.names(lang).first().cloned().unwrap_or_default();
                missing.push(pos(
                    1,
                    format!("{name}｜{} — {}", sec.question(lang), sec.writes(lang)),
                ));
            }
            None => {}
        }
    }
    out.extend(finding(
        RULE_MISSING_SECTION,
        Severity::Error,
        "この型の文書に要る節が無い",
        missing,
    ));

    let mut disorder = Vec::new();
    for w in found.windows(2) {
        if w[1].1 < w[0].1 {
            let h = &heads[w[1].1];
            disorder.push(pos(
                h.block.line,
                format!("{}｜{}", h.block.text, "推奨する順序より前にある"),
            ));
        }
    }
    out.extend(finding(
        RULE_SECTION_ORDER,
        Severity::Warning,
        "節の順序が推奨する順序と違う",
        disorder,
    ));
    out
}

/// 承認したプロットと本文の見出しを突き合わせる。
///
/// プロットは版管理しない作業用の文書であり、`.suikou/plans/` に置く。
/// そのため CI では検査できない。手元とエージェントのループの中で使う。
pub fn check_plot(doc: &Document, plot_headings: &[String]) -> Vec<LocalFinding> {
    let heads: Vec<(usize, String)> = sections(doc)
        .iter()
        .map(|o| (o.block.line, o.block.text.clone()))
        .collect();
    let mut diff = Vec::new();
    for h in plot_headings {
        if !heads.iter().any(|(_, x)| x == h) {
            diff.push(pos(1, format!("プロットにあって本文に無い｜{h}")));
        }
    }
    for (line, h) in &heads {
        if !plot_headings.iter().any(|x| x == h) {
            diff.push(pos(*line, format!("本文にあってプロットに無い｜{h}")));
        }
    }
    finding(
        RULE_PLOT_MISMATCH,
        Severity::Warning,
        "承認したプロットと構成が違う。プロットを直すか、本文を直す",
        diff,
    )
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(src: &str) -> Document {
        Document::parse(src)
    }

    fn rules(f: &[LocalFinding]) -> Vec<&str> {
        f.iter().map(|x| x.rule_id.as_str()).collect()
    }

    #[test]
    fn every_doctype_names_its_sections_in_both_languages() {
        for (name, dt) in doctypes() {
            assert!(
                !dt.description(Lang::Ja).is_empty(),
                "{name} ja description"
            );
            assert!(
                !dt.description(Lang::En).is_empty(),
                "{name} en description"
            );
            for s in &dt.sections {
                for lang in [Lang::Ja, Lang::En] {
                    assert!(!s.names(lang).is_empty(), "{name}/{} names {lang:?}", s.id);
                    assert!(
                        !s.question(lang).is_empty(),
                        "{name}/{} question {lang:?}",
                        s.id
                    );
                    assert!(
                        !s.writes(lang).is_empty(),
                        "{name}/{} writes {lang:?}",
                        s.id
                    );
                }
            }
        }
    }

    #[test]
    fn missing_required_section_is_an_error() {
        let dt = doctypes().get("decision").unwrap();
        let src = "# 決定\n\n## 文脈\n\n事情を書く。\n\n## 決定\n\n決めた。\n";
        let f = check_doctype(&doc(src), Lang::Ja, dt);
        let m = f
            .iter()
            .find(|x| x.rule_id == RULE_MISSING_SECTION)
            .unwrap();
        assert_eq!(m.severity, Severity::Error);
        assert_eq!(m.positions.len(), 1);
        // 欠落を告げるだけでなく、その節に何を書くかまで示す。
        assert!(m.positions[0].text.contains("帰結"));
        assert!(m.positions[0].text.contains("良い面と悪い面"));
    }

    #[test]
    fn a_complete_document_reports_nothing() {
        let dt = doctypes().get("decision").unwrap();
        let src = "# 決定\n\n## 文脈\n\n事情。\n\n## 決定\n\n決めた。\n\n## 帰結\n\nこうなる。\n";
        assert!(check_doctype(&doc(src), Lang::Ja, dt).is_empty());
    }

    #[test]
    fn synonyms_satisfy_a_required_section() {
        let dt = doctypes().get("decision").unwrap();
        let src = "# 決定\n\n## 背景\n\n事情。\n\n## 決定\n\n決めた。\n\n## 影響\n\nこうなる。\n";
        assert!(check_doctype(&doc(src), Lang::Ja, dt).is_empty());
    }

    #[test]
    fn section_order_is_a_warning() {
        let dt = doctypes().get("decision").unwrap();
        let src = "# 決定\n\n## 決定\n\n決めた。\n\n## 文脈\n\n事情。\n\n## 帰結\n\nこうなる。\n";
        let f = check_doctype(&doc(src), Lang::Ja, dt);
        let o = f.iter().find(|x| x.rule_id == RULE_SECTION_ORDER).unwrap();
        assert_eq!(o.severity, Severity::Warning);
        assert_eq!(o.positions[0].text.split('｜').next().unwrap(), "決定");
    }

    #[test]
    fn english_headings_satisfy_the_same_doctype() {
        let dt = doctypes().get("decision").unwrap();
        let src = "# ADR\n\n## Context\n\nThe situation.\n\n## Decision\n\nWe chose it.\n\n\
                   ## Consequences\n\nThis follows.\n";
        assert!(check_doctype(&doc(src), Lang::En, dt).is_empty());
    }

    #[test]
    fn a_paragraph_over_the_limit_is_flagged() {
        let long = "短い文である。".repeat(MAX_SENTENCES_PER_PARAGRAPH + 1);
        let src = format!("# 見出し\n\n{long}\n");
        let f = check_universal(&doc(&src), Lang::Ja);
        let p = f.iter().find(|x| x.rule_id == RULE_LONG_PARAGRAPH).unwrap();
        assert_eq!(p.severity, Severity::Warning);
    }

    #[test]
    fn a_paragraph_at_the_limit_passes() {
        let ok = "短い文である。".repeat(MAX_SENTENCES_PER_PARAGRAPH);
        let src = format!("# 見出し\n\n{ok}\n");
        assert!(!rules(&check_universal(&doc(&src), Lang::Ja)).contains(&RULE_LONG_PARAGRAPH));
    }

    // 英語の文の切り分けが小数点で切れると、段落の文の数が水増しされる。
    #[test]
    fn a_decimal_point_does_not_count_as_a_sentence_end() {
        let src = "# Heading\n\nThe value is 11.095 and the other is 8.5 in the same run.\n";
        assert!(!rules(&check_universal(&doc(src), Lang::En)).contains(&RULE_LONG_PARAGRAPH));
        assert_eq!(
            sentence_count("The value is 11.095 and 8.5 here. Then it ends.", Lang::En),
            2
        );
    }

    #[test]
    fn a_leaf_section_without_a_body_is_flagged() {
        let src = "# 見出し\n\n本文である。\n\n## 空の節\n\n## 次の節\n\n本文である。\n";
        let f = check_universal(&doc(src), Lang::Ja);
        let e = f.iter().find(|x| x.rule_id == RULE_EMPTY_SECTION).unwrap();
        assert_eq!(e.positions.len(), 1);
        assert_eq!(e.positions[0].text, "空の節");
    }

    // RFC 7322 は、下位の節だけを含む節を認めている。
    #[test]
    fn a_section_holding_only_subsections_passes() {
        let src = "# 見出し\n\n本文である。\n\n## 親\n\n### 子\n\n本文である。\n";
        assert!(!rules(&check_universal(&doc(src), Lang::Ja)).contains(&RULE_EMPTY_SECTION));
    }

    // コードだけの節は本文を持つものとして数える。
    #[test]
    fn a_section_whose_body_is_only_code_passes() {
        let src = "# 見出し\n\n本文である。\n\n## 使い方\n\n```sh\nsuikou check\n```\n";
        assert!(!rules(&check_universal(&doc(src), Lang::Ja)).contains(&RULE_EMPTY_SECTION));
    }

    #[test]
    fn plot_mismatch_reports_both_directions() {
        let src = "# 題\n\n## ある節\n\n本文である。\n\n## 余った節\n\n本文である。\n";
        let plot = vec!["ある節".to_string(), "足りない節".to_string()];
        let f = check_plot(&doc(src), &plot);
        let m = &f[0];
        assert_eq!(m.rule_id, RULE_PLOT_MISMATCH);
        assert_eq!(m.positions.len(), 2);
        assert!(m.positions[0].text.contains("足りない節"));
        assert!(m.positions[1].text.contains("余った節"));
        // 位置は本文の見出しの行を指す。
        assert_eq!(m.positions[1].line, 7);
    }

    #[test]
    fn a_body_matching_the_plot_reports_nothing() {
        let src = "# 題\n\n## ある節\n\n本文である。\n";
        let plot = vec!["ある節".to_string()];
        assert!(check_plot(&doc(src), &plot).is_empty());
    }

    #[test]
    fn writing_rules_exist_in_both_languages() {
        for lang in [Lang::Ja, Lang::En] {
            let r = writing_rules(lang);
            assert!(r.lines().count() >= 4, "{lang:?}: {r}");
            // 上限は定数から取る。定数を変えたときに文面が取り残されないようにする。
            assert!(r.contains(&MAX_SENTENCES_PER_PARAGRAPH.to_string()), "{r}");
        }
    }

    // 標題直下の導入は、最初の節の代わりになる。README がこの形を取る。
    #[test]
    fn a_lead_paragraph_stands_in_for_the_first_section() {
        let dt = doctypes().get("overview").unwrap();
        let src = "# suikou\n\n何であるかをここで述べる。\n\n## 使い方\n\n手順である。\n";
        assert!(check_doctype(&Document::parse(src), Lang::Ja, dt).is_empty());
    }

    // 代えられるのは最初の節だけとする。ほかの節は見出しを求める。
    #[test]
    fn a_lead_paragraph_does_not_stand_in_for_a_later_section() {
        let dt = doctypes().get("overview").unwrap();
        let src = "# suikou\n\n何であるかをここで述べる。\n\n## なぜ\n\n理由である。\n";
        let f = check_doctype(&Document::parse(src), Lang::Ja, dt);
        let m = f
            .iter()
            .find(|x| x.rule_id == RULE_MISSING_SECTION)
            .unwrap();
        assert_eq!(m.positions.len(), 1);
        assert!(m.positions[0].text.starts_with("使い方"));
    }

    // 標題のすぐ後に節が続く文書には導入が無い。
    #[test]
    fn a_document_with_no_lead_still_needs_the_first_section() {
        let dt = doctypes().get("overview").unwrap();
        let src = "# suikou\n\n## 使い方\n\n手順である。\n";
        let f = check_doctype(&Document::parse(src), Lang::Ja, dt);
        let m = f
            .iter()
            .find(|x| x.rule_id == RULE_MISSING_SECTION)
            .unwrap();
        assert!(m.positions[0].text.starts_with("何であるか"));
    }

    // 節の中の小見出しは節ではない。型が定めるのは最上位の並びである。
    #[test]
    fn a_subheading_does_not_satisfy_a_section() {
        let dt = doctypes().get("decision").unwrap();
        let src = "# 題\n\n## 文脈\n\n事情。\n\n### 帰結\n\n下位の見出し。\n\n\
                   ## 決定\n\n決めた。\n";
        let f = check_doctype(&Document::parse(src), Lang::Ja, dt);
        let m = f
            .iter()
            .find(|x| x.rule_id == RULE_MISSING_SECTION)
            .unwrap();
        assert!(m.positions[0].text.starts_with("帰結"));
    }
}
