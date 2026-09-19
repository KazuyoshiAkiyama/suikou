//! 辞書の読み込み時間を測る。hooks から高頻度で起動する用途の起動コストを見積もるために使う。
//!
//! ```sh
//! cargo run --release --features lindera-unidic --example dict_load
//! ```
//!
//! daemon モードを用意するかどうかの判断が、この値にかかっている。

use std::time::Instant;
use suikou_core::tokenizer::{LinderaMorphology, Morphology};

fn main() -> anyhow::Result<()> {
    let t0 = Instant::now();
    let morph = LinderaMorphology::new()?;
    let load = t0.elapsed();

    // 読み込み後の解析そのものの費用と分けて示す。
    let t1 = Instant::now();
    let n = morph.tokenize("設定を変更し、確認する。").len();
    let tokenize = t1.elapsed();

    println!("辞書の読み込みと自己診断: {load:?}");
    println!("1文の解析: {tokenize:?} ({n} トークン)");
    Ok(())
}
