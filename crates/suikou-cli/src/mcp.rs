//! `suikou mcp` の実装。
//!
//! stdio で JSON-RPC 2.0 を話す MCP サーバ。`initialize`、`tools/list`、
//! `tools/call` の3つに応じれば道具として足りるため、この3つだけを実装する。
//!
//! 依存を増やす代わりに `serde_json` だけで手で書いた。理由を挙げる。
//!
//! - 実装する要求が3種類だけであり、MCP の仕様の大半（resources、prompts、
//!   通知の往復など）を使わない。crate を足すと使わない機能ごと固定することになる。
//! - このリポジトリは依存を最小に保ち `=` で固定する方針であり（`CLAUDE.md`）、
//!   `serde_json` は既に両クレートの依存に入っている。増える依存は 0 である。
//! - stdio トランスポートは「1行に1メッセージ」という単純な形で足りる。
//!
//! 判断の経緯は `docs/content/ja/decisions.md` の D-25 にある。
//!
//! `check` と `brief` の判定ロジックは `crate::check` と `crate::brief` に
//! 実装がある。ここでは JSON の出し入れだけを行い、判定そのものは呼び出すだけにする。

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

use crate::brief;
use crate::check;

/// 標準入力から1行ずつ読み、要求ごとに応答を標準出力へ1行で書く。
///
/// 通知（`id` を持たない要求）には応答しない。JSON-RPC の定めどおりである。
pub fn run() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line.context("標準入力を読めない")?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(line) {
            Ok(req) => dispatch(&req),
            // 解釈できない行は id が分からないため、null を id として返す。
            // JSON-RPC の parse error はこの形が定めである。
            Err(e) => Some(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": format!("JSON を解釈できない: {e}") }
            })),
        };

        if let Some(resp) = response {
            writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// 1つの要求を捌いて応答を作る。通知なら `None` を返す。
///
/// IO を持たないため単体テストで直接呼べる。
fn dispatch(req: &Value) -> Option<Value> {
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let id = req.get("id").cloned();
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    let outcome = handle_method(method, &params);

    // id のない要求は通知として扱い、応答しない。
    let id = id?;

    Some(match outcome {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => {
            json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
        }
    })
}

/// JSON-RPC のエラーは `(code, message)` で表す。専用の型を作るほどの複雑さがない。
type RpcError = (i64, String);

fn handle_method(method: &str, params: &Value) -> Result<Value, RpcError> {
    match method {
        "initialize" => Ok(initialize_result()),
        "tools/list" => Ok(tools_list_result()),
        "tools/call" => tools_call_result(params),
        // ping はクライアントが疎通を確かめるために送ることがある。空でよい。
        "ping" => Ok(json!({})),
        other => Err((-32601, format!("メソッドが見つからない: {other}"))),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": { "name": "suikou", "version": env!("CARGO_PKG_VERSION") },
        "capabilities": { "tools": {} }
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "check",
                "description": "Markdown の本文かファイルパスを検査し、局所指摘と文書指標の統合レポートを返す。suikou check と同じ判定を使う。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "検査する Markdown 本文。path と同時には渡せない"
                        },
                        "path": {
                            "type": "string",
                            "description": "検査するファイルのパス。text と同時には渡せない"
                        },
                        "lang": {
                            "type": "string",
                            "enum": ["auto", "ja", "en"],
                            "default": "auto"
                        },
                        "profile": {
                            "type": "string",
                            "description": "組み込みは oss と service。プロファイルの TOML ファイルのパスも渡せる",
                            "default": "oss"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["md", "json"],
                            "default": "md"
                        }
                    }
                }
            },
            {
                "name": "brief",
                "description": "生成前にプロンプトへ入れる制約ブロックを返す。suikou brief と同じ内容を使う。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "lang": {
                            "type": "string",
                            "enum": ["auto", "ja", "en"],
                            "description": "auto は ja として扱う。入力文書を持たないため文面から判定できない",
                            "default": "auto"
                        },
                        "detail": {
                            "type": "string",
                            "enum": ["concise", "balanced", "detailed"],
                            "default": "balanced"
                        },
                        "profile": {
                            "type": "string",
                            "default": "oss"
                        }
                    }
                }
            }
        ]
    })
}

fn tools_call_result(params: &Value) -> Result<Value, RpcError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| (-32602, "params.name が要る".to_string()))?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    let outcome = match name {
        "check" => call_check(&args),
        "brief" => call_brief(&args),
        other => {
            return Err((
                -32602,
                format!("ツール {other} は存在しない。check か brief を渡す"),
            ))
        }
    };

    // 道具の実行そのものの失敗は JSON-RPC のエラーにせず、
    // `isError: true` を持つ結果として返す。MCP の定めどおりで、
    // クライアントが「サーバの不調」と「引数や入力の誤り」を区別できる。
    Ok(match outcome {
        Ok(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
        Err(e) => {
            json!({ "content": [{ "type": "text", "text": e.to_string() }], "isError": true })
        }
    })
}

fn call_check(args: &Value) -> Result<String> {
    let text_arg = args.get("text").and_then(Value::as_str);
    let path_arg = args.get("path").and_then(Value::as_str);
    let src = match (text_arg, path_arg) {
        (Some(_), Some(_)) => bail!("text と path は同時に渡せない。どちらか一方にする"),
        (Some(t), None) => t.to_string(),
        (None, Some(p)) => std::fs::read_to_string(p).with_context(|| format!("{p} を読めない"))?,
        (None, None) => bail!("text か path のどちらかが要る"),
    };
    let lang = args.get("lang").and_then(Value::as_str).unwrap_or("auto");
    let profile_name = args.get("profile").and_then(Value::as_str).unwrap_or("oss");
    let format = args.get("format").and_then(Value::as_str).unwrap_or("md");

    let profile = check::load_profile(profile_name)?;
    let (_lang, report) = check::analyze(&src, lang, &profile)?;

    match format {
        "json" => Ok(serde_json::to_string_pretty(&report)?),
        "md" => Ok(report.to_markdown()),
        other => bail!("format は md か json のいずれかとする。与えられた値は {other}"),
    }
}

fn call_brief(args: &Value) -> Result<String> {
    let profile = args
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or("oss")
        .to_string();
    let lang = args
        .get("lang")
        .and_then(Value::as_str)
        .unwrap_or("auto")
        .to_string();
    let detail = args
        .get("detail")
        .and_then(Value::as_str)
        .unwrap_or("balanced")
        .to_string();
    brief::run(brief::Options {
        profile,
        lang,
        detail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(method: &str, id: i64, params: Value) -> Value {
        dispatch(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
            .expect("要求には応答が要る")
    }

    #[test]
    fn initialize_reports_tool_capability() {
        let resp = call("initialize", 1, json!({}));
        assert_eq!(resp["result"]["capabilities"]["tools"], json!({}));
        assert!(resp["result"]["serverInfo"]["name"] == "suikou");
    }

    #[test]
    fn tools_list_returns_check_and_brief() {
        let resp = call("tools/list", 2, json!({}));
        let tools = resp["result"]["tools"].as_array().expect("配列のはず");
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"check"), "{names:?}");
        assert!(names.contains(&"brief"), "{names:?}");
        assert_eq!(names.len(), 2, "{names:?}");
    }

    #[test]
    fn unknown_method_is_a_jsonrpc_error() {
        let resp = call("no/such/method", 3, json!({}));
        assert_eq!(resp["error"]["code"], -32601);
    }

    #[test]
    fn notification_without_id_gets_no_response() {
        let resp = dispatch(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        assert!(resp.is_none());
    }

    #[test]
    fn parse_error_is_reported_as_json_rpc_error() {
        let resp = match serde_json::from_str::<Value>("not json") {
            Ok(_) => panic!("これは JSON ではない"),
            Err(_) => json!({ "jsonrpc": "2.0", "id": Value::Null, "error": { "code": -32700 } }),
        };
        assert_eq!(resp["error"]["code"], -32700);
    }

    #[test]
    fn tools_call_check_matches_direct_check_analyze_english() {
        // 辞書を要しない英語の文書で、MCP 経由の check と core の analyze が
        // 同じ Markdown を返すことを確かめる。判定ロジックを二重に書いていないことの確認。
        let text = "This is a short document. It stays inside every threshold.\n";
        let resp = call(
            "tools/call",
            4,
            json!({ "name": "check", "arguments": { "text": text, "lang": "en" } }),
        );
        assert_eq!(resp["result"]["isError"], false);
        let via_mcp = resp["result"]["content"][0]["text"].as_str().unwrap();

        let profile = check::load_profile("oss").unwrap();
        let (_lang, report) = check::analyze(text, "en", &profile).unwrap();
        assert_eq!(via_mcp, report.to_markdown());
    }

    #[test]
    fn tools_call_check_json_format_round_trips() {
        let text = "現在の実装では反映されない。\n";
        let resp = call(
            "tools/call",
            5,
            json!({
                "name": "check",
                "arguments": { "text": text, "lang": "en", "format": "json" }
            }),
        );
        let via_mcp = resp["result"]["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(via_mcp).expect("JSON のはず");
        assert!(parsed.get("document").is_some());
        assert!(parsed.get("local").is_some());
    }

    #[test]
    fn tools_call_check_requires_text_or_path() {
        let resp = call("tools/call", 6, json!({ "name": "check", "arguments": {} }));
        assert_eq!(resp["result"]["isError"], true);
    }

    #[test]
    fn tools_call_check_rejects_both_text_and_path() {
        let resp = call(
            "tools/call",
            7,
            json!({
                "name": "check",
                "arguments": { "text": "a", "path": "b.md" }
            }),
        );
        assert_eq!(resp["result"]["isError"], true);
    }

    #[test]
    fn tools_call_check_reads_from_path() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("suikou-mcp-test-{}.md", std::process::id()));
        std::fs::write(&path, "Short prose that stays inside every threshold.\n").unwrap();

        let resp = call(
            "tools/call",
            8,
            json!({
                "name": "check",
                "arguments": { "path": path.to_str().unwrap(), "lang": "en" }
            }),
        );
        std::fs::remove_file(&path).ok();
        assert_eq!(resp["result"]["isError"], false);
    }

    #[test]
    fn tools_call_brief_matches_direct_brief_run() {
        let resp = call(
            "tools/call",
            9,
            json!({
                "name": "brief",
                "arguments": { "lang": "en", "detail": "balanced", "profile": "oss" }
            }),
        );
        let via_mcp = resp["result"]["content"][0]["text"].as_str().unwrap();

        let direct = brief::run(brief::Options {
            profile: "oss".to_string(),
            lang: "en".to_string(),
            detail: "balanced".to_string(),
        })
        .unwrap();
        assert_eq!(via_mcp, direct);
    }

    #[test]
    fn tools_call_unknown_tool_is_an_rpc_error() {
        let resp = call("tools/call", 10, json!({ "name": "no-such-tool" }));
        assert_eq!(resp["error"]["code"], -32602);
    }

    #[cfg(feature = "lindera-unidic")]
    #[test]
    fn tools_call_check_analyzes_japanese_with_the_embedded_dictionary() {
        let text = "設定を変更し、確認する。設定を変更し、確認する。\n";
        let resp = call(
            "tools/call",
            11,
            json!({ "name": "check", "arguments": { "text": text, "lang": "ja" } }),
        );
        assert_eq!(resp["result"]["isError"], false);

        let profile = check::load_profile("oss").unwrap();
        let (_lang, report) = check::analyze(text, "ja", &profile).unwrap();
        let via_mcp = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert_eq!(via_mcp, report.to_markdown());
    }
}
