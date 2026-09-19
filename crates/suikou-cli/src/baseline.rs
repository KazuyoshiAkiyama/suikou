//! `suikou baseline` の実装。
//!
//! 与えられた文書から指標を測り、指標ごとに標本を集めて、`suikou_core::baseline::calibrate`
//! で閾値を較正する。較正の根拠（Tukey の外れ値境界を使うこと）は core 側に書いてある。
//!
//! `guidance` と `direction` と `severity` は、同梱の oss プロファイルの文面をそのまま
//! 引き継ぐ。ここで書き換えるのは `value` だけである。呼び出しのたびに文面を書き写すと、
//! oss.toml を直したときに baseline の出力だけ古いまま残る経路ができる。

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::io::Write as _;
use suikou_core::baseline::{calibrate, CalibrationError, MIN_SAMPLES};
use suikou_core::check::measure;
use suikou_core::markdown::Document;
use suikou_core::profile::{Profile, Threshold};

use crate::check::{collect, resolve_lang};
use crate::morphology;

pub struct Options {
    pub paths: Vec<String>,
    pub lang: String,
    pub name: String,
    pub out: Option<String>,
}

/// 指標名から標本の列へ。
type Samples = BTreeMap<String, Vec<f64>>;

/// 対象の文書から指標を測り、指標名ごとに標本を集める。
fn collect_samples(paths: &[String], lang_spec: &str) -> Result<Samples> {
    let files = collect(paths)?;
    let mut samples: Samples = BTreeMap::new();
    for path in &files {
        let src = std::fs::read_to_string(path)
            .with_context(|| format!("{} を読めない", path.display()))?;
        let doc = Document::parse(&src);
        let lang = resolve_lang(lang_spec, &src)?;
        let morph = morphology::load(lang)?;
        let values = measure(&doc, lang, morph.as_ref());
        for (name, value) in values {
            samples.entry(name).or_default().push(value);
        }
    }
    Ok(samples)
}

/// 標本から閾値を較正し、oss プロファイルの guidance と direction と severity を添えて
/// 新しいプロファイルに仕立てる。
///
/// 較正できない指標が2種類ある。標本数が `suikou_core::baseline::MIN_SAMPLES` に満たない
/// 指標と、below 方向の境界が指標の値域の外（0以下）まで下がって永久に発火しなくなった
/// 指標である。後者は `suikou_core::baseline` のモジュール文書に経緯が書いてある。
/// どちらも 0 や仮の値で埋めて出すと、使う人が根拠のない閾値、あるいは
/// 死んだ閾値に気づけない。そのため、その指標は出力から外して標準エラーに理由を出す。
fn build_profile(name: String, spec: &Profile, samples: &Samples) -> Profile {
    let mut thresholds = BTreeMap::new();
    for (metric, t) in &spec.thresholds {
        let Some(values) = samples.get(metric) else {
            continue;
        };
        let value = match calibrate(values, t.direction) {
            Ok(value) => value,
            Err(CalibrationError::TooFewSamples { count }) => {
                eprintln!(
                    "{metric}: 標本が {count} 件しかなく較正できない。最低 {MIN_SAMPLES} 件が要る"
                );
                continue;
            }
            Err(CalibrationError::Unreachable { threshold }) => {
                eprintln!(
                    "{metric}: 較正した境界 {threshold} が値域の外に出て、\
                     この閾値は永久に発火しない。較正しない"
                );
                continue;
            }
        };
        thresholds.insert(
            metric.clone(),
            Threshold {
                value,
                direction: t.direction,
                severity: t.severity,
                guidance: t.guidance.clone(),
            },
        );
    }
    Profile { name, thresholds }
}

pub fn run(opts: Options) -> Result<()> {
    let samples = collect_samples(&opts.paths, &opts.lang)?;

    // guidance と direction と severity の対応表として oss プロファイルを使う。
    let spec = Profile::builtin("oss").expect("組み込みの oss プロファイルを読めない");
    let profile = build_profile(opts.name, &spec, &samples);

    let rendered = toml::to_string_pretty(&profile).context("プロファイルを TOML にできない")?;
    match opts.out {
        Some(path) => {
            std::fs::write(&path, &rendered).with_context(|| format!("{path} に書けない"))?;
        }
        None => {
            std::io::stdout().write_all(rendered.as_bytes())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use suikou_core::report::{Direction, Severity};

    fn spec() -> Profile {
        Profile::builtin("oss").unwrap()
    }

    #[test]
    fn build_profile_round_trips_through_toml() {
        let mut samples: Samples = BTreeMap::new();
        // 8件。ja.mattr100 は "above" 方向。
        samples.insert(
            "ja.mattr100".into(),
            vec![0.55, 0.56, 0.57, 0.58, 0.59, 0.60, 0.61, 0.62],
        );
        let profile = build_profile("custom".into(), &spec(), &samples);

        let rendered = toml::to_string_pretty(&profile).unwrap();
        let back: Profile = toml::from_str(&rendered).expect("読み戻せる形になっていない");

        assert_eq!(back.name, "custom");
        let t = back
            .thresholds
            .get("ja.mattr100")
            .expect("較正できたはずの指標が出ていない");
        assert_eq!(t.direction, Direction::Above);
        assert_eq!(t.severity, Severity::Warning);
        assert!(!t.guidance.is_empty());
        // 較正値そのものは suikou_core::baseline のテストで確かめてあるため、
        // ここでは oss.toml の値と単純に一致しないことだけ確かめる。
        assert_ne!(t.value, spec().thresholds["ja.mattr100"].value);
    }

    #[test]
    fn metrics_without_enough_samples_are_skipped() {
        let mut samples: Samples = BTreeMap::new();
        // MIN_SAMPLES(4) に満たない。
        samples.insert("ja.mattr100".into(), vec![0.55, 0.60]);
        let profile = build_profile("custom".into(), &spec(), &samples);
        assert!(!profile.thresholds.contains_key("ja.mattr100"));
    }

    #[test]
    fn metrics_absent_from_the_input_are_skipped() {
        let samples: Samples = BTreeMap::new();
        let profile = build_profile("custom".into(), &spec(), &samples);
        assert!(profile.thresholds.is_empty());
    }

    /// 実際にコーパスへ走らせて見つかった欠陥の再現。
    /// `ja.demonstrative_per_1k` は below 方向で、値域が0以上の千字あたりの出現数である。
    /// 標本がひとかたまりに寄っていると Q1 - 1.5*IQR が0を下回り、その境界は
    /// どんな実測値も下回れなくなる。較正せずに外すこと。
    #[test]
    fn metrics_with_an_unreachable_fence_are_skipped() {
        let mut samples: Samples = BTreeMap::new();
        // Q1=1.0、Q3=3.0、IQR=2.0 なので below の境界は 1.0 - 1.5*2.0 = -2.0 になる。
        samples.insert(
            "ja.demonstrative_per_1k".into(),
            vec![0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0],
        );
        let profile = build_profile("custom".into(), &spec(), &samples);
        assert!(
            !profile.thresholds.contains_key("ja.demonstrative_per_1k"),
            "発火しえない閾値が出力に紛れ込んでいる: {:?}",
            profile.thresholds.get("ja.demonstrative_per_1k")
        );
    }

    /// 較正できた below 方向の閾値が、実際に `evaluate` を通して発火することを確かめる。
    /// 指摘ゼロだけでは、較正が正しく働いたのか、閾値が死んでいて何も引っかからなかった
    /// だけなのかを区別できない。ここでは閾値を跨ぐ文書を用意し、跨いだ側では発火し、
    /// 跨がない側では発火しないことを両方見る。
    #[test]
    fn a_calibrated_below_threshold_fires_when_a_document_crosses_it() {
        use suikou_core::lang::Lang;
        use suikou_core::markdown::Document;
        use suikou_core::report::Direction as Dir;
        use suikou_core::tokenizer::{FakeMorphology, Morphology};

        // 指示詞が多い標本と少ない標本を混ぜ、below の境界が0より大きくなるようにする。
        let mut samples: Samples = BTreeMap::new();
        samples.insert(
            "ja.demonstrative_per_1k".into(),
            vec![40.0, 42.0, 44.0, 46.0, 48.0, 50.0, 52.0, 54.0],
        );
        let profile = build_profile("custom".into(), &spec(), &samples);
        let t = profile
            .thresholds
            .get("ja.demonstrative_per_1k")
            .expect("この標本なら較正できるはずである");
        assert_eq!(t.direction, Dir::Below);
        assert!(
            t.value > 0.0,
            "below の閾値が発火しうる値域に入っていない: {}",
            t.value
        );

        // 「これ」を指示詞として認識させる。ほかの語は自立語として数えない設定にする。
        let morph = FakeMorphology::from_spec("これ|代名詞|||和 設定|名詞|普通名詞||漢");
        assert!(!morph.tokenize("これ").is_empty(), "テスト設定を確かめる");

        // 境界を下回る（指示詞が少ない）文書。findings に出るはずである。
        let sparse = Document::parse("設定を確認する。\n");
        let r = suikou_core::check::evaluate(&sparse, Lang::Ja, &morph, &profile);
        assert!(
            r.document
                .iter()
                .any(|m| m.metric == "ja.demonstrative_per_1k"),
            "境界を下回ったのに発火していない: {:?}",
            r.document
        );

        // 境界を上回る（指示詞が多い）文書。findings に出ないはずである。
        let dense = Document::parse(
            "これ。これ。これ。これ。これ。これ。これ。これ。これ。これ。\
             これ。これ。これ。これ。これ。これ。これ。これ。これ。これ。\n",
        );
        let r = suikou_core::check::evaluate(&dense, Lang::Ja, &morph, &profile);
        assert!(
            !r.document
                .iter()
                .any(|m| m.metric == "ja.demonstrative_per_1k"),
            "境界を上回ったのに発火している: {:?}",
            r.document
        );
    }
}
