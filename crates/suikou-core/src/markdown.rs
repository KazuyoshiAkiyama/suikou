//! Markdown をブロックに分ける。
//!
//! 行ベースで実装する。Markdown パーサを使わないのは、
//! `research/` の参照実装が行ベースであり、ゴールデンテストの値を一致させるためである。
//! 依存が減る分、バージョン差による事故も減る。
//!
//! 前処理の順序が結果を左右する。`docs/content/ja/metrics.md` に定めた順に行う。

use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BlockKind {
    Prose,
    ListItem,
    Heading,
    Table,
    Quote,
}

#[derive(Debug, Clone, Serialize)]
pub struct Block {
    pub kind: BlockKind,
    /// 1 始まりの行番号。
    pub line: usize,
    /// 記法を取り除いた本文。
    pub text: String,
    /// 箇条書きの入れ子の深さ。先頭の空白2個を1段とする。
    pub depth: usize,
    /// 番号付きリストかどうか。
    pub ordered: bool,
    /// 記法を取り除く前の行。M 系の規則が使う。
    pub raw: String,
    /// 直前が空行かどうか。段落の切れ目を知るために持つ。
    pub blank_before: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Document {
    pub blocks: Vec<Block>,
    /// 空行を除いた行数。箇条書き割合の分母になる。
    pub non_empty_lines: usize,
    /// コードブロックがあった行。前処理で取り除くため、本文の有無を見る規則が要る。
    pub code_lines: Vec<usize>,
}

macro_rules! re {
    ($name:ident, $pat:expr) => {
        fn $name() -> &'static Regex {
            static R: OnceLock<Regex> = OnceLock::new();
            R.get_or_init(|| Regex::new($pat).unwrap())
        }
    };
}

re!(re_frontmatter, r"(?s)\A---\n.*?\n---\n");
re!(re_escape, r"\\([.\-+*_#`\[\]()<>|])");
re!(re_html, r"<[^>]+>");
re!(re_link, r"!?\[([^\]]*)\]\([^)]*\)");
re!(re_code_span, r"`[^`]*`");
re!(re_emphasis, r"\*\*([^*]*)\*\*|\*([^*]*)\*");
re!(re_list, r"^(\s*)([-*+]|\d+[.)])\s+(.*)$");
// 見出しの字下げは3桁までとする。CommonMark がそう定めている。
// 4桁以上の字下げはコードであり、その中の `#` を見出しとして読んではならない。
// Rust RFC の本文でこの誤読が起き、節がひとつも見つからなくなった。
re!(re_heading, r"^ {0,3}#{1,6}\s+(.*)$");

/// 取り除いた範囲を、同じ数の改行に置き換える。
///
/// 中身ごと消すと、後続の行の番号が前へずれる。
/// 指摘の行番号が利用者の見ているファイルと食い違うと、その指摘は使えない。
fn blank_out(caps: &regex::Captures) -> String {
    let m = caps.get(0).map_or("", |m| m.as_str());
    "\n".repeat(m.matches('\n').count())
}

/// 中身がコードである Hugo のショートコード。
///
/// これ以外のショートコード（`note`、`caution` など）は地の文を包むため、
/// 取り除くと本文そのものが消える。ここに挙げるものだけを対象とする。
/// `code_sample` は外部のファイルを参照するだけで、中身を持たないため含めない。
/// `tab` は名前では決まらない。`codelang` を宣言したものだけを対象とする。
const CODE_SHORTCODES: [&str; 2] = ["highlight", "mermaid"];

/// 前処理。順序を変えてはならない。
/// エスケープ解除を飛ばすと文分割が成立しない。
pub fn preprocess(source: &str) -> String {
    let s = re_frontmatter().replace(source, blank_out);
    let s = blank_lines(&s, &code_fence_lines(&s));
    let s = blank_code_shortcodes(&s);
    let s = re_escape().replace_all(&s, "$1");
    let s = re_html().replace_all(&s, blank_out);
    s.into_owned()
}

/// コードを持つショートコードが占める行を、1 始まりで返す。
///
/// 閉じが見つからない開きは対象にしない。
/// 対応が崩れている文書で、そこから先の本文をすべて落とすことになるためである。
fn code_shortcode_lines(source: &str) -> Vec<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some(name) = shortcode_open(lines[i]) else {
            i += 1;
            continue;
        };
        match (i + 1..lines.len()).find(|j| has_shortcode_close(lines[*j], name)) {
            Some(end) => {
                out.extend(i + 1..=end + 1);
                i = end + 1;
            }
            None => i += 1,
        }
    }
    out
}

/// コードを持つショートコードの中身を、同じ数の改行に置き換える。
///
/// 囲みのコードブロックと同じ扱いにする。
/// 取り除かないと、YAML のコメントの `#` が見出しとして読まれる。
/// Kubernetes の英語文書で、本文の無い節の指摘77件はすべてこれを出どころとしていた。
fn blank_code_shortcodes(source: &str) -> String {
    if !source.contains("{{<") {
        return source.to_string();
    }
    let drop: std::collections::HashSet<usize> = code_shortcode_lines(source).into_iter().collect();
    let mut out = String::with_capacity(source.len());
    for (i, line) in source.lines().enumerate() {
        if !drop.contains(&(i + 1)) {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// コードを持つショートコードの開きなら、閉じを探すための名前を返す。
///
/// 名前で決まるもののほかに、`codelang` を宣言した `tab` がある。
/// どちらも文書の側が中身をコードだと宣言しているため、推測にはならない。
fn shortcode_open(line: &str) -> Option<&'static str> {
    let t = line.trim_start().strip_prefix("{{<")?.trim_start();
    if let Some(n) = CODE_SHORTCODES.into_iter().find(|n| t.starts_with(n)) {
        return Some(n);
    }
    (t.starts_with("tab ") && t.contains("codelang")).then_some("tab")
}

/// 閉じは行の途中にも現れる。行の先頭に限定しない。
fn has_shortcode_close(line: &str, name: &str) -> bool {
    line.split("{{<").skip(1).any(|t| {
        t.trim_start()
            .strip_prefix('/')
            .is_some_and(|t| t.trim_start().starts_with(name))
    })
}

/// 指定した行を、同じ数の改行に置き換える。
fn blank_lines(source: &str, drop: &[usize]) -> String {
    let drop: std::collections::HashSet<usize> = drop.iter().copied().collect();
    let mut out = String::with_capacity(source.len());
    for (i, line) in source.lines().enumerate() {
        if !drop.contains(&(i + 1)) {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// 囲みのコードブロックの開きなら、その記号と長さを返す。
///
/// CommonMark は記号3個以上を開きとし、字下げは3桁までを認める。
fn fence_open(line: &str) -> Option<(char, usize)> {
    let indent = line.len() - line.trim_start().len();
    if indent > 3 {
        return None;
    }
    let t = line.trim_start();
    let c = t.chars().next()?;
    if c != '`' && c != '~' {
        return None;
    }
    let n = t.chars().take_while(|x| *x == c).count();
    (n >= 3).then_some((c, n))
}

/// 囲みのコードブロックが占める行を、1 始まりで返す。
///
/// 前処理はコードを取り除くため、そのままでは「本文の無い節」と見分けがつかない。
/// コードだけを載せる節は正しい形なので、行を覚えておいて判定に使う。
///
/// 閉じは、開きと同じ記号で、開き以上の長さで、そのあとに何も無い行だけとする。
/// CommonMark がそう定めている。短い囲みを長い囲みの中に入れる書き方が実際にあり、
/// 内側の囲みで閉じたことにすると、そこから先のコードが本文として読まれる。
/// Rust RFC の `` ```` `` の中に `` ``` `` を入れた例で、コードの `#` が見出しになった。
fn code_fence_lines(source: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut open: Option<(char, usize)> = None;
    for (i, line) in source.lines().enumerate() {
        match open {
            Some((c, n)) => {
                out.push(i + 1);
                let t = line.trim();
                if t.chars().all(|x| x == c) && t.chars().count() >= n && !t.is_empty() {
                    open = None;
                }
            }
            None => {
                if let Some(f) = fence_open(line) {
                    open = Some(f);
                    out.push(i + 1);
                }
            }
        }
    }
    out
}

fn strip_inline(line: &str) -> String {
    let s = re_link().replace_all(line, "$1");
    let s = re_code_span().replace_all(&s, "");
    let s = re_emphasis().replace_all(&s, "$1$2");
    s.trim().to_string()
}

pub fn count_chars(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

pub fn is_japanese_char(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x30FF | 0x4E00..=0x9FFF)
}

pub fn is_kanji(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF)
}

impl Document {
    pub fn parse(source: &str) -> Self {
        // ショートコードの中身も、節が本文を持つかどうかの判定ではコードとして数える。
        let mut code_lines = code_fence_lines(source);
        code_lines.extend(code_shortcode_lines(source));
        code_lines.sort_unstable();
        code_lines.dedup();
        let text = preprocess(source);
        let mut blocks = Vec::new();
        let mut non_empty = 0usize;

        let mut blank_before = true;
        for (i, raw_line) in text.lines().enumerate() {
            let line_no = i + 1;
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                blank_before = true;
                continue;
            }
            non_empty += 1;

            if let Some(c) = re_heading().captures(raw_line) {
                blocks.push(Block {
                    kind: BlockKind::Heading,
                    line: line_no,
                    text: strip_inline(c.get(1).map_or("", |m| m.as_str())),
                    depth: 0,
                    ordered: false,
                    raw: raw_line.to_string(),
                    blank_before: std::mem::replace(&mut blank_before, false),
                });
                continue;
            }
            if let Some(c) = re_list().captures(raw_line) {
                let indent_src = c.get(1).map_or("", |m| m.as_str());
                let indent = indent_src.chars().filter(|ch| *ch == ' ').count()
                    + indent_src.chars().filter(|ch| *ch == '\t').count() * 2;
                let marker = c.get(2).map_or("", |m| m.as_str());
                blocks.push(Block {
                    kind: BlockKind::ListItem,
                    line: line_no,
                    text: strip_inline(c.get(3).map_or("", |m| m.as_str())),
                    depth: indent / 2,
                    ordered: !matches!(marker, "-" | "*" | "+"),
                    raw: raw_line.to_string(),
                    blank_before: std::mem::replace(&mut blank_before, false),
                });
                continue;
            }
            let kind = if trimmed.starts_with('|') {
                BlockKind::Table
            } else if trimmed.starts_with('>') {
                BlockKind::Quote
            } else {
                BlockKind::Prose
            };
            blocks.push(Block {
                kind,
                line: line_no,
                text: strip_inline(trimmed),
                depth: 0,
                ordered: false,
                raw: raw_line.to_string(),
                blank_before: std::mem::replace(&mut blank_before, false),
            });
        }
        Document {
            blocks,
            non_empty_lines: non_empty,
            code_lines,
        }
    }

    fn join(&self, kinds: &[BlockKind]) -> String {
        self.blocks
            .iter()
            .filter(|b| kinds.contains(&b.kind))
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 文構造系の指標が対象とする本文。
    pub fn prose(&self) -> String {
        self.join(&[BlockKind::Prose])
    }

    /// 語彙系の指標が対象とする本文。地の文と箇条書きの本文を含む。
    /// 地の文だけで計算してはならない。出力の大半が箇条書きのモデルがあり、
    /// その場合に本文の8割以上を捨てることになる。
    pub fn body(&self) -> String {
        self.join(&[BlockKind::Prose, BlockKind::ListItem])
    }

    pub fn headings(&self) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|b| b.kind == BlockKind::Heading)
            .collect()
    }

    /// D2 地の文比率。
    pub fn prose_ratio(&self) -> f64 {
        let prose = count_chars(&self.prose());
        let body = count_chars(&self.body());
        if body == 0 {
            0.0
        } else {
            prose as f64 / body as f64
        }
    }

    /// D8 箇条書き割合。
    pub fn list_ratio(&self) -> f64 {
        if self.non_empty_lines == 0 {
            return 0.0;
        }
        let n = self
            .blocks
            .iter()
            .filter(|b| b.kind == BlockKind::ListItem)
            .count();
        n as f64 / self.non_empty_lines as f64
    }

    /// 連続するリスト項目のまとまりを返す。M1 と M5 が使う。
    pub fn list_blocks(&self) -> Vec<Vec<&Block>> {
        let mut out: Vec<Vec<&Block>> = Vec::new();
        let mut cur: Vec<&Block> = Vec::new();
        for b in &self.blocks {
            if b.kind == BlockKind::ListItem {
                cur.push(b);
            } else if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        out
    }

    /// 連続する地の文の行を段落としてまとめる。
    ///
    /// 空行が段落の切れ目になる。行ごとにブロックを持つ作りのため、
    /// 段落の単位で見る規則はこれを使う。
    ///
    /// 地の文以外のブロックも切れ目になる。
    /// 見出しや箇条書きを読み飛ばすだけにすると、それをまたいだ地の文が
    /// ひとつの段落として連結され、文の数が水増しされる。
    pub fn paragraphs(&self) -> Vec<(usize, String)> {
        let mut out: Vec<(usize, String)> = Vec::new();
        let mut prev_was_prose = false;
        for b in &self.blocks {
            if b.kind != BlockKind::Prose {
                prev_was_prose = false;
                continue;
            }
            match out.last_mut() {
                Some(last) if prev_was_prose && !b.blank_before => {
                    last.1.push(' ');
                    last.1.push_str(&b.text);
                }
                _ => out.push((b.line, b.text.clone())),
            }
            prev_was_prose = true;
        }
        out
    }

    /// 指定した行より前にある最後のブロックを返す。M1 の導入文判定が使う。
    pub fn block_before(&self, line: usize) -> Option<&Block> {
        self.blocks.iter().rfind(|b| b.line < line)
    }
}

/// 日本語の文分割。`。！？` の直後で切る。
/// 5文字未満、または日本語の文字を含まない断片は捨てる。
pub fn split_sentences_ja(prose: &str) -> Vec<String> {
    let mut out = Vec::new();
    for para in prose.lines() {
        let mut cur = String::new();
        for ch in para.chars() {
            cur.push(ch);
            if matches!(ch, '。' | '！' | '？') {
                push_ja(&mut out, &cur);
                cur.clear();
            }
        }
        push_ja(&mut out, &cur);
    }
    out
}

fn push_ja(out: &mut Vec<String>, s: &str) {
    let t = s.trim();
    if t.chars().count() >= 5 && t.chars().any(is_japanese_char) {
        out.push(t.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescapes_before_splitting() {
        let src = "The validator reports each error\\. It also reports a line number\\.";
        let out = preprocess(src);
        assert!(
            out.contains("error. It"),
            "エスケープが解除されていない: {out}"
        );
    }

    #[test]
    fn keeps_line_numbers_across_removed_blocks() {
        // フロントマターとコードブロックを消しても、後続の行番号を保つ。
        // 番号がずれると、指摘の位置が利用者の見ているファイルと食い違う。
        let src = "---\ntitle: x\n---\n\n本文である。\n\n```\ncode\ncode\n```\n\n後の行である。\n";
        let d = Document::parse(src);
        let last = d.blocks.last().unwrap();
        assert_eq!(last.text, "後の行である。");
        assert_eq!(last.line, 12, "{:?}", d.blocks);
    }

    #[test]
    fn removes_code_fences_and_frontmatter() {
        let src = "---\ntitle: x\n---\nhello\n```\ncode. code.\n```\nworld\n";
        let out = preprocess(src);
        assert!(!out.contains("title"));
        assert!(!out.contains("code."));
        assert!(out.contains("hello") && out.contains("world"));
    }

    #[test]
    fn classifies_blocks() {
        let src = "# 見出し\n\n本文である。\n\n- 項目1\n- 項目2\n\n| a | b |\n> quote\n";
        let d = Document::parse(src);
        let kinds: Vec<BlockKind> = d.blocks.iter().map(|b| b.kind).collect();
        assert_eq!(
            kinds,
            vec![
                BlockKind::Heading,
                BlockKind::Prose,
                BlockKind::ListItem,
                BlockKind::ListItem,
                BlockKind::Table,
                BlockKind::Quote,
            ]
        );
        assert_eq!(d.non_empty_lines, 6);
    }

    #[test]
    fn ordered_flag_and_depth() {
        let src = "1. first\n2. second\n  - nested\n";
        let d = Document::parse(src);
        assert!(d.blocks[0].ordered);
        assert!(!d.blocks[2].ordered);
        assert_eq!(d.blocks[2].depth, 1);
    }

    #[test]
    fn body_includes_list_items_prose_does_not() {
        let src = "本文である。\n\n- 項目の本文\n";
        let d = Document::parse(src);
        assert!(!d.prose().contains("項目の本文"));
        assert!(d.body().contains("項目の本文"));
    }

    #[test]
    fn prose_ratio_drops_when_document_is_mostly_lists() {
        let src = "短い導入。\n\n- ああああああああああ\n- いいいいいいいいいい\n- うううううううううう\n";
        let d = Document::parse(src);
        assert!(d.prose_ratio() < 0.3, "ratio={}", d.prose_ratio());
    }

    #[test]
    fn strips_inline_markup() {
        let src = "**強調**と`コード`と[リンク](http://example.com)。\n";
        let d = Document::parse(src);
        assert_eq!(d.blocks[0].text, "強調ととリンク。");
    }

    #[test]
    fn splits_japanese_sentences() {
        let s = split_sentences_ja("これは一文目である。これは二文目である。これは三文目だ。");
        assert_eq!(s.len(), 3);
        assert!(s[0].ends_with('。'));
    }

    #[test]
    fn drops_short_fragments() {
        let s = split_sentences_ja("ああ。これは十分に長い文である。");
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn groups_consecutive_list_items() {
        let src = "導入である。\n\n- a\n- b\n\n地の文。\n\n- c\n";
        let d = Document::parse(src);
        let g = d.list_blocks();
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].len(), 2);
        assert_eq!(g[1].len(), 1);
    }

    #[test]
    fn finds_block_before_a_list() {
        let src = "導入である。\n\n- a\n";
        let d = Document::parse(src);
        let list_line = d.list_blocks()[0][0].line;
        let before = d.block_before(list_line).unwrap();
        assert_eq!(before.kind, BlockKind::Prose);
        assert_eq!(before.text, "導入である。");
    }

    #[test]
    fn paragraphs_join_consecutive_prose_lines() {
        let doc = Document::parse("一行目である。\n二行目である。\n\n別の段落である。\n");
        let p = doc.paragraphs();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].1, "一行目である。 二行目である。");
        assert_eq!(p[1].0, 4);
    }

    // 見出しや箇条書きをまたいで地の文が連結されると、文の数が水増しされる。
    #[test]
    fn a_heading_breaks_a_paragraph() {
        let doc = Document::parse("前の段落である。\n\n## 見出し\n\n後の段落である。\n");
        let p = doc.paragraphs();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].1, "前の段落である。");
    }

    #[test]
    fn a_list_breaks_a_paragraph() {
        let doc = Document::parse("前の段落である。\n\n- 項目である。\n  続きである。\n");
        let p = doc.paragraphs();
        assert_eq!(p[0].1, "前の段落である。");
        assert!(p.iter().all(|(_, t)| !t.contains("前の段落である。 ")));
    }

    // Hugo のショートコードの中の YAML コメントが見出しとして読まれていた。
    #[test]
    fn a_code_shortcode_is_blanked() {
        let src = "# 題\n\n{{< highlight yaml >}}\n# CAUTION: not a heading\nkey: value\n                   {{< /highlight >}}\n\n本文である。\n";
        let doc = Document::parse(src);
        assert_eq!(doc.headings().len(), 1);
        // 行番号は元のファイルに一致したままとする。
        assert_eq!(doc.blocks.last().unwrap().line, 8);
    }

    // 中身のあるショートコードを消すと本文が落ちる。対象を限る。
    #[test]
    fn a_prose_shortcode_is_kept() {
        let src = "# 題\n\n{{< note >}}\n本文である。\n{{< /note >}}\n";
        let doc = Document::parse(src);
        assert!(doc.body().contains("本文である。"));
    }

    // 閉じの無い開きで、そこから先の本文をすべて落としてはならない。
    #[test]
    fn an_unclosed_shortcode_leaves_the_document_alone() {
        let src = "# 題\n\n{{< highlight yaml >}}\n\n本文である。\n";
        let doc = Document::parse(src);
        assert!(doc.body().contains("本文である。"));
    }

    // 閉じが行の途中に現れる書き方が実際のコーパスにある。
    #[test]
    fn a_close_inside_a_line_ends_the_shortcode() {
        let src = "# 題\n\n{{< highlight text >}}\n# not a heading\nx{{< /highlight >}}.\n\n                   本文である。\n";
        let doc = Document::parse(src);
        assert_eq!(doc.headings().len(), 1);
        assert!(doc.body().contains("本文である。"));
    }

    // tab は名前では決まらない。codelang を宣言したものだけがコードを持つ。
    #[test]
    fn a_tab_with_codelang_is_blanked() {
        let src = "# 題\n\n{{< tab name=\"Linux\" codelang=\"yaml\" >}}\n                   # not a heading\n{{< /tab >}}\n\n本文である。\n";
        let doc = Document::parse(src);
        assert_eq!(doc.headings().len(), 1);
        assert!(doc.body().contains("本文である。"));
    }

    #[test]
    fn a_tab_without_codelang_is_kept() {
        let src = "# 題\n\n{{< tab name=\"手順\" >}}\n本文である。\n{{< /tab >}}\n";
        let doc = Document::parse(src);
        assert!(doc.body().contains("本文である。"));
    }

    // ショートコードだけを含む節は、本文を持つものとして数える。
    #[test]
    fn a_shortcode_counts_as_a_body() {
        let src = "# 題\n\n## 図\n\n{{< mermaid >}}\ngraph TD;\n{{< /mermaid >}}\n";
        let doc = Document::parse(src);
        assert!(doc.code_lines.contains(&6));
    }

    // 4桁以上の字下げはコードである。その中の `#` を見出しとして読んではならない。
    #[test]
    fn an_indented_hash_is_not_a_heading() {
        let doc = Document::parse("## 節\n\n本文である。\n\n    # これはコードである\n");
        assert_eq!(doc.headings().len(), 1);
    }

    // CommonMark は3桁までの字下げを見出しとして認める。
    #[test]
    fn a_heading_indented_up_to_three_spaces_is_a_heading() {
        let doc = Document::parse("   ## 節\n\n本文である。\n");
        assert_eq!(doc.headings().len(), 1);
    }

    // 長い囲みの中に短い囲みを入れる書き方が実際にある。
    // 内側で閉じたことにすると、そこから先のコードが本文として読まれる。
    #[test]
    fn a_short_fence_inside_a_long_one_does_not_close_it() {
        let src = "## 節\n\n````rust\n# ```cargo\n# [dependencies]\n# ```\n\
                   fn main() {}\n````\n\n本文である。\n";
        let doc = Document::parse(src);
        assert_eq!(doc.headings().len(), 1);
        assert!(doc.body().contains("本文である。"));
    }

    #[test]
    fn a_fence_closes_on_an_equal_or_longer_marker() {
        let doc = Document::parse("```\ncode\n```\n\n本文である。\n");
        assert!(doc.body().contains("本文である。"));
        assert_eq!(doc.code_lines, vec![1, 2, 3]);
    }

    // 閉じの無い囲みは、そこから先をすべてコードとして扱う。
    // 開いたままの囲みの中身を本文として読むほうが害が大きい。
    #[test]
    fn an_unclosed_fence_runs_to_the_end() {
        let doc = Document::parse("## 節\n\n```\ncode\n# not a heading\n");
        assert_eq!(doc.headings().len(), 1);
    }
}
