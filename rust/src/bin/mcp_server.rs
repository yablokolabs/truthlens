use std::io::{self, BufRead, Write};
use truthlens::mcp::{JsonRpcRequest, failure, handle_request};

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

        if let Some(resp) = handle_request(req) {
            writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap()).unwrap();
            stdout.flush().unwrap();
        }
    }
}
