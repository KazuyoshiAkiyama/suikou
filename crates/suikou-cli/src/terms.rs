//! `suikou terms` の実装。
//!
//! 文書群から分野で確立した語を取り出し、`.suikou/terms.toml` に書く。
//! 抽出そのもの（複合名詞のまとめ方、英語の手掛かり、頻度の下限）は
//! `suikou-core` の `terms.rs` に置いてある。単体テストしやすくするためと、
//! `check` と同じく出力の形によらず抽出のロジックを保つためである。
//! ここでは、ファイルを集めて言語ごとに振り分け、TOML を書き出すだけを行う。

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;
use suikou_core::lang::Lang;
use suikou_core::markdown::Document;
use suikou_core::terms::{self, Terms};
use suikou_core::tokenizer::Morphology;

use crate::check::{collect, resolve_lang};
use crate::morphology;

pub struct Options {
    pub paths: Vec<String>,
    pub out: String,
    pub lang: String,
    pub min_count: usize,
}

pub fn run(opts: Options) -> Result<()> {
    let files = collect(&opts.paths)?;

    let mut ja_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut en_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut ja_morph: Option<Box<dyn Morphology>> = None;

    for path in &files {
        let src = std::fs::read_to_string(path)
            .with_context(|| format!("{} を読めない", path.display()))?;
        let doc = Document::parse(&src);
        let lang = resolve_lang(&opts.lang, &src)?;
        match lang {
            Lang::Ja => {
                if ja_morph.is_none() {
                    ja_morph = Some(morphology::load(Lang::Ja)?);
                }
                let morph = ja_morph.as_ref().unwrap().as_ref();
                terms::count_ja(&doc.body(), morph, &mut ja_counts);
            }
            Lang::En => terms::count_en(&doc, &mut en_counts),
        }
    }

    let mut list = terms::build_terms(&ja_counts, Lang::Ja, opts.min_count);
    list.extend(terms::build_terms(&en_counts, Lang::En, opts.min_count));
    let out = Terms { terms: list }.sorted();

    let body = toml::to_string_pretty(&out).context("TOML にできない")?;
    let out_path = Path::new(&opts.out);
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("{} を作れない", parent.display()))?;
        }
    }
    std::fs::write(out_path, body).with_context(|| format!("{} に書けない", out_path.display()))?;

    println!(
        "{} 語を {} に書いた（下限 {} 回、対象 {} ファイル）",
        out.terms.len(),
        out_path.display(),
        opts.min_count,
        files.len()
    );
    Ok(())
}
