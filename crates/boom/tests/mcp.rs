//! `boom mcp`: boom's commands served to an agent's program as tools
//! over the Model Context Protocol, none of them a person's act
//! (agent-tools; record 0009).

mod support;

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::{json, Value};
use support::{initialize, request, text, Place, SCENARIO};

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

/// The arguments of `propose` for a first version of
/// `money-has-currency`.
fn a_proposal() -> Value {
    json!({
        "kind": "invariant",
        "name": "money-has-currency",
        "statement": "Every amount names its currency.",
        "scenarios": [SCENARIO],
        "provenance": "run-1",
    })
}

#[test]
fn a_proposal_through_its_tool() {
    let place = Place::new();
    place.made();
    let before = place.log().len();
    let result = place.call(
        "propose",
        json!({
            "kind": "invariant",
            "name": "money-has-currency",
            "statement": "Every amount names its currency.",
            "scenarios": ["Given an amount, then it names its currency."],
            "repositories": ["shop"],
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-propose/1", "{answer}");
    assert_eq!(answer["claim"], "money-has-currency");
    assert_eq!(answer["version"].as_str().unwrap().len(), 12, "{answer}");
    assert_eq!(answer["repositories"], json!(["shop"]));
    let log = place.log();
    assert_eq!(log.len(), before + 1);
    let entry = log.last().unwrap();
    assert_eq!(entry["act"], "proposed");
    assert_eq!(entry["actor"], json!({"agent": {"provenance": "run-1"}}));
    assert_eq!(entry["signature"], Value::Null);
}

#[test]
fn the_commands_own_answer() {
    let place = Place::new();
    let ada = place.made();
    let proposed = place.json(&[
        "propose",
        "invariant",
        "money-has-currency",
        "Every amount names its currency.",
        "--scenario",
        SCENARIO,
        "--repository",
        "shop",
        "--provenance",
        "run-1",
    ]);
    let version = proposed["version"].as_str().unwrap();
    place.accepted("money-has-currency", version, &ada);
    let result = place.call("query", json!({"repository": "shop"}));
    assert_eq!(result["isError"], false, "{result}");
    let command = place.run(&["query", "--repository", "shop", "--json"]);
    assert_eq!(command.code, 0, "{command:?}");
    assert_eq!(result["content"][0]["text"], command.stdout.trim());
    assert_eq!(text(&result)["claims"][0]["name"], "money-has-currency");
}

#[test]
fn a_refusal_is_an_error() {
    let place = Place::new();
    place.made();
    let before = place.log();
    let mut arguments = a_proposal();
    arguments.as_object_mut().unwrap().remove("provenance");
    let result = place.call("propose", arguments);
    assert_eq!(result["isError"], true, "{result}");
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-refused/1", "{answer}");
    let reason = answer["refused"]["reason"].as_str().unwrap();
    assert!(
        reason.contains("an agent's proposal names its provenance"),
        "{reason}"
    );
    assert_eq!(place.log(), before);
}

#[test]
fn not_run_is_an_error() {
    let place = Place::new();
    let result = place.call("show", json!({"target": "money-has-currency"}));
    assert_eq!(result["isError"], true, "{result}");
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-not-run/1", "{answer}");
    let reason = answer["not_run"]["reason"].as_str().unwrap();
    assert!(reason.contains("no store at"), "{reason}");
    assert!(!place.store.exists(), "no store is made");
}

#[test]
fn words_beginning_with_a_hyphen() {
    let place = Place::new();
    place.made();
    let result = place.call(
        "propose",
        json!({
            "kind": "invariant",
            "name": "no-negative-zero",
            "statement": "-0 is never an amount.",
            "scenarios": ["--given an amount of -0, then it is refused."],
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    let version = text(&result)["version"].as_str().unwrap().to_owned();
    let shown = place.show(&format!("no-negative-zero@{version}"));
    assert_eq!(shown["statement"], "-0 is never an amount.");
    assert_eq!(
        shown["scenarios"],
        json!(["--given an amount of -0, then it is refused."])
    );
}

#[test]
fn the_working_folders_repository() {
    let place = Place::new();
    place.made();
    let repo = place.repository("shop-repo", Some("repository = \"shop\"\n"));
    let result = place.call_in(&repo, "query", json!({}));
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-query/1", "{answer}");
    assert_eq!(answer["repository"], "shop");
}
