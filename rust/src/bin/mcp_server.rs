use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Write};
use truthlens::{analyze, analyze_with_verification};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

fn success(id: Option<Value>, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

fn failure(id: Option<Value>, code: i32, message: impl Into<String>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.into(),
        }),
    }
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "analyze_text",
            "description": "Analyze AI-generated or general text for claim trust/risk.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "Text to analyze"},
                    "verify": {"type": "boolean", "description": "Enable entity verification"}
                },
                "required": ["text"]
            }
        },
        {
            "name": "analyze_file",
            "description": "Read a local text file and analyze its contents.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to a UTF-8 text file"},
                    "verify": {"type": "boolean", "description": "Enable entity verification"}
                },
                "required": ["path"]
            }
        }
    ])
}

fn handle_call_tool(name: &str, arguments: Option<&Value>) -> Result<Value, String> {
    match name {
        "analyze_text" => {
            let args = arguments.ok_or("Missing arguments")?;
            let text = args
                .get("text")
                .and_then(Value::as_str)
                .ok_or("Missing required field: text")?;
            let verify = args.get("verify").and_then(Value::as_bool).unwrap_or(false);
            let report = if verify {
                analyze_with_verification(text)
            } else {
                analyze(text)
            };
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&report).unwrap()
                    }
                ],
                "structuredContent": report
            }))
        }
        "analyze_file" => {
            let args = arguments.ok_or("Missing arguments")?;
            let path = args
                .get("path")
                .and_then(Value::as_str)
                .ok_or("Missing required field: path")?;
            let verify = args.get("verify").and_then(Value::as_bool).unwrap_or(false);
            let md = fs::metadata(path).map_err(|e| format!("Failed to stat file: {e}"))?;
            if !md.is_file() {
                return Err("Path is not a regular file".to_string());
            }
            if md.len() > 1024 * 1024 {
                return Err("File too large (>1MB)".to_string());
            }
            let text = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read file as UTF-8 text: {e}"))?;
            let report = if verify {
                analyze_with_verification(&text)
            } else {
                analyze(&text)
            };
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&report).unwrap()
                    }
                ],
                "structuredContent": report,
                "path": path
            }))
        }
        _ => Err(format!("Unknown tool: {name}")),
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            Ok(_) => continue,
            Err(e) => {
                eprintln!("stdin read error: {e}");
                break;
            }
        };

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = failure(None, -32700, format!("Parse error: {e}"));
                writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap()).unwrap();
                stdout.flush().unwrap();
                continue;
            }
        };

        let id = req.id.clone();
        let resp = match req.method.as_str() {
            "initialize" => success(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "truthlens-mcp",
                        "version": "0.6.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                }),
            ),
            "notifications/initialized" => continue,
            "tools/list" => success(id, json!({ "tools": tool_definitions() })),
            "tools/call" => {
                let params = req
                    .params
                    .as_ref()
                    .ok_or("Missing params")
                    .map_err(|e| e.to_string());
                match params {
                    Ok(params) => {
                        let name = params.get("name").and_then(Value::as_str);
                        match name {
                            Some(name) => match handle_call_tool(name, params.get("arguments")) {
                                Ok(result) => success(id, result),
                                Err(msg) => failure(id, -32000, msg),
                            },
                            None => failure(id, -32602, "Missing tool name"),
                        }
                    }
                    Err(msg) => failure(id, -32602, msg),
                }
            }
            _ => failure(id, -32601, format!("Method not found: {}", req.method)),
        };

        writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap()).unwrap();
        stdout.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn tools_list_contains_expected_tools() {
        let tools = tool_definitions();
        let arr = tools.as_array().expect("tools should be an array");
        let names: Vec<&str> = arr
            .iter()
            .filter_map(|t| t.get("name").and_then(Value::as_str))
            .collect();
        assert!(names.contains(&"analyze_text"));
        assert!(names.contains(&"analyze_file"));
    }

    #[test]
    fn analyze_text_returns_structured_content() {
        let result = handle_call_tool(
            "analyze_text",
            Some(&json!({"text": "Einstein was born in 1879 in Ulm."})),
        )
        .expect("analyze_text should succeed");
        assert!(result.get("structuredContent").is_some());
        let content = result
            .get("content")
            .and_then(Value::as_array)
            .expect("content array expected");
        assert!(!content.is_empty());
    }

    #[test]
    fn analyze_file_rejects_missing_file() {
        let result = handle_call_tool(
            "analyze_file",
            Some(&json!({"path": "/definitely/not/a/real/file.txt"})),
        );
        assert!(result.is_err());
    }

    #[test]
    fn analyze_file_reads_text_file() {
        let tmp_name = format!(
            "/tmp/truthlens-mcp-test-{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        fs::write(&tmp_name, "This claim might be true.").unwrap();
        let result = handle_call_tool("analyze_file", Some(&json!({"path": tmp_name})))
            .expect("analyze_file should succeed for text file");
        assert!(result.get("structuredContent").is_some());
    }
}
