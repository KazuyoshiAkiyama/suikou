//! 英語の文書指標。
//!
//! 検証を通ったのは MATTR、地の文比率、直喩・例示標識の三つだけである。
//! 現在分詞節と名詞化は人間帯と重なったため採用しない。
//! Reinhart らが報告した5.3倍という効果量は、文書作成というタスクに転移しなかった。

use crate::metrics::{mattr, per_1k};
use serde::Serialize;

/// AI は隠喩的な語彙を使う一方で、比喩であることを標識しない。
pub const SIMILE_MARKERS: &[&str] = &[
    "like a",
    "like the",
    "as if",
    "as though",
    "akin to",
    "similar to",
    "think of it as",
    "imagine",
    "analogous to",
    "for example",
    "for instance",
];

#[derive(Debug, Clone, Default, Serialize)]
pub struct EnMetrics {
    pub mattr100: f64,
    /// D9。人間は 0.84 から 2.54。AI は 0.00 から 0.60 で明確に少ない。
    pub simile_marker_per_1k: f64,
    pub words: usize,
}

pub fn tokenize_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

pub fn analyze(_prose: &str, body: &str) -> EnMetrics {
    let words = tokenize_words(body);
    let lower = body.to_lowercase();
    let mut hits = 0usize;
    for m in SIMILE_MARKERS {
        hits += lower.matches(m).count();
    }
    EnMetrics {
        mattr100: mattr(&words, 100),
        simile_marker_per_1k: per_1k(hits, words.len()),
        words: words.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_and_lowercases() {
        let w = tokenize_words("The Cache, stored.");
        assert_eq!(w, vec!["the", "cache", "stored"]);
    }

    #[test]
    fn counts_simile_markers() {
        let m = analyze(
            "",
            "It behaves like a queue. For example, consider a burst.",
        );
        assert!(m.simile_marker_per_1k > 0.0);
    }

    #[test]
    fn no_markers_gives_zero() {
        let m = analyze(
            "",
            "The cache stores a computed result and returns it later.",
        );
        assert_eq!(m.simile_marker_per_1k, 0.0);
    }
}
