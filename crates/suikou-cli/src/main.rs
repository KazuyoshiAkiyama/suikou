//! suikou の CLI。
//!
//! textlint を局所規則のために呼び出し、自前の文書解析と統合して一つのレポートにする。
//! 文書指標を textlint の中に収めない理由は、収めると解析基盤が JS に縛られ、
//! 検証済みの指標を実装の都合で捨てることになるためである。

mod baseline;
mod brief;
mod check;
mod mcp;
mod morphology;
mod plot;
mod terms;

use anyhow::Result;
use clap::{Parser, Subcommand};

// 辞書の有無は `--version` で判別できるようにする。
// どちらのビルドを掴んでいるか分からないと、日本語が解析されない理由が読めない。
#[cfg(feature = "lindera-unidic")]
const LONG_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\n辞書: UniDic 同梱");
#[cfg(not(feature = "lindera-unidic"))]
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n辞書: なし。日本語は解析できない"
);

#[derive(Parser)]
#[command(
    name = "suikou",
    version,
    long_version = LONG_VERSION,
    about = "技術文書のlintとハーネス"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 生成前にプロンプトへ入れる制約ブロックを出力する
    Brief {
        #[arg(long, default_value = "oss")]
        profile: String,
        #[arg(long, default_value = "auto")]
        lang: String,
        /// 指示の詳細度。構造の規則は concise で守られるが、時点依存語は detailed が要る
        #[arg(long, default_value = "balanced")]
        detail: String,
        /// 文書の型。与えると、節の型枠と書き方の指針を先に出す
        #[arg(long)]
        doctype: Option<String>,
    },
    /// 局所指摘と文書指標の統合レポートを出力する
    Check {
        paths: Vec<String>,
        #[arg(long, default_value = "md")]
        format: String,
        #[arg(long, default_value = "auto")]
        lang: String,
        #[arg(long, default_value = "oss")]
        profile: String,
        /// レポートの上限。severity の順に切り詰める
        #[arg(long)]
        budget: Option<usize>,
        /// exit code だけを返す
        #[arg(long)]
        quiet: bool,
    },
    /// コーパスから閾値を較正しプロファイルを生成する
    Baseline {
        paths: Vec<String>,
        #[arg(long, default_value = "auto")]
        lang: String,
        /// 生成するプロファイルの名前
        #[arg(long, default_value = "custom")]
        name: String,
        /// 出力先。省略時は標準出力に出す
        #[arg(long)]
        out: Option<String>,
    },
    /// 文書群から分野で確立した語を取り出し、たとえの判定から除く語彙集を作る
    Terms {
        paths: Vec<String>,
        #[arg(long, default_value = ".suikou/terms.toml")]
        out: String,
        #[arg(long, default_value = "auto")]
        lang: String,
        /// 出現回数の下限。1回しか出ない語は分野の語と偶然の言い回しを見分けられない
        #[arg(long, default_value_t = suikou_core::terms::DEFAULT_MIN_COUNT)]
        min_count: usize,
    },
    /// MCP サーバとして起動する
    Mcp,
    /// 常駐モードについて、置かないと決めた理由を出す
    Daemon,
    /// 書く前に構成を決めるための型枠を出す
    Plot {
        path: String,
        /// 文書の型。design、decision、howto、reference、overview
        #[arg(long, default_value = "design")]
        doctype: String,
        #[arg(long, default_value = "ja")]
        lang: String,
        /// 既にあるプロットを作り直す
        #[arg(long)]
        force: bool,
    },
    /// インストールされたバイナリが期待どおりかを確かめる
    Selftest,
}

/// 配布物の自己診断。
///
/// 辞書の有無を `#[cfg]` で表示するだけでは足りない。
/// 到達経路のない辞書はリンカに落とされるため、cfg は真でも実体がない場合がある。
/// 実際に読み込んで解析させ、動くことを確かめる。
fn selftest() -> Result<()> {
    use suikou_core::Lang;

    println!("版: {}", env!("CARGO_PKG_VERSION"));

    // 英語の経路は形態素解析器を必要としない。
    morphology::load(Lang::En)?;
    println!("英語: 解析できる");

    if morphology::HAS_DICTIONARY {
        let morph = morphology::load(Lang::Ja)?;
        let n = morph.tokenize("設定を変更し、確認する。").len();
        println!("日本語: 解析できる。辞書: UniDic 同梱（{n} トークン）");
    } else {
        println!("日本語: 解析できない。辞書: なし");
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Plot {
            path,
            doctype,
            lang,
            force,
        } => plot::run(plot::Options {
            path,
            doctype,
            lang,
            force,
        }),
        Command::Selftest => selftest(),
        Command::Brief {
            profile,
            lang,
            detail,
            doctype,
        } => {
            let text = brief::run(brief::Options {
                profile,
                lang,
                detail,
                doctype,
            })?;
            print!("{text}");
            Ok(())
        }
        Command::Check {
            paths,
            format,
            lang,
            profile,
            budget,
            quiet,
        } => {
            let code = check::run(check::Options {
                paths,
                format,
                lang,
                profile,
                budget,
                quiet,
            })?;
            std::process::exit(code);
        }
        Command::Baseline {
            paths,
            lang,
            name,
            out,
        } => baseline::run(baseline::Options {
            paths,
            lang,
            name,
            out,
        }),
        Command::Terms {
            paths,
            out,
            lang,
            min_count,
        } => terms::run(terms::Options {
            paths,
            out,
            lang,
            min_count,
        }),
        Command::Mcp => mcp::run(),
        Command::Daemon => {
            // T9 で、辞書の読み込み（0.7 ミリ秒）に続いてプロセス起動そのものの
            // 費用を測った。release ビルドで `check` が数ミリ秒、`--version` が
            // 1 ミリ秒台であり、hooks から Write や Edit のたびに起動する使い方でも
            // 頭打ちにならない。常駐の形を置く根拠がないため実装しない。
            // 測った値と判断の経緯は decisions.md の D-25 にある。
            println!(
                "daemon は用意しない。release ビルドでの起動は check で数ミリ秒、\n\
                 --version で1ミリ秒台に収まり、辞書の読み込みを合わせても\n\
                 hooks から都度起動する使い方で頭打ちにならないと測ってある。\n\
                 測った値と判断は docs/content/ja/decisions.md の D-25 にある。"
            );
            Ok(())
        }
    }
}
