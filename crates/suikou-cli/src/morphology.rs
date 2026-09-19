//! 形態素解析器の取得。辞書を同梱したビルドかどうかの分岐をここに閉じ込める。
//!
//! 辞書なしのビルドで日本語を解析しようとしたら落とす。
//! 黙って空の解析結果を返すと、漢語率も連用中止も 0 になり、
//! 「指摘なし」という誤った結論が出る。`verify_schema` と同じ理由で、落ちる方がよい。

use anyhow::Result;
use suikou_core::tokenizer::{Morphology, Token};
use suikou_core::Lang;

/// 辞書を同梱しているかどうか。`--version` の表示に使う。
pub const HAS_DICTIONARY: bool = cfg!(feature = "lindera-unidic");

/// 英語の経路は形態素解析器を参照しない。
/// 参照されたら前提が崩れているため、黙って進まずに落とす。
struct UnusedMorphology;

impl Morphology for UnusedMorphology {
    fn tokenize(&self, _text: &str) -> Vec<Token> {
        unreachable!("英語の経路で形態素解析器が呼ばれた")
    }
}

pub fn load(lang: Lang) -> Result<Box<dyn Morphology>> {
    match lang {
        Lang::En => Ok(Box::new(UnusedMorphology)),
        Lang::Ja => load_ja(),
    }
}

#[cfg(feature = "lindera-unidic")]
fn load_ja() -> Result<Box<dyn Morphology>> {
    use suikou_core::tokenizer::LinderaMorphology;
    Ok(Box::new(LinderaMorphology::new()?))
}

#[cfg(not(feature = "lindera-unidic"))]
fn load_ja() -> Result<Box<dyn Morphology>> {
    anyhow::bail!(
        "このビルドは辞書を含まないため日本語を解析できない。\n\
         辞書を同梱した版を GitHub Releases から取得するか、\n\
         `cargo install --git https://github.com/KazuyoshiAkiyama/suikou suikou-cli --features lindera-unidic` \
         で入れ直すこと。"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_does_not_need_a_dictionary() {
        assert!(load(Lang::En).is_ok());
    }

    #[cfg(not(feature = "lindera-unidic"))]
    #[test]
    fn japanese_fails_loudly_without_a_dictionary() {
        let e = match load(Lang::Ja) {
            Ok(_) => panic!("辞書なしのビルドで日本語が通ってしまった"),
            Err(e) => e.to_string(),
        };
        assert!(e.contains("辞書を含まない"), "{e}");
    }

    #[cfg(feature = "lindera-unidic")]
    #[test]
    fn japanese_loads_the_embedded_dictionary() {
        assert!(load(Lang::Ja).is_ok());
    }
}
