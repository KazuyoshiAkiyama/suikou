//! 構造の規則が人間の文書でどれだけ発火するかを測る。
//!
//! ```sh
//! cargo run --release --example structure_rate -- corpus/cache/pro_en en
//! ```
//!
//! `suikou check` は規則ごとに示す位置を `MAX_POSITIONS_PER_RULE` 件で打ち切るため、
//! CLI の出力からは本当の件数を数えられない。ここでは規則を直に呼ぶ。
//!
//! 見るのは型に依らない規則だけである。
//! 必須の節と順序は文書の型の宣言を要求するため、型を宣言していないコーパスでは測れない。

use std::path::Path;
use suikou_core::lang::Lang;
use suikou_core::markdown::Document;
use suikou_core::structure::{outline, sentence_count, MAX_SENTENCES_PER_PARAGRAPH};

#[derive(Default)]
struct Totals {
    files: usize,
    paragraphs: usize,
    long_paragraphs: usize,
    files_with_long: usize,
    leaf_sections: usize,
    empty_sections: usize,
    files_with_empty: usize,
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = args.next().unwrap_or_else(|| {
        eprintln!("使い方: structure_rate <ディレクトリ> <ja|en>");
        std::process::exit(2)
    });
    let lang = match args.next().as_deref() {
        Some("en") => Lang::En,
        _ => Lang::Ja,
    };

    let mut t = Totals::default();
    let mut files: Vec<_> = std::fs::read_dir(Path::new(&dir))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    files.sort();

    for path in &files {
        let src = std::fs::read_to_string(path)?;
        let doc = Document::parse(&src);
        t.files += 1;

        let mut long_here = 0;
        for (_, text) in doc.paragraphs() {
            t.paragraphs += 1;
            if sentence_count(&text, lang) > MAX_SENTENCES_PER_PARAGRAPH {
                long_here += 1;
            }
        }
        t.long_paragraphs += long_here;
        t.files_with_long += usize::from(long_here > 0);

        let mut empty_here = 0;
        for o in outline(&doc) {
            if o.has_child {
                continue;
            }
            t.leaf_sections += 1;
            if !o.has_body {
                empty_here += 1;
            }
        }
        t.empty_sections += empty_here;
        t.files_with_empty += usize::from(empty_here > 0);
    }

    let pct = |n: usize, d: usize| {
        if d == 0 {
            0.0
        } else {
            n as f64 * 100.0 / d as f64
        }
    };
    println!("対象 {} ファイル（{dir}、{lang:?}）", t.files);
    println!(
        "段落 {} 件のうち {} 件が {} 文を超える（{:.2}%）",
        t.paragraphs,
        t.long_paragraphs,
        MAX_SENTENCES_PER_PARAGRAPH,
        pct(t.long_paragraphs, t.paragraphs)
    );
    println!(
        "  1件以上を含むファイルは {} 件（{:.1}%）",
        t.files_with_long,
        pct(t.files_with_long, t.files)
    );
    println!(
        "末端の節 {} 件のうち {} 件が本文を持たない（{:.2}%）",
        t.leaf_sections,
        t.empty_sections,
        pct(t.empty_sections, t.leaf_sections)
    );
    println!(
        "  1件以上を含むファイルは {} 件（{:.1}%）",
        t.files_with_empty,
        pct(t.files_with_empty, t.files)
    );
    Ok(())
}
