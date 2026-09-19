//! 日本語の文書指標。UniDic の語種と活用形を使う。
//! IPADIC では語種も活用形も得られないため、辞書は UniDic 系に限る。

use crate::markdown::{count_chars, is_japanese_char, is_kanji, split_sentences_ja};
use crate::metrics::{mattr, per_1k};
use crate::tokenizer::Morphology;
use serde::Serialize;

/// AI はこれらを避け、漢語の名詞化に置き換える傾向がある。
pub const KEISHIKI_MEISHI: &[&str] = &["こと", "もの", "ため", "よう", "点", "場合"];

pub const DEMONSTRATIVES: &[&str] = &[
    "これ",
    "それ",
    "この",
    "その",
    "これら",
    "それら",
    "こちら",
    "そちら",
    "ここ",
    "そこ",
];

/// テ形接続の分母がこれを下回る文書では比を計算しない。
pub const MIN_TE_FOR_RATIO: f64 = 0.05;

#[derive(Debug, Clone, Default, Serialize)]
pub struct JaMetrics {
    pub mattr100: f64,
    /// D3。人間は 2.0 前後、AI は 3.6 から 15.9。最も強く分離する。
    /// 分母が小さすぎる場合は None。
    pub renyo_te_ratio: Option<f64>,
    pub renyo_per_1k: f64,
    pub te_per_1k: f64,
    /// D5。AI は人間より少ない。欠落の指摘になる。
    pub keishiki_meishi_per_1k: f64,
    /// D6。AI は人間より少ない。日本語のゼロ照応と関係する。
    pub demonstrative_per_1k: f64,
    /// D7。自立語のうち語種が漢であるものの割合。
    pub kango_ratio: f64,
    pub kanji_ratio: f64,
    pub sentences: usize,
    pub prose_chars: usize,
}

pub fn analyze(prose: &str, body: &str, morph: &dyn Morphology) -> JaMetrics {
    let body_toks = morph.tokenize(body);
    let surfaces: Vec<String> = body_toks.iter().map(|t| t.surface.clone()).collect();
    let body_chars = count_chars(body);
    let prose_chars = count_chars(prose);

    let mut kango = 0usize;
    let mut goshu_total = 0usize;
    for t in &body_toks {
        if t.is_content_word() {
            match t.goshu.as_str() {
                "漢" => {
                    kango += 1;
                    goshu_total += 1;
                }
                "和" | "外" | "混" => goshu_total += 1,
                _ => {}
            }
        }
    }

    let keishiki = body_toks
        .iter()
        .filter(|t| t.pos1 == "名詞" && KEISHIKI_MEISHI.contains(&t.surface.as_str()))
        .count();
    let demo = body_toks
        .iter()
        .filter(|t| DEMONSTRATIVES.contains(&t.surface.as_str()))
        .count();

    let prose_toks = morph.tokenize(prose);
    let mut renyo = 0usize;
    let mut te = 0usize;
    for i in 0..prose_toks.len().saturating_sub(1) {
        let next_is_comma = prose_toks[i + 1].surface == "、";
        if !next_is_comma {
            continue;
        }
        if prose_toks[i].is_renyo_verb() {
            renyo += 1;
        }
        if prose_toks[i].is_te_conjunctive() {
            te += 1;
        }
    }

    let kanji = body.chars().filter(|c| is_kanji(*c)).count();
    let ja_chars = body.chars().filter(|c| is_japanese_char(*c)).count();

    let renyo_per_1k = per_1k(renyo, prose_chars);
    let te_per_1k = per_1k(te, prose_chars);
    let ratio = if te_per_1k < MIN_TE_FOR_RATIO {
        None
    } else {
        Some(renyo_per_1k / te_per_1k)
    };

    JaMetrics {
        mattr100: mattr(&surfaces, 100),
        renyo_te_ratio: ratio,
        renyo_per_1k,
        te_per_1k,
        keishiki_meishi_per_1k: per_1k(keishiki, body_chars),
        demonstrative_per_1k: per_1k(demo, body_chars),
        kango_ratio: if goshu_total == 0 {
            0.0
        } else {
            kango as f64 / goshu_total as f64
        },
        kanji_ratio: if ja_chars == 0 {
            0.0
        } else {
            kanji as f64 / ja_chars as f64
        },
        sentences: split_sentences_ja(prose).len(),
        prose_chars,
    }
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
    fn counts_renyo_before_comma() {
        let m = analyze(
            "設定を変更し、設定を変更する。",
            "設定を変更し、設定を変更する。",
            &morph(),
        );
        assert!(m.renyo_per_1k > 0.0);
    }

    #[test]
    fn counts_te_before_comma() {
        let m = analyze("設定を変更して、", "設定を変更して、", &morph());
        assert!(m.te_per_1k > 0.0);
    }

    #[test]
    fn ratio_is_none_when_te_is_absent() {
        let m = analyze("設定を変更し、", "設定を変更し、", &morph());
        assert!(m.renyo_te_ratio.is_none());
    }

    #[test]
    fn kango_ratio_counts_only_content_words() {
        // 自立語は 設定(漢) 変更(漢) する(和) こと(和) の4語であり 2/4 になる。
        // 助詞「を」を分母に入れると 2/5 = 0.4 に下がる。
        let m = analyze("設定を変更すること", "設定を変更すること", &morph());
        assert!(
            (m.kango_ratio - 0.5).abs() < 1e-9,
            "kango={}",
            m.kango_ratio
        );
    }

    #[test]
    fn keishiki_meishi_is_counted_on_body_not_prose() {
        let m = analyze("", "ことこと", &morph());
        assert!(m.keishiki_meishi_per_1k > 0.0);
    }

    #[test]
    fn demonstratives_are_counted() {
        let m = analyze("", "これこれ", &morph());
        assert!(m.demonstrative_per_1k > 0.0);
    }
}
