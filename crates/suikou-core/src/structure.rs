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
pub const RULE_LEAD_OPENER: &str = "structure/lead-opener";
pub const RULE_WORK_NOTE: &str = "structure/work-note";

/// 段落の文の数の上限。これを超えたら分割の目安とする。
/// Google のスタイルガイドが「5〜6文を超えたら多くを詰め込みすぎている目安」と定める。
pub const MAX_SENTENCES_PER_PARAGRAPH: usize = 6;

#[derive(Debug, Deserialize)]
struct RawStructure {
    doctype: std::collections::HashMap<String, DocType>,
    #[serde(default)]
    select: Vec<Question>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocType {
    description: String,
    #[serde(default)]
    description_en: String,
    /// 作業について述べた文が、この型では読み手に向けた中身になるかどうか。
    ///
    /// 何を試して何が駄目だったかは、設計と決定の記録では伝えるべき中身である。
    /// README と参照の文書では読み手に何も与えない。
    /// 同じ文が、型によって中身になるか混ざり物になるかが変わる。
    #[serde(default)]
    pub work_notes_ok: bool,
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

/// 型を選ぶための問い。`structure.toml` の `[[select]]` から読む。
///
/// ツールは問わない。木を出力して、呼び出す側が読者に問う。
/// CI とエージェントから呼ばれるため、対話的な入力を置かない。
#[derive(Debug, Clone, Deserialize)]
pub struct Question {
    pub id: String,
    ja: String,
    en: String,
    pub options: Vec<Choice>,
}

impl Question {
    pub fn text(&self, lang: Lang) -> &str {
        match lang {
            Lang::Ja => &self.ja,
            Lang::En => &self.en,
        }
    }
}

/// 問いの選択肢。次の問いか、決まった型のどちらかを指す。
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    ja: String,
    en: String,
    /// 次の問いの id。
    #[serde(default)]
    pub next: Option<String>,
    /// 決まった型の名前。
    #[serde(default)]
    pub doctype: Option<String>,
}

impl Choice {
    pub fn text(&self, lang: Lang) -> &str {
        match lang {
            Lang::Ja => &self.ja,
            Lang::En => &self.en,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub id: String,
    #[serde(default)]
    pub required: bool,
    /// 標題の直下に置いた地の文で、この節を代えられるかどうか。
    ///
    /// 主題を述べる節にだけ与える。
    /// RFC 7322 の abstract が標題と目次の間に見出し無しで入る前例であり、
    /// README も同じ形を取る。見出しを強いると、かえって読みにくくなる。
    ///
    /// 前提や文脈のような節は、導入では代えられない。
    /// 位置だけで決めると、最初の節が何であっても代えられてしまう。
    #[serde(default)]
    pub lead_ok: bool,
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
    /// どれだけ書くか。検出ではなく誘導のために持つ。
    ///
    /// 規則にはしない。著名な OSS の README 221件で四分位が中央値の 2.5 倍から
    /// 3 倍に広がり、人間の文書が分離しない。分離しない値を閾値にはできない。
    ///
    /// 絶対の字数ではなく、節のあいだの順序と中身の構成で示す。
    /// 節ごとの実測は D-46 にある。
    #[serde(default)]
    scale: String,
    #[serde(default)]
    scale_en: String,
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

    /// その節にどれだけ書くか。持たない節では空を返す。
    pub fn scale(&self, lang: Lang) -> &str {
        match lang {
            Lang::Ja => &self.scale,
            Lang::En => &self.scale_en,
        }
    }

    /// 見出しがこの節にあたるかどうか。表記の揺れを同義語で吸収する。
    ///
    /// 照合は両方の言語の同義語に対して行う。
    /// 日本語の文書が英語の見出しを持つことがあるためである。
    /// Kubernetes の日本語訳は `{{% heading "prerequisites" %}}` をそのまま使っており、
    /// 日本語の同義語だけで照合すると111件中110件で節が無いと出た。
    ///
    /// 表示に使う名前は言語ごとに選ぶ。照合と表示で別の関数を使う。
    pub fn matches(&self, heading: &str, _lang: Lang) -> bool {
        let h = heading.to_lowercase();
        self.ja
            .iter()
            .chain(self.en.iter())
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
/// 型を選ぶための問い。並びは `structure.toml` に書いた順である。
pub fn questions() -> &'static [Question] {
    static Q: OnceLock<Vec<Question>> = OnceLock::new();
    Q.get_or_init(|| {
        let raw: RawStructure = toml::from_str(include_str!("../data/structure.toml"))
            .expect("structure.toml を解釈できない");
        raw.select
    })
}

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
             - 節は見出しが示す問いに答える形で書く。答えていない段落はその節に置かない。\n\
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

/// 作業そのものについて述べた形。
///
/// 読み手に向けた文書に、著者が自分のために残した文脈が混ざることがある。
/// 試した案、判明したこと、作業の段階、以前の状態である。
/// 読み手はそこから次の行動を得られない。
///
/// 同じ形が、設計と決定の記録では伝えるべき中身になる。
/// `work_notes_ok` を持つ型では検出しない。
///
/// 採否は実測で決めた。文書数に対する割合である。
///
/// | 形 | README | 参照(英) | 参照(日) | Rust RFC | ADR |
/// |---|---|---|---|---|---|
/// | 試した案 | 0.4% | 0.2% | 0.1% | 4.5% | 3.2% |
/// | 判明したこと | 0% | 0.3% | 0.1% | 7.5% | 1.6% |
/// | 作業の段階 | 0% | 0% | 0.1% | 0% | 0.5% |
/// | 以前はこうだった | 0% | 0.3% | 0.3% | 5.0% | 0.5% |
///
/// `変更の経緯`（`we refactored`、`移動した`）は候補から除いた。
/// 英語では 11 件のうち 8 件が読み手に行き先を与えており、
/// `moved to their own repositories` のように、どこを見ればよいかを伝えている。
/// 日本語では `フォーカスを移動した` のような通常の動詞と区別できない。
/// 43 件すべてがこの種類の誤検出であった。経緯は D-40 にある。
const WORK_NOTE_JA: &[(&str, &str)] = &[
    (
        r"(試し|検討し|実験し)(た|たが|てみた)(が|結果|ところ|。)|うまく(いかな|動かな)かった",
        "試した案を述べている。読み手に要るのは結果だけである。経緯は決定の記録へ移す",
    ),
    (
        r"(判明した|分かった|気付いた|発覚した)(。|ため|ので)",
        "作業の途中で判明したことを述べている。読み手に向けた結論に書き換える",
    ),
    (
        r"(第|フェーズ|段階|ステップ)\s*\d+\s*(は|が)?(完了|終了|済み|進行中)|(実装|移行|対応)(は|が)(完了|進行中|途中)(した|である|。)",
        "作業の段階を述べている。読み手が使えるかどうかを書く",
    ),
    (
        r"(もともと|元々|当初|以前)は[^。]{0,40}(だった|していた)|(旧|古い)(版|バージョン)では",
        "以前の状態を述べている。読み手に行き先が要るなら版と移行先を示す",
    ),
];

const WORK_NOTE_EN: &[(&str, &str)] = &[
    (
        r"(?i)\b(we|i) (tried|attempted|experimented with|considered)\b|\bafter (trying|testing|experimenting)\b",
        "This recounts what was tried. The reader needs the outcome; move the account to the decision record",
    ),
    (
        r"(?i)\b(we|i) (found|noticed|realized|discovered|learned) that\b|\bit turn(s|ed) out\b",
        "This recounts what was discovered while working. State the conclusion for the reader instead",
    ),
    (
        r"(?i)\b(phase|step|stage|milestone) \d+ (is|was|has been) (complete|completed|done|finished)\b|\b(implementation|migration|refactor(ing)?|rewrite) is (complete|done|ongoing|in progress)\b",
        "This reports the state of the work. Say whether the reader can use it",
    ),
    (
        r"(?i)\boriginally (this|it|the|we|there)\b|\bused to (be|use|have|work|live)\b|\bin (earlier|previous) versions,? (this|it|we)\b",
        "This recounts a former state. If the reader needs it, give the version and where to go now",
    ),
];

/// 作業について述べた文を、読み手に向けた型の文書で検出する。
pub fn check_work_notes(doc: &Document, lang: Lang, dt: &DocType) -> Vec<LocalFinding> {
    // 設計と決定の記録では、何を試したかが伝えるべき中身である。
    if dt.work_notes_ok {
        return Vec::new();
    }
    let table = match lang {
        Lang::Ja => WORK_NOTE_JA,
        _ => WORK_NOTE_EN,
    };
    let mut out = Vec::new();
    for (pat, msg) in table {
        let re = regex::Regex::new(pat).expect("作業の形の正規表現");
        let mut positions = Vec::new();
        for (line, text) in doc.paragraphs() {
            if let Some(m) = re.find(&text) {
                let excerpt: String = text[m.start()..].chars().take(36).collect();
                positions.push(pos(line, excerpt));
            }
        }
        out.extend(finding(RULE_WORK_NOTE, Severity::Warning, msg, positions));
    }
    out
}

/// 先頭の段落。標題の後、最初の節の見出しより前にある最初の地の文である。
fn lead_paragraph(doc: &Document) -> Option<(usize, String)> {
    let first_section = sections(doc).first().map(|o| o.block.line);
    doc.paragraphs()
        .into_iter()
        .find(|(line, _)| first_section.is_none_or(|h| *line < h))
}

/// 文書の書き出しとして避ける形。
///
/// 書き手が自分のために残す覚書は、書き出しに現れる。
/// 一般論、自分の経緯、文書の目的の宣言である。
/// 読み手はそれを必要としない。Diátaxis が言う、
/// 一つの文書が複数の必要に応えようとしている状態である。
///
/// 採否は実測で決めた。先頭の段落 831 件（著名 OSS の README 199、
/// Rust RFC 20、ADR 10、参照コーパスの英語 602）で、
/// ここに挙げた3つの形はいずれも0件であった。
/// 日本語も参照コーパスの先頭の段落 653 件で0件であった。
///
/// 自己言及（`本文書は`、`This document describes`）はここに入れない。
/// 参照コーパスの英語で 37/602 件（6%）現れる。API の説明では成り立つ形である。
/// README と入口の文書に限って別に見る。経緯は D-39 にある。
const LEAD_AVOID_JA: &[(&str, &str)] = &[
    (
        r"^(近年|昨今|現代|今日|最近|かつて|従来)(では|は|、|において)",
        "一般論から始めている。主題を主語にして、何であるかから書く",
    ),
    (
        r"^(筆者|私|我々|著者)(は|が)|^(もともと|元々|当初)(は|、)",
        "書き手の経緯から始めている。読み手に要るのは経緯ではなく主題である",
    ),
    (
        r"^(本文書|この文書|本稿|本書)の(目的|狙い|ねらい)は",
        "文書の目的を宣言している。目的ではなく主題を述べる",
    ),
];

const LEAD_AVOID_EN: &[(&str, &str)] = &[
    (
        r"(?i)^(in recent years|as (software|systems|applications|projects)|traditionally|nowadays|in today's|modern (software|development)|with the (rise|growth|increase))",
        "The lead opens on generalities. Make the subject the subject, and say what it is",
    ),
    (
        r"(?i)^(i (built|made|wrote|created|started|have)|we (built|made|started|decided)|this (started|began) as|originally)",
        "The lead opens on how it came about. The reader needs the subject, not the history",
    ),
    (
        r"(?i)^(the (goal|purpose|aim|objective) of this|this (aims|intends|seeks) to)",
        "The lead declares an intent. State the subject instead",
    ),
];

/// 先頭の段落の書き出しを見る。
pub fn check_lead(doc: &Document, lang: Lang) -> Vec<LocalFinding> {
    let Some((line, text)) = lead_paragraph(doc) else {
        return Vec::new();
    };
    let head = text.trim_start();
    let table = match lang {
        Lang::Ja => LEAD_AVOID_JA,
        _ => LEAD_AVOID_EN,
    };
    for (pat, msg) in table {
        let re = regex::Regex::new(pat).expect("書き出しの正規表現");
        if re.is_match(head) {
            let excerpt: String = head.chars().take(40).collect();
            return finding(
                RULE_LEAD_OPENER,
                Severity::Warning,
                msg,
                vec![pos(line, excerpt)],
            )
            .into_iter()
            .collect();
        }
    }
    Vec::new()
}

/// 標題の直後に、最初の節の見出しより前の地の文があるかどうか。
///
/// ここに置いた段落は、`lead_ok` を持つ節の役目を果たす。
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
            // 主題を述べる節だけは、標題の直下の導入で代えられる。
            None if i == 0 && lead && sec.lead_ok => {}
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

    // 日本語の文書が英語の見出しを持つことがある。両方の同義語で照合する。
    #[test]
    fn an_english_heading_satisfies_a_section_in_a_japanese_document() {
        let dt = doctypes().get("howto").unwrap();
        let src = "# 手順\n\n## Before you begin\n\n用意する。\n\n## やり方\n\n操作する。\n";
        assert!(check_doctype(&Document::parse(src), Lang::Ja, dt).is_empty());
    }

    #[test]
    fn a_japanese_heading_satisfies_a_section_in_an_english_document() {
        let dt = doctypes().get("howto").unwrap();
        let src = "# Procedure\n\n## 前提\n\nSet up.\n\n## Steps\n\nRun it.\n";
        assert!(check_doctype(&Document::parse(src), Lang::En, dt).is_empty());
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

    // 導入で代えられるのは、主題を述べる節だけとする。
    // 位置だけで決めると、最初の節が何であっても代えられてしまう。
    #[test]
    fn a_lead_paragraph_does_not_stand_in_for_a_section_that_is_not_the_subject() {
        let dt = doctypes().get("howto").unwrap();
        let src = "# 手順\n\n本文である。\n\n## やり方\n\n操作する。\n";
        let f = check_doctype(&Document::parse(src), Lang::Ja, dt);
        let m = f
            .iter()
            .find(|x| x.rule_id == RULE_MISSING_SECTION)
            .unwrap();
        assert!(
            m.positions[0].text.starts_with("前提"),
            "{:?}",
            m.positions[0].text
        );
    }

    // 代えられるのは最初の節だけとする。ほかの節は見出しを求める。
    // overview は必須の後続節を持たないため、design で確かめる。
    // design は summary が導入で代えられ、motivation と design を必須に持つ。
    #[test]
    fn a_lead_paragraph_does_not_stand_in_for_a_later_section() {
        let dt = doctypes().get("design").unwrap();
        let src = "# suikou\n\n何についての文書かをここで述べる。\n\n## 設計\n\n構成である。\n";
        let f = check_doctype(&Document::parse(src), Lang::Ja, dt);
        let m = f
            .iter()
            .find(|x| x.rule_id == RULE_MISSING_SECTION)
            .unwrap();
        // 導入は summary を代える。残る必須は motivation だけである。
        assert_eq!(m.positions.len(), 1);
        assert!(m.positions[0].text.starts_with("動機"));
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

    // 書き出しの規則。実測で0件だった3つの形だけを見る。
    #[test]
    fn a_lead_that_opens_on_generalities_is_reported() {
        let src = "# suikou\n\n近年、技術文書の品質が問題になっている。そこでこの道具を作った。\n";
        let f = check_lead(&Document::parse(src), Lang::Ja);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule_id, RULE_LEAD_OPENER);
    }

    #[test]
    fn a_lead_that_opens_on_the_writer_is_reported() {
        let src = "# suikou\n\n筆者は技術文書の lint を探していたが、見つからなかった。\n";
        let f = check_lead(&Document::parse(src), Lang::Ja);
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn a_lead_that_declares_an_intent_is_reported() {
        let src = "# suikou\n\n本文書の目的は、この道具の使い方を説明することである。\n";
        let f = check_lead(&Document::parse(src), Lang::Ja);
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn a_lead_that_names_the_subject_is_not_reported() {
        let src = "# suikou\n\nsuikou は技術文書の lint である。日本語と英語を扱う。\n";
        assert!(check_lead(&Document::parse(src), Lang::Ja).is_empty());
    }

    // 自己言及は規則に入れない。参照コーパスの英語で6%現れるためである。
    #[test]
    fn a_lead_that_refers_to_the_document_is_not_reported() {
        let src = "# suikou\n\n本文書は、この道具が課す規則を述べる。\n";
        assert!(check_lead(&Document::parse(src), Lang::Ja).is_empty());
    }

    #[test]
    fn the_english_lead_rule_fires_on_the_same_three_forms() {
        for src in [
            "# suikou\n\nIn recent years, documentation quality has become a problem.\n",
            "# suikou\n\nI built this because no linter covered technical Japanese.\n",
            "# suikou\n\nThe goal of this document is to explain how the tool works.\n",
        ] {
            let f = check_lead(&Document::parse(src), Lang::En);
            assert_eq!(f.len(), 1, "{src}");
        }
    }

    #[test]
    fn an_english_lead_that_names_the_subject_is_not_reported() {
        let src =
            "# suikou\n\nsuikou is a linter for technical documentation in Japanese and English.\n";
        assert!(check_lead(&Document::parse(src), Lang::En).is_empty());
    }

    // 先頭の段落が無い文書には当てるものが無い。
    #[test]
    fn a_document_with_no_lead_is_not_reported() {
        let src = "# suikou\n\n## 概要\n\n近年、問題になっている。\n";
        assert!(check_lead(&Document::parse(src), Lang::Ja).is_empty());
    }

    // readme の型は240件で測った。発火率が高すぎないことを確かめる。
    #[test]
    fn the_readme_doctype_requires_only_the_lead() {
        let dt = doctypes().get("readme").unwrap();
        let req: Vec<&str> = dt
            .sections
            .iter()
            .filter(|s| s.required)
            .map(|s| s.id.as_str())
            .collect();
        assert_eq!(req, vec!["what"]);
        let src = "# suikou\n\nsuikou は技術文書の lint である。\n";
        assert!(check_doctype(&Document::parse(src), Lang::Ja, dt).is_empty());
    }

    // 作業について述べた文。型によって中身になるか混ざり物になるかが変わる。
    #[test]
    fn a_work_note_in_a_readme_is_reported() {
        let dt = doctypes().get("readme").unwrap();
        let src = "# suikou\n\nsuikou は lint である。\n\n## 使い方\n\n\
                   別の案も試したが、うまく動かなかった。\n";
        let f = check_work_notes(&Document::parse(src), Lang::Ja, dt);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule_id, RULE_WORK_NOTE);
    }

    // 設計と決定の記録では、何を試したかが伝えるべき中身である。
    #[test]
    fn a_work_note_in_a_decision_record_is_not_reported() {
        for name in ["design", "decision"] {
            let dt = doctypes().get(name).unwrap();
            let src = "# x\n\n## 文脈\n\n別の案も試したが、うまく動かなかった。\n";
            assert!(
                check_work_notes(&Document::parse(src), Lang::Ja, dt).is_empty(),
                "{name}"
            );
        }
    }

    #[test]
    fn the_state_of_the_work_is_reported_in_a_readme() {
        let dt = doctypes().get("readme").unwrap();
        let src = "# suikou\n\nsuikou は lint である。\n\nフェーズ 2 は完了している。\n";
        assert_eq!(
            check_work_notes(&Document::parse(src), Lang::Ja, dt).len(),
            1
        );
    }

    // 読み手に向けた文は検出しない。
    #[test]
    fn a_sentence_for_the_reader_is_not_reported() {
        let dt = doctypes().get("readme").unwrap();
        let src = "# suikou\n\nsuikou は lint である。\n\n\
                   設定は .suikou/profile.toml で変更する。\n";
        assert!(check_work_notes(&Document::parse(src), Lang::Ja, dt).is_empty());
    }

    #[test]
    fn the_english_work_note_rule_fires_on_the_same_forms() {
        let dt = doctypes().get("readme").unwrap();
        for src in [
            "# x\n\nIt is a linter.\n\nWe tried a different approach first.\n",
            "# x\n\nIt is a linter.\n\nWe found that the parser was too slow.\n",
            "# x\n\nIt is a linter.\n\nPhase 2 is complete.\n",
            "# x\n\nIt is a linter.\n\nOriginally this used a different format.\n",
        ] {
            assert_eq!(
                check_work_notes(&Document::parse(src), Lang::En, dt).len(),
                1,
                "{src}"
            );
        }
    }

    #[test]
    fn an_english_sentence_for_the_reader_is_not_reported() {
        let dt = doctypes().get("readme").unwrap();
        let src = "# x\n\nIt is a linter.\n\nThe docs moved to the project wiki.\n";
        assert!(check_work_notes(&Document::parse(src), Lang::En, dt).is_empty());
    }

    // 型の旗が意図した型にだけ立っていることを確かめる。
    //
    // design と decision は、何を試したかが伝えるべき中身である。
    // log は記録を束ねた文書であり、記録そのものが中身である。
    // 旗が増えると検出が静かに消えるため、一覧で固定する。
    #[test]
    fn only_the_three_record_types_allow_work_notes() {
        let mut ok: Vec<&str> = doctypes()
            .iter()
            .filter(|(_, dt)| dt.work_notes_ok)
            .map(|(n, _)| n.as_str())
            .collect();
        ok.sort_unstable();
        assert_eq!(ok, vec!["decision", "design", "log"]);
    }

    // 記録を束ねた型では、作業について述べた文を検出しない。
    #[test]
    fn a_log_keeps_its_account_of_the_work() {
        let dt = doctypes().get("log").unwrap();
        let src = "# 作業の一覧\n\n別の案も試したが、うまく動かなかった。\n";
        assert!(check_work_notes(&Document::parse(src), Lang::Ja, dt).is_empty());
        assert!(dt.sections.is_empty());
    }

    // どれだけ書くかは、持っている型でだけ示す。規則にはしない。
    #[test]
    fn only_the_readme_type_carries_a_scale() {
        let mut with: Vec<&str> = doctypes()
            .iter()
            .filter(|(_, dt)| dt.sections.iter().any(|s| !s.scale(Lang::Ja).is_empty()))
            .map(|(n, _)| n.as_str())
            .collect();
        with.sort_unstable();
        assert_eq!(with, vec!["readme"]);
    }

    // 持つ節は、日本語と英語の両方を持つ。片方だけでは出力が欠ける。
    #[test]
    fn a_scale_is_given_in_both_languages() {
        for (name, dt) in doctypes() {
            for s in &dt.sections {
                assert_eq!(
                    s.scale(Lang::Ja).is_empty(),
                    s.scale(Lang::En).is_empty(),
                    "{name} の {} が片方の言語しか持たない",
                    s.id
                );
            }
        }
    }

    // 型を選ぶ木と型の一覧を、試験で同期させる。
    // 選択肢の行き先が実在しないと、問いに答えても型に届かない。
    #[test]
    fn every_choice_points_at_something_that_exists() {
        let ids: Vec<&str> = questions().iter().map(|q| q.id.as_str()).collect();
        for q in questions() {
            assert!(!q.options.is_empty(), "{} に選択肢が無い", q.id);
            for c in &q.options {
                match (&c.next, &c.doctype) {
                    (Some(n), None) => assert!(ids.contains(&n.as_str()), "{n} という問いが無い"),
                    (None, Some(d)) => assert!(doctypes().contains_key(d), "{d} という型が無い"),
                    _ => panic!("{} の選択肢は next か doctype のどちらか一方を持つ", q.id),
                }
            }
        }
    }

    // すべての型に木から到達できる。到達できない型は選べない。
    #[test]
    fn every_doctype_is_reachable_from_the_tree() {
        let mut reached: Vec<&str> = questions()
            .iter()
            .flat_map(|q| q.options.iter())
            .filter_map(|c| c.doctype.as_deref())
            .collect();
        reached.sort_unstable();
        reached.dedup();
        let mut all: Vec<&str> = doctypes().keys().map(String::as_str).collect();
        all.sort_unstable();
        assert_eq!(reached, all, "木から到達できない型がある");
    }

    #[test]
    fn every_question_is_given_in_both_languages() {
        for q in questions() {
            assert!(
                !q.text(Lang::Ja).is_empty() && !q.text(Lang::En).is_empty(),
                "{}",
                q.id
            );
            for c in &q.options {
                assert!(
                    !c.text(Lang::Ja).is_empty() && !c.text(Lang::En).is_empty(),
                    "{} の選択肢が片方の言語しか持たない",
                    q.id
                );
            }
        }
    }

    // 最初の問いは先頭に置く。出力の順序がこれに依る。
    #[test]
    fn the_first_question_comes_first() {
        assert_eq!(questions().first().map(|q| q.id.as_str()), Some("purpose"));
    }
}
