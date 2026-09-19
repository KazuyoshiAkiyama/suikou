//! 形態素解析の境界。
//!
//! lindera の API はバージョンによって変わる。バージョンに依存するコードを
//! このファイルの `LinderaMorphology` だけに閉じ込めてある。
//! ほかのモジュールは `Morphology` トレイト越しにしか形態素解析を使わない。
//!
//! この分離により、形態素解析を持たない環境でも本体のロジックを
//! `FakeMorphology` でテストできる。

/// UniDic の素性から必要なものだけを取り出した形。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub surface: String,
    /// 品詞大分類。名詞、動詞、助詞、補助記号など。
    pub pos1: String,
    /// 品詞中分類。接続助詞、普通名詞など。
    pub pos2: String,
    /// 活用形。連用形-一般、終止形-一般など。活用しない語は空。
    pub cform: String,
    /// 語種。和、漢、外、混、記号、固。
    pub goshu: String,
    /// 語彙素。
    pub lemma: String,
}

impl Token {
    pub fn is_content_word(&self) -> bool {
        matches!(
            self.pos1.as_str(),
            "名詞" | "動詞" | "形容詞" | "副詞" | "形状詞"
        )
    }

    pub fn is_renyo_verb(&self) -> bool {
        self.pos1 == "動詞" && self.cform.starts_with("連用形")
    }

    pub fn is_te_conjunctive(&self) -> bool {
        (self.surface == "て" || self.surface == "で") && self.pos2 == "接続助詞"
    }

    /// 体言として文や項目を止められる品詞かどうか。
    pub fn is_taigen(&self) -> bool {
        matches!(self.pos1.as_str(), "名詞" | "接尾辞" | "代名詞" | "形状詞")
    }
}

pub trait Morphology {
    fn tokenize(&self, text: &str) -> Vec<Token>;
}

/// テスト用。空白区切りの `表層/品詞1/品詞2/活用形/語種` を読む。
/// 形態素解析器を用意せずにロジックを検証するために使う。
#[derive(Debug, Default)]
pub struct FakeMorphology {
    pub table: Vec<Token>,
}

impl FakeMorphology {
    /// `設定|名詞|普通名詞||漢 を|助詞|格助詞||和` のような記法から作る。
    pub fn from_spec(spec: &str) -> Self {
        let mut table = Vec::new();
        for item in spec.split_whitespace() {
            let f: Vec<&str> = item.split('|').collect();
            table.push(Token {
                surface: f.first().copied().unwrap_or_default().to_string(),
                pos1: f.get(1).copied().unwrap_or_default().to_string(),
                pos2: f.get(2).copied().unwrap_or_default().to_string(),
                cform: f.get(3).copied().unwrap_or_default().to_string(),
                goshu: f.get(4).copied().unwrap_or_default().to_string(),
                lemma: f.first().copied().unwrap_or_default().to_string(),
            });
        }
        Self { table }
    }
}

impl Morphology for FakeMorphology {
    /// 入力文字列を、登録済みの表層形で先頭から最長一致で切る。
    /// 一致しない文字は1文字ずつ未知語として返す。
    fn tokenize(&self, text: &str) -> Vec<Token> {
        let chars: Vec<char> = text.chars().collect();
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < chars.len() {
            let rest: String = chars[i..].iter().collect();
            let mut best: Option<&Token> = None;
            for t in &self.table {
                if rest.starts_with(&t.surface)
                    && !t.surface.is_empty()
                    && best.is_none_or(|b| t.surface.chars().count() > b.surface.chars().count())
                {
                    best = Some(t);
                }
            }
            match best {
                Some(t) => {
                    i += t.surface.chars().count();
                    out.push(t.clone());
                }
                None => {
                    let c = chars[i];
                    i += 1;
                    let pos1 = if c == '、' || c == '。' {
                        "補助記号"
                    } else {
                        "名詞"
                    };
                    let pos2 = if c == '、' {
                        "読点"
                    } else if c == '。' {
                        "句点"
                    } else {
                        ""
                    };
                    out.push(Token {
                        surface: c.to_string(),
                        pos1: pos1.to_string(),
                        pos2: pos2.to_string(),
                        cform: String::new(),
                        goshu: String::new(),
                        lemma: c.to_string(),
                    });
                }
            }
        }
        out
    }
}

#[cfg(feature = "lindera-unidic")]
pub use lindera_impl::LinderaMorphology;

#[cfg(feature = "lindera-unidic")]
mod lindera_impl {
    use super::{Morphology, Token};
    use anyhow::{anyhow, Result};
    use lindera::dictionary::{load_embedded_dictionary, DictionaryKind};
    use lindera::mode::Mode;
    use lindera::segmenter::Segmenter;
    use std::borrow::Cow;

    /// UniDic 2.1.2 の素性の並び。
    /// 品詞大分類 / 品詞中分類 / 品詞小分類 / 品詞細分類 / 活用型 / 活用形 /
    /// 語彙素読み / 語彙素 / 書字形出現形 / 発音形出現形 / 書字形基本形 /
    /// 発音形基本形 / 語種 / 語頭変化型 / 語頭変化形 / 語末変化型 / 語末変化形
    ///
    /// **lindera を更新したら `verify_schema` を必ず通すこと。**
    /// 並びが変わっていれば起動時に落ちる。黙って誤った値を返すより落ちる方がよい。
    const IDX_POS1: usize = 0;
    const IDX_POS2: usize = 1;
    const IDX_CFORM: usize = 5;
    const IDX_LEMMA: usize = 7;
    const IDX_GOSHU: usize = 12;
    const MIN_DETAILS: usize = 13;

    pub struct LinderaMorphology {
        segmenter: Segmenter,
    }

    impl LinderaMorphology {
        /// 辞書は埋め込み版を使う。読み込みは重いので、プロセスにつき一度だけ行う。
        /// hooks から高頻度で起動される用途では daemon モードを使う。
        pub fn new() -> Result<Self> {
            let dictionary = load_embedded_dictionary(DictionaryKind::UniDic)
                .map_err(|e| anyhow!("UniDic の読み込みに失敗した: {e}"))?;
            let me = Self {
                segmenter: Segmenter::new(Mode::Normal, dictionary, None),
            };
            me.verify_schema()?;
            Ok(me)
        }

        fn field(f: &[&str], i: usize) -> String {
            match f.get(i) {
                Some(&"*") | None => String::new(),
                Some(v) => v.to_string(),
            }
        }

        /// 素性列から Token を組み立てる。ここだけがバージョンに依存する。
        pub fn token_from_features(surface: &str, f: &[&str]) -> Token {
            Token {
                surface: surface.to_string(),
                pos1: Self::field(f, IDX_POS1),
                pos2: Self::field(f, IDX_POS2),
                cform: Self::field(f, IDX_CFORM),
                goshu: Self::field(f, IDX_GOSHU),
                lemma: Self::field(f, IDX_LEMMA),
            }
        }

        /// 素性の並びが想定どおりかを起動時に確かめる。
        ///
        /// 辞書やバージョンが変わって並びがずれると、語種と活用形が静かに壊れる。
        /// 漢語率と連用中止がこの二つに依存しているため、黙って誤った値を返すと
        /// 指摘の内容そのものが誤りになる。落ちる方がよい。
        pub fn verify_schema(&self) -> Result<()> {
            let probe = "設定を変更し、確認する。";
            let toks = self.tokenize(probe);
            let find = |s: &str| toks.iter().find(|t| t.surface == s).cloned();

            let setting =
                find("設定").ok_or_else(|| anyhow!("自己診断: 「設定」が切り出せなかった"))?;
            if setting.pos1 != "名詞" {
                return Err(anyhow!(
                    "自己診断: 品詞の位置がずれている。「設定」の pos1 が {:?}",
                    setting.pos1
                ));
            }
            if setting.goshu != "漢" {
                return Err(anyhow!(
                    "自己診断: 語種の位置がずれている。「設定」の語種が {:?}。\n\
                     IDX_GOSHU={} を辞書の素性の並びに合わせて直すこと",
                    setting.goshu,
                    IDX_GOSHU
                ));
            }
            let shi = find("し").ok_or_else(|| anyhow!("自己診断: 「し」が切り出せなかった"))?;
            if !shi.cform.starts_with("連用形") {
                return Err(anyhow!(
                    "自己診断: 活用形の位置がずれている。「し」の活用形が {:?}。\n\
                     IDX_CFORM={} を直すこと",
                    shi.cform,
                    IDX_CFORM
                ));
            }
            Ok(())
        }
    }

    impl Morphology for LinderaMorphology {
        fn tokenize(&self, text: &str) -> Vec<Token> {
            let mut out = Vec::new();
            let tokens = match self.segmenter.segment(Cow::Borrowed(text)) {
                Ok(t) => t,
                Err(_) => return out,
            };
            for mut t in tokens {
                let surface = t.surface.to_string();
                let details = t.details();
                if details.len() < MIN_DETAILS {
                    // 未知語は素性が短い。品詞だけ拾って残りは空にする。
                    out.push(Token {
                        surface,
                        pos1: Self::field(&details, IDX_POS1),
                        pos2: Self::field(&details, IDX_POS2),
                        cform: String::new(),
                        goshu: String::new(),
                        lemma: String::new(),
                    });
                    continue;
                }
                out.push(Self::token_from_features(&surface, &details));
            }
            out
        }
    }
}

#[cfg(all(test, feature = "lindera-unidic"))]
mod lindera_tests {
    use super::*;

    /// T2 の完了条件。素性の並びが想定どおりであることを実際の辞書で確かめる。
    /// `new` が `verify_schema` を呼ぶため、構築が通ること自体が自己診断の通過を意味する。
    #[test]
    fn reads_goshu_and_cform_from_the_real_dictionary() {
        let m = LinderaMorphology::new().expect("辞書の読み込みと自己診断");
        let toks = m.tokenize("設定を変更し、確認する。");
        let find = |s: &str| {
            toks.iter()
                .find(|t| t.surface == s)
                .unwrap_or_else(|| panic!("「{s}」が切り出せない: {toks:?}"))
        };
        assert_eq!(find("設定").goshu, "漢");
        assert_eq!(find("設定").pos1, "名詞");
        let shi = find("し");
        assert!(shi.cform.starts_with("連用形"), "cform={:?}", shi.cform);
    }

    /// 読点が読点として返ることを確かめる。連用中止とテ形接続の判定が直後の読点に依存する。
    #[test]
    fn counts_renyo_and_te_on_real_text() {
        let m = LinderaMorphology::new().expect("辞書の読み込み");
        let renyo = m.tokenize("設定を変更し、確認する。");
        assert!(renyo.iter().any(|t| t.is_renyo_verb()));
        let te = m.tokenize("設定を変更して、確認する。");
        assert!(te.iter().any(|t| t.is_te_conjunctive()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake() -> FakeMorphology {
        FakeMorphology::from_spec(
            "設定|名詞|普通名詞||漢 を|助詞|格助詞||和 変更|名詞|普通名詞||漢 \
             し|動詞|非自立可能|連用形-一般|和 行う|動詞|一般|終止形-一般|和 \
             て|助詞|接続助詞||和 こと|名詞|普通名詞||和 これ|代名詞|||和",
        )
    }

    #[test]
    fn splits_by_longest_match() {
        let m = fake();
        let t = m.tokenize("設定を変更し、");
        let surfaces: Vec<&str> = t.iter().map(|x| x.surface.as_str()).collect();
        assert_eq!(surfaces, vec!["設定", "を", "変更", "し", "、"]);
    }

    #[test]
    fn marks_reading_comma() {
        let m = fake();
        let t = m.tokenize("設定、");
        assert_eq!(t.last().unwrap().pos2, "読点");
    }

    #[test]
    fn detects_renyo_and_te() {
        let m = fake();
        let t = m.tokenize("変更して");
        assert!(t.iter().any(|x| x.is_renyo_verb()));
        assert!(t.iter().any(|x| x.is_te_conjunctive()));
    }

    #[test]
    fn content_word_classification() {
        let m = fake();
        let t = m.tokenize("設定を");
        assert!(t[0].is_content_word());
        assert!(!t[1].is_content_word());
    }
}
