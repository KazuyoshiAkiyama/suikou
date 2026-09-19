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

impl Profile {
    pub fn builtin(name: &str) -> Option<Self> {
        let src = match name {
            "oss" => include_str!("../profiles/oss.toml"),
            "service" => include_str!("../profiles/service.toml"),
            _ => return None,
        };
        toml::from_str(src).ok()
    }
}
