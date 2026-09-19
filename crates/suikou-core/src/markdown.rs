//! Markdown をブロックに分ける。
//!
//! 行ベースで実装する。Markdown パーサを使わないのは、
//! `research/` の参照実装が行ベースであり、ゴールデンテストの値を一致させるためである。
//! 依存が減る分、バージョン差による事故も減る。
//!
//! 前処理の順序が結果を左右する。`docs/METRICS.md` に定めた順に行う。

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
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Document {
    pub blocks: Vec<Block>,
    /// 空行を除いた行数。箇条書き割合の分母になる。
    pub non_empty_lines: usize,
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
re!(re_fence, r"(?s)```.*?```|~~~.*?~~~");
re!(re_escape, r"\\([.\-+*_#`\[\]()<>|])");
re!(re_html, r"<[^>]+>");
re!(re_link, r"!?\[([^\]]*)\]\([^)]*\)");
re!(re_code_span, r"`[^`]*`");
re!(re_emphasis, r"\*\*([^*]*)\*\*|\*([^*]*)\*");
re!(re_list, r"^(\s*)([-*+]|\d+[.)])\s+(.*)$");
re!(re_heading, r"^#{1,6}\s+(.*)$");

/// 前処理。順序を変えてはならない。
/// エスケープ解除を飛ばすと文分割が成立しない。
pub fn preprocess(source: &str) -> String {
    let s = re_frontmatter().replace(source, "");
    let s = re_fence().replace_all(&s, "");
    let s = re_escape().replace_all(&s, "$1");
    let s = re_html().replace_all(&s, " ");
    s.into_owned()
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
        let text = preprocess(source);
        let mut blocks = Vec::new();
        let mut non_empty = 0usize;

        for (i, raw_line) in text.lines().enumerate() {
            let line_no = i + 1;
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                continue;
            }
            non_empty += 1;

            if let Some(c) = re_heading().captures(trimmed) {
                blocks.push(Block {
                    kind: BlockKind::Heading,
                    line: line_no,
                    text: strip_inline(c.get(1).map_or("", |m| m.as_str())),
                    depth: 0,
                    ordered: false,
                    raw: raw_line.to_string(),
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
            });
        }
        Document {
            blocks,
            non_empty_lines: non_empty,
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
}
