//! `boom mcp` (agent-tools; record 0011): boom served over the Model
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

use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::path::Path;

use clap::{Arg, ArgAction};
use serde_json::{json, Map, Value};

use crate::cli::Env;
use crate::sign::{Key, Keys, Signer};
use crate::Failure;

/// Each command served as a tool, by its words (design 3).
pub(crate) const SERVED: &[&str] = &[
    "propose",
    "record change",
    "show",
    "log",
    "lineage",
    "query",
    "search",
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
    (
        "verify",
        "it checks the whole log and builds the views again, a check a person or a host's schedule runs",
    ),
    (
        "repo add",
        "registering a repository is an owner's act, signed with their security key",
    ),
    (
        "repo set",
        "changing a repository's settings is an owner's act, signed with their security key",
    ),
    (
        "repo show",
        "a person reads a store's repositories from the command line in this release",
    ),
    (
        "terms check",
        "the hook of the plugin runs it on what an agent writes, and the gate on what a change touches",
    ),
    (
        "approve",
        "only a person approves a plan, in their own terminal, signing with their security key",
    ),
    (
        "append",
        "an entry of another tool's type is that tool's act, which the tool appends from its own program",
    ),
    (
        "withdraw",
        "an author withdraws its own proposal from the command line in this release",
    ),
    (
        "supersede",
        "an agent proposes a supersession from the command line in this release",
    ),
    ("mcp", "it is the server itself"),
    (
        "hook session-start",
        "the harness of an agent runs it when a session starts, and an agent does not call a hook",
    ),
    (
        "hook pre-tool-use",
        "the harness of an agent runs it before an edit, and an agent does not call a hook",
    ),
    (
        "note",
        "a note is the act of a person, on the page or with boom note in their own terminal; only a person writes a note",
    ),
    (
        "page",
        "it serves a page until it is stopped, which no call of a tool can do; a person opens it with boom page",
    ),
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

/// What `initialize` says waits on a person (design 7).
pub const INSTRUCTIONS: &str =
    "boom keeps this project's claims. Its tools propose claims and read \
what stands; none accepts, rejects or registers. Only a person accepts \
or rejects, in their own terminal, signing with their security key: \
boom accept <name>@<version> --key <key>, or boom reject \
<name>@<version> --reason <why> --key <key>. Only a person writes a note \
on a paragraph of a narrative, on the page that boom page serves or with \
boom note; a new version of the narrative answers each open note, with \
propose and its answers.";

/// The tools that only read, as flume marks `status`.
const READ_ONLY: &[&str] = &["show", "log", "lineage", "query", "search"];

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

/// The key source every command the server runs is given: it signs
/// nothing, so no call reaches a person's `ssh-keygen` or sets off their
/// key's prompt, even by an argument a later change lets through
/// (design 5).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoKeys;

impl Keys for NoKeys {
    fn signer(&self, key: &Key) -> Result<Box<dyn Signer + '_>, Failure> {
        Err(Failure::refused(format!(
            "boom mcp signs nothing, with {} or any key: a person accepts, rejects or registers in their own terminal",
            key.path.display()
        )))
    }
}

/// The command line's words for a call to `tool` with `given` (design
/// 4): its command's words; each option as `--<long>=<value>`, a flag
/// taken again and again once for each item, a true switch as `--<long>`
/// and a false one left out; `--json`; `--store=<path>` where `boom mcp`
/// was given one; `--`; and the positional values in their order. The
/// `=` and the `--` keep a value beginning with a hyphen from being read
/// as a switch.
fn words(tool: &Tool, given: &Map<String, Value>, store: Option<&Path>) -> Vec<OsString> {
    let mut words: Vec<OsString> = tool.words.split(' ').map(OsString::from).collect();
    let mut positional = Vec::new();
    for arg in arguments(&tool.command) {
        let Some(value) = given.get(arg.get_id().as_str()) else {
            continue;
        };
        let values: Vec<&str> = match value {
            Value::String(value) => vec![value.as_str()],
            Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        if arg.is_positional() {
            positional.extend(values);
            continue;
        }
        let long = arg.get_long().unwrap_or_else(|| arg.get_id().as_str());
        if shape(arg) == Shape::Switch {
            if value == &Value::Bool(true) {
                words.push(format!("--{long}").into());
            }
            continue;
        }
        for value in values {
            words.push(format!("--{long}={value}").into());
        }
    }
    words.push("--json".into());
    if let Some(store) = store {
        let mut word = OsString::from("--store=");
        word.push(store);
        words.push(word);
    }
    words.push("--".into());
    words.extend(positional.into_iter().map(OsString::from));
    words
}

/// A call's result: the command's JSON object as one text item, an
/// error where it refused or could not run.
fn result(text: &str, error: bool) -> Value {
    json!({"content": [{"type": "text", "text": text}], "isError": error})
}

/// A call's arguments, `raw`, held to `tool`'s schema before anything runs
/// (design 4): absent read as `{}`, and refused where they are not an
/// object, name an argument the tool does not take, or give one a value
/// of another type than its property's. The command line's own refusal
/// of a flag it does not take, said in the tool's terms.
fn checked(tool: &Tool, raw: &Value) -> Result<Map<String, Value>, String> {
    let given = match raw {
        Value::Null => return Ok(Map::new()),
        Value::Object(given) => given,
        _ => {
            return Err(format!(
                "the tool {} takes its arguments as an object, not {}",
                tool.name,
                type_words(raw)
            ))
        }
    };
    for (name, value) in given {
        let Some(arg) = arguments(&tool.command).find(|arg| arg.get_id() == name) else {
            let why = LEFT_OUT
                .iter()
                .chain(GLOBAL)
                .find(|(left_out, _)| left_out == name)
                .map_or(String::new(), |(_, why)| format!(": {why}"));
            return Err(format!(
                "the tool {} takes no argument {name}{why}",
                tool.name
            ));
        };
        let (fits, asked) = match shape(arg) {
            Shape::Switch => (value.is_boolean(), "a boolean"),
            Shape::Many => (
                value
                    .as_array()
                    .is_some_and(|items| items.iter().all(Value::is_string)),
                "an array of strings",
            ),
            Shape::One => (value.is_string(), "a string"),
        };
        if !fits {
            return Err(format!(
                "the tool {} takes {name} as {asked}, not {}",
                tool.name,
                type_words(value)
            ));
        }
    }
    Ok(given.clone())
}

/// A JSON value's type, in words.
fn type_words(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(items) if items.iter().all(Value::is_string) => "an array of strings",
        Value::Array(_) => "an array holding other than strings",
        Value::Object(_) => "an object",
    }
}

/// Runs `tool`'s command with the arguments `raw`, once they hold to its schema,
/// through the command line's own parser and code, giving it [`NoKeys`].
fn call(tool: &Tool, raw: &Value, store: Option<&Path>, env: &Env) -> Value {
    let given = match checked(tool, raw) {
        Ok(given) => given,
        Err(why) => {
            let refused = crate::cli::failed(true, Failure::Refused(why));
            return result(refused.stdout.trim(), true);
        }
    };
    let outcome = crate::cli::run_with_input(
        words(tool, &given, store),
        &NoKeys,
        env,
        &mut std::io::empty(),
    );
    result(outcome.stdout.trim(), outcome.code != 0)
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
fn answer(request: &Value, store: Option<&Path>, env: &Env) -> Option<Value> {
    let id = request.get("id")?.clone();
    let method = request["method"].as_str().unwrap_or_default();
    let result = match method {
        "initialize" => json!({
            "protocolVersion": request["params"]["protocolVersion"]
                .as_str()
                .unwrap_or(PROTOCOL),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "boom", "version": env!("CARGO_PKG_VERSION")},
            "instructions": INSTRUCTIONS,
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tools().iter().map(listed).collect::<Vec<_>>()}),
        "tools/call" => {
            let name = &request["params"]["name"];
            if let Some(tool) = tools().iter().find(|tool| name == tool.name.as_str()) {
                return Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": call(tool, &request["params"]["arguments"], store, env),
                }));
            }
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
    fn the_words_for_a_call() {
        let tools = tools();
        let propose = tools.iter().find(|tool| tool.name == "propose").unwrap();
        let given = json!({
            "kind": "invariant",
            "name": "money-has-currency",
            "statement": "-0 is never an amount.",
            "provenance": "run-1",
            "repositories": ["shop", "--ledger"],
        });
        let words = words(
            propose,
            given.as_object().unwrap(),
            Some(Path::new("/stores/boom.sqlite")),
        );
        let words: Vec<&str> = words.iter().map(|word| word.to_str().unwrap()).collect();
        assert_eq!(
            words,
            [
                "propose",
                "--provenance=run-1",
                "--repository=shop",
                "--repository=--ledger",
                "--json",
                "--store=/stores/boom.sqlite",
                "--",
                "invariant",
                "money-has-currency",
                "-0 is never an amount.",
            ]
        );
        let unset = json!({"kind": "invariant", "system": false});
        let words = super::words(propose, unset.as_object().unwrap(), None);
        assert_eq!(words, ["propose", "--json", "--", "invariant"]);
    }

    #[test]
    fn the_server_holds_no_key() {
        for key in ["/home/ada/.ssh/id_ed25519_sk", "keys/ada", ""] {
            match NoKeys.signer(&Key::at(key)) {
                Err(Failure::Refused(why)) => {
                    assert!(why.contains("boom mcp signs nothing"), "{why}");
                    assert!(why.contains("their own terminal"), "{why}");
                }
                Err(other) => panic!("{other:?}"),
                Ok(_) => panic!("NoKeys gave a signer for {key}"),
            }
        }
        // A command that reaches for a key is refused before it writes.
        let dir = tempfile::tempdir().unwrap();
        let store = dir.path().join("boom.sqlite");
        let outcome = crate::cli::run(
            [
                "init",
                "--person",
                "ada",
                "--key",
                "keys/ada",
                "--json",
                "--store",
                store.to_str().unwrap(),
            ],
            &NoKeys,
            &Env::default(),
        );
        assert_eq!(outcome.code, 2, "{outcome:?}");
        assert!(
            outcome.stdout.contains("boom mcp signs nothing"),
            "{outcome:?}"
        );
        assert!(!store.exists(), "no store is made");
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
