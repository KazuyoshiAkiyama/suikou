//! 見出しに当てる規則。
//!
//! 見出しの文字列を対象とする判定をここに集める。ほかのブロックは見ない。
//! 規則ごとに対象ブロックを個別に絞る形では、どの規則が見出しに当たるかが
//! コードを読むまで分からない。その境界をこのモジュールで明示する。
//!
//! | 層 | 見出し |
//! |---|---|
//! | 文書指標（`ja.*`、`en.*`） | 見ない。`Document::body` が地の文とリスト項目だけを連結する |
//! | `register/rare-wago`、`register/prefer-katakana` | 見ない |
//! | `glossary/variant` | 見ない |
//! | `structure/heading-form` | 見る。章タイトルだけを見る |
//! | `maint/numbered-heading` | 見る。標題を除く見出しを見る |
//! | `maint/time-dependent` | 見る。標題を含むすべての見出しを見る |
//! | `glossary/defined-name` | 見る。本文と合わせて見る |
//!
//! 規則の識別子ごとに `LocalFinding` は1件だけ作る。`report.rs` の `to_markdown` は
//! `local` の要素ごとに節を出力するため、同じ識別子で2件返すと説明文が二重に出る。
//! 本文と見出しの双方に当たる規則は、ここが `Position` を返して本文側が束ねる。

/// 見出しの末尾から、形の判定に関わらない部分を落とす。
///
/// `（完了）` のような注記と句読点を落とす。落とさないと最後の形態素が
/// 補助記号になり、`## 設定を変更する（任意）` の用言止めを取り逃がす。
///
/// 開き括弧の無い閉じ括弧では何も落とさない。`## 設定する（任意` のような
/// 閉じ忘れで、落とす範囲を決められないためである。
pub fn trim_tail(text: &str) -> &str {
    let mut s = text.trim();
    loop {
        let before = s;
        if let Some(rest) = s.strip_suffix('）') {
            if let Some(i) = rest.rfind('（') {
                s = &rest[..i];
            }
        } else if let Some(rest) = s.strip_suffix(')') {
            if let Some(i) = rest.rfind('(') {
                s = &rest[..i];
            }
        }
        s = s.trim_end_matches(|c: char| {
            matches!(c, '。' | '：' | ':' | '、' | '！' | '？' | '.' | ' ' | '　')
        });
        if s == before {
            return s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tail_of_a_heading_is_trimmed() {
        assert_eq!(trim_tail("設定を変更する（任意）"), "設定を変更する");
        assert_eq!(
            trim_tail("T14 ルールを導入する（済み）"),
            "T14 ルールを導入する"
        );
        assert_eq!(trim_tail("検証の方法。"), "検証の方法");
        assert_eq!(
            trim_tail("register/rare-wago: 文体にそぐわない和語"),
            "register/rare-wago: 文体にそぐわない和語"
        );
        assert_eq!(trim_tail("List lead-ins (M1)"), "List lead-ins");
        assert_eq!(trim_tail("設定する（任意"), "設定する（任意");
        assert_eq!(trim_tail("（）"), "");
        assert_eq!(trim_tail("   "), "");
    }
}
