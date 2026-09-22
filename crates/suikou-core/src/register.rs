//! 文体に合わない和語を見つける。
//!
//! 生成モデルは、英語から日本語にするときに漢語を和語へ置き換えることがある。
//! 「確認する」が「確かめる」に、「必要である」が「要る」になる類である。
//! 技術文書の文体としては硬さが足りず、速く読めなくなる。
//!
//! 何が文体に合うかは、プロの日本語訳のコーパスで測って決める。
//! `data/register_ja.toml` は Kubernetes、MDN、Vue の日本語訳から作った和語の頻度表であり、
//! 作り直す手順は `research/lexshift/build_reference.py` にある。
//! 語の対応表は持たない。分野ごとの言い回しの違いをすべて数え上げることになるためである。
//!
//! 判定は、調べる文書での頻度と参照の頻度を対数尤度比で比べる形とする。
//! あるかないかの二値にしないのは、参照を大きくすると稀な語でも数回は現れ、
//! 二値では「文体から外れた語」と「ただ珍しい語」を見分けられなくなるためである。
//! 統計は Dunning (1993) の対数尤度比であり、二つのコーパスを比べる標準の方法である。
//!
//! この規則は日本語だけに当てる。同じ方法を英語で試したが成立しなかった。
//! 語種という手掛かりが無いため、話題の語と文体の語を頻度では分けられない。
//! 経緯は決定の記録の D-28 にある。

use crate::markdown::{Block, BlockKind, Document};
use crate::report::{LocalFinding, Position, Severity};
use crate::tokenizer::Morphology;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::OnceLock;

pub const RULE_RARE_WAGO: &str = "register/rare-wago";
pub const RULE_PREFER_KATAKANA: &str = "register/prefer-katakana";

/// 同じ語がこの回数以上出たときだけ指摘する。
/// 一度きりの語は、文体から外れているのか言い回しの綾なのかを判別できない。
pub const MIN_OCCURRENCES: usize = 3;

/// 対数尤度比の臨界値。自由度1で p < 0.0001 にあたる慣例の値である。
pub const G2_CRITICAL: f64 = 15.13;

/// 効果量の下限。Hardie (2014) の Log Ratio で、参照の 2^7 = 128 倍を境にする。
///
/// 対数尤度比だけでは足りない。標本が大きいと、普通の語を少し多く使っただけでも
/// 有意になる。文体から外れているかどうかは、有意かどうかではなく偏りの大きさで決まる。
///
/// 値は測定で決めた。参照に使っていないプロの日本語訳12頁と、
/// このリポジトリの日本語の文書とで Log Ratio の分布を比べた。
/// 4 ではプロの文書に9語、6 では1語の誤検出が出る。7 でゼロになる。
/// 設計の方針では、人間の書いた文書に警告が出ることを偽陽性として扱う。
/// 取りこぼしは許すが、プロの文書に出る誤検出は許さない側に寄せる。
pub const LOG_RATIO_MIN: f64 = 7.0;

const CONTENT: [&str; 5] = ["名詞", "動詞", "形容詞", "形状詞", "副詞"];

#[derive(Debug, Deserialize)]
struct RawReference {
    meta: RawMeta,
    words: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct RawPairs {
    pairs: std::collections::HashMap<String, String>,
}

/// 漢語よりカタカナ語が優勢な対応表。
///
/// プロの日本語訳で測ると、同じ意味に漢語とカタカナ語の両方が使われる組がある。
/// 分野ごとに逆転する語もあるが、合計で優勢な側に従う。
/// 多義語を避けるため、プロがその漢語をほとんど使っていない組だけを載せてある。
/// 作り直す手順は `research/lexshift/kata_pairs.py` にある。
pub fn katakana_pairs() -> &'static std::collections::HashMap<String, String> {
    static P: OnceLock<std::collections::HashMap<String, String>> = OnceLock::new();
    P.get_or_init(|| {
        let raw: RawPairs = toml::from_str(include_str!("../data/kata_pairs_ja.toml"))
            .expect("同梱の kata_pairs_ja.toml を読めない");
        raw.pairs
    })
}

/// 対応表に載る漢語を見つけて、カタカナ語を示す。
///
/// 統計ではなく対応表で判定するため、出現ごとに指摘する。
/// 文書の分野で別の意味に使っている語は `.suikou/register-allow.toml` で外す。
pub fn check_katakana(
    doc: &Document,
    morph: &dyn Morphology,
    pairs: &std::collections::HashMap<String, String>,
    allow: &HashSet<String>,
) -> Option<LocalFinding> {
    let mut positions = Vec::new();
    for b in body_blocks(doc) {
        let tokens = morph.tokenize(&b.text);
        for (i, t) in tokens.iter().enumerate() {
            if t.pos1 != "名詞" || t.goshu != "漢" {
                continue;
            }
            // 直前が名詞なら複合語の一部とみなして見送る。
            // 「英語版」「第3版」の「版」を「バージョン」に替えると誤りになる。
            if i > 0 && tokens[i - 1].pos1 == "名詞" {
                continue;
            }
            let lemma = if t.lemma.is_empty() {
                &t.surface
            } else {
                &t.lemma
            };
            if allow.contains(lemma) || allow.contains(&t.surface) {
                continue;
            }
            if let Some(kata) = pairs.get(lemma.as_str()) {
                let excerpt: String = b.text.chars().take(44).collect();
                positions.push(Position {
                    line: b.line,
                    column: 1,
                    text: format!("{lemma}→{kata}｜{excerpt}"),
                });
            }
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
        rule_id: RULE_PREFER_KATAKANA.to_string(),
        severity: Severity::Warning,
        message: "プロの技術文書ではカタカナ語の方が優勢である。示した語に置き換える".to_string(),
        truncated: total - kept.len(),
        positions: kept,
    })
}

#[derive(Debug, Deserialize)]
struct RawMeta {
    content_words: u64,
}

/// プロの日本語訳での和語の頻度。語彙素で持つ。
#[derive(Debug, Clone, Default)]
pub struct Reference {
    words: std::collections::HashMap<String, u64>,
    content_words: u64,
}

fn parse(src: &str, name: &str) -> Reference {
    let raw: RawReference =
        toml::from_str(src).unwrap_or_else(|e| panic!("同梱の {name} を読めない: {e}"));
    Reference {
        words: raw.words,
        content_words: raw.meta.content_words,
    }
}

impl Reference {
    /// 日本語の参照。和語の頻度表である。
    pub fn builtin() -> &'static Reference {
        static R: OnceLock<Reference> = OnceLock::new();
        R.get_or_init(|| parse(include_str!("../data/register_ja.toml"), "register_ja.toml"))
    }

    pub fn from_counts<I: IntoIterator<Item = (String, u64)>>(
        words: I,
        content_words: u64,
    ) -> Self {
        Reference {
            words: words.into_iter().collect(),
            content_words,
        }
    }

    pub fn count(&self, lemma: &str) -> u64 {
        self.words.get(lemma).copied().unwrap_or(0)
    }

    /// 対数尤度比。参照より文書の方が多いときだけ正の判定に使う。
    fn over_used(&self, lemma: &str, in_doc: u64, doc_words: u64) -> bool {
        let (a, b) = (in_doc as f64, self.count(lemma) as f64);
        let (n1, n2) = (doc_words.max(1) as f64, self.content_words.max(1) as f64);
        if a / n1 <= b / n2 {
            return false;
        }
        let e1 = n1 * (a + b) / (n1 + n2);
        let e2 = n2 * (a + b) / (n1 + n2);
        let mut g = 0.0;
        if a > 0.0 {
            g += a * (a / e1).ln();
        }
        if b > 0.0 {
            g += b * (b / e2).ln();
        }
        if 2.0 * g < G2_CRITICAL {
            return false;
        }
        // 0.5 を足すのは、参照に一度も出ない語で割り算が壊れるのを防ぐためである。
        let log_ratio = (((a + 0.5) / n1) / ((b + 0.5) / n2)).log2();
        log_ratio >= LOG_RATIO_MIN
    }
}

fn is_target(pos1: &str, pos2: &str, goshu: &str, surface: &str) -> bool {
    if goshu != "和" || !CONTENT.contains(&pos1) || pos2 == "数詞" {
        return false;
    }
    // 1文字のカタカナは、複合語が割れた断片であることが多い。
    let mut chars = surface.chars();
    !matches!(
        (chars.next(), chars.next()),
        (Some('ァ'..='ヶ' | 'ー'), None)
    )
}

/// 語彙の指標と同じ範囲を見る。地の文と箇条書きの本文を対象とする。
fn body_blocks(doc: &Document) -> impl Iterator<Item = &Block> {
    doc.blocks
        .iter()
        .filter(|b| matches!(b.kind, BlockKind::Prose | BlockKind::ListItem))
}

/// 一覧に無い和語が `MIN_OCCURRENCES` 回以上出ていれば指摘する。
///
/// `allow` には、その文書の分野で定まった語を渡す。`suikou terms` の出力が使える。
pub fn check(
    doc: &Document,
    morph: &dyn Morphology,
    reference: &Reference,
    allow: &HashSet<String>,
) -> Option<LocalFinding> {
    let mut hits: Vec<(String, Position)> = Vec::new();
    let mut counts: std::collections::BTreeMap<String, u64> = Default::default();
    let mut doc_words: u64 = 0;
    for b in body_blocks(doc) {
        for t in morph.tokenize(&b.text) {
            if CONTENT.contains(&t.pos1.as_str()) {
                doc_words += 1;
            }
            if !is_target(&t.pos1, &t.pos2, &t.goshu, &t.surface) {
                continue;
            }
            let lemma = if t.lemma.is_empty() {
                t.surface.clone()
            } else {
                t.lemma.clone()
            };
            if allow.contains(&lemma) || allow.contains(&t.surface) {
                continue;
            }
            *counts.entry(lemma.clone()).or_default() += 1;
            // どの語が指摘されたのかを位置に添える。
            // 語が分からないと、どこを直せばよいか読み取れない。
            let excerpt: String = b.text.chars().take(48).collect();
            hits.push((
                lemma.clone(),
                Position {
                    line: b.line,
                    column: 1,
                    text: format!("{lemma}｜{excerpt}"),
                },
            ));
        }
    }
    let positions: Vec<Position> = hits
        .into_iter()
        .filter(|(lemma, _)| {
            let n = counts[lemma];
            n >= MIN_OCCURRENCES as u64 && reference.over_used(lemma, n, doc_words)
        })
        .map(|(_, p)| p)
        .collect();
    if positions.is_empty() {
        return None;
    }
    let total = positions.len();
    let kept: Vec<Position> = positions
        .into_iter()
        .take(crate::report::Report::MAX_POSITIONS_PER_RULE)
        .collect();
    Some(LocalFinding {
        rule_id: RULE_RARE_WAGO.to_string(),
        severity: Severity::Warning,
        message: "プロの技術文書で使われない和語を繰り返し使っている。硬い文体に直す".to_string(),
        truncated: total - kept.len(),
        positions: kept,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::FakeMorphology;

    fn morph() -> FakeMorphology {
        FakeMorphology::from_spec(
            "確かめる|動詞|一般|終止形-一般|和 確認|名詞|普通名詞||漢 する|動詞|非自立可能|終止形-一般|和 \
             を|助詞|格助詞||和 設定|名詞|普通名詞||漢 と|助詞|格助詞||和 \
             三|名詞|数詞||和 形態素解析|名詞|普通名詞||漢 癖|名詞|普通名詞||和",
        )
    }

    fn reference() -> Reference {
        // プロのコーパスを模した頻度。「する」は普通に使われ、「確かめる」はごく稀とする。
        Reference::from_counts(
            [
                ("する".to_string(), 3000u64),
                ("確かめる".to_string(), 2u64),
            ],
            100_000,
        )
    }

    fn doc_of(text: &str) -> Document {
        Document::parse(text)
    }

    #[test]
    fn repeated_word_outside_the_reference_is_reported() {
        let d = doc_of("設定を確かめる。\n\n設定を確かめる。\n\n設定を確かめる。\n");
        let f = check(&d, &morph(), &reference(), &HashSet::new()).expect("指摘が出ていない");
        assert_eq!(f.rule_id, RULE_RARE_WAGO);
        assert_eq!(f.positions.len(), 3);
    }

    #[test]
    fn a_word_used_as_often_as_in_the_reference_is_not_reported() {
        let d = doc_of("設定をする。\n\n設定をする。\n\n設定をする。\n");
        assert!(check(&d, &morph(), &reference(), &HashSet::new()).is_none());
    }

    #[test]
    fn fewer_occurrences_than_the_threshold_are_not_reported() {
        let d = doc_of("設定を確かめる。\n\n設定を確かめる。\n");
        assert!(check(&d, &morph(), &reference(), &HashSet::new()).is_none());
    }

    #[test]
    fn a_word_in_the_project_glossary_is_not_reported() {
        let d = doc_of("設定を確かめる。\n\n設定を確かめる。\n\n設定を確かめる。\n");
        let allow: HashSet<String> = ["確かめる".to_string()].into_iter().collect();
        assert!(check(&d, &morph(), &reference(), &allow).is_none());
    }

    #[test]
    fn kango_is_not_reported_even_when_it_is_absent_from_the_reference() {
        // 分野の用語は漢語やカタカナ語が多い。和語に絞ることで誤って拾わずに済む。
        let d = doc_of("形態素解析をする。\n\n形態素解析をする。\n\n形態素解析をする。\n");
        assert!(check(&d, &morph(), &reference(), &HashSet::new()).is_none());
    }

    #[test]
    fn numerals_are_not_reported() {
        let d = doc_of("三を見る。\n\n三を見る。\n\n三を見る。\n");
        assert!(check(&d, &morph(), &reference(), &HashSet::new()).is_none());
    }

    #[test]
    fn a_kango_in_the_pair_table_is_reported_with_its_katakana() {
        let m = FakeMorphology::from_spec(
            "版|名詞|普通名詞||漢 を|助詞|格助詞||和 見る|動詞|一般|終止形-一般|和",
        );
        let d = Document::parse("版を見る。\n");
        let pairs = [("版".to_string(), "バージョン".to_string())]
            .into_iter()
            .collect();
        let f = check_katakana(&d, &m, &pairs, &HashSet::new()).expect("指摘が出ていない");
        assert_eq!(f.rule_id, RULE_PREFER_KATAKANA);
        assert!(
            f.positions[0].text.starts_with("版→バージョン"),
            "{:?}",
            f.positions[0].text
        );
    }

    #[test]
    fn a_kango_the_project_uses_in_another_sense_can_be_excluded() {
        let m = FakeMorphology::from_spec(
            "版|名詞|普通名詞||漢 を|助詞|格助詞||和 見る|動詞|一般|終止形-一般|和",
        );
        let d = Document::parse("版を見る。\n");
        let pairs = [("版".to_string(), "バージョン".to_string())]
            .into_iter()
            .collect();
        let allow: HashSet<String> = ["版".to_string()].into_iter().collect();
        assert!(check_katakana(&d, &m, &pairs, &allow).is_none());
    }

    #[test]
    fn the_builtin_pair_table_prefers_katakana_for_measured_words() {
        let p = katakana_pairs();
        assert_eq!(p.get("版").map(String::as_str), Some("バージョン"));
        assert_eq!(p.get("利用者").map(String::as_str), Some("ユーザー"));
        // 多義語は載せない。プロも別の意味で使うためである。
        for w in ["対象", "対応", "場合", "設定", "状態"] {
            assert!(!p.contains_key(w), "{w} が対応表にある");
        }
    }

    #[test]
    fn the_builtin_reference_separates_common_words_from_soft_ones() {
        let r = Reference::builtin();
        // 普通に使われる語は、文書で繰り返し出ても指摘しない。
        for w in ["使う", "作る", "持つ", "含む", "行う", "書く"] {
            assert!(!r.over_used(w, 10, 5000), "{w} を指摘してしまう");
        }
        // 文体から外れた語は、同じ回数でも指摘する。
        for w in ["要る", "足す", "確かめる", "揃う", "癖"] {
            assert!(r.over_used(w, 10, 5000), "{w} を指摘できない");
        }
    }

    #[test]
    fn a_rare_word_used_once_or_twice_is_not_reported() {
        // 珍しい語がたまに出るだけなら指摘しない。統計が有意にならないためである。
        let r = Reference::builtin();
        assert!(!r.over_used("確かめる", 2, 5000));
    }
}
