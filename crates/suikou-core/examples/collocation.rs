//! 名詞と動詞の組を数える。共起が成立しない組を見つけるために使う。
//!
//! ```sh
//! cargo run --release --features lindera-unidic --example collocation -- <ディレクトリ>...
//! ```
//!
//! 候補を推測で挙げず、人間の文書と AI の文書で分布を比べて挙げる。
//! 1つめのディレクトリを人間の側、残りを AI の側として扱う。
//!
//! 部分一致では名詞が取れない。`規範を置く` の `規範` を取るには形態素解析が要る。
//! 既存の `Morphology` トレイト越しに使い、この例でも境を越えない。

use std::collections::HashMap;
use std::path::Path;
use suikou_core::markdown::Document;
use suikou_core::tokenizer::{LinderaMorphology, Morphology};

/// 意味の薄い動詞。これが取る名詞を見る。
/// 名詞の側に意味が寄るため、動詞が当たっていなくても文が成立しているように見える。
const LIGHT: [&str; 12] = [
    "置く",
    "行う",
    "図る",
    "担保する",
    "実施する",
    "持つ",
    "与える",
    "立てる",
    "取る",
    "作る",
    "付ける",
    "掛ける",
];

fn collect(dir: &Path, morph: &dyn Morphology) -> HashMap<String, usize> {
    let mut out: HashMap<String, usize> = HashMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    paths.sort();
    for p in paths {
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        let doc = Document::parse(&src);
        let tokens = morph.tokenize(&doc.body());
        for w in tokens.windows(3) {
            if w[0].pos1 != "名詞" || w[1].surface != "を" || w[2].pos1 != "動詞" {
                continue;
            }
            // 「する」が続く場合は複合動詞として扱う。担保する、実施する。
            let verb = w[2].lemma.as_str();
            let noun = if w[0].lemma.is_empty() {
                &w[0].surface
            } else {
                &w[0].lemma
            };
            if !LIGHT.contains(&verb) {
                continue;
            }
            *out.entry(format!("{noun}を{verb}")).or_default() += 1;
        }
    }
    out
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("使い方: collocation <人間の側> <AIの側>...");
        std::process::exit(2);
    }
    let morph = LinderaMorphology::new()?;
    let human = collect(Path::new(&args[0]), &morph);
    let mut ai: HashMap<String, usize> = HashMap::new();
    for d in &args[1..] {
        for (k, v) in collect(Path::new(d), &morph) {
            *ai.entry(k).or_default() += v;
        }
    }

    let mut only: Vec<(&String, &usize)> =
        ai.iter().filter(|(k, _)| !human.contains_key(*k)).collect();
    only.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    println!("人間の側 {} 組 / AI の側 {} 組", human.len(), ai.len());
    println!("\n人間の側に現れず AI の側に2回以上あるもの:");
    for (k, v) in only.iter().filter(|(_, v)| **v >= 2) {
        println!("  {k}  ×{v}");
    }
    println!("\n両方に現れるもののうち AI の側が多いもの:");
    let mut both: Vec<(&String, usize, usize)> = ai
        .iter()
        .filter_map(|(k, a)| human.get(k).map(|h| (k, *a, *h)))
        .collect();
    both.sort_by_key(|x| std::cmp::Reverse(x.1 * 100 / x.2.max(1)));
    for (k, a, h) in both.iter().take(12) {
        println!("  {k}  AI {a} / 人間 {h}");
    }
    Ok(())
}
