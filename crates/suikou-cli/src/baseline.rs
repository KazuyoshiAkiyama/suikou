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
use suikou_core::baseline::{calibrate, MIN_SAMPLES};
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
/// 標本数が `MIN_SAMPLES` に満たない指標は較正できない。
/// 較正できない指標を 0 や仮の値で埋めて出すと、使う人が根拠のない閾値に気づけない。
/// そのため、その指標は出力から外して標準エラーに理由を出す。
fn build_profile(name: String, spec: &Profile, samples: &Samples) -> Profile {
    let mut thresholds = BTreeMap::new();
    for (metric, t) in &spec.thresholds {
        let Some(values) = samples.get(metric) else {
            continue;
        };
        let Some(value) = calibrate(values, t.direction) else {
            eprintln!(
                "{metric}: 標本が {} 件しかなく較正できない。最低 {MIN_SAMPLES} 件が要る",
                values.len()
            );
            continue;
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
}
