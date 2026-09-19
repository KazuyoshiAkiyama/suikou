//! 出力スキーマ。局所指摘と文書スコープ指標を分けて持つ。
//! 文書指標は指摘位置を持たない。多くが「欠落」の指摘であり、原理的に位置がないため。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Above,
    Below,
}

/// 文書スコープの指標。閾値を外れたものだけを載せる。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetric {
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub direction: Direction,
    pub severity: Severity,
    /// 数値ではなく指示文。修正方針の合成に使う。
    pub guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub text: String,
}

/// 局所指摘。ルール単位でまとめる。説明文をルールにつき一度だけ出すため。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFinding {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub positions: Vec<Position>,
    /// 上限を超えて省いた件数。
    pub truncated: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Report {
    pub document: Vec<DocumentMetric>,
    pub local: Vec<LocalFinding>,
}

impl Report {
    pub const MAX_POSITIONS_PER_RULE: usize = 20;

    pub fn worst_severity(&self) -> Option<Severity> {
        let mut worst = None;
        for s in self
            .local
            .iter()
            .map(|f| f.severity)
            .chain(self.document.iter().map(|m| m.severity))
        {
            worst = Some(match worst {
                None => s,
                Some(Severity::Error) => Severity::Error,
                Some(Severity::Warning) if s == Severity::Error => s,
                Some(Severity::Info) if s != Severity::Info => s,
                Some(w) => w,
            });
        }
        worst
    }

    /// 出力のトークン数の見積もり。
    ///
    /// 正確な数は模型ごとに違うため、文字数からの概算で足りる。
    /// 日本語と英語で1トークンあたりの文字数が異なるので、中間の値を取る。
    /// 切り詰めの判断にしか使わない。
    pub const CHARS_PER_TOKEN: usize = 3;

    pub fn estimated_tokens(&self) -> usize {
        self.to_markdown().chars().count() / Self::CHARS_PER_TOKEN
    }

    /// 見積もりが budget に収まるまで、severity の低いものから落とす。
    ///
    /// 落とす順序は info、warning、error とする。
    /// メンテナンス性の指摘を最後まで残すのは、閾値の問題ではなく規定違反だからである。
    pub fn trim_to(&mut self, budget: usize) {
        for level in [Severity::Info, Severity::Warning, Severity::Error] {
            if self.estimated_tokens() <= budget {
                return;
            }
            self.document.retain(|m| m.severity != level);
            if self.estimated_tokens() <= budget {
                return;
            }
            // 同じ severity の中では、位置の多いものから減らす。
            while self.estimated_tokens() > budget {
                let Some(f) = self
                    .local
                    .iter_mut()
                    .filter(|f| f.severity == level && !f.positions.is_empty())
                    .max_by_key(|f| f.positions.len())
                else {
                    break;
                };
                f.positions.pop();
                f.truncated += 1;
            }
            // 位置を全部落としても、ルールと件数は残す。
            // 何件あるかが分からないと、直し切れたかを判断できない。
        }
    }

    /// LLM が読む Markdown。方針を先に、位置を後に置く。
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        if !self.document.is_empty() {
            out.push_str("## 書き直しの方針\n\n");
            for m in &self.document {
                out.push_str("- ");
                out.push_str(&m.guidance);
                out.push('\n');
            }
            out.push('\n');
        }
        if !self.local.is_empty() {
            out.push_str("## 直す箇所\n\n");
            for f in &self.local {
                out.push_str(&format!(
                    "### {} — {}（{}件）\n\n",
                    f.message,
                    f.rule_id,
                    f.positions.len() + f.truncated
                ));
                for p in &f.positions {
                    // 同じ行に複数ある指摘を見分けられるように、列を持つものは列も出す。
                    if p.column > 1 {
                        out.push_str(&format!("- L{}:{} {}\n", p.line, p.column, p.text));
                    } else {
                        out.push_str(&format!("- L{} {}\n", p.line, p.text));
                    }
                }
                if f.truncated > 0 {
                    out.push_str(&format!("- ほか{}件\n", f.truncated));
                }
                out.push('\n');
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(sev: Severity, n: usize) -> LocalFinding {
        LocalFinding {
            rule_id: "maint/x".into(),
            severity: sev,
            message: "説明".into(),
            positions: (0..n)
                .map(|i| Position {
                    line: i + 1,
                    column: 1,
                    text: "あ".repeat(40),
                })
                .collect(),
            truncated: 0,
        }
    }

    #[test]
    fn worst_severity_prefers_error() {
        let r = Report {
            document: vec![],
            local: vec![finding(Severity::Info, 1), finding(Severity::Error, 1)],
        };
        assert_eq!(r.worst_severity(), Some(Severity::Error));
    }

    #[test]
    fn empty_report_has_no_severity() {
        assert_eq!(Report::default().worst_severity(), None);
    }

    #[test]
    fn trim_drops_info_before_error() {
        let mut r = Report {
            document: vec![],
            local: vec![finding(Severity::Info, 10), finding(Severity::Error, 10)],
        };
        let before = r.estimated_tokens();
        r.trim_to(before / 2);
        assert!(r.local.iter().any(|f| f.severity == Severity::Error));
        assert!(r.estimated_tokens() <= before);
    }

    #[test]
    fn trim_keeps_the_count_of_dropped_positions() {
        let mut r = Report {
            document: vec![],
            local: vec![finding(Severity::Info, 10)],
        };
        r.trim_to(10);
        let kept: usize = r.local.iter().map(|f| f.positions.len()).sum();
        let dropped: usize = r.local.iter().map(|f| f.truncated).sum();
        assert_eq!(kept + dropped, 10, "{:?}", r.local);
    }
}
