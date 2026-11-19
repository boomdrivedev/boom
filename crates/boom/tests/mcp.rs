//! `boom mcp`: boom's commands served to an agent's program as tools
//! over the Model Context Protocol, none of them a person's act
//! (agent-tools; record 0009).

mod support;

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::{json, Value};
use support::{initialize, request, Place};

#[test]
fn initialized() {
    let place = Place::new();
    let answers = place.mcp(&[
        initialize(1),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        request(2, "ping", json!({})),
    ]);
    assert_eq!(answers.len(), 2, "{answers:?}");
    let first = &answers[0];
    assert_eq!(first["jsonrpc"], "2.0");
    assert_eq!(first["id"], 1);
    assert_eq!(first["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(first["result"]["capabilities"]["tools"], json!({}));
    assert_eq!(first["result"]["serverInfo"]["name"], "boom");
    assert_eq!(
        first["result"]["serverInfo"]["version"],
        env!("CARGO_PKG_VERSION")
    );
    assert_eq!(answers[1]["id"], 2);
    assert_eq!(answers[1]["result"], json!({}));
}

#[test]
fn a_line_that_is_not_json() {
    let place = Place::new();
    let answers = place.mcp_lines(
        place.dir.path(),
        &[
            "this is not json".to_owned(),
            request(1, "ping", json!({})).to_string(),
        ],
    );
    assert_eq!(answers.len(), 2, "{answers:?}");
    assert_eq!(answers[0]["error"]["code"], -32700, "{answers:?}");
    assert_eq!(answers[0]["id"], Value::Null, "{answers:?}");
    assert_eq!(answers[1]["id"], 1);
    assert_eq!(answers[1]["result"], json!({}));
}

#[test]
fn a_method_it_does_not_serve() {
    let place = Place::new();
    let answers = place.mcp(&[request(1, "resources/list", json!({}))]);
    assert_eq!(answers.len(), 1, "{answers:?}");
    assert_eq!(answers[0]["id"], 1);
    assert_eq!(answers[0]["error"]["code"], -32601, "{answers:?}");
    let message = answers[0]["error"]["message"].as_str().unwrap();
    assert!(message.contains("resources/list"), "{message}");
}

#[test]
fn asked_for_one_json_object() {
    let place = Place::new();
    let outcome = place.run(&["mcp", "--json"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-refused/1");
    let reason = answer["refused"]["reason"].as_str().unwrap();
    assert!(
        reason.contains("boom mcp answers in the protocol's messages"),
        "{reason}"
    );
}

#[test]
fn one_answer_before_the_next_request() {
    let place = Place::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_boom"))
        .args(["mcp", "--store", &place.store_arg()])
        .env("XDG_DATA_HOME", place.dir.path().join("data"))
        .env("HOME", place.dir.path().join("home"))
        .current_dir(place.dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the binary runs");
    let mut stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    // Each line the server writes is read on a thread of its own, so a
    // server holding its answers fails the wait below rather than hangs.
    let (lines, read) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in stdout.lines() {
            if lines.send(line.unwrap()).is_err() {
                return;
            }
        }
    });
    let within = Duration::from_secs(10);
    writeln!(stdin, "{}", initialize(1)).unwrap();
    stdin.flush().unwrap();
    let first: Value = serde_json::from_str(
        &read
            .recv_timeout(within)
            .expect("boom mcp answers initialize within ten seconds, before its input closes"),
    )
    .unwrap();
    assert_eq!(first["id"], 1, "{first}");
    assert_eq!(first["result"]["serverInfo"]["name"], "boom", "{first}");
    writeln!(stdin, "{}", request(2, "tools/list", json!({}))).unwrap();
    stdin.flush().unwrap();
    let second: Value = serde_json::from_str(
        &read
            .recv_timeout(within)
            .expect("boom mcp answers tools/list within ten seconds, before its input closes"),
    )
    .unwrap();
    assert_eq!(second["id"], 2, "{second}");
    assert!(second["result"]["tools"].is_array(), "{second}");
    drop(stdin);
    let status = child.wait().unwrap();
    reader.join().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(read.try_recv().is_err(), "boom mcp writes nothing else");
}

/// The tools `tools/list` gives.
fn tools(place: &Place) -> Vec<Value> {
    let answers = place.mcp(&[initialize(1), request(2, "tools/list", json!({}))]);
    answers[1]["result"]["tools"].as_array().unwrap().clone()
}

/// The tool `name` among `tools`.
fn tool<'a>(tools: &'a [Value], name: &str) -> &'a Value {
    tools
        .iter()
        .find(|tool| tool["name"] == name)
        .unwrap_or_else(|| panic!("no tool {name}"))
}

/// A tool's arguments, sorted, and the ones it requires, sorted.
fn arguments(tool: &Value) -> (Vec<String>, Vec<String>) {
    let mut taken: Vec<String> = tool["inputSchema"]["properties"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    taken.sort();
    let mut required: Vec<String> = tool["inputSchema"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().to_owned())
        .collect();
    required.sort();
    (taken, required)
}

fn names(names: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
    names.sort();
    names
}

#[test]
fn the_tools_listed() {
    let place = Place::new();
    let tools = tools(&place);
    let mut listed: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    listed.sort_unstable();
    assert_eq!(
        listed,
        [
            "import_adr",
            "import_rules",
            "log",
            "propose",
            "query",
            "show"
        ]
    );
    for tool in &tools {
        assert!(!tool["description"].as_str().unwrap().is_empty(), "{tool}");
        assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
        assert_eq!(tool["inputSchema"]["additionalProperties"], false, "{tool}");
    }
}

#[test]
fn a_tools_arguments_are_its_commands() {
    let place = Place::new();
    let tools = tools(&place);
    let propose = tool(&tools, "propose");
    assert_eq!(
        arguments(propose),
        (
            names(&[
                "kind",
                "name",
                "statement",
                "scenarios",
                "amends",
                "body",
                "provenance",
                "repositories",
                "system"
            ]),
            names(&["kind", "name", "statement"])
        )
    );
    let properties = &propose["inputSchema"]["properties"];
    assert_eq!(
        properties["kind"]["enum"],
        json!(["invariant", "decision", "assumption"])
    );
    assert_eq!(properties["scenarios"]["type"], "array");
    assert_eq!(properties["scenarios"]["items"]["type"], "string");
    assert_eq!(properties["system"]["type"], "boolean");
    assert_eq!(properties["statement"]["type"], "string");
    assert_eq!(
        arguments(tool(&tools, "show")),
        (names(&["target"]), names(&["target"]))
    );
    assert_eq!(arguments(tool(&tools, "log")), (vec![], vec![]));
    assert_eq!(
        arguments(tool(&tools, "query")),
        (names(&["repository"]), vec![])
    );
    let import = ["repository", "drafts", "provenance", "dry_run"];
    for (name, path) in [("import_adr", "dir"), ("import_rules", "page")] {
        let mut taken = import.to_vec();
        taken.push(path);
        assert_eq!(
            arguments(tool(&tools, name)),
            (names(&taken), names(&[path])),
            "{name}"
        );
    }
    for name in ["show", "log", "query"] {
        assert_eq!(tool(&tools, name)["annotations"]["readOnlyHint"], true);
    }
}

#[test]
fn no_persons_act_listed() {
    let place = Place::new();
    for tool in tools(&place) {
        let name = tool["name"].as_str().unwrap();
        assert!(
            !["init", "register", "accept", "reject", "mcp"].contains(&name),
            "{name} is listed"
        );
        assert!(
            tool["inputSchema"]["properties"].get("key").is_none(),
            "{name} takes key"
        );
    }
}
