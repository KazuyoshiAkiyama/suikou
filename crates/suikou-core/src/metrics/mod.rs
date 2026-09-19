//! 文書スコープの指標。位置を持たない。
//! 多くが「欠落」の指摘であり、原理的に位置がない。
pub mod en;
pub mod ja;

/// 移動窓による語彙多様性。窓幅は 100 トークン。
/// 日英どちらでも AI が人間帯を上回る、最も頑健な指標。
pub fn mattr(tokens: &[String], window: usize) -> f64 {
    use std::collections::HashSet;
    if tokens.is_empty() {
        return 0.0;
    }
    if tokens.len() < window {
        let uniq: HashSet<&String> = tokens.iter().collect();
        return uniq.len() as f64 / tokens.len() as f64;
    }
    let n = tokens.len() - window + 1;
    let mut sum = 0.0;
    for i in 0..n {
        let uniq: HashSet<&String> = tokens[i..i + window].iter().collect();
        sum += uniq.len() as f64 / window as f64;
    }
    sum / n as f64
}

/// 1000 単位あたりに正規化する。分母が 0 なら 0 を返す。
pub fn per_1k(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64 * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn mattr_of_all_distinct_is_one() {
        let t = toks(&["a", "b", "c"]);
        assert!((mattr(&t, 100) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mattr_of_all_same_is_small() {
        let t = toks(&["a", "a", "a", "a"]);
        assert!((mattr(&t, 100) - 0.25).abs() < 1e-9);
    }

    #[test]
    fn mattr_uses_window_when_long_enough() {
        let t: Vec<String> = (0..10).map(|i| (i % 2).to_string()).collect();
        assert!((mattr(&t, 4) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn per_1k_handles_zero_denominator() {
        assert_eq!(per_1k(5, 0), 0.0);
        assert!((per_1k(5, 1000) - 5.0).abs() < 1e-9);
    }
}
