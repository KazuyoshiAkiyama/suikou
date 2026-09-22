//! `suikou brief` の実装。
//!
//! 生成の前にプロンプトへ貼る制約ブロックを作る。そのまま貼れる形で標準出力に返す。
//!
//! 中身は書式層と保守性層だけに絞ってある。MATTR や連用中止の比といった語彙と
//! 文の組み立ての指標はここに出さない。書式を抑える実験で、書式は指示で完全に
//! 消えるが語彙と組み立ては指示では変わらないと分かっているためである。
//! 詳しい根拠は `docs/content/ja/design.md` の「brief が出す縛り」を見よ。
//!
//! 保守性の規則も一様には書かない。M1（導入文）、M5（項目のそろい）、
//! M7（「など」終わり）は1文で足りる。指示がなくても起きない M2、M3、M4 は
//! 入れない。M6（時点依存語）だけは簡潔な指示では守られないと実測で分かって
//! いるため、水準によらず禁じる語をすべて列挙する。

use anyhow::{bail, Context, Result};
use suikou_core::lang::Lang;
use suikou_core::rules::{TIME_DEPENDENT_EN, TIME_DEPENDENT_JA};
use suikou_core::structure::{doctypes, writing_rules, DocType};

use crate::check::load_profile;

pub struct Options {
    pub profile: String,
    pub lang: String,
    pub detail: String,
    /// 文書の型。与えると、禁止の一覧の前に節の型枠を出す。
    pub doctype: Option<String>,
}

/// `--detail` の水準。
///
/// 短い指示には出力を縮める働きがあるため、既定は balanced とする。
/// concise はさらに切り詰め、detailed は規則ごとに根拠まで書く。
/// どの水準でも M6 の禁止語一覧だけは省略しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Detail {
    Concise,
    Balanced,
    Detailed,
}

impl Detail {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "concise" => Ok(Detail::Concise),
            "balanced" => Ok(Detail::Balanced),
            "detailed" => Ok(Detail::Detailed),
            other => bail!(
                "--detail は concise、balanced、detailed のいずれかとする。与えられた値は {other}"
            ),
        }
    }
}

/// brief は入力ファイルを取らないため、`auto` では文面から言語を判定できない。
/// 既定を日本語とする。
fn resolve_lang(spec: &str) -> Result<Lang> {
    match spec {
        "auto" | "ja" => Ok(Lang::Ja),
        "en" => Ok(Lang::En),
        other => bail!("--lang は auto、ja、en のいずれかとする。与えられた値は {other}"),
    }
}

pub fn run(opts: Options) -> Result<String> {
    // brief の中身はプロファイルの閾値に左右されないが、存在しない名前は
    // check と同じく早く落とす。プロファイルに brief 専用の設定は足していない。
    // 理由は D-21 にある。
    load_profile(&opts.profile)?;
    let lang = resolve_lang(&opts.lang)?;
    let detail = Detail::parse(&opts.detail)?;
    let dt = match &opts.doctype {
        Some(name) => Some(lookup_doctype(name)?),
        None => None,
    };
    let mut out = String::new();
    if let Some(dt) = dt {
        out.push_str(&render_structure(dt, lang));
        out.push('\n');
    }
    out.push_str(&render(lang, detail));
    Ok(out)
}

fn lookup_doctype(name: &str) -> Result<&'static DocType> {
    doctypes().get(name).with_context(|| {
        let mut names: Vec<&str> = doctypes().keys().map(String::as_str).collect();
        names.sort_unstable();
        format!("型 {name} を知らない。使えるのは {}", names.join("、"))
    })
}

/// 節の型枠と書き方の指針。禁止の一覧より前に置く。
///
/// 禁止だけを渡すと、避けるべき形は分かっても書くべき形が決まらない。
/// 節ごとに、その節が答える問いと何を書くかを示す。
/// `suikou plot` が出すものと同じ内容であり、指針は一か所から取る。
fn render_structure(dt: &DocType, lang: Lang) -> String {
    let mut out = String::new();
    match lang {
        Lang::Ja => {
            out.push_str(&format!(
                "# 文書の組み立て\n\n型: {}\n\n",
                dt.description(lang)
            ));
            out.push_str("次の節をこの順に置く。節ごとに、その節が答える問いを示す。\n\n");
            for s in &dt.sections {
                let name = s
                    .names(lang)
                    .first()
                    .cloned()
                    .unwrap_or_else(|| s.id.clone());
                let mark = if s.required { "" } else { "（任意）" };
                out.push_str(&format!(
                    "- {name}{mark}… {} / {}\n",
                    s.question(lang),
                    s.writes(lang)
                ));
            }
            out.push_str("\n# 書き方\n\n");
        }
        Lang::En => {
            out.push_str(&format!(
                "# Structure\n\nDoctype: {}\n\n",
                dt.description(lang)
            ));
            out.push_str(
                "Use the sections below, in this order. Each one names the question it \
                 answers.\n\n",
            );
            for s in &dt.sections {
                let name = s
                    .names(lang)
                    .first()
                    .cloned()
                    .unwrap_or_else(|| s.id.clone());
                let mark = if s.required { "" } else { " (optional)" };
                out.push_str(&format!(
                    "- {name}{mark} — {} / {}\n",
                    s.question(lang),
                    s.writes(lang)
                ));
            }
            out.push_str("\n# How to write it\n\n");
        }
    }
    out.push_str(&writing_rules(lang));
    out
}

fn render(lang: Lang, detail: Detail) -> String {
    match lang {
        Lang::Ja => render_ja(detail),
        Lang::En => render_en(detail),
    }
}

fn render_ja(detail: Detail) -> String {
    let words = TIME_DEPENDENT_JA.join("、");
    let mut out = String::from("# 生成するときに守ること\n\n");
    match detail {
        Detail::Concise => {
            out.push_str("- 箇条書きの導入文は完全な文で書き、項目と文法的につなげない。\n");
            out.push_str("- 同じ箇条書きの項目は末尾の形をそろえる。\n");
            out.push_str("- 列挙を「など」「等」で終えない。\n");
            out.push_str(&format!("- 次の語を使わない。{words}\n"));
        }
        Detail::Balanced => {
            out.push_str(
                "- 箇条書きの導入文は完全な文で書く。項目と文法的につながる形にしない。\n",
            );
            out.push_str(
                "- 同じ箇条書きの中で項目の末尾の形をそろえる。文で終える項目と体言で終える項目を混ぜない。\n",
            );
            out.push_str(
                "- 列挙を「など」「等」で終えない。網羅していないことは導入文の側で示す。\n",
            );
            out.push_str("- 次の語は時点に依存し、文書を古びさせる。使わない。\n");
            out.push_str(&format!("  {words}\n"));
        }
        Detail::Detailed => {
            out.push_str("## 箇条書きの導入文\n\n");
            out.push_str(
                "箇条書きの前に置く文は、それだけで完結した文にする。助詞や連用形で終えて\
                 項目とつながる形にすると、項目を増減したときに導入文だけが文法的に壊れ、\
                 直し忘れが起きる。体言止めでコロンを置く形は認めるが、助詞や連用形で\
                 終える形は避ける。\n\n",
            );
            out.push_str("## 項目の末尾のそろえ\n\n");
            out.push_str(
                "同じ箇条書きの中で、項目の文法的な終わり方をそろえる。文で終える項目と\
                 体言で終える項目が混在すると、読み手が箇条書き全体の構造を読み違える。\n\n",
            );
            out.push_str("## 「など」終わりの禁止\n\n");
            out.push_str(
                "列挙の末尾を「など」「等」で締めない。網羅していないことを示す必要が\
                 あるときは、導入文の側で明示する。\n\n",
            );
            out.push_str("## 時点に依存する語の禁止\n\n");
            out.push_str(
                "次の語を使わない。使うと、文書が時間の経過とともに実情と合わなくなる。\n\n",
            );
            out.push_str(&format!("{words}\n"));
        }
    }
    out
}

fn render_en(detail: Detail) -> String {
    let words = TIME_DEPENDENT_EN.join(", ");
    let mut out = String::from("# Constraints for this generation\n\n");
    match detail {
        Detail::Concise => {
            out.push_str(
                "- Introduce every list with a complete sentence; do not let it connect \
                 grammatically into the items.\n",
            );
            out.push_str("- Keep list items parallel in form.\n");
            out.push_str("- Do not end an enumeration with \"etc.\" or \"and so on\".\n");
            out.push_str(&format!("- Do not use: {words}\n"));
        }
        Detail::Balanced => {
            out.push_str(
                "- Introduce every list with a complete sentence; a lead-in that grammatically \
                 continues into the items breaks whenever an item is added or removed.\n",
            );
            out.push_str(
                "- Keep every item in a list parallel in form; do not mix sentence-ending items \
                 with noun-phrase items in the same list.\n",
            );
            out.push_str(
                "- Do not end an enumeration with \"etc.\" or \"and so on\"; state \
                 incompleteness in the lead-in instead.\n",
            );
            out.push_str(
                "- The words below are time-dependent and make the document go stale. Do not \
                 use them.\n",
            );
            out.push_str(&format!("  {words}\n"));
        }
        Detail::Detailed => {
            out.push_str("## The list lead-in\n\n");
            out.push_str(
                "Introduce every bulleted or numbered list with a sentence that stands on its \
                 own. A lead-in that ends with a preposition or a participle and grammatically \
                 continues into the items breaks, on its own, whenever an item is added or \
                 removed. A noun phrase followed by a colon is fine; a fragment that depends on \
                 the first item is not.\n\n",
            );
            out.push_str("## Parallel items\n\n");
            out.push_str(
                "Keep the grammatical ending of every item in one list the same. Mixing \
                 sentence-ending items with noun-phrase items in a single list makes the \
                 structure of the list hard to read at a glance.\n\n",
            );
            out.push_str("## No trailing \"etc.\"\n\n");
            out.push_str(
                "Do not close an enumeration with \"etc.\" or \"and so on\". If the list is not \
                 exhaustive, say so in the lead-in instead.\n\n",
            );
            out.push_str("## Time-dependent words\n\n");
            out.push_str(
                "Do not use the words below. Each one anchors the sentence to the moment it \
                 was written, so the document goes stale as soon as that moment passes.\n\n",
            );
            out.push_str(&format!("{words}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ja_contains_every_forbidden_word() {
        let out = render_ja(Detail::Balanced);
        for w in TIME_DEPENDENT_JA {
            assert!(out.contains(w), "missing {w} in:\n{out}");
        }
    }

    #[test]
    fn en_contains_every_forbidden_word() {
        let out = render_en(Detail::Balanced);
        for w in TIME_DEPENDENT_EN {
            assert!(out.contains(w), "missing {w} in:\n{out}");
        }
    }

    #[test]
    fn forbidden_words_present_at_every_detail_level() {
        for d in [Detail::Concise, Detail::Balanced, Detail::Detailed] {
            let ja = render_ja(d);
            let en = render_en(d);
            for w in TIME_DEPENDENT_JA {
                assert!(ja.contains(w), "{d:?} ja missing {w}");
            }
            for w in TIME_DEPENDENT_EN {
                assert!(en.contains(w), "{d:?} en missing {w}");
            }
        }
    }

    // 語彙・文構造系の指標は指示で変わらないと実測で分かっている。
    // brief が誤ってこれらを混入させていないことを確かめる。
    #[test]
    fn does_not_mention_vocabulary_or_structure_metrics() {
        let forbidden_ja = ["連用中止", "漢語率", "MATTR", "変動係数", "テ形接続"];
        let forbidden_en = ["MATTR", "renyo", "coefficient of variation", "kango"];
        for d in [Detail::Concise, Detail::Balanced, Detail::Detailed] {
            let ja = render_ja(d);
            let en = render_en(d);
            for w in forbidden_ja {
                assert!(!ja.contains(w), "{d:?} ja unexpectedly mentions {w}");
            }
            for w in forbidden_en {
                assert!(
                    !en.to_lowercase().contains(&w.to_lowercase()),
                    "{d:?} en unexpectedly mentions {w}"
                );
            }
        }
    }

    #[test]
    fn detail_level_changes_length() {
        let ja_concise = render_ja(Detail::Concise).len();
        let ja_balanced = render_ja(Detail::Balanced).len();
        let ja_detailed = render_ja(Detail::Detailed).len();
        assert!(ja_concise < ja_balanced, "{ja_concise} < {ja_balanced}");
        assert!(ja_balanced < ja_detailed, "{ja_balanced} < {ja_detailed}");

        let en_concise = render_en(Detail::Concise).len();
        let en_balanced = render_en(Detail::Balanced).len();
        let en_detailed = render_en(Detail::Detailed).len();
        assert!(en_concise < en_balanced, "{en_concise} < {en_balanced}");
        assert!(en_balanced < en_detailed, "{en_balanced} < {en_detailed}");
    }

    #[test]
    fn m2_m3_m4_are_left_out() {
        // M2（項目数）、M3（見出しの連番）、M4（手動番号）は指示がなくても
        // 起きないため brief に入れない。規則名や説明が出ないことで確かめる。
        for d in [Detail::Concise, Detail::Balanced, Detail::Detailed] {
            let ja = render_ja(d);
            assert!(!ja.contains("項目数"));
            assert!(!ja.contains("連番"));
            assert!(!ja.contains("手動で番号"));
        }
    }

    #[test]
    fn auto_lang_defaults_to_japanese() {
        assert_eq!(resolve_lang("auto").unwrap(), Lang::Ja);
    }

    #[test]
    fn unknown_lang_is_an_error() {
        assert!(resolve_lang("fr").is_err());
    }

    #[test]
    fn unknown_detail_is_an_error() {
        assert!(Detail::parse("verbose").is_err());
    }

    #[test]
    fn run_with_builtin_profile_succeeds() {
        let out = run(Options {
            profile: "oss".to_string(),
            lang: "en".to_string(),
            detail: "balanced".to_string(),
            doctype: None,
        })
        .unwrap();
        assert!(out.contains("currently"));
    }

    #[test]
    fn run_with_unknown_profile_fails() {
        let err = run(Options {
            profile: "no-such-profile".to_string(),
            lang: "ja".to_string(),
            detail: "balanced".to_string(),
            doctype: None,
        });
        assert!(err.is_err());
    }

    #[test]
    fn the_doctype_skeleton_precedes_the_constraints() {
        for (lang, spec, head) in [
            (Lang::Ja, "ja", "# 文書の組み立て"),
            (Lang::En, "en", "# Structure"),
        ] {
            let out = run(Options {
                profile: "oss".to_string(),
                lang: spec.to_string(),
                detail: "concise".to_string(),
                doctype: Some("design".to_string()),
            })
            .unwrap();
            // 書くべき形を先に示し、禁止はその後に置く。
            assert!(out.starts_with(head), "{lang:?}: {out}");
            for s in &doctypes()["design"].sections {
                assert!(
                    out.contains(&s.names(lang)[0].to_string()),
                    "{lang:?} section"
                );
                assert!(out.contains(s.question(lang)), "{lang:?} question");
            }
            for line in writing_rules(lang).lines() {
                assert!(out.contains(line), "{lang:?} rule {line}");
            }
        }
    }

    #[test]
    fn without_a_doctype_the_output_is_unchanged() {
        let opts = |dt| Options {
            profile: "oss".to_string(),
            lang: "ja".to_string(),
            detail: "balanced".to_string(),
            doctype: dt,
        };
        assert_eq!(run(opts(None)).unwrap(), render_ja(Detail::Balanced));
        assert!(
            run(opts(Some("design".to_string()))).unwrap().len() > run(opts(None)).unwrap().len()
        );
    }

    #[test]
    fn an_unknown_doctype_is_an_error() {
        let err = run(Options {
            profile: "oss".to_string(),
            lang: "ja".to_string(),
            detail: "balanced".to_string(),
            doctype: Some("no-such-type".to_string()),
        });
        assert!(err.is_err());
    }
}
