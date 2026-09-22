//! `suikou plot` の実装。書く前に構成を決めるための型枠を出す。
//!
//! 出すのは見出しと、その節が答える問いだけである。本文は書かない。
//! これを人が承認してから本文を書く流れにすると、構成の誤りを本文を書く前に見つけられる。
//!
//! プロットは `.suikou/plans/` に置く。思考の側の文書であり、版管理しない。
//! 設計の「思考ファイルと成果物ファイルの分離」に従う。
//! そのため CI からは見えない。プロットとの突き合わせは手元でのみ働く。

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use suikou_core::lang::Lang;
use suikou_core::structure::{doctypes, writing_rules, DocType};

pub struct Options {
    pub path: String,
    pub doctype: String,
    pub lang: String,
    pub force: bool,
}

/// 成果物のパスから、対応するプロットのパスを作る。
pub fn plan_path(target: &Path) -> PathBuf {
    Path::new(".suikou/plans").join(target)
}

/// プロットの先頭に置く、型の id の印。
///
/// `check` がここから型を読む。説明の文だけでは機械が読み戻せない。
/// HTML のコメントにしてあるのは、Markdown として表示したときに見えないためである。
const DOCTYPE_MARK: &str = "<!-- suikou:doctype";

fn render(dt: &DocType, dt_id: &str, lang: Lang, target: &str) -> String {
    match lang {
        Lang::Ja => render_ja(dt, dt_id, target),
        Lang::En => render_en(dt, dt_id, target),
    }
}

fn render_ja(dt: &DocType, dt_id: &str, target: &str) -> String {
    let lang = Lang::Ja;
    let mut out = format!(
        "# {target} のプロット\n\n{DOCTYPE_MARK} {dt_id} -->\n型: {}\n\n",
        dt.description(lang)
    );
    out.push_str(
        "各節に、その節で述べる主張を一行で書く。書けない節は、まだ考えが足りていない。\n\
         承認を得てから本文を書く。本文を書いたあとに構成を変えた場合は、この文書も直す。\n\n",
    );
    for s in &dt.sections {
        let name = s
            .names(lang)
            .first()
            .cloned()
            .unwrap_or_else(|| s.id.clone());
        let mark = if s.required { "" } else { "（任意）" };
        out.push_str(&format!("## {name}{mark}\n\n"));
        out.push_str(&format!("問い: {}\n", s.question(lang)));
        out.push_str(&format!("書くこと: {}\n", s.writes(lang)));
        out.push_str("主張: \n\n");
    }
    out.push_str("## 書き方\n\n");
    out.push_str(&writing_rules(lang));
    out
}

fn render_en(dt: &DocType, dt_id: &str, target: &str) -> String {
    let lang = Lang::En;
    let mut out = format!(
        "# Plot for {target}\n\n{DOCTYPE_MARK} {dt_id} -->\nDoctype: {}\n\n",
        dt.description(lang)
    );
    out.push_str(
        "Write the claim each section makes, in one line. A section you cannot fill in is a \
         section you have not thought through yet.\nGet the plot approved before writing the \
         body. If the structure changes while writing, change this document too.\n\n",
    );
    for s in &dt.sections {
        let name = s
            .names(lang)
            .first()
            .cloned()
            .unwrap_or_else(|| s.id.clone());
        let mark = if s.required { "" } else { " (optional)" };
        out.push_str(&format!("## {name}{mark}\n\n"));
        out.push_str(&format!("Question: {}\n", s.question(lang)));
        out.push_str(&format!("What goes here: {}\n", s.writes(lang)));
        out.push_str("Claim: \n\n");
    }
    out.push_str("## How to write it\n\n");
    out.push_str(&writing_rules(lang));
    out
}

pub fn run(opts: Options) -> Result<()> {
    let dt = doctypes().get(&opts.doctype).with_context(|| {
        let names: Vec<&str> = doctypes().keys().map(String::as_str).collect();
        format!(
            "型 {} を知らない。使えるのは {}",
            opts.doctype,
            names.join("、")
        )
    })?;
    let lang = match opts.lang.as_str() {
        "en" => Lang::En,
        _ => Lang::Ja,
    };
    let target = Path::new(&opts.path);
    let dest = plan_path(target);
    if dest.exists() && !opts.force {
        bail!(
            "{} が既にある。作り直すなら --force を付ける",
            dest.display()
        );
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("{} を作れない", parent.display()))?;
    }
    std::fs::write(&dest, render(dt, &opts.doctype, lang, &opts.path))
        .with_context(|| format!("{} に書けない", dest.display()))?;
    println!(
        "{} を作った。主張を書き込んで、承認を得てから本文に進む。",
        dest.display()
    );
    Ok(())
}

/// プロットが宣言した型を読む。
///
/// フロントマターを持てない文書でも、プロットがあれば型が決まる。
/// プロットは `--doctype` を与えて作るため、型はそこに既にある。
pub fn plot_doctype(target: &Path) -> Option<String> {
    let src = std::fs::read_to_string(plan_path(target)).ok()?;
    src.lines()
        .find_map(|l| l.trim().strip_prefix(DOCTYPE_MARK))
        .map(|v| v.trim().trim_end_matches("-->").trim().to_string())
        .filter(|v| !v.is_empty())
}

/// プロットから見出しを読む。突き合わせに使う。
pub fn plot_headings(target: &Path) -> Option<Vec<String>> {
    let src = std::fs::read_to_string(plan_path(target)).ok()?;
    let heads: Vec<String> = src
        .lines()
        .filter_map(|l| l.strip_prefix("## "))
        .map(|h| {
            h.trim()
                .trim_end_matches("（任意）")
                .trim_end_matches("(optional)")
                .trim()
                .to_string()
        })
        .filter(|h| h != "書き方" && h != "How to write it")
        .collect();
    (!heads.is_empty()).then_some(heads)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(name: &str) -> &'static DocType {
        doctypes().get(name).unwrap()
    }

    #[test]
    fn the_plot_holds_a_heading_for_every_section() {
        for lang in [Lang::Ja, Lang::En] {
            let out = render(dt("design"), "design", lang, "docs/x.md");
            for s in &dt("design").sections {
                let name = &s.names(lang)[0];
                assert!(
                    out.contains(&format!("## {name}")),
                    "{lang:?} missing {name}"
                );
                assert!(
                    out.contains(s.question(lang)),
                    "{lang:?} missing the question"
                );
            }
        }
    }

    // プロットは主張を書く枠であって、本文ではない。空欄で出す。
    #[test]
    fn the_claim_line_is_left_blank() {
        assert!(render(dt("decision"), "decision", Lang::Ja, "x.md").contains("主張: \n"));
        assert!(render(dt("decision"), "decision", Lang::En, "x.md").contains("Claim: \n"));
    }

    #[test]
    fn the_plot_carries_the_writing_rules() {
        for lang in [Lang::Ja, Lang::En] {
            let out = render(dt("howto"), "howto", lang, "x.md");
            for line in writing_rules(lang).lines() {
                assert!(out.contains(line), "{lang:?} missing {line}");
            }
        }
    }

    // 読み戻した見出しがそのまま突き合わせに使える形であること。
    #[test]
    fn headings_read_back_match_the_section_names() {
        for lang in [Lang::Ja, Lang::En] {
            let src = render(dt("design"), "design", lang, "x.md");
            let heads: Vec<String> = src
                .lines()
                .filter_map(|l| l.strip_prefix("## "))
                .map(|h| {
                    h.trim()
                        .trim_end_matches("（任意）")
                        .trim_end_matches("(optional)")
                        .trim()
                        .to_string()
                })
                .filter(|h| h != "書き方" && h != "How to write it")
                .collect();
            let want: Vec<String> = dt("design")
                .sections
                .iter()
                .map(|s| s.names(lang)[0].clone())
                .collect();
            assert_eq!(heads, want, "{lang:?}");
        }
    }

    #[test]
    fn the_plot_sits_outside_the_repository_tree() {
        let p = plan_path(Path::new("docs/content/ja/design.md"));
        assert_eq!(
            p,
            Path::new(".suikou/plans/docs/content/ja/design.md").to_path_buf()
        );
    }

    #[test]
    fn an_unknown_doctype_is_an_error() {
        let err = run(Options {
            path: "x.md".to_string(),
            doctype: "no-such-type".to_string(),
            lang: "ja".to_string(),
            force: false,
        });
        assert!(err.is_err());
    }

    // プロットは型を機械が読める形で持つ。文書の側に書き足さずに済む。
    #[test]
    fn the_plot_declares_its_doctype() {
        for lang in [Lang::Ja, Lang::En] {
            let out = render(dt("decision"), "decision", lang, "x.md");
            assert!(out.contains("<!-- suikou:doctype decision -->"), "{out}");
        }
    }

    #[test]
    fn the_declared_doctype_reads_back() {
        let src = render(dt("howto"), "howto", Lang::Ja, "x.md");
        let line = src
            .lines()
            .find_map(|l| l.trim().strip_prefix(DOCTYPE_MARK))
            .unwrap();
        assert_eq!(line.trim().trim_end_matches("-->").trim(), "howto");
    }

    // 印は HTML のコメントにしてある。Markdown として見たときに出てはならない。
    #[test]
    fn the_mark_is_not_read_as_a_heading() {
        let src = render(dt("design"), "design", Lang::Ja, "x.md");
        let doc = suikou_core::markdown::Document::parse(&src);
        assert!(!doc.body().contains("suikou:doctype"));
    }
}
