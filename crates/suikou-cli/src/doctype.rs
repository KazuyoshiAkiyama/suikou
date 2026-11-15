//! 型を選ぶための問いを出す。
//!
//! ツールは問わない。木を出力して、呼び出す側が読者に問う。
//! CI とエージェントから呼ばれるため、対話的な入力を置かない。
//! `brief` が指示を出力して書く側に委ねるのと同じ形である。
//!
//! 木は `structure.toml` の `[[select]]` にある。
//! 型を足したら木も伸ばす。試験が両者の同期を確かめている。

use anyhow::Result;
use suikou_core::lang::Lang;
use suikou_core::structure::{doctypes, questions};

pub struct Options {
    pub lang: String,
    /// 問いを出さず、型の一覧だけを出す。
    pub list: bool,
}

pub fn run(opts: Options) -> Result<()> {
    let lang = match opts.lang.as_str() {
        "en" => Lang::En,
        _ => Lang::Ja,
    };
    print!("{}", render(lang, opts.list));
    Ok(())
}

fn render(lang: Lang, list_only: bool) -> String {
    let mut out = String::new();
    if !list_only {
        out.push_str(&heading(lang));
        for q in questions() {
            out.push_str(&format!("\n## {}\n\n", q.text(lang)));
            for c in &q.options {
                let dest = match (&c.next, &c.doctype) {
                    (Some(n), _) => next_label(lang, n),
                    (_, Some(d)) => doctype_label(lang, d),
                    _ => String::new(),
                };
                // 区切りも言語に合わせる。
                let sep = match lang {
                    Lang::Ja => "…",
                    Lang::En => "—",
                };
                out.push_str(&format!("- {} {sep} {dest}\n", c.text(lang)));
            }
        }
        out.push('\n');
    }
    out.push_str(&list_heading(lang));
    let mut names: Vec<&String> = doctypes().keys().collect();
    names.sort();
    for name in names {
        let dt = &doctypes()[name];
        let sep = match lang {
            Lang::Ja => "…",
            Lang::En => "—",
        };
        out.push_str(&format!("- `{name}` {sep} {}\n", dt.description(lang)));
    }
    out.push_str(&footer(lang));
    out
}

fn heading(lang: Lang) -> String {
    match lang {
        Lang::Ja => "# 文書の型を選ぶ\n\n\
                     読者に次の問いを当てて、型を決める。最大2問で決まる。\n\
                     決まったら `suikou plot <path> --doctype <型>` で型枠を出す。\n"
            .to_string(),
        Lang::En => "# Choosing a doctype\n\n\
                     Put the questions below to the reader; two at most settle it.\n\
                     Then run `suikou plot <path> --doctype <type>` for the skeleton.\n"
            .to_string(),
    }
}

fn next_label(lang: Lang, id: &str) -> String {
    let q = questions().iter().find(|q| q.id == id);
    let text = q.map(|q| q.text(lang)).unwrap_or(id);
    match lang {
        Lang::Ja => format!("次の問いへ: {text}"),
        Lang::En => format!("next question: {text}"),
    }
}

fn doctype_label(lang: Lang, name: &str) -> String {
    let desc = doctypes()
        .get(name)
        .map(|d| d.description(lang))
        .unwrap_or("");
    // 括弧は言語に合わせる。英語の出力に全角が混ざると読みにくい。
    match lang {
        Lang::Ja => format!("`{name}`（{desc}）"),
        Lang::En => format!("`{name}` ({desc})"),
    }
}

fn list_heading(lang: Lang) -> String {
    match lang {
        Lang::Ja => "# 型の一覧\n\n".to_string(),
        Lang::En => "# The doctypes\n\n".to_string(),
    }
}

fn footer(lang: Lang) -> String {
    match lang {
        Lang::Ja => {
            "\n型ごとの必須の節と、節が答える問いは `suikou brief --doctype <型>` で出る。\n"
                .to_string()
        }
        Lang::En => "\n`suikou brief --doctype <type>` gives each type's required sections \
                     and the question each answers.\n"
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tree_and_the_list_both_appear() {
        let out = render(Lang::Ja, false);
        assert!(out.contains("文書の型を選ぶ"));
        assert!(out.contains("型の一覧"));
        for name in doctypes().keys() {
            assert!(out.contains(&format!("`{name}`")), "{name} が出ていない");
        }
    }

    #[test]
    fn list_only_leaves_the_questions_out() {
        let out = render(Lang::Ja, true);
        assert!(!out.contains("文書の型を選ぶ"));
        assert!(out.contains("型の一覧"));
    }

    // 選択肢の行き先が、読める言葉で出ることを確かめる。
    // id がそのまま出ると、読者には意味が分からない。
    #[test]
    fn a_choice_shows_where_it_leads() {
        let out = render(Lang::Ja, false);
        assert!(out.contains("次の問いへ:"));
        assert!(!out.contains(" proposal\n"), "問いの id が露出している");
    }

    #[test]
    fn english_renders_in_english() {
        let out = render(Lang::En, false);
        assert!(out.contains("Choosing a doctype"));
        assert!(out.contains("next question:"));
        assert!(!out.contains("型"));
        // 全角の括弧と中黒が混ざると読みにくい。
        for c in ['（', '）', '…', '。'] {
            assert!(!out.contains(c), "英語の出力に {c} が混ざっている");
        }
    }
}
