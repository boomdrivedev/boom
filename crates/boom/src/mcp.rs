//! `boom mcp` (agent-tools; record 0009): boom served over the Model
//! Context Protocol on standard input and output, one JSON-RPC message a
//! line, to an agent's program.
//!
//! The design is `flume mcp`'s, which flume owns and boom cites (flume's
//! `crates/flume/src/commands/mcp.rs`, its clause 60 and record 0069): a
//! loop over `serde_json`, each answer one line, flushed before the next
//! line is read, with no protocol crate (core.6).
//!
//! Its tools are the commands an agent may run, each read from the
//! command line's own definition (design 3): no tool exists without its
//! command, and every command boom has is served or named absent.

use std::io::{BufRead, Write};
use std::path::Path;

use clap::{Arg, ArgAction};
use serde_json::{json, Value};

use crate::cli::Env;

/// Each command served as a tool, by its words (design 3).
pub(crate) const SERVED: &[&str] = &[
    "propose",
    "show",
    "log",
    "query",
    "import adr",
    "import rules",
];

/// Each command no tool serves, by its words, with why (design 3).
pub(crate) const ABSENT: &[(&str, &str)] = &[
    (
        "init",
        "making a store is its first owner's act, signed with their security key",
    ),
    (
        "register",
        "registering a person is an owner's act, signed with their security key",
    ),
    (
        "accept",
        "only a person accepts, in their own terminal, signing with their security key",
    ),
    (
        "reject",
        "only a person rejects, in their own terminal, signing with their security key",
    ),
    ("mcp", "it is the server itself"),
];

/// The arguments of a served command no tool takes, with why (design
/// 3, 5).
pub(crate) const LEFT_OUT: &[(&str, &str)] = &[(
    "key",
    "a person signs with their key in their own terminal, and boom mcp signs nothing",
)];

/// The command line's global arguments, which no tool takes (design 4,
/// 6).
const GLOBAL: &[(&str, &str)] = &[
    (
        "store",
        "each tool runs on the store boom mcp was started with",
    ),
    ("json", "each tool answers its command's JSON"),
];

/// The tools that only read, as flume marks `status`.
const READ_ONLY: &[&str] = &["show", "log", "query"];

/// What a tool's property is, read from its argument's action: a switch
/// a boolean, a flag taken again and again an array of strings, any
/// other a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Switch,
    Many,
    One,
}

fn shape(arg: &Arg) -> Shape {
    match arg.get_action() {
        ArgAction::SetTrue => Shape::Switch,
        ArgAction::Append => Shape::Many,
        _ => Shape::One,
    }
}

/// The command `words` names in `definition`.
fn leaf<'a>(definition: &'a clap::Command, words: &str) -> Option<&'a clap::Command> {
    words
        .split(' ')
        .try_fold(definition, |command, word| command.find_subcommand(word))
}

/// The arguments of `command` its tool takes: all but [`LEFT_OUT`], the
/// global ones and clap's own help and version.
fn arguments(command: &clap::Command) -> impl Iterator<Item = &Arg> {
    command.get_arguments().filter(|arg| {
        let id = arg.get_id().as_str();
        !matches!(
            arg.get_action(),
            ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
        ) && !LEFT_OUT.iter().chain(GLOBAL).any(|(name, _)| *name == id)
    })
}

/// A served command as a tool.
struct Tool {
    /// Its command's words joined by `_`.
    name: String,
    /// Its command's words.
    words: &'static str,
    /// Its command, as the command line defines it.
    command: clap::Command,
}

/// The tools, one for each command served, read from the command line's
/// definition.
fn tools() -> Vec<Tool> {
    let definition = crate::cli::definition();
    SERVED
        .iter()
        .map(|&words| Tool {
            name: words.replace(' ', "_"),
            words,
            command: leaf(&definition, words)
                .expect("each command served is one the command line defines")
                .clone(),
        })
        .collect()
}

/// A tool as `tools/list` gives it: its name, its command's `about`, and
/// an input schema of its command's arguments by their clap ids.
fn listed(tool: &Tool) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for arg in arguments(&tool.command) {
        let id = arg.get_id().as_str();
        let mut property = match shape(arg) {
            Shape::Switch => json!({"type": "boolean"}),
            Shape::Many => json!({"type": "array", "items": {"type": "string"}}),
            Shape::One => {
                let values: Vec<String> = arg
                    .get_possible_values()
                    .iter()
                    .filter(|value| !value.is_hide_set())
                    .map(|value| value.get_name().to_owned())
                    .collect();
                if values.is_empty() {
                    json!({"type": "string"})
                } else {
                    json!({"type": "string", "enum": values})
                }
            }
        };
        if let Some(help) = arg.get_help() {
            property["description"] = json!(help.to_string());
        }
        properties.insert(id.to_owned(), property);
        if arg.is_required_set() {
            required.push(id.to_owned());
        }
    }
    json!({
        "name": tool.name,
        "description": tool.command.get_about().map(ToString::to_string).unwrap_or_default(),
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
        },
        "annotations": {"readOnlyHint": READ_ONLY.contains(&tool.words)},
    })
}

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

/// Why no tool is named `name`, with why where boom leaves its command
/// out.
fn no_tool(name: &str) -> String {
    match ABSENT
        .iter()
        .find(|(words, _)| words.replace(' ', "_") == name)
    {
        Some((words, why)) => {
            format!("no tool is named {name}: boom mcp serves no {words}, as {why}")
        }
        None => format!("no tool is named {name}"),
    }
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
        "tools/list" => json!({"tools": tools().iter().map(listed).collect::<Vec<_>>()}),
        "tools/call" => {
            let name = &request["params"]["name"];
            let name = name
                .as_str()
                .map_or_else(|| name.to_string(), str::to_owned);
            return Some(error(id, -32602, no_tool(&name)));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every command the command line defines, by its words, down to its
    /// leaves.
    fn leaves(command: &clap::Command, above: &str, found: &mut Vec<String>) {
        for sub in command.get_subcommands() {
            let words = if above.is_empty() {
                sub.get_name().to_owned()
            } else {
                format!("{above} {}", sub.get_name())
            };
            if sub.has_subcommands() {
                leaves(sub, &words, found);
            } else {
                found.push(words);
            }
        }
    }

    #[test]
    fn each_command_served_or_absent() {
        let mut found = Vec::new();
        leaves(&crate::cli::definition(), "", &mut found);
        assert!(found.contains(&"import adr".to_owned()), "{found:?}");
        for words in &found {
            let served = SERVED.contains(&words.as_str());
            let absent = ABSENT.iter().any(|(absent, _)| absent == words);
            assert!(
                served ^ absent,
                "boom {words} is to be served or named absent, never both: served {served}, absent {absent}"
            );
        }
        for words in SERVED.iter().chain(ABSENT.iter().map(|(words, _)| words)) {
            assert!(
                found.contains(&(*words).to_owned()),
                "{words} is no command boom has"
            );
        }
    }

    #[test]
    fn each_argument_of_a_served_command_is_its_tools() {
        let definition = crate::cli::definition();
        let tools = tools();
        for words in SERVED {
            let command = leaf(&definition, words).unwrap();
            let tool = tools.iter().find(|tool| tool.words == *words).unwrap();
            let listed = listed(tool);
            let properties = listed["inputSchema"]["properties"].as_object().unwrap();
            let mut expected = Vec::new();
            for arg in command.get_arguments() {
                let id = arg.get_id().as_str();
                if GLOBAL.iter().any(|(name, _)| *name == id) {
                    continue;
                }
                if LEFT_OUT.iter().any(|(name, _)| *name == id) {
                    assert!(!properties.contains_key(id), "{words} takes {id}");
                    continue;
                }
                assert!(
                    properties.contains_key(id),
                    "the tool {} lacks {id}",
                    tool.name
                );
                expected.push(id.to_owned());
            }
            let mut taken: Vec<String> = properties.keys().cloned().collect();
            taken.sort();
            expected.sort();
            assert_eq!(taken, expected, "the tool {}'s arguments", tool.name);
        }
    }
}
