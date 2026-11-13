//! 閾値のプロファイル。
//!
//! 初期値は corpus/baselines.toml の実測値から導いた。
//! ジャンルによる変動が大きいため、`suikou baseline` で自前のコーパスから作り直せるようにする。

use crate::report::{Direction, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    pub value: f64,
    pub direction: Direction,
    pub severity: Severity,
    pub guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    /// 指標名から閾値へ。指標名は "ja.renyo_te_ratio" のような形にする。
    pub thresholds: BTreeMap<String, Threshold>,
}

/// プロファイルに重ねる上書き。`.suikou/profile.toml` に書く。
///
/// プロファイルごと差し替えると、変えたい1つの値のために全部を写すことになる。
/// 写した側は、組み込みの値が変わっても追従しない。
/// 変えたい閾値だけを書けるようにして、残りは組み込みのままにする。
///
/// 漢語率と漢字率は分野によって幅が大きく、どこを警告とするかは文書の性質で変わる。
/// この上書きは、その判断をプロジェクトに渡すために置く。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProfileOverride {
    /// 出さない指標の名前。閾値を消して黙らせる。
    #[serde(default)]
    pub disable: Vec<String>,
    /// 書いた欄だけを差し替える。
    #[serde(default)]
    pub thresholds: BTreeMap<String, ThresholdPatch>,
}

/// 閾値の一部だけを差し替えるための形。書かなかった欄は組み込みのままとする。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ThresholdPatch {
    pub value: Option<f64>,
    pub direction: Option<Direction>,
    pub severity: Option<Severity>,
    pub guidance: Option<String>,
}

impl Profile {
    /// 上書きを重ねる。知らない指標名はそのまま足す。
    ///
    /// 足せるようにしてあるのは、組み込みが閾値を持たない指標
    /// （`ja.kanji_ratio` など）をプロジェクトが有効にできるようにするためである。
    /// ただし `value` と `direction` の両方が無い新しい指標は足さない。
    /// 片方だけでは判定できず、黙って無視するより気付ける形にする。
    pub fn apply(&mut self, ov: &ProfileOverride) -> Result<(), String> {
        for (metric, patch) in &ov.thresholds {
            match self.thresholds.get_mut(metric) {
                Some(t) => {
                    if let Some(v) = patch.value {
                        t.value = v;
                    }
                    if let Some(d) = patch.direction {
                        t.direction = d;
                    }
                    if let Some(s) = patch.severity {
                        t.severity = s;
                    }
                    if let Some(g) = &patch.guidance {
                        t.guidance = g.clone();
                    }
                }
                None => {
                    let (Some(value), Some(direction)) = (patch.value, patch.direction) else {
                        return Err(format!(
                            "{metric} は組み込みに閾値が無い。足すには value と direction の両方が要る"
                        ));
                    };
                    self.thresholds.insert(
                        metric.clone(),
                        Threshold {
                            value,
                            direction,
                            severity: patch.severity.unwrap_or(Severity::Info),
                            guidance: patch.guidance.clone().unwrap_or_default(),
                        },
                    );
                }
            }
        }
        for metric in &ov.disable {
            self.thresholds.remove(metric);
        }
        Ok(())
    }

    pub fn builtin(name: &str) -> Option<Self> {
        let src = match name {
            "oss" => include_str!("../profiles/oss.toml"),
            "service" => include_str!("../profiles/service.toml"),
            _ => return None,
        };
        toml::from_str(src).ok()
    }
}

#[cfg(test)]
mod override_tests {
    use super::*;

    fn oss() -> Profile {
        Profile::builtin("oss").unwrap()
    }

    fn parse(src: &str) -> ProfileOverride {
        toml::from_str(src).unwrap()
    }

    // 書いた欄だけが変わる。残りは組み込みのままとする。
    #[test]
    fn a_patch_replaces_only_the_fields_it_names() {
        let mut p = oss();
        let before = p.thresholds["ja.kango_ratio"].clone();
        p.apply(&parse("[thresholds.\"ja.kango_ratio\"]\nvalue = 0.55\n"))
            .unwrap();
        let after = &p.thresholds["ja.kango_ratio"];
        assert_eq!(after.value, 0.55);
        assert_eq!(after.direction, before.direction);
        assert_eq!(after.severity, before.severity);
        assert_eq!(after.guidance, before.guidance);
    }

    #[test]
    fn a_patch_can_raise_the_severity() {
        let mut p = oss();
        p.apply(&parse(
            "[thresholds.\"ja.kango_ratio\"]\nseverity = \"warning\"\n",
        ))
        .unwrap();
        assert_eq!(p.thresholds["ja.kango_ratio"].severity, Severity::Warning);
    }

    // 漢語率を黙らせられる。割合を重く見すぎないための出口である。
    #[test]
    fn disable_removes_the_threshold() {
        let mut p = oss();
        p.apply(&parse("disable = [\"ja.kango_ratio\"]\n")).unwrap();
        assert!(!p.thresholds.contains_key("ja.kango_ratio"));
    }

    // 組み込みが閾値を持たない指標を、プロジェクトが有効にできる。
    #[test]
    fn a_new_metric_can_be_added() {
        let mut p = oss();
        assert!(!p.thresholds.contains_key("ja.kanji_ratio"));
        p.apply(&parse(
            "[thresholds.\"ja.kanji_ratio\"]\nvalue = 0.40\ndirection = \"above\"\n",
        ))
        .unwrap();
        let t = &p.thresholds["ja.kanji_ratio"];
        assert_eq!(t.value, 0.40);
        assert_eq!(t.severity, Severity::Info);
    }

    // 片方だけでは判定できない。黙って無視せず落とす。
    #[test]
    fn a_new_metric_without_a_direction_is_an_error() {
        let mut p = oss();
        let err = p
            .apply(&parse("[thresholds.\"ja.kanji_ratio\"]\nvalue = 0.40\n"))
            .unwrap_err();
        assert!(err.contains("ja.kanji_ratio"), "{err}");
    }

    #[test]
    fn an_empty_override_changes_nothing() {
        let mut p = oss();
        let before = p.thresholds.len();
        p.apply(&ProfileOverride::default()).unwrap();
        assert_eq!(p.thresholds.len(), before);
    }

    // 上書きと disable を同じファイルに書いた場合、disable が後に効く。
    #[test]
    fn disable_wins_over_a_patch_for_the_same_metric() {
        let mut p = oss();
        p.apply(&parse(
            "disable = [\"ja.kango_ratio\"]\n\n[thresholds.\"ja.kango_ratio\"]\nvalue = 0.9\n",
        ))
        .unwrap();
        assert!(!p.thresholds.contains_key("ja.kango_ratio"));
    }
}
