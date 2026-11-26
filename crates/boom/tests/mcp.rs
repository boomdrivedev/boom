//! `boom mcp`: boom's commands served to an agent's program as tools
//! over the Model Context Protocol, none of them a person's act
//! (agent-tools; record 0011).

mod support;

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::{json, Value};
use support::{initialize, request, text, Place};

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

fn properties_of(tool: &Value) -> &Value {
    &tool["inputSchema"]["properties"]
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
            "record_change",
            "search",
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
                "amends",
                "body",
                "provenance",
                "repositories",
                "contexts",
                "system",
                "replaces",
                "source",
                "changes",
                "origin",
                "paths",
                "includes",
                "answers",
                "view",
                "narratives",
                "cause"
            ]),
            names(&["kind", "name", "statement"])
        )
    );
    for edge in [
        "source",
        "changes",
        "origin",
        "contexts",
        "paths",
        "includes",
        "answers",
        "narratives",
    ] {
        assert_eq!(properties_of(propose)[edge]["type"], "array", "{edge}");
        assert_eq!(
            properties_of(propose)[edge]["items"]["type"],
            "string",
            "{edge}"
        );
    }
    let properties = &propose["inputSchema"]["properties"];
    assert_eq!(
        properties["kind"]["enum"],
        json!([
            "invariant",
            "decision",
            "assumption",
            "term",
            "context",
            "narrative",
            "outline"
        ])
    );
    assert_eq!(properties["replaces"]["type"], "array");
    assert_eq!(properties["replaces"]["items"]["type"], "string");
    assert_eq!(properties["repositories"]["type"], "array");
    assert_eq!(properties["repositories"]["items"]["type"], "string");
    assert_eq!(
        properties.get("scenarios"),
        None,
        "a claim takes no scenarios"
    );
    assert_eq!(properties["system"]["type"], "boolean");
    assert_eq!(properties["statement"]["type"], "string");
    assert_eq!(
        arguments(tool(&tools, "show")),
        (names(&["target", "repository", "artifact"]), vec![])
    );
    assert_eq!(arguments(tool(&tools, "log")), (vec![], vec![]));
    let query = tool(&tools, "query");
    assert_eq!(
        arguments(query),
        (
            names(&["repository", "if_declared", "paths", "elaboration"]),
            vec![]
        )
    );
    assert_eq!(
        query["inputSchema"]["properties"]["elaboration"]["type"],
        "string"
    );
    assert_eq!(query["inputSchema"]["properties"]["paths"]["type"], "array");
    assert_eq!(
        query["inputSchema"]["properties"]["paths"]["items"]["type"],
        "string"
    );
    assert_eq!(
        query["inputSchema"]["properties"]["if_declared"]["type"],
        "boolean"
    );
    let search = tool(&tools, "search");
    assert_eq!(
        arguments(search),
        (
            names(&["words", "proposed", "repository", "all", "limit"]),
            names(&[])
        )
    );
    let properties = &search["inputSchema"]["properties"];
    assert_eq!(properties["words"]["type"], "array");
    assert_eq!(properties["words"]["items"]["type"], "string");
    assert_eq!(properties["all"]["type"], "boolean");
    assert_eq!(properties["limit"]["type"], "string");
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
    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        assert_eq!(
            tool["annotations"]["readOnlyHint"],
            ["show", "log", "query", "search"].contains(&name),
            "{name}"
        );
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
            "repositories": ["shop"],
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-propose/2", "{answer}");
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
fn the_mcp_tool_warns_too() {
    let place = Place::new();
    place.made();
    let result = place.call(
        "propose",
        json!({
            "kind": "invariant",
            "name": "money-has-currency",
            "statement": "The roadmap gives every amount its currency.",
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-propose/2", "{answer}");
    assert_eq!(
        answer["plan_words"],
        json!([{"word": "roadmap", "in": "statement", "line": null}])
    );
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
fn a_search_through_its_tool() {
    let place = Place::new();
    let ada = place.made();
    let proposed = place.json(&[
        "propose",
        "invariant",
        "money-has-currency",
        "Every amount names its currency.",
        "--repository",
        "shop",
        "--provenance",
        "run-1",
    ]);
    let version = proposed["version"].as_str().unwrap();
    place.accepted("money-has-currency", version, &ada);
    let result = place.call(
        "search",
        json!({"words": ["currency"], "repository": "shop"}),
    );
    assert_eq!(result["isError"], false, "{result}");
    let command = place.run(&["search", "currency", "--repository", "shop", "--json"]);
    assert_eq!(command.code, 0, "{command:?}");
    assert_eq!(result["content"][0]["text"], command.stdout.trim());
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-search/2", "{answer}");
    assert_eq!(answer["claims"][0]["name"], "money-has-currency");
    assert_eq!(answer["claims"][0]["version"], version);
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
            "provenance": "--run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    let version = text(&result)["version"].as_str().unwrap().to_owned();
    let shown = place.show(&format!("no-negative-zero@{version}"));
    assert_eq!(shown["statement"], "-0 is never an amount.");
    assert_eq!(
        shown["proposer"],
        json!({"agent": {"provenance": "--run-1"}})
    );
}

#[test]
fn the_working_folders_repository() {
    let place = Place::new();
    let ada = place.made();
    let repo = place.repository("shop-repo", Some(support::SHOP_REMOTE));
    place.repo_added(&ada, "shop", support::SHOP_REMOTE);
    let result = place.call_in(&repo, "query", json!({}));
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(answer["format"], "boom-query/2", "{answer}");
    assert_eq!(answer["repository"], "shop");
}

/// The result's refusal's reason, asserting it is an error holding a
/// `boom-refused/1` object.
fn refusal(result: &Value) -> String {
    assert_eq!(result["isError"], true, "{result}");
    let answer = text(result);
    assert_eq!(answer["format"], "boom-refused/1", "{answer}");
    answer["refused"]["reason"].as_str().unwrap().to_owned()
}

#[test]
fn a_store_given() {
    let place = Place::new();
    place.made();
    let other = place.dir.path().join("other.sqlite");
    let result = place.call(
        "show",
        json!({"target": "money-has-currency", "store": support::path_arg(&other)}),
    );
    let reason = refusal(&result);
    assert!(reason.contains("argument store"), "{reason}");
    assert!(!other.exists(), "the other file is not opened");
}

#[test]
fn a_value_of_another_type() {
    let place = Place::new();
    place.made();
    let before = place.log();
    let mut arguments = a_proposal();
    arguments["repositories"] = json!("shop");
    let reason = refusal(&place.call("propose", arguments));
    assert!(reason.contains("repositories"), "{reason}");
    assert_eq!(place.log(), before);
}

#[test]
fn scenarios_given() {
    let place = Place::new();
    place.made();
    let before = place.log();
    let mut arguments = a_proposal();
    arguments["scenarios"] = json!(["Given an amount, then it names its currency."]);
    let reason = refusal(&place.call("propose", arguments));
    assert!(reason.contains("scenarios"), "{reason}");
    assert_eq!(place.log(), before);
}

#[test]
fn a_misspelled_switch() {
    let place = Place::new();
    place.made();
    let dir = place.folder(
        "adr",
        &[(
            "0001-amounts-in-cents.md",
            "---\nstatus: accepted\ndate: 2026-01-04\n---\n\n# Amounts are kept in cents\n\nEvery amount is an integer of cents.\n",
        )],
    );
    let drafts = place.file(
        "drafts.json",
        &json!({
            "format": "boom-import-drafts/2",
            "records": {"0001-amounts-in-cents.md": {}},
        })
        .to_string(),
    );
    let mut arguments = json!({
        "dir": dir,
        "repository": "shop",
        "drafts": drafts,
        "provenance": "run-1",
        "dryrun": true,
    });
    let before = place.log();
    let reason = refusal(&place.call("import_adr", arguments.clone()));
    assert!(reason.contains("dryrun"), "{reason}");
    assert_eq!(place.log(), before);
    // The same call without the misspelled switch imports for real.
    arguments.as_object_mut().unwrap().remove("dryrun");
    let result = place.call("import_adr", arguments);
    assert_eq!(result["isError"], false, "{result}");
    assert_eq!(place.log().len(), before.len() + 1);
}

#[test]
fn a_key_given_to_propose() {
    let place = Place::new();
    let ada = place.made();
    let signatures = place.keys.signatures();
    let before = place.log();
    let mut arguments = a_proposal();
    arguments.as_object_mut().unwrap().remove("provenance");
    arguments["key"] = json!(ada);
    let reason = refusal(&place.call("propose", arguments));
    assert!(reason.contains("argument key"), "{reason}");
    assert_eq!(place.keys.signatures(), signatures, "a key signed");
    assert_eq!(place.log(), before);
}

#[test]
fn an_accept_asked_of_the_server() {
    let place = Place::new();
    place.made();
    let version = place.proposed("money-has-currency", "Every amount names its currency.");
    let target = format!("money-has-currency@{version}");
    let answers = place.mcp(&[
        initialize(1),
        request(
            2,
            "tools/call",
            json!({"name": "accept", "arguments": {"targets": [target]}}),
        ),
    ]);
    assert_eq!(answers.len(), 2, "{answers:?}");
    assert_eq!(answers[1]["id"], 2);
    assert_eq!(answers[1]["error"]["code"], -32602, "{answers:?}");
    let message = answers[1]["error"]["message"].as_str().unwrap();
    assert!(message.contains("accept"), "{message}");
    assert_eq!(place.status("money-has-currency", &version), "proposed");
}

#[test]
fn the_instructions() {
    let place = Place::new();
    let answers = place.mcp(&[initialize(1)]);
    let instructions = answers[0]["result"]["instructions"].as_str().unwrap();
    assert!(instructions.contains("boom accept"), "{instructions}");
    assert!(instructions.contains("boom reject"), "{instructions}");
    assert!(
        instructions.contains("in their own terminal"),
        "{instructions}"
    );
}

/// The arguments the command line takes for [`a_proposal`].
const PROPOSE: [&str; 6] = [
    "propose",
    "invariant",
    "money-has-currency",
    "Every amount names its currency.",
    "--provenance",
    "run-1",
];

#[test]
fn a_proposals_accept() {
    let by_json = Place::new();
    by_json.made();
    let answer = by_json.json(&PROPOSE);
    let version = answer["version"].as_str().unwrap().to_owned();
    let accept = format!("boom accept money-has-currency@{version} --key <key>");
    assert_eq!(answer["status"], "proposed");
    assert_eq!(answer["accept"], accept.as_str());

    let by_words = Place::new();
    by_words.made();
    let outcome = by_words.run(&PROPOSE);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(
        outcome
            .stdout
            .contains(&format!("\na person accepts it with: {accept}\n")),
        "{outcome:?}"
    );

    let by_tool = Place::new();
    by_tool.made();
    let result = by_tool.call("propose", a_proposal());
    assert_eq!(result["isError"], false, "{result}");
    assert_eq!(text(&result)["accept"], accept.as_str());
}

#[test]
fn a_version_that_stands() {
    let place = Place::new();
    let ada = place.made();
    let version = place.json(&PROPOSE)["version"].as_str().unwrap().to_owned();
    place.accepted("money-has-currency", &version, &ada);
    let result = place.call("propose", a_proposal());
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(answer["version"], version.as_str());
    assert_eq!(answer["status"], "accepted", "{answer}");
    assert_eq!(answer["accept"], Value::Null, "{answer}");
    let outcome = place.run(&PROPOSE);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(
        !outcome.stdout.contains("a person accepts it with"),
        "{outcome:?}"
    );
}

#[test]
fn a_claim_proposed_with_its_edges_through_its_tool() {
    let place = Place::new();
    let ada = place.made();
    let resident = place.proposed(
        "flume-has-a-resident",
        "flume has one resident on each host.",
    );
    place.accepted("flume-has-a-resident", &resident, &ada);
    let commit = "0123456789abcdef0123456789abcdef01234567";
    let reference = format!("git:sluice@{commit}:design/overview.svg");
    let result = place.call(
        "propose",
        json!({
            "kind": "invariant",
            "name": "the-runner-serves-a-socket",
            "statement": "The runner serves a socket.",
            "source": ["flume-has-a-resident"],
            "origin": [reference],
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(
        answer["source"],
        json!([format!("flume-has-a-resident@{resident}")])
    );
    assert_eq!(answer["origin"][0]["reference"], reference);
    let signed: Value =
        serde_json::from_str(place.log().last().unwrap()["signed"].as_str().unwrap()).unwrap();
    assert_eq!(
        signed["claim"]["source"],
        json!([format!("flume-has-a-resident@{resident}")])
    );
    assert_eq!(signed["claim"]["origin"][0]["git"]["commit"], commit);
}

#[test]
fn an_artifact_read_through_its_tool() {
    let place = Place::new();
    place.made();
    let mockup = place.file("mockup.svg", "<svg/>\n");
    let answer = place.json(&[
        "propose",
        "invariant",
        "status-on-a-socket",
        "Status is read from a socket.",
        "--origin",
        &mockup,
        "--provenance",
        "run-1",
    ]);
    let fingerprint = answer["origin"][0]["fingerprint"].as_str().unwrap();
    let result = place.call("show", json!({"artifact": fingerprint}));
    assert_eq!(result["isError"], false, "{result}");
    let shown = text(&result);
    assert_eq!(shown["name"], "mockup.svg");
    assert_eq!(shown["text"], "<svg/>\n");
    let result = place.call(
        "show",
        json!({"target": "status-on-a-socket", "artifact": fingerprint}),
    );
    assert_eq!(result["isError"], true, "{result}");
    assert!(
        text(&result)["refused"]["reason"]
            .as_str()
            .unwrap()
            .contains("boom show takes a target or an artifact, and not both"),
        "{result}"
    );
}

#[test]
fn a_note_asked_of_the_server() {
    let place = Place::new();
    place.made_by("chuck");
    let version = place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &[],
    )["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = place.log();
    let answers = place.mcp(&[
        initialize(1),
        request(
            2,
            "tools/call",
            json!({"name": "note", "arguments": {
                "target": format!("boom-end-to-end@{version}"),
                "paragraph": "1",
                "words": "A word.",
            }}),
        ),
    ]);
    assert_eq!(answers[1]["error"]["code"], -32602, "{answers:?}");
    let message = answers[1]["error"]["message"].as_str().unwrap();
    assert!(message.contains("note"), "{message}");
    assert_eq!(place.log(), before);
}

#[test]
fn the_instructions_name_the_note() {
    let place = Place::new();
    let answers = place.mcp(&[initialize(1)]);
    let instructions = answers[0]["result"]["instructions"].as_str().unwrap();
    assert!(
        instructions.contains("Only a person writes a note"),
        "{instructions}"
    );
    assert!(instructions.contains("boom page"), "{instructions}");
    assert!(instructions.contains("boom note"), "{instructions}");
}

#[test]
fn an_outline_proposed_through_its_tool() {
    let place = Place::new();
    place.made();
    place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &[],
    );
    let result = place.call(
        "propose",
        json!({
            "kind": "outline",
            "name": "boomdrive-design-book",
            "statement": "The design book of Boomdrive",
            "view": "book",
            "narratives": ["boom-end-to-end"],
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    let answer = text(&result);
    assert_eq!(answer["kind"], "outline");
    assert_eq!(answer["status"], "proposed");
    let last = place.log().pop().unwrap();
    assert_eq!(last["act"], "proposed");
    let signed: Value = serde_json::from_str(last["signed"].as_str().unwrap()).unwrap();
    assert_eq!(signed["claim"]["kind"], "outline");
    let held = signed["claim"]["narratives"][0].as_str().unwrap();
    assert!(held.starts_with("boom-end-to-end@"), "{held}");
}

#[test]
fn the_open_notes_of_an_elaboration_through_its_tool() {
    let place = Place::new();
    place.made_by("chuck");
    let narrative = place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &["--origin", "elaboration:boom/the-design-book"],
    );
    let version = narrative["version"].as_str().unwrap();
    place.noted(
        &format!("boom-end-to-end@{version}"),
        3,
        "Say what a drive is.",
    );
    let result = place.call("query", json!({"elaboration": "boom/the-design-book"}));
    assert_eq!(result["isError"], false, "{result}");
    let command = place.run(&["query", "--elaboration", "boom/the-design-book", "--json"]);
    assert_eq!(command.code, 0, "{command:?}");
    assert_eq!(result["content"][0]["text"], command.stdout.trim());
    assert_eq!(text(&result)["notes"][0]["narrative"], "boom-end-to-end");
}
