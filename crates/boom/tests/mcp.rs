//! `boom mcp`: boom's commands served to an agent's program as tools
//! over the Model Context Protocol, none of them a person's act
//! (agent-tools; record 0009).

mod support;

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
