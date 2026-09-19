//! suikou の解析中核。
//!
//! 検出対象は二つの集団に分かれる。文や行の単位で真偽が決まり位置を持つものと、
//! 文書全体でしか計算できず位置を持たないものである。
//! 後者の多くは欠落の指摘であり、原理的に位置がない。
//! この分割が「一度の実行で、一度の修正で済ませる」という要件の実装形になる。
//!
//! 形態素解析への依存は `tokenizer` に閉じ込めてある。
//! 既定の feature では lindera を含まないため、形態素解析器がなくても
//! 本体のロジックを `FakeMorphology` でテストできる。

pub mod lang;
pub mod markdown;
pub mod metrics;
pub mod profile;
pub mod report;
pub mod rules;
pub mod tokenizer;

pub use lang::{Lang, Style};
pub use markdown::Document;
pub use report::{Report, Severity};
pub use tokenizer::{Morphology, Token};
