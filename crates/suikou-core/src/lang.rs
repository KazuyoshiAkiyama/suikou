//! 言語と文体の判定。
//!
//! 文体は文書単位ではなくブロック単位で判定する。本文が敬体、箇条書きが常体という
//! 構成が正当であるため、文書単位では粒度が粗すぎる。
//! 文書指標の閾値は文体で分けない。実測でモデル間の方向が一致しなかったため。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Ja,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    /// ですます調
    Desumasu,
    /// である調
    Dearu,
    Unknown,
}

pub fn detect_lang(text: &str) -> Lang {
    let ja = text
        .chars()
        .filter(|c| matches!(*c as u32, 0x3040..=0x30FF | 0x4E00..=0x9FFF))
        .count();
    if ja * 20 > text.chars().count() {
        Lang::Ja
    } else {
        Lang::En
    }
}

/// 文末の形から文体を判定する。
pub fn detect_style(sentences: &[String]) -> Style {
    let mut desumasu = 0usize;
    let mut dearu = 0usize;
    for s in sentences {
        let t = s.trim_end_matches(['。', '！', '？']);
        if t.ends_with("です")
            || t.ends_with("ます")
            || t.ends_with("ました")
            || t.ends_with("ません")
            || t.ends_with("でしょう")
            || t.ends_with("ください")
        {
            desumasu += 1;
        } else if t.ends_with("である") || t.ends_with("だ") || t.ends_with("た") {
            dearu += 1;
        }
    }
    match (desumasu, dearu) {
        (0, 0) => Style::Unknown,
        (d, j) if d > j => Style::Desumasu,
        _ => Style::Dearu,
    }
}
