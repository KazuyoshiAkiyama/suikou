//! M1 から M7。メンテナンス性の局所規則。
//!
//! この層は AI 臭さではなくメンテナンス性を根拠に持つ。AI 以前から存在する問題であり、
//! モデルの世代交代でも陳腐化しない。出典は Google developer documentation style guide。
//!
//! 人間コーパスの平均に合わせてはならない。箇条書きの導入文を正しく書いているのは
//! AWS S3 ユーザーガイドだけで、Linux kernel は不完全な導入が優勢だった。
//! 最良実践に合わせる。

use crate::lang::Lang;
use crate::markdown::{Block, BlockKind, Document};
use crate::report::{LocalFinding, Position, Severity};
use crate::tokenizer::Morphology;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

pub const RULE_LIST_LEAD_IN: &str = "maint/list-lead-in";
pub const RULE_ITEM_COUNT: &str = "maint/item-count";
pub const RULE_NUMBERED_HEADING: &str = "maint/numbered-heading";
pub const RULE_MANUAL_NUMBER: &str = "maint/manual-number";
pub const RULE_PARALLEL_ITEMS: &str = "maint/parallel-items";
pub const RULE_TIME_DEPENDENT: &str = "maint/time-dependent";
pub const RULE_TRAILING_ETC: &str = "maint/trailing-etc";

/// M6。厳格版に絞る。new / now / future / existing を含めると偽陽性が多い。
pub const TIME_DEPENDENT_JA: &[&str] = &[
    "現在",
    "現時点",
    "最新の",
    "新しい",
    "今後",
    "将来的に",
    "まもなく",
    "既存の",
    "目下",
    "現行の",
    // 書いた時点を起点にする語。Google の timeless documentation が
    // recently と soon と new を挙げており、その日本語側にあたる。
    "最近",
    "近年",
    "直近",
    "先日",
    "従来",
    "当面",
    "順次",
];

/// M6 の英語版。語の一覧は `TIME_DEPENDENT_JA` と対になる。
/// `brief` サブコマンドが禁止語を列挙するとき、この一覧を正本として使う。
pub const TIME_DEPENDENT_EN: &[&str] = &[
    "currently",
    "presently",
    "eventually",
    "soon",
    "latest",
    "newest",
    "newer",
    "as of this writing",
    "at present",
    "in the future",
    "in the near future",
    "for now",
    // 書いた時点を起点にする語。日本語側の追加と対になる。
    "recently",
    "lately",
    "these days",
    "nowadays",
    "so far",
    "for the time being",
    "up to now",
];

/// M1。英語の導入文がこれらを含めば完全な導入とみなす。
/// 厳密には定形動詞の有無を見るべきだが、この判定だけで
/// AWS S3 ユーザーガイドと Linux kernel を分離できている。
pub const EN_LEAD_IN_MARKERS: &[&str] = &[
    "the following",
    "as follows",
    "these",
    "below",
    "steps",
    "following",
];

macro_rules! re {
    ($name:ident, $pat:expr) => {
        fn $name() -> &'static Regex {
            static R: OnceLock<Regex> = OnceLock::new();
            R.get_or_init(|| Regex::new($pat).unwrap())
        }
    };
}

re!(
    re_m2_ja,
    r"[0-9０-９一二三四五六七八九十]+\s*[つ個点種類]の\s*(?:理由|方法|ポイント|点|要素|ステップ|手順|方針|特徴|観点|側面|利点|欠点|課題|原則|要因|種類)|(?:以下|次)の\s*[0-9０-９一二三四五六七八九十]+\s*[つ個点]|[0-9０-９一二三四五六七八九十]+\s*[つ個]あ(?:る|ります)"
);
re!(
    re_m2_en,
    r"(?i)\b(two|three|four|five|six|seven|2|3|4|5|6|7)\s+(reasons|ways|steps|things|types|categories|benefits|points|factors|approaches|options|principles|rules|components|parts)\b"
);
re!(
    re_m3,
    r"^#{1,6}\s*(?:第?\s*[0-9０-９]+\s*[.．、)）章節]|[0-9０-９]+\s*[-–]\s)"
);
re!(re_m4, r"^\s*[-*+]\s+[0-9０-９]+\s*[.)．）、]\s");
re!(re_m7_ja, r"(?:など|等)[。、）\)]?\s*$");

/// 絶対の係留。これがある文では M6 を見送る。
///
/// 時点に触れること自体は誤りではない。読む時点で意味が変わることが誤りである。
/// 西暦、バージョン番号、`YYYY-MM` の形は、読む時点によらず同じ時点を指す。
/// 「lindera 6.0.0 で確認した」は過去形だが古びない。
///
/// この見送りが、「意図して時点に触れる場合は除く」という約束の運用形である。
/// 判断に委ねると検査できないため、係留の有無という形に落とす。
fn re_anchor() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?x)
            (19|20)\d{2}\s*(年|-|/)     # 西暦
          | v\d+\.\d+                 # v の付いたバージョン
          | \d+\.\d+\.\d+               # 3つ組のバージョン
          | (RFC|KEP|ADR|D-)\s*\d+       # 番号の付いた文書
        ",
        )
        .unwrap()
    })
}

/// 係留のない相対の期間。語の一覧では拾えない形をここで見る。
///
/// 「直近3ヶ月」「過去2年」「先週」は、読む時点によって指す範囲が変わる。
/// 数を伴うため、語の一覧に並べても網羅できない。
fn re_relative_period() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(直近|過去|ここ|この)\s*[0-9０-９一二三四五六七八九十]+\s*(年|ヶ月|か月|箇月|週間|日間)|(先|今|来)(週|月|年度|年)")
            .unwrap()
    })
}

/// 英語の相対の期間。日本語側と対になる。
fn re_relative_period_en() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)\b((in|over|for|during)\s+the\s+(past|last)\s+\w+\s+(years?|months?|weeks?|days?)|(last|next)\s+(week|month|quarter|year))\b")
            .unwrap()
    })
}

/// M6 の日本語。語の一覧は `TIME_DEPENDENT_JA` を正本とし、そこから組み立てる。
/// 出現ごとに数えるため、`contains` ではなく正規表現にしてある。
fn re_m6_ja() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(&TIME_DEPENDENT_JA.join("|")).unwrap())
}
/// M6 の英語。語の一覧は `TIME_DEPENDENT_EN` を正本とし、そこから組み立てる。
fn re_m6_en() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(&format!(r"(?i)\b({})\b", TIME_DEPENDENT_EN.join("|"))).unwrap())
}
re!(re_m7_en, r"(?i)(etc\.|and so on|and more)\s*[.)]?\s*$");

/// M1 の導入文の分類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeadIn {
    /// 完全な文、または体言＋コロン。
    Ok,
    /// 項目と文法的に結合している。
    Ng,
    /// 判定しない。
    Other,
}

pub fn classify_lead_in_ja(line: &str, morph: &dyn Morphology) -> LeadIn {
    let s = line.trim();
    if s.is_empty() {
        return LeadIn::Other;
    }
    let last = s.chars().last().unwrap();
    if matches!(last, '。' | '！' | '？') {
        return LeadIn::Ok;
    }
    if matches!(last, ':' | '：') {
        let core: String = s.chars().take(s.chars().count() - 1).collect();
        let toks = morph.tokenize(core.trim());
        return match toks.last() {
            Some(t) if t.is_taigen() => LeadIn::Ok,
            Some(_) => LeadIn::Ng,
            None => LeadIn::Other,
        };
    }
    let toks = morph.tokenize(s);
    match toks.last() {
        Some(t) if t.pos1 == "助詞" => LeadIn::Ng,
        Some(t) if (t.pos1 == "動詞" || t.pos1 == "助動詞") && t.cform.starts_with("連用形") => {
            LeadIn::Ng
        }
        _ => LeadIn::Other,
    }
}

pub fn classify_lead_in_en(line: &str) -> LeadIn {
    let s = line.trim();
    if s.is_empty() {
        return LeadIn::Other;
    }
    if s.ends_with(':') {
        let lower = s.to_lowercase();
        if EN_LEAD_IN_MARKERS.iter().any(|m| lower.contains(m)) {
            return LeadIn::Ok;
        }
        return LeadIn::Ng;
    }
    if s.ends_with('.') || s.ends_with('!') || s.ends_with('?') {
        return LeadIn::Ok;
    }
    LeadIn::Other
}

/// M5。項目末尾の形の分類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemEnding {
    Kuten,
    Taigen,
    Yougen,
    Period,
    None,
    Other,
}

pub fn item_ending_ja(text: &str, morph: &dyn Morphology) -> ItemEnding {
    let t = text.trim();
    if t.is_empty() {
        return ItemEnding::Other;
    }
    if matches!(t.chars().last().unwrap(), '。' | '！' | '？') {
        return ItemEnding::Kuten;
    }
    match morph.tokenize(t).last() {
        Some(x) if x.is_taigen() => ItemEnding::Taigen,
        Some(_) => ItemEnding::Yougen,
        None => ItemEnding::Other,
    }
}

pub fn item_ending_en(text: &str) -> ItemEnding {
    let t = text.trim();
    if t.ends_with('.') || t.ends_with('!') || t.ends_with('?') {
        ItemEnding::Period
    } else {
        ItemEnding::None
    }
}

fn finding(rule: &str, sev: Severity, msg: &str, pos: Vec<Position>) -> Option<LocalFinding> {
    if pos.is_empty() {
        return None;
    }
    let total = pos.len();
    let kept: Vec<Position> = pos
        .into_iter()
        .take(crate::report::Report::MAX_POSITIONS_PER_RULE)
        .collect();
    let truncated = total - kept.len();
    Some(LocalFinding {
        rule_id: rule.to_string(),
        severity: sev,
        message: msg.to_string(),
        positions: kept,
        truncated,
    })
}

/// 一致した箇所ごとに Position を作る。1行に複数あっても取りこぼさない。
///
/// 取りこぼすと、指摘された一方だけを直した文書が `check --quiet` で再び落ちる。
/// 層ごとに lint と修正を往復させないという設計が、そこで崩れる。
/// 係留の無い文の中の一致だけを位置にする。
///
/// 係留は文ごとに見る。同じ段落の別の文が年号を持っていても、
/// この文が時点に依存することは変わらない。
fn pos_of_unanchored_matches(b: &Block, re: &Regex, lang: Lang) -> Vec<Position> {
    let ends = sentence_ends(&b.text, lang);
    re.find_iter(&b.text)
        .filter(|m| {
            let (from, to) = enclosing_sentence(&ends, m.start(), b.text.len());
            !re_anchor().is_match(&b.text[from..to])
        })
        .map(|m| Position {
            line: b.line,
            column: b.text[..m.start()].chars().count() + 1,
            text: b.text.chars().take(60).collect(),
        })
        .collect()
}

/// 文の終わりのバイト位置。区切りの記号を含めた位置を返す。
fn sentence_ends(text: &str, lang: Lang) -> Vec<usize> {
    text.char_indices()
        .filter(|(_, c)| match lang {
            Lang::Ja => matches!(c, '。' | '！' | '？'),
            Lang::En => matches!(c, '.' | '!' | '?'),
        })
        .map(|(i, c)| i + c.len_utf8())
        .collect()
}

fn enclosing_sentence(ends: &[usize], at: usize, len: usize) -> (usize, usize) {
    let from = ends.iter().rev().find(|e| **e <= at).copied().unwrap_or(0);
    let to = ends.iter().find(|e| **e > at).copied().unwrap_or(len);
    (from, to)
}

fn pos_of_matches(b: &Block, re: &Regex) -> Vec<Position> {
    re.find_iter(&b.text)
        .map(|m| Position {
            line: b.line,
            column: b.text[..m.start()].chars().count() + 1,
            text: b.text.chars().take(60).collect(),
        })
        .collect()
}

fn pos_of(b: &Block) -> Position {
    Position {
        line: b.line,
        column: 1,
        text: b.text.chars().take(60).collect(),
    }
}

pub fn check_all(doc: &Document, lang: Lang, morph: &dyn Morphology) -> Vec<LocalFinding> {
    let mut out = Vec::new();
    let mut m1 = Vec::new();
    let mut m5 = Vec::new();
    let mut m4 = Vec::new();
    let mut m7 = Vec::new();

    for group in doc.list_blocks() {
        let first = group[0];
        if let Some(before) = doc.block_before(first.line) {
            if before.kind == BlockKind::Prose {
                let kind = match lang {
                    Lang::Ja => classify_lead_in_ja(&before.text, morph),
                    Lang::En => classify_lead_in_en(&before.text),
                };
                if kind == LeadIn::Ng {
                    m1.push(pos_of(before));
                }
            }
        }
        if group.len() >= 2 {
            let kinds: HashSet<ItemEnding> = group
                .iter()
                .map(|b| match lang {
                    Lang::Ja => item_ending_ja(&b.text, morph),
                    Lang::En => item_ending_en(&b.text),
                })
                .collect();
            if kinds.len() > 1 {
                m5.push(pos_of(first));
            }
        }
        for b in &group {
            if re_m4().is_match(&b.raw) {
                m4.push(pos_of(b));
            }
            let m7hit = match lang {
                Lang::Ja => re_m7_ja().is_match(b.text.trim()),
                Lang::En => re_m7_en().is_match(b.text.trim()),
            };
            if m7hit {
                m7.push(pos_of(b));
            }
        }
    }

    let mut m2 = Vec::new();
    let mut m3 = Vec::new();
    let mut m6 = Vec::new();
    for b in &doc.blocks {
        match b.kind {
            BlockKind::Heading => {
                if re_m3().is_match(b.raw.trim()) {
                    m3.push(pos_of(b));
                }
            }
            BlockKind::Prose => {
                m2.extend(pos_of_matches(
                    b,
                    match lang {
                        Lang::Ja => re_m2_ja(),
                        Lang::En => re_m2_en(),
                    },
                ));
                if match lang {
                    Lang::Ja => re_m7_ja().is_match(b.text.trim()),
                    Lang::En => re_m7_en().is_match(b.text.trim()),
                } {
                    m7.push(pos_of(b));
                }
            }
            _ => {}
        }
        // 絶対の係留がある文は見送る。時点に触れること自体は誤りではない。
        let (words, period) = match lang {
            Lang::Ja => (re_m6_ja(), re_relative_period()),
            Lang::En => (re_m6_en(), re_relative_period_en()),
        };
        m6.extend(pos_of_unanchored_matches(b, words, lang));
        m6.extend(pos_of_unanchored_matches(b, period, lang));
        // 「直近3ヶ月」は語の一覧と期間の型の両方に当たる。
        // 同じ位置を二度示すと、直す側は同じ箇所を二度読むことになる。
        m6.sort_by_key(|p| (p.line, p.column));
        m6.dedup_by_key(|p| (p.line, p.column));
    }

    out.extend(finding(
        RULE_LIST_LEAD_IN,
        Severity::Error,
        "箇条書きの導入文が項目と文法的に結合している。項目を増減すると壊れる",
        m1,
    ));
    out.extend(finding(
        RULE_ITEM_COUNT,
        Severity::Error,
        "地の文に項目数を書いている。項目を増減すると本文が合わなくなる",
        m2,
    ));
    out.extend(finding(
        RULE_NUMBERED_HEADING,
        Severity::Error,
        "見出しに節の連番がある。節を増減すると振り直しが要る",
        m3,
    ));
    out.extend(finding(
        RULE_MANUAL_NUMBER,
        Severity::Error,
        "箇条書きの項目に手動で番号を書いている",
        m4,
    ));
    out.extend(finding(
        RULE_PARALLEL_ITEMS,
        Severity::Error,
        "同じ箇条書きの中で項目の末尾の形が揃っていない",
        m5,
    ));
    out.extend(finding(
        RULE_TIME_DEPENDENT,
        Severity::Error,
        "時点に依存する語を使っている。文書が古びる",
        m6,
    ));
    out.extend(finding(
        RULE_TRAILING_ETC,
        Severity::Error,
        "列挙が「など」で終わっている。網羅的でないことは導入文で示す",
        m7,
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::FakeMorphology;

    fn morph() -> FakeMorphology {
        FakeMorphology::from_spec(
            "以下|名詞|普通名詞||漢 の|助詞|格助詞||和 目的|名詞|普通名詞||漢 \
             で|助詞|格助詞||和 使い|動詞|一般|連用形-一般|和 ます|助動詞||終止形-一般|和 \
             手順|名詞|普通名詞||漢 は|助詞|係助詞||和 次|名詞|普通名詞||漢 \
             とおり|名詞|普通名詞||和 検証|名詞|普通名詞||漢 する|動詞|非自立可能|終止形-一般|和",
        )
    }

    #[test]
    fn lead_in_complete_sentence_is_ok() {
        assert_eq!(
            classify_lead_in_ja("手順は次のとおり。", &morph()),
            LeadIn::Ok
        );
    }

    #[test]
    fn lead_in_taigen_colon_is_ok() {
        assert_eq!(classify_lead_in_ja("手順:", &morph()), LeadIn::Ok);
    }

    #[test]
    fn lead_in_particle_ending_is_ng() {
        assert_eq!(classify_lead_in_ja("以下の目的で", &morph()), LeadIn::Ng);
    }

    #[test]
    fn en_lead_in_bare_colon_is_ng() {
        assert_eq!(
            classify_lead_in_en("Use the reload command to:"),
            LeadIn::Ng
        );
    }

    #[test]
    fn en_lead_in_with_marker_is_ok() {
        assert_eq!(
            classify_lead_in_en("Apply the change by doing the following:"),
            LeadIn::Ok
        );
    }

    #[test]
    fn detects_item_count_in_prose() {
        let d = Document::parse("よくある失敗は3つある。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_ITEM_COUNT));
    }

    #[test]
    fn detects_numbered_heading() {
        let d = Document::parse("## 2. 検証の方法\n\n本文である。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_NUMBERED_HEADING));
    }

    #[test]
    fn detects_manual_number_in_bullet() {
        let d = Document::parse("導入である。\n\n- 1. 最初\n- 2. 次\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_MANUAL_NUMBER));
    }

    #[test]
    fn detects_mixed_item_endings() {
        let d = Document::parse("導入である。\n\n- 構文を検証する。\n- サービスを再読み込み\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_PARALLEL_ITEMS));
    }

    #[test]
    fn detects_time_dependent_words() {
        let d = Document::parse("現在の実装では反映されない。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_TIME_DEPENDENT));
    }

    // 語の一覧では拾えない相対の期間。数を伴うため列挙できない。
    #[test]
    fn detects_a_relative_period() {
        for src in [
            "直近3ヶ月でリクエスト数が倍になった。\n",
            "過去2年の運用で問題は出ていない。\n",
            "先週の計測では収まっていた。\n",
        ] {
            let d = Document::parse(src);
            let f = check_all(&d, Lang::Ja, &morph());
            assert!(
                f.iter().any(|x| x.rule_id == RULE_TIME_DEPENDENT),
                "拾えていない: {src}"
            );
        }
    }

    // 絶対の係留がある文は見送る。時点に触れること自体は誤りではない。
    #[test]
    fn an_absolute_anchor_suppresses_the_finding() {
        for src in [
            "lindera 6.0.0 で現在の素性の順序を確認した。\n",
            "2026年3月の計測では、直近3ヶ月で倍になっていた。\n",
            "RFC 7322 は現在も必須の節を定めている。\n",
        ] {
            let d = Document::parse(src);
            let f = check_all(&d, Lang::Ja, &morph());
            assert!(
                !f.iter().any(|x| x.rule_id == RULE_TIME_DEPENDENT),
                "係留があるのに発火した: {src}"
            );
        }
    }

    // 係留は文ごとに見る。同じ段落の別の文の係留を借りてはならない。
    #[test]
    fn an_anchor_does_not_carry_to_another_sentence() {
        let d = Document::parse("2026年3月に計測した。現在の実装では反映されない。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        let t = f.iter().find(|x| x.rule_id == RULE_TIME_DEPENDENT).unwrap();
        assert_eq!(t.positions.len(), 1, "{:?}", t.positions);
    }

    #[test]
    fn an_anchor_does_not_carry_to_another_block() {
        let d = Document::parse("2026年3月に計測した。\n\n現在の実装では反映されない。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_TIME_DEPENDENT));
    }

    // 語の一覧と期間の型が同じ箇所に当たっても、示す位置はひとつとする。
    #[test]
    fn an_overlapping_match_is_reported_once() {
        let d = Document::parse("直近3ヶ月で負荷が上がった。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        let t = f.iter().find(|x| x.rule_id == RULE_TIME_DEPENDENT).unwrap();
        assert_eq!(t.positions.len(), 1, "{:?}", t.positions);
    }

    // 小数をバージョン番号と取り違えてはならない。「2.5倍」は係留ではない。
    #[test]
    fn a_decimal_is_not_an_anchor() {
        let d = Document::parse("直近3ヶ月で約2.5倍に増加した。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_TIME_DEPENDENT));
    }

    #[test]
    fn detects_a_relative_period_en() {
        let d = Document::parse("Requests doubled over the past three months.\n");
        let f = check_all(&d, Lang::En, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_TIME_DEPENDENT));
    }

    #[test]
    fn an_anchor_suppresses_the_finding_en() {
        let d = Document::parse("Measured on 2026-03-01, requests doubled recently.\n");
        let f = check_all(&d, Lang::En, &morph());
        assert!(!f.iter().any(|x| x.rule_id == RULE_TIME_DEPENDENT));
    }

    #[test]
    fn detects_trailing_etc_en() {
        let d = Document::parse("Errors include braces, unknown keys, etc.\n");
        let f = check_all(&d, Lang::En, &morph());
        assert!(f.iter().any(|x| x.rule_id == RULE_TRAILING_ETC));
    }

    #[test]
    fn clean_document_produces_no_findings() {
        let d = Document::parse("手順は次のとおり。\n\n- 構文を検証する。\n- ログを確認する。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn counts_every_occurrence_on_one_line() {
        // 1行に2語あるとき、2件として位置を持つ。列が異なる。
        let d = Document::parse("現在の実装は、今後の版で変わる。\n");
        let f = check_all(&d, Lang::Ja, &morph());
        let t = f.iter().find(|x| x.rule_id == RULE_TIME_DEPENDENT).unwrap();
        assert_eq!(t.positions.len(), 2, "{:?}", t.positions);
        assert_ne!(t.positions[0].column, t.positions[1].column);
    }

    #[test]
    fn truncates_positions_at_the_cap() {
        let mut src = String::new();
        for _ in 0..30 {
            src.push_str("現在の実装では反映されない。\n\n");
        }
        let d = Document::parse(&src);
        let f = check_all(&d, Lang::Ja, &morph());
        let t = f.iter().find(|x| x.rule_id == RULE_TIME_DEPENDENT).unwrap();
        assert_eq!(
            t.positions.len(),
            crate::report::Report::MAX_POSITIONS_PER_RULE
        );
        assert_eq!(t.truncated, 10);
    }
}
