//! コーパスの実測値から閾値を較正する。
//!
//! `docs/content/*/design.md` は「閾値は、人間の幅の上と AI の幅の下との間に置く」と定める。
//! だが `suikou baseline` が見るのは人間コーパスだけで、AI の幅は分からない。
//! そこで、人間の分布そのものから見て「外れ値と言える」境界を出すことにする。
//! この境界が実際の AI 帯の下限より内側にあるかどうかは、AI コーパスとの突き合わせでしか
//! 確かめられない。その突き合わせは本実装の時点では行っていない。
//! 理由は `docs/content/*/decisions.md` の D-22 にある。
//!
//! 採った方法は Tukey の外れ値境界（fence）である。
//! 第1四分位と第3四分位の差（四分位範囲、IQR）の `FENCE_K` 倍を四分位の外側へ足した点を
//! 境界とする。統計で広く使われている「外れ値」の標準的な定義であり、
//! 恣意的な余裕を独自に決めるより根拠を示しやすい。

use crate::report::Direction;

/// 四分位範囲の何倍を境界に足すかの係数。
///
/// 1.5 は Tukey (1977) の "outlier"（外れ値）の定義で使われる値である。
/// 3.0 の "far out"（極端な外れ値）はもっと緩く、境界が外側に寄りすぎる。
/// baseline は少ない標本数で較正することが多く、境界を広げすぎると
/// 較正した意味が薄れる。標準的な 1.5 を採る。
pub const FENCE_K: f64 = 1.5;

/// 四分位を割るために最低限要る標本数。
///
/// 3件以下では、上半分か下半分のどちらかが1件になり、
/// 四分位範囲が分布の広がりでなく個々の値でほぼ決まってしまう。
/// 4件あれば上下の半分がそれぞれ2件以上になり、範囲としての意味を持つ。
pub const MIN_SAMPLES: usize = 4;

/// 中央値。ソート済みの列を前提とする。
fn median(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

/// 第1四分位と第3四分位を Tukey の hinge 法で求める。
///
/// 中央値そのものは、奇数個の場合にどちらの半分にも含めない。
/// 標本数が2以上であることを呼び出し側が保証すること。
fn quartiles(values: &[f64]) -> (f64, f64) {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("較正の対象に NaN が混じっている"));
    let n = sorted.len();
    let mid = n / 2;
    let (lower, upper) = if n % 2 == 0 {
        (&sorted[..mid], &sorted[mid..])
    } else {
        (&sorted[..mid], &sorted[mid + 1..])
    };
    (median(lower), median(upper))
}

/// 標本から閾値を較正する。
///
/// 標本数が `MIN_SAMPLES` に満たない場合は `None` を返す。
/// 較正できないことを黙って 0 や標本の最大値で埋めると、根拠のない閾値が紛れ込む。
pub fn calibrate(values: &[f64], direction: Direction) -> Option<f64> {
    if values.len() < MIN_SAMPLES {
        return None;
    }
    let (q1, q3) = quartiles(values);
    let iqr = q3 - q1;
    Some(match direction {
        Direction::Above => q3 + FENCE_K * iqr,
        Direction::Below => q1 - FENCE_K * iqr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quartiles_use_the_tukey_hinge_method() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        // 下半分 [1,2,3,4] の中央値は 2.5。上半分 [5,6,7,8] の中央値は 6.5。
        assert_eq!(quartiles(&values), (2.5, 6.5));
    }

    #[test]
    fn quartiles_exclude_the_median_when_the_count_is_odd() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];
        // 中央値の 3.0 はどちらの半分にも入らない。下半分 [1,2]→1.5、上半分 [4,5]→4.5。
        assert_eq!(quartiles(&values), (1.5, 4.5));
    }

    #[test]
    fn calibrate_above_adds_the_fence_to_q3() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let t = calibrate(&values, Direction::Above).unwrap();
        // IQR = 6.5 - 2.5 = 4.0。境界は 6.5 + 1.5 * 4.0 = 12.5。
        assert!((t - 12.5).abs() < 1e-9, "{t}");
    }

    #[test]
    fn calibrate_below_subtracts_the_fence_from_q1() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let t = calibrate(&values, Direction::Below).unwrap();
        // 境界は 2.5 - 1.5 * 4.0 = -3.5。
        assert!((t - (-3.5)).abs() < 1e-9, "{t}");
    }

    #[test]
    fn calibrate_refuses_too_few_samples() {
        assert_eq!(calibrate(&[1.0, 2.0, 3.0], Direction::Above), None);
    }
}
