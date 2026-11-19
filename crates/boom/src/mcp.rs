//! `boom mcp` (agent-tools; record 0009): boom served over the Model
//! Context Protocol on standard input and output, one JSON-RPC message a
//! line, to an agent's program.
//!
//! The design is `flume mcp`'s, which flume owns and boom cites (flume's
//! `crates/flume/src/commands/mcp.rs`, its clause 60 and record 0069): a
//! loop over `serde_json`, each answer one line, flushed before the next
//! line is read, with no protocol crate (core.6).

use std::io::{BufRead, Write};
use std::path::Path;

use serde_json::{json, Value};

use crate::cli::Env;

/// The protocol's version the server answers where the client names
/// none, as flume's does.
pub const PROTOCOL: &str = "2025-06-18";

/// A JSON-RPC error answering `id`.
fn error(id: Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message},
    })
}

/// The answer to one request, none for a notification.
fn answer(request: &Value, _store: Option<&Path>, _env: &Env) -> Option<Value> {
    let id = request.get("id")?.clone();
    let method = request["method"].as_str().unwrap_or_default();
    let result = match method {
        "initialize" => json!({
            "protocolVersion": request["params"]["protocolVersion"]
                .as_str()
                .unwrap_or(PROTOCOL),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "boom", "version": env!("CARGO_PKG_VERSION")},
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": []}),
        "tools/call" => {
            let name = &request["params"]["name"];
            let name = name
                .as_str()
                .map_or_else(|| name.to_string(), str::to_owned);
            return Some(error(id, -32602, format!("no tool is named {name}")));
        }
        _ => return Some(error(id, -32601, format!("no method {method}"))),
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

/// Serves requests read from `input`, one a line, answering each on
/// `output` before reading the next, until `input` ends or `output`
/// closes. Each tool runs on `store`, where `boom mcp` was given one,
/// with `env`.
pub fn serve(input: impl BufRead, mut output: impl Write, store: Option<&Path>, env: &Env) {
    for line in input.lines() {
        let Ok(line) = line else {
            return;
        };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => answer(&request, store, env),
            Err(parse) => Some(error(Value::Null, -32700, parse.to_string())),
        };
        if let Some(reply) = reply {
            if writeln!(output, "{reply}")
                .and_then(|()| output.flush())
                .is_err()
            {
                return;
            }
        }
    }
}
