//! 文書群から分野で確立した語を取り出す。
//!
//! たとえの層は、ここに載る語を「確立した用語」として扱い、たとえの判定から除く。
//! cache や handshake が分野の語であってたとえではないのと同じ理由で、
//! 手元の文書に繰り返し出る語も分野の語として扱ってよい。
//! 同じ数え上げが、語の一貫性を確かめる材料にもなる。
//!
//! 辞書を同梱しないのは `docs/content/ja/decisions.md` の D-11 に書いたとおりである。
//! プロジェクトごとに語が違ううえ、指摘されて有名になった語ほど早く廃れるため、
//! 固定した辞書は古びる。文書から作った語彙集は古びない。

use crate::lang::Lang;
use crate::markdown::{BlockKind, Document};
use crate::metrics::ja::KEISHIKI_MEISHI;
use crate::tokenizer::Morphology;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// 語ひとつと出現回数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Term {
    pub word: String,
    pub lang: Lang,
    pub count: usize,
}

/// `.suikou/terms.toml` の形。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Terms {
    pub terms: Vec<Term>,
}

impl Terms {
    /// 出現回数の多い順に並べる。同数なら語の文字順とする。
    /// 出力を決定的にしておくと、差分レビューで無関係な並び替えが混ざらない。
    pub fn sorted(mut self) -> Self {
        self.terms
            .sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.word.cmp(&b.word)));
        self
    }
}

/// 出現回数の下限の既定値。
///
/// 1回しか出ない語は、たまたまの言い回しと分野の語を見分ける材料がない。
/// terms のもう一つの目的である語の一貫性の検査も、同じ語が2回以上出て初めて成り立つ。
/// この値は実測ではなく、この二つの目的から導く下限である。
pub const DEFAULT_MIN_COUNT: usize = 2;

/// 頻度表から、下限を満たす語だけを `Term` に直す。
pub fn build_terms(counts: &BTreeMap<String, usize>, lang: Lang, min_count: usize) -> Vec<Term> {
    counts
        .iter()
        .filter(|(_, &c)| c >= min_count)
        .map(|(word, &count)| Term {
            word: word.clone(),
            lang,
            count,
        })
        .collect()
}

// ---------------------------------------------------------------------
// 日本語
// ---------------------------------------------------------------------

/// 名詞または名詞に準じる接尾辞が連続する範囲を1語の候補としてまとめて数える。
///
/// 「形態素解析」「文書指標」のような分野の語は、名詞が連続した複合語であることが多い。
/// 1トークンずつ数えると複合語が分断されるため、名詞の連なりをまとめて1語として扱う。
///
/// 「素」「性」「化」のような接尾辞は UniDic では pos1 が「接尾辞」になり、
/// 「名詞」ではない。この境界をそのまま使うと「形態素解析」が「形態」と「解析」に
/// 割れ、「保守性」が「保守」だけになる。`tokenizer.rs` の `Token::is_taigen` が
/// 体言として扱う品詞に「名詞」と「接尾辞」を含めているのと同じ理由で、
/// ここでも両方を複合語の構成要素として扱う。ただし `is_taigen` と異なり
/// 「代名詞」（これ・それ）は指示語であって分野の語ではないため対象から外す。
///
/// 接尾辞は語幹に束縛される形態素であり、それ単独では語にならない。
/// 「3つ」の「つ」のような助数詞も接尾辞として切り出されるが、UniDic の素性には
/// これを助数詞だけに絞り込める粒度の情報がなく（`tokenizer.rs` に保持しているのは
/// pos1 と pos2 までで、より細かい品詞細分類は保持していない）、表層形を数える
/// アプローチも助数詞の一覧を新たに作ることになり `KEISHIKI_MEISHI` と同じ問題を
/// 繰り返す。そこで品詞ではなく構造で絞る。接尾辞は、直前に名詞の連なりが
/// 既にある場合に限って連なりを伸ばす。連なりが空のところに接尾辞だけが
/// 現れた場合は対象にしない。これにより「3つ」の「つ」のように直前が数詞で
/// 断ち切られた接尾辞は候補から漏れ、「形態素」の「素」や「保守性」の「性」の
/// ように直前の名詞に束縛された接尾辞だけが複合語を構成する。
///
/// 動詞や助詞は連なりを断ち切る。用言を含む言い回し（「解析する」の「する」など）は
/// たとえの層が扱う対象語彙ではなく、ここでは対象にしない。
/// 数詞（「3つ」の「3」）も断ち切る。数量は分野の語にならない。
///
/// 候補が形式名詞1語だけからなる場合は捨てる。
/// 「こと」「もの」のような形式名詞は `crates/suikou-core/src/metrics/ja.rs` の
/// `KEISHIKI_MEISHI` で既に一般語と定めてあり、ここで新たに一般語の一覧を
/// 作り直すことはしない。形式名詞以外の一般語（「方法」など）をどう除くかは
/// `docs/content/ja/decisions.md` の D-24 に理由を書いた。
pub fn count_ja(body: &str, morph: &dyn Morphology, counts: &mut BTreeMap<String, usize>) {
    let toks = morph.tokenize(body);
    let mut run: Vec<&str> = Vec::new();
    for t in &toks {
        let qualifies = match t.pos1.as_str() {
            "名詞" => t.pos2 != "数詞",
            "接尾辞" => !run.is_empty(),
            _ => false,
        };
        if qualifies {
            run.push(t.surface.as_str());
        } else {
            flush_run_ja(&run, counts);
            run.clear();
        }
    }
    flush_run_ja(&run, counts);
}

fn flush_run_ja(run: &[&str], counts: &mut BTreeMap<String, usize>) {
    if run.is_empty() {
        return;
    }
    if run.len() == 1 && KEISHIKI_MEISHI.contains(&run[0]) {
        return;
    }
    let word: String = run.concat();
    *counts.entry(word).or_insert(0) += 1;
}

// ---------------------------------------------------------------------
// 英語
// ---------------------------------------------------------------------
//
// 英語には形態素解析器がない（`TASKS.md` の T3 に書いたとおり、文の切り分けも
// まだ持たない）。そのため品詞に頼れず、次の3つの手掛かりで候補を拾う。
//
// 1. 大文字と数字とハイフンだけからなる語（JSON、CLI、D-04、M1）。
//    頭字語や ID は位置によらず分野の語である。
// 2. 文の先頭ではない位置で大文字始まりの語（GitHub、Rust、UniDic）。
//    固有名詞や製品名は、文中でも大文字始まりを保つ。
//    文の先頭かどうかは、直前の語の直後に `.` `!` `?` があるか、
//    その語がブロックの最初の語であるかで見る。行が段落の折り返しの途中から
//    始まる場合はこの判定が効かないため、英語の文区切りが未実装であるという
//    制約を引き継いでいる。誤って文頭語を候補に含めても、出現回数の下限と
//    人の目視で弾かれる前提を置く。
//    「D-04 The tokenizer hides behind a trait」のように ID が文の主語の前に
//    立つ見出しもある。ブロックの先頭から ID 風の語が続く間は、まだ文頭の
//    延長とみなして次の語の文頭判定を続ける（`count_en_words` の実装を参照）。
// 3. インラインコードの中身が識別子1個の形をしている場合（`Document::parse`、
//    `lindera-unidic`）。空白を含むコード片はコマンド全体である場合が多く、
//    どの部分語が語かを決める根拠がないため、単語に割らずスパンごと捨てるか
//    採るかの二択にする。

/// 英字で始まる語1個を拾う。頭字語と大文字始まりの語の両方をこれで拾い分ける。
fn re_word() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[A-Za-z][A-Za-z0-9''-]*").unwrap())
}

fn re_code_span() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"`([^`\n]+)`").unwrap())
}

fn re_bare_ident() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^[A-Za-z][A-Za-z0-9_:.\-]*$").unwrap())
}

/// 頭字語や `D-04`、`M1` のような ID 風の語。ハイフンと数字を含めてよい。
/// `AWS-S3` のような固有名の綴りにもこの形が出る。
fn is_all_caps_word(w: &str) -> bool {
    w.len() >= 2
        && w.chars().any(|c| c.is_ascii_alphabetic())
        && w.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

fn is_capitalized_word(w: &str) -> bool {
    let mut chars = w.chars();
    match chars.next() {
        Some(c0) => c0.is_ascii_uppercase() && w.chars().count() >= 2,
        None => false,
    }
}

/// ブロックの地の文と箇条書きから、大文字始まりの語の連なりを数える。
/// 見出しも同じ規則で扱う。このリポジトリの英語の見出しは文頭だけを
/// 大文字にする書き方であり（Title Case ではない）、見出しを特別扱いする
/// 根拠がないためである。
///
/// この文書自身の decisions.md には「D-04 The tokenizer hides behind a trait」の
/// ような見出しがある。文の最初の語は「D-04」という ID であって、文としての
/// 主語は次の「The」である。先頭の語だけを文頭とみなすと「The」を普通の語と
/// 誤認する。そこで、ブロックの先頭から ID 風の語（`is_all_caps_word` が
/// 真になる語）が連続する間は、まだ文頭の続きとみなして判定を続ける。
/// 小文字を含む語に出会った時点でこの「先頭の続き」を終える。
fn count_en_words(text: &str, counts: &mut BTreeMap<String, usize>) {
    let mut run: Vec<&str> = Vec::new();
    let mut prev_end = 0usize;
    // ブロック先頭から ID 風の語が続いている間は真のまま。
    let mut leading = true;
    for m in re_word().find_iter(text) {
        let word = m.as_str();
        let between = text[prev_end..m.start()].trim();
        let sentence_initial = leading || between.ends_with(['.', '!', '?']);
        let punct_break = !between.is_empty();
        prev_end = m.end();

        if punct_break {
            flush_run_en(&run, counts);
            run.clear();
        }

        let qualifies = is_all_caps_word(word) || (is_capitalized_word(word) && !sentence_initial);
        if qualifies {
            run.push(word);
        } else {
            flush_run_en(&run, counts);
            run.clear();
        }

        leading = leading && !punct_break && is_all_caps_word(word);
    }
    flush_run_en(&run, counts);
}

fn flush_run_en(run: &[&str], counts: &mut BTreeMap<String, usize>) {
    if run.is_empty() {
        return;
    }
    let word = run.join(" ");
    *counts.entry(word).or_insert(0) += 1;
}

/// インラインコードのうち、空白を含まず識別子1個の形をしているものを数える。
fn count_en_code_spans(raw: &str, counts: &mut BTreeMap<String, usize>) {
    for c in re_code_span().captures_iter(raw) {
        let inner = c.get(1).map_or("", |m| m.as_str()).trim();
        if re_bare_ident().is_match(inner) {
            *counts.entry(inner.to_string()).or_insert(0) += 1;
        }
    }
}

/// 文書1つ分の英語の候補を数える。
/// コード片は前処理後の生の行（`Block::raw`）から、語は記法を除いた
/// 本文（`Block::text`）から拾う。`Block::text` は既にインラインコードを
/// 取り除いてあるため、二重に数えることはない。
///
/// 大文字始まりの語の連なりは、地の文と箇条書きと見出しだけを対象にする。
/// 表の行は対象にしない。表の見出し列や値の列（High、Low のような区分）は、
/// 文でないため文頭かどうかの手掛かりが働かず、かつ同じ値が行をまたいで
/// 繰り返し出るため出現回数の下限でも弾けない。実際にこのリポジトリの
/// 英語文書で試して、表の列の値が語彙集に混じることを確かめた上で外した。
/// コード片の抽出は対象を絞らない。識別子は表の中でも文の中でも同じ形で
/// 現れ、表特有のノイズを持ち込まないためである。
pub fn count_en(doc: &Document, counts: &mut BTreeMap<String, usize>) {
    for block in &doc.blocks {
        count_en_code_spans(&block.raw, counts);
        if matches!(
            block.kind,
            BlockKind::Prose | BlockKind::ListItem | BlockKind::Heading
        ) {
            count_en_words(&block.text, counts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::FakeMorphology;

    fn fake() -> FakeMorphology {
        // UniDic は「素」「性」のような接尾辞を pos1 = 接尾辞 として切り出す。
        // 「形態素」を「形態」+「素」に割って、複合語の構成要素として扱われることを確かめる。
        FakeMorphology::from_spec(
            "形態|名詞|普通名詞||漢 素|接尾辞|名詞的|||和 解析|名詞|普通名詞||漢 \
             性|接尾辞|名詞的|||和 器|接尾辞|名詞的|||和 \
             を|助詞|格助詞||和 使う|動詞|一般|終止形-一般|和 。|補助記号|句点||和 \
             文書|名詞|普通名詞||漢 指標|名詞|普通名詞||漢 は|助詞|係助詞||和 \
             こと|名詞|普通名詞||和 が|助詞|格助詞||和 大事|形状詞|一般|||和 \
             だ|助動詞||終止形-一般|和 3|名詞|数詞|||和 つ|接尾辞|名詞的|助数詞||和 \
             ある|動詞|非自立可能|終止形-一般|和 保守|名詞|普通名詞||漢",
        )
    }

    #[test]
    fn merges_consecutive_nouns_into_one_compound() {
        let mut counts = BTreeMap::new();
        count_ja("形態素解析を使う。形態素解析を使う。", &fake(), &mut counts);
        assert_eq!(counts.get("形態素解析"), Some(&2));
    }

    #[test]
    fn a_suffix_extends_the_compound_rather_than_breaking_it() {
        // 「形態」(名詞) + 「素」(接尾辞) + 「解析」(名詞) がひとつながりの語になる。
        let mut counts = BTreeMap::new();
        count_ja("形態素解析を使う。形態素解析を使う。", &fake(), &mut counts);
        assert!(!counts.contains_key("形態"), "{counts:?}");
        assert!(!counts.contains_key("解析"), "{counts:?}");
    }

    #[test]
    fn a_derivational_suffix_forms_a_new_compound() {
        // 「保守」(名詞) + 「性」(接尾辞) が「保守性」としてまとまる。
        let mut counts = BTreeMap::new();
        count_ja("保守性を保つ。保守性を保つ。", &fake(), &mut counts);
        assert_eq!(counts.get("保守性"), Some(&2));
    }

    #[test]
    fn a_bare_formal_noun_is_not_a_term() {
        let mut counts = BTreeMap::new();
        count_ja("ことが大事だ。", &fake(), &mut counts);
        assert!(counts.is_empty(), "{counts:?}");
    }

    #[test]
    fn numerals_break_the_run() {
        let mut counts = BTreeMap::new();
        count_ja("3つある。", &fake(), &mut counts);
        assert!(!counts.contains_key("3つ"));
    }

    #[test]
    fn a_counter_suffix_cut_off_by_a_numeral_is_not_a_lone_term() {
        // 実際の UniDic は助数詞「つ」も pos2 = 名詞的 で切り出し、助数詞専用の
        // 値を持たない。数詞で連なりが断たれたあとの接尾辞が単独で語にならない
        // ことを確かめる。ここで防いでいるのは、実際にこの語がノイズとして
        // 出た D-24 の事例そのものである。
        let mut counts = BTreeMap::new();
        count_ja("3つ指標がある。3つ指標がある。", &fake(), &mut counts);
        assert!(!counts.contains_key("つ"), "{counts:?}");
    }

    #[test]
    fn verbs_do_not_join_the_compound() {
        let mut counts = BTreeMap::new();
        count_ja("文書指標は大事だ。", &fake(), &mut counts);
        assert_eq!(counts.get("文書指標"), Some(&1));
        assert!(!counts.contains_key("文書指標は"));
    }

    #[test]
    fn min_count_filters_single_occurrences() {
        let mut counts = BTreeMap::new();
        count_ja("形態素解析を使う。", &fake(), &mut counts);
        let terms = build_terms(&counts, Lang::Ja, DEFAULT_MIN_COUNT);
        assert!(terms.is_empty(), "{terms:?}");
    }

    #[test]
    fn acronyms_are_counted_regardless_of_position() {
        let mut counts = BTreeMap::new();
        count_en_words("JSON is a format. Every API returns JSON.", &mut counts);
        assert_eq!(counts.get("JSON"), Some(&2));
        assert_eq!(counts.get("API"), Some(&1));
    }

    #[test]
    fn sentence_initial_capitals_are_skipped() {
        let mut counts = BTreeMap::new();
        count_en_words("Rust is fast. The Rust compiler checks types.", &mut counts);
        // 文頭の Rust と The は数えない。文中の Rust だけを数える。
        assert_eq!(counts.get("Rust"), Some(&1));
        assert!(!counts.contains_key("The"));
    }

    #[test]
    fn a_leading_id_does_not_make_the_next_word_look_mid_sentence() {
        // このリポジトリ自身の decisions.md にある見出しの形。
        // 「D-04」は ID であって文の主語ではない。文の主語は次の「The」であり、
        // これも文頭として扱う必要がある。
        let mut counts = BTreeMap::new();
        count_en_words("D-04 The tokenizer hides behind a trait", &mut counts);
        assert!(!counts.contains_key("The"), "{counts:?}");
        assert_eq!(counts.get("D-04"), Some(&1));
    }

    #[test]
    fn adjacent_capitals_merge_into_one_term() {
        let mut counts = BTreeMap::new();
        count_en_words("It runs on GitHub Actions every night.", &mut counts);
        assert_eq!(counts.get("GitHub Actions"), Some(&1));
        assert!(!counts.contains_key("GitHub"));
    }

    #[test]
    fn a_comma_breaks_the_compound_run() {
        let mut counts = BTreeMap::new();
        count_en_words("It uses Rust, Markdown, and TOML.", &mut counts);
        assert_eq!(counts.get("Rust"), Some(&1));
        assert_eq!(counts.get("Markdown"), Some(&1));
        assert!(!counts.contains_key("Rust Markdown"));
    }

    #[test]
    fn bare_identifier_code_spans_are_counted() {
        let mut counts = BTreeMap::new();
        count_en_code_spans("Call `Document::parse` twice.", &mut counts);
        assert_eq!(counts.get("Document::parse"), Some(&1));
    }

    #[test]
    fn multi_word_code_spans_are_skipped() {
        let mut counts = BTreeMap::new();
        count_en_code_spans("Run `cargo test --all` first.", &mut counts);
        assert!(counts.is_empty(), "{counts:?}");
    }

    #[test]
    fn count_en_reads_raw_for_code_and_text_for_words() {
        let doc = Document::parse("Use `lindera` to tokenize. Use `lindera` again.\n");
        let mut counts = BTreeMap::new();
        count_en(&doc, &mut counts);
        assert_eq!(counts.get("lindera"), Some(&2));
    }

    #[test]
    fn table_rows_do_not_feed_the_capitalized_word_heuristic() {
        // 表の列の値は文ではないため文頭判定が働かず、行をまたいで同じ値が
        // 繰り返されて出現回数の下限もすり抜ける。表の行は対象から外してある。
        let src = "| Register | Severity |\n\
                    |---|---|\n\
                    | Formal | High |\n\
                    | Formal | High |\n";
        let doc = Document::parse(src);
        let mut counts = BTreeMap::new();
        count_en(&doc, &mut counts);
        assert!(counts.is_empty(), "{counts:?}");
    }

    #[test]
    fn prose_still_feeds_the_capitalized_word_heuristic() {
        let doc = Document::parse("It runs GitHub Actions. It runs GitHub Actions.\n");
        let mut counts = BTreeMap::new();
        count_en(&doc, &mut counts);
        assert_eq!(counts.get("GitHub Actions"), Some(&2));
    }

    #[test]
    fn terms_sort_by_count_then_word() {
        let terms = Terms {
            terms: vec![
                Term {
                    word: "b".into(),
                    lang: Lang::En,
                    count: 3,
                },
                Term {
                    word: "a".into(),
                    lang: Lang::En,
                    count: 5,
                },
                Term {
                    word: "c".into(),
                    lang: Lang::En,
                    count: 5,
                },
            ],
        }
        .sorted();
        let words: Vec<&str> = terms.terms.iter().map(|t| t.word.as_str()).collect();
        assert_eq!(words, vec!["a", "c", "b"]);
    }
}
