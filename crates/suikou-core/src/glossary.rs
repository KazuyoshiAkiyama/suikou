//! 用語集に沿わない書き方を見る規則。
//!
//! 見るのは2つである。
//!
//! - 表記。同じ語の書き分けを1つに決める。`サーバ` を `サーバー` に寄せる。
//! - 多字の直訳語。英語を一語ずつ写した言い方を、定着した言い方に寄せる。
//!
//! 語の選択そのものは `register/prefer-katakana` が持つ。
//! 同じ箇所を2つの規則が指すと、直す側は同じ指摘を二度読むことになる。
//! この規則は、そちらが持たない表記と多字の直訳語だけを見る。
//!
//! 単字の直訳語（版、枝、題、頭、印）はここで見ない。
//! 部分一致では「問題」の中の「題」を拾ってしまい、形態素解析が要る。
//! `prefer-katakana` が形態素で見ており、重ねる必要がない。
//!
//! 用語集の出どころは `data/glossary_ja.toml` にある。
//! 表記は JTCA の規約を出発点にして参照コーパスで確かめ、
//! 規約と実測が食い違う語は実測に従っている。

use crate::markdown::{BlockKind, Document};
use crate::report::{LocalFinding, Position, Severity};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::OnceLock;

pub const RULE_VARIANT: &str = "glossary/variant";

#[derive(Debug, Deserialize)]
struct Raw {
    #[serde(default)]
    orthography: Vec<Orthography>,
    #[serde(default)]
    term: Vec<Term>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Orthography {
    pub preferred: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Term {
    pub concept: String,
    pub preferred: String,
    #[serde(default)]
    pub rejected: Vec<String>,
    /// 文字どおりの意味で使う形。ここに当たる箇所は置き換えない。
    #[serde(default)]
    pub keep: Vec<String>,
}

/// 置き換えの一覧。見つける形と、寄せる先を持つ。
pub struct Entry {
    pub from: String,
    pub to: String,
    /// この形の中に現れる場合は見送る。
    pub keep: Vec<String>,
    /// 直後に長音符号が続く場合は見送る。`サーバ` と `サーバー` を分けるために要る。
    pub guard_choon: bool,
}

pub fn entries() -> &'static Vec<Entry> {
    static E: OnceLock<Vec<Entry>> = OnceLock::new();
    E.get_or_init(|| {
        let raw: Raw = toml::from_str(include_str!("../data/glossary_ja.toml"))
            .expect("同梱の glossary_ja.toml を読めない");
        let mut out = Vec::new();
        for o in &raw.orthography {
            for v in &o.variants {
                // 寄せる先が見つける形を含む場合だけ、長音の番を置く。
                // `サーバ` は `サーバー` の一部であり、守らないと正しい表記を指摘する。
                let guard_choon = o.preferred.starts_with(v.as_str());
                out.push(Entry {
                    from: v.clone(),
                    to: o.preferred.clone(),
                    keep: Vec::new(),
                    guard_choon,
                });
            }
        }
        for t in &raw.term {
            for r in &t.rejected {
                // 単字は部分一致で誤るため、ここでは見ない。
                if r.chars().count() < 2 {
                    continue;
                }
                out.push(Entry {
                    from: r.clone(),
                    to: t.preferred.clone(),
                    keep: t.keep.clone(),
                    guard_choon: false,
                });
            }
        }
        out
    })
}

/// 用語集の語をすべて返す。許可の一覧と突き合わせるために公開する。
pub fn concepts() -> Vec<String> {
    let raw: Raw = toml::from_str(include_str!("../data/glossary_ja.toml")).expect("読めない");
    raw.term.iter().map(|t| t.concept.clone()).collect()
}

fn body_blocks(doc: &Document) -> impl Iterator<Item = &crate::markdown::Block> {
    doc.blocks
        .iter()
        .filter(|b| matches!(b.kind, BlockKind::Prose | BlockKind::ListItem))
}

pub fn check(doc: &Document, allow: &HashSet<String>) -> Option<LocalFinding> {
    let mut positions: Vec<Position> = Vec::new();
    for b in body_blocks(doc) {
        for e in entries() {
            if allow.contains(&e.from) {
                continue;
            }
            for (at, _) in b.text.match_indices(&e.from) {
                let rest = &b.text[at + e.from.len()..];
                if e.guard_choon && rest.starts_with('ー') {
                    continue;
                }
                // 文字どおりの意味で使う形に入っている箇所は置き換えない。
                if e.keep.iter().any(|k| {
                    b.text
                        .match_indices(k.as_str())
                        .any(|(ks, _)| ks <= at && at + e.from.len() <= ks + k.len())
                }) {
                    continue;
                }
                let column = b.text[..at].chars().count() + 1;
                let excerpt: String = b.text.chars().take(40).collect();
                positions.push(Position {
                    line: b.line,
                    column,
                    text: format!("{}→{}｜{excerpt}", e.from, e.to),
                });
            }
        }
    }
    if positions.is_empty() {
        return None;
    }
    positions.sort_by_key(|p| (p.line, p.column));
    positions.dedup_by_key(|p| (p.line, p.column));
    let total = positions.len();
    let kept: Vec<Position> = positions
        .into_iter()
        .take(crate::report::Report::MAX_POSITIONS_PER_RULE)
        .collect();
    Some(LocalFinding {
        rule_id: RULE_VARIANT.to_string(),
        severity: Severity::Warning,
        message: "用語集の表記と違う。示した形に寄せる".to_string(),
        truncated: total - kept.len(),
        positions: kept,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn flags_a_variant_spelling() {
        let d = Document::parse("サーバの設定を変更する。\n");
        let f = check(&d, &none()).unwrap();
        assert_eq!(f.rule_id, RULE_VARIANT);
        assert_eq!(f.severity, Severity::Warning);
        assert!(f.positions[0].text.starts_with("サーバ→サーバー"));
    }

    // 正しい表記を指摘してはならない。`サーバ` は `サーバー` の一部である。
    #[test]
    fn the_preferred_spelling_is_not_flagged() {
        for src in ["サーバーの設定を変更する。\n", "ユーザーに権限を与える。\n"]
        {
            assert!(check(&Document::parse(src), &none()).is_none(), "{src}");
        }
    }

    // 規約と実測が食い違う語は実測に従う。長音を付けた形を指摘する。
    #[test]
    fn a_measured_exception_runs_the_other_way() {
        let d = Document::parse("コンテナーを起動する。\n");
        let f = check(&d, &none()).unwrap();
        assert!(f.positions[0].text.starts_with("コンテナー→コンテナ"));
        assert!(check(&Document::parse("コンテナを起動する。\n"), &none()).is_none());
    }

    #[test]
    fn flags_a_multi_character_calque() {
        let d = Document::parse("根因を記録する。\n");
        let f = check(&d, &none()).unwrap();
        assert!(f.positions[0].text.starts_with("根因→根本原因"));
    }

    // 単字は部分一致で誤る。`問題` の中の `題` を拾ってはならない。
    #[test]
    fn a_single_character_calque_is_left_to_the_morphological_rule() {
        assert!(entries().iter().all(|e| e.from.chars().count() >= 2));
        assert!(check(&Document::parse("問題を記録する。\n"), &none()).is_none());
    }

    // 文字どおりの意味で使う形は置き換えない。
    #[test]
    fn a_literal_sense_is_kept() {
        assert!(check(&Document::parse("基準値を上回る。\n"), &none()).is_none());
    }

    #[test]
    fn an_allowed_word_is_skipped() {
        let allow: HashSet<String> = ["サーバ".to_string()].into_iter().collect();
        assert!(check(&Document::parse("サーバの設定。\n"), &allow).is_none());
    }

    // 見出しとコードは対象にしない。地の文と箇条書きだけを見る。
    #[test]
    fn a_heading_is_not_checked() {
        assert!(check(&Document::parse("## サーバの設定\n"), &none()).is_none());
    }

    #[test]
    fn every_entry_declares_both_sides() {
        for e in entries() {
            assert!(!e.from.is_empty() && !e.to.is_empty());
            assert_ne!(e.from, e.to);
        }
    }

    #[test]
    fn concepts_are_named() {
        let c = concepts();
        assert!(c.iter().all(|x| !x.is_empty()));
        assert!(c.contains(&"version".to_string()));
    }
}
