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
                    out.push_str(&format!("- L{} {}\n", p.line, p.text));
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
