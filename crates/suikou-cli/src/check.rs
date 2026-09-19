//! `suikou check` の実装。
//!
//! 一度の実行で全ての層の指摘を出す。層ごとに lint と修正を往復させないための構成である。
//! 文書指標は位置を持たないため方針として、局所指摘は位置つきで出す。

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use suikou_core::check::evaluate;
use suikou_core::lang::{detect_lang, Lang};
use suikou_core::markdown::Document;
use suikou_core::profile::Profile;
use suikou_core::report::{Report, Severity};

use crate::morphology;

pub struct Options {
    pub paths: Vec<String>,
    pub format: String,
    pub lang: String,
    pub profile: String,
    pub budget: Option<usize>,
    pub quiet: bool,
}

/// 終了コード。hooks から差し戻すかどうかの判断に使う。
pub const EXIT_OK: i32 = 0;
pub const EXIT_ERROR: i32 = 1;

// brief からも使うため crate 内に公開する。プロファイルの読み込み方を
// 二重に書かないための共有である。
pub(crate) fn load_profile(name: &str) -> Result<Profile> {
    if let Some(p) = Profile::builtin(name) {
        return Ok(p);
    }
    let path = Path::new(name);
    if !path.exists() {
        bail!("プロファイル {name} が見つからない。組み込みは oss と service");
    }
    let src = std::fs::read_to_string(path).with_context(|| format!("{name} を読めない"))?;
    toml::from_str(&src).with_context(|| format!("{name} を解釈できない"))
}

// `baseline` と `terms` も同じ集め方と言語判定を要るため、crate 内に公開する。
// 重複させると、パスの集め方がサブコマンドごとに黙って食い違う経路ができる。
pub(crate) fn resolve_lang(spec: &str, text: &str) -> Result<Lang> {
    match spec {
        "auto" => Ok(detect_lang(text)),
        "ja" => Ok(Lang::Ja),
        "en" => Ok(Lang::En),
        other => bail!("--lang は auto、ja、en のいずれかとする。与えられた値は {other}"),
    }
}

pub(crate) fn collect(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for p in paths {
        let path = PathBuf::from(p);
        if path.is_dir() {
            collect_dir(&path, &mut out)?;
        } else {
            out.push(path);
        }
    }
    if out.is_empty() {
        bail!("対象のファイルがない");
    }
    out.sort();
    Ok(out)
}

fn collect_dir(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("{} を読めない", dir.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect_dir(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}

fn text_format(path: &Path, report: &Report) -> String {
    let mut out = String::new();
    for m in &report.document {
        out.push_str(&format!(
            "{}: {} {}\n",
            path.display(),
            m.metric,
            m.guidance
        ));
    }
    for f in &report.local {
        for p in &f.positions {
            out.push_str(&format!(
                "{}:{}:{}: {} [{}]\n",
                path.display(),
                p.line,
                p.column,
                f.message,
                f.rule_id
            ));
        }
    }
    out
}

pub fn run(opts: Options) -> Result<i32> {
    let profile = load_profile(&opts.profile)?;
    let files = collect(&opts.paths)?;
    let many = files.len() > 1;

    let mut worst: Option<Severity> = None;
    let mut json = serde_json::Map::new();
    let mut rendered = String::new();

    for path in &files {
        let src = std::fs::read_to_string(path)
            .with_context(|| format!("{} を読めない", path.display()))?;
        let doc = Document::parse(&src);
        let lang = resolve_lang(&opts.lang, &src)?;
        let morph = morphology::load(lang)?;
        let mut report = evaluate(&doc, lang, morph.as_ref(), &profile);
        if let Some(b) = opts.budget {
            report.trim_to(b / files.len().max(1));
        }

        if let Some(s) = report.worst_severity() {
            worst = Some(match worst {
                None => s,
                Some(Severity::Error) => Severity::Error,
                Some(_) if s == Severity::Error => Severity::Error,
                Some(Severity::Info) => s,
                Some(w) => w,
            });
        }

        match opts.format.as_str() {
            "json" => {
                json.insert(
                    path.display().to_string(),
                    serde_json::to_value(&report).context("JSON にできない")?,
                );
            }
            "text" => rendered.push_str(&text_format(path, &report)),
            "md" => {
                let body = report.to_markdown();
                if body.is_empty() {
                    continue;
                }
                if many {
                    rendered.push_str(&format!("# {}\n\n", path.display()));
                }
                rendered.push_str(&body);
            }
            other => bail!("--format は md、json、text のいずれかとする。与えられた値は {other}"),
        }
    }

    if !opts.quiet {
        if opts.format == "json" {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            print!("{rendered}");
        }
    }

    // error があるときだけ 1 を返す。warning と info では差し戻さない。
    Ok(match worst {
        Some(Severity::Error) => EXIT_ERROR,
        _ => EXIT_OK,
    })
}
