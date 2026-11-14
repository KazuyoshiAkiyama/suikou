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
    /// 型ごとの調整。型を宣言した文書にだけ重ねる。
    ///
    /// 同じ閾値を README と参照の文書に当てることはできない。
    /// 著名な OSS の README 240件（英語）では `en.mattr100` が 70% で発火し、
    /// 参照コーパス 652件では 15% であった。差は 4.7 倍である。
    /// 発火率が 70% の指標は、型と閾値が合っていないことを示す。経緯は D-41 にある。
    #[serde(default)]
    pub doctype: BTreeMap<String, DoctypeAdjust>,
}

/// 型ごとの調整。閾値と、黙らせる局所の規則を持つ。
///
/// `reason` は調整の経緯を残すために置く。
/// 緩めた判断は、後から読む人が同じ検討を繰り返さずに済む形で残す。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DoctypeAdjust {
    /// なぜこの型で変えたのか。実測した値か、利用者との調整の結果を書く。
    #[serde(default)]
    pub reason: String,
    /// この型では出さない指標の名前。
    #[serde(default)]
    pub disable: Vec<String>,
    /// この型では出さない局所の規則の id。`structure/long-paragraph` のような形にする。
    #[serde(default)]
    pub rules: Vec<String>,
    /// この型で差し替える閾値。書いた欄だけを変える。
    #[serde(default)]
    pub thresholds: BTreeMap<String, ThresholdPatch>,
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
    /// 型ごとの調整。組み込みの型の調整に重ねる。
    ///
    /// 入り口の README と参照の文書で硬さが違うという判断を、
    /// 型ごとにプロジェクトの側で決められるようにするために置く。
    #[serde(default)]
    pub doctype: BTreeMap<String, DoctypeAdjust>,
}

/// 閾値の一部だけを差し替えるための形。書かなかった欄は組み込みのままとする。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    /// 型ごとの調整を重ねた複製を返す。黙らせる規則の一覧も返す。
    ///
    /// 重ねる順序は、組み込みの型の調整、利用者の全体の上書き、利用者の型の調整とする。
    /// 利用者が書いたものが常に後に来る。
    /// 組み込みの調整を利用者が戻せない形にすると、実測と合わない現場で逃げ道が無くなる。
    pub fn for_doctype(&self, doctype: &str, ov: Option<&ProfileOverride>) -> (Self, Vec<String>) {
        let mut out = self.clone();
        let mut silenced = Vec::new();

        // 組み込みの型の調整。
        if let Some(a) = self.doctype.get(doctype) {
            out.absorb(a, &mut silenced);
        }
        if let Some(ov) = ov {
            // 利用者の全体の上書きは、呼ぶ側が既に当てている場合もある。
            // ここでは型の調整だけを当てる。二重に当てても結果は同じになる。
            if let Some(a) = ov.doctype.get(doctype) {
                out.absorb(a, &mut silenced);
            }
        }
        silenced.sort_unstable();
        silenced.dedup();
        (out, silenced)
    }

    fn absorb(&mut self, a: &DoctypeAdjust, silenced: &mut Vec<String>) {
        for (metric, patch) in &a.thresholds {
            if let Some(t) = self.thresholds.get_mut(metric) {
                if let Some(v) = patch.value {
                    t.value = v;
                }
                if let Some(d) = patch.direction {
                    t.direction = d;
                }
                if let Some(sv) = patch.severity {
                    t.severity = sv;
                }
                if let Some(g) = &patch.guidance {
                    t.guidance = g.clone();
                }
            }
            // 組み込みに無い指標は型の調整では足さない。
            // 足すのは全体の上書きの役目とする。型ごとに別の指標が現れると追えなくなる。
        }
        for metric in &a.disable {
            self.thresholds.remove(metric);
        }
        silenced.extend(a.rules.iter().cloned());
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

    // 型ごとの調整。入り口と参照で硬さが違うという判断を、型で表す。
    #[test]
    fn a_doctype_adjustment_changes_only_that_doctype() {
        let base = Profile::builtin("oss").unwrap();
        let before = base.thresholds["en.mattr100"].value;
        let (readme, _) = base.for_doctype("readme", None);
        let (reference, _) = base.for_doctype("reference", None);
        assert!(readme.thresholds["en.mattr100"].value > before);
        assert_eq!(reference.thresholds["en.mattr100"].value, before);
    }

    // 較正できない指標は、型ごとに除外できる。
    // ADR 190件で `en.prose_ratio` の較正値が値域の外に出たため `decision` で外した。
    #[test]
    fn a_doctype_adjustment_can_remove_a_metric() {
        let base = Profile::builtin("oss").unwrap();
        assert!(base.thresholds.contains_key("en.prose_ratio"));
        let (decision, _) = base.for_doctype("decision", None);
        assert!(!decision.thresholds.contains_key("en.prose_ratio"));
        // ほかの型からは消えない。
        let (reference, _) = base.for_doctype("reference", None);
        assert!(reference.thresholds.contains_key("en.prose_ratio"));
    }

    // 比喩の標識は型ごとに外さない。語数の門で制御する。
    // どの型でも閾値は残り、語数が足りない文書で比べないだけである。
    #[test]
    fn the_simile_metric_is_not_removed_by_any_doctype() {
        let base = Profile::builtin("oss").unwrap();
        for dt in ["readme", "design", "decision", "reference", "howto", "log"] {
            let (p, _) = base.for_doctype(dt, None);
            assert!(
                p.thresholds.contains_key("en.simile_marker_per_1k"),
                "{dt} から消えている"
            );
        }
    }

    #[test]
    fn an_unknown_doctype_leaves_the_profile_alone() {
        let base = Profile::builtin("oss").unwrap();
        let (same, silenced) = base.for_doctype("no-such-type", None);
        assert_eq!(same.thresholds.len(), base.thresholds.len());
        assert!(silenced.is_empty());
    }

    #[test]
    fn a_project_can_silence_a_rule_for_one_doctype() {
        let base = Profile::builtin("oss").unwrap();
        let ov: ProfileOverride = toml::from_str(
            r#"
            [doctype.readme]
            reason = "入り口は箇条書きが主になる"
            rules = ["structure/long-paragraph"]
            "#,
        )
        .unwrap();
        let (_, silenced) = base.for_doctype("readme", Some(&ov));
        assert_eq!(silenced, vec!["structure/long-paragraph"]);
        // ほかの型には及ばない。
        let (_, none) = base.for_doctype("reference", Some(&ov));
        assert!(none.is_empty());
    }

    #[test]
    fn a_project_override_wins_over_the_builtin_doctype_adjustment() {
        let base = Profile::builtin("oss").unwrap();
        let ov: ProfileOverride = toml::from_str(
            r#"
            [doctype.readme.thresholds."en.mattr100"]
            value = 0.95
            "#,
        )
        .unwrap();
        let (readme, _) = base.for_doctype("readme", Some(&ov));
        assert_eq!(readme.thresholds["en.mattr100"].value, 0.95);
    }

    // 組み込みの調整は理由を持つ。緩めた判断を後から読めるようにするためである。
    #[test]
    fn every_builtin_doctype_adjustment_records_a_reason() {
        for name in ["oss", "service"] {
            let p = Profile::builtin(name).unwrap();
            for (dt, a) in &p.doctype {
                assert!(!a.reason.is_empty(), "{name} の {dt} に理由が無い");
            }
        }
    }

    // 型の調整では指標を足せない。足すのは全体の上書きの役目とする。
    #[test]
    fn a_doctype_adjustment_does_not_add_a_metric() {
        let base = Profile::builtin("oss").unwrap();
        let ov: ProfileOverride = toml::from_str(
            r#"
            [doctype.readme.thresholds."en.no_such_metric"]
            value = 1.0
            direction = "above"
            "#,
        )
        .unwrap();
        let (readme, _) = base.for_doctype("readme", Some(&ov));
        assert!(!readme.thresholds.contains_key("en.no_such_metric"));
    }
}
