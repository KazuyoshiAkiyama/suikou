//! `suikou check` の実装。
//!
//! 一度の実行で全ての層の指摘を出す。層ごとに lint と修正を往復させないための構成である。
//! 文書指標は位置を持たないため方針として、局所指摘は位置つきで出す。

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use suikou_core::check::evaluate_with_glossary;
use suikou_core::lang::{detect_lang, Lang};
use suikou_core::markdown::Document;
use suikou_core::profile::Profile;
use suikou_core::report::{Report, Severity};

use crate::morphology;

/// 1つの文書を解析してレポートを作る。
///
/// `run` のループと `mcp` サブコマンドの `check` ツールから共有して使う。
/// 判定のロジックを CLI と MCP で二重に書かないための関数である。
#[derive(serde::Deserialize, Default)]
struct AllowFile {
    #[serde(default)]
    words: Vec<String>,
}

/// 文体の指摘から外す語を集める。
///
/// `.suikou/terms.toml` は `suikou terms` が作る分野の語である。
/// `.suikou/register-allow.toml` は、参照コーパスに無いが正しいと
/// プロジェクトが判断した語を書く。参照コーパスの分野が偏っている分を補う。
fn glossary() -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    if let Ok(src) = std::fs::read_to_string(".suikou/terms.toml") {
        if let Ok(t) = toml::from_str::<suikou_core::terms::Terms>(&src) {
            out.extend(t.terms.into_iter().map(|x| x.word));
        }
    }
    if let Ok(src) = std::fs::read_to_string(".suikou/register-allow.toml") {
        if let Ok(a) = toml::from_str::<AllowFile>(&src) {
            out.extend(a.words);
        }
    }
    out
}

/// `.suikou/structure.toml` の中身。
///
/// パスの表と、ほかの道具が付けた宣言の読み替えを持つ。
#[derive(serde::Deserialize, Default)]
struct StructureCfg {
    #[serde(default)]
    paths: std::collections::HashMap<String, String>,
    #[serde(default)]
    front_matter: FrontMatterCfg,
}

/// フロントマターの別の欄を型の宣言として読む設定。
///
/// Hugo の `content_type` のように、型を既に宣言している文書がある。
/// その場合に `doctype:` を書き足させると、文書を触らずに済む道を閉ざす。
#[derive(serde::Deserialize, Default)]
struct FrontMatterCfg {
    #[serde(default)]
    field: String,
    #[serde(default)]
    map: std::collections::HashMap<String, String>,
}

fn structure_cfg() -> StructureCfg {
    std::fs::read_to_string(".suikou/structure.toml")
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn front_matter(src: &str) -> Option<&str> {
    src.strip_prefix("---\n")
        .and_then(|r| r.split_once("\n---"))
        .map(|x| x.0)
}

fn field_value<'a>(fm: &'a str, field: &str) -> Option<&'a str> {
    fm.lines().find_map(|l| {
        l.strip_prefix(field)?
            .strip_prefix(':')
            .map(|v| v.trim().trim_matches('"'))
    })
}

/// 文書の型を決める。近いところから順に見る。
///
/// フロントマターの `doctype:`、ほかの道具が付けた宣言の読み替え、
/// `.suikou/structure.toml` のパスの表、そしてプロットが宣言した型の順とする。
///
/// プロットは `--doctype` を与えて作るため、型を既に持っている。
/// 文書の側に書き足さなくても、プロットがあれば型が決まる。
///
/// どこにも無ければ型に依る規則は当てない。
/// 型を宣言していない文書に、後から規則が増えて落ちることがないようにするためである。
fn resolve_doctype(src: &str, path: &std::path::Path) -> Option<String> {
    declared_doctype(&structure_cfg(), src, path).or_else(|| crate::plot::plot_doctype(path))
}

/// 文書と設定だけから型を決める部分。プロットを読む前の段階にあたる。
fn declared_doctype(cfg: &StructureCfg, src: &str, path: &std::path::Path) -> Option<String> {
    if let Some(fm) = front_matter(src) {
        if let Some(v) = field_value(fm, "doctype") {
            return Some(v.to_string());
        }
        if !cfg.front_matter.field.is_empty() {
            if let Some(v) = field_value(fm, &cfg.front_matter.field) {
                // 表に無い値は型を決めない。知らない値を黙って落とすほうが、
                // こちらの見立てで割り当てるより安全である。
                if let Some(name) = cfg.front_matter.map.get(v) {
                    return Some(name.clone());
                }
            }
        }
    }
    doctype_from_paths(cfg, path)
}

fn doctype_from_paths(cfg: &StructureCfg, path: &std::path::Path) -> Option<String> {
    let p = path.to_string_lossy();
    // 長い接頭辞を先に見る。細かい指定が大まかな指定に勝つ。
    let mut keys: Vec<&String> = cfg.paths.keys().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    keys.into_iter()
        .find(|k| p.starts_with(k.as_str()))
        .map(|k| cfg.paths[k].clone())
}

pub(crate) fn analyze(
    src: &str,
    lang_spec: &str,
    profile: &Profile,
    path: Option<&std::path::Path>,
) -> Result<(Lang, Report)> {
    let doc = Document::parse(src);
    let lang = resolve_lang(lang_spec, src)?;
    let morph = morphology::load(lang)?;
    let mut report = evaluate_with_glossary(&doc, lang, morph.as_ref(), profile, &glossary());
    // 構造の規則は言語を問わない。型を宣言していない文書にも段落と節の規則は当てる。
    report
        .local
        .extend(suikou_core::structure::check_universal(&doc, lang));
    if let Some(path) = path {
        if let Some(name) = resolve_doctype(src, path) {
            // 知らない名前を黙って読み飛ばすと、綴りを誤ったときに検査が消える。
            // 較正で同じ失敗をした経緯が D-22 にある。ここでは落とす。
            let dt = suikou_core::structure::doctypes()
                .get(&name)
                .ok_or_else(|| {
                    let mut names: Vec<&str> = suikou_core::structure::doctypes()
                        .keys()
                        .map(String::as_str)
                        .collect();
                    names.sort_unstable();
                    anyhow::anyhow!(
                        "{} が宣言した型 {name} を知らない。使えるのは {}",
                        path.display(),
                        names.join("、")
                    )
                })?;
            report
                .local
                .extend(suikou_core::structure::check_doctype(&doc, lang, dt));
        }
        if let Some(heads) = crate::plot::plot_headings(path) {
            report
                .local
                .extend(suikou_core::structure::check_plot(&doc, &heads));
        }
    }
    Ok((lang, report))
}

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
        let (_lang, mut report) = analyze(&src, &opts.lang, &profile, Some(path))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(src: &str) -> StructureCfg {
        toml::from_str(src).unwrap()
    }

    #[test]
    fn the_front_matter_doctype_wins() {
        let c = cfg("[paths]\n\"docs/\" = \"reference\"\n");
        let src = "---\ntitle: \"x\"\ndoctype: \"design\"\n---\n\n本文。\n";
        let got = declared_doctype(&c, src, Path::new("docs/x.md"));
        assert_eq!(got.as_deref(), Some("design"));
    }

    #[test]
    fn the_path_table_applies_when_the_front_matter_is_silent() {
        let c = cfg("[paths]\n\"docs/\" = \"reference\"\n\"docs/how/\" = \"howto\"\n");
        // 長い接頭辞が勝つ。
        let got = declared_doctype(&c, "本文。\n", Path::new("docs/how/x.md"));
        assert_eq!(got.as_deref(), Some("howto"));
    }

    // ほかの道具が付けた宣言を読み替える。文書を書き換えずに型を当てられる。
    #[test]
    fn a_foreign_front_matter_field_is_mapped() {
        let c = cfg("[front_matter]\nfield = \"content_type\"\n\n\
             [front_matter.map]\ntask = \"howto\"\n");
        let src = "---\ntitle: \"x\"\ncontent_type: task\n---\n\nBody.\n";
        let got = declared_doctype(&c, src, Path::new("x.md"));
        assert_eq!(got.as_deref(), Some("howto"));
    }

    // 表に無い値は型を決めない。こちらの見立てで割り当てないためである。
    #[test]
    fn an_unmapped_value_declares_nothing() {
        let c = cfg("[front_matter]\nfield = \"content_type\"\n\n\
             [front_matter.map]\ntask = \"howto\"\n");
        let src = "---\ncontent_type: feature_gate\n---\n\nBody.\n";
        assert_eq!(declared_doctype(&c, src, Path::new("x.md")), None);
    }

    #[test]
    fn our_own_field_beats_the_foreign_one() {
        let c = cfg("[front_matter]\nfield = \"content_type\"\n\n\
             [front_matter.map]\ntask = \"howto\"\n");
        let src = "---\ncontent_type: task\ndoctype: \"reference\"\n---\n\nBody.\n";
        let got = declared_doctype(&c, src, Path::new("x.md"));
        assert_eq!(got.as_deref(), Some("reference"));
    }

    #[test]
    fn a_document_declaring_nothing_gets_no_doctype() {
        assert_eq!(
            declared_doctype(&StructureCfg::default(), "本文。\n", Path::new("x.md")),
            None
        );
    }
}
