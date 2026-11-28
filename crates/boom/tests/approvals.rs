//! A person's signed approval of a plan (plan-approvals).

mod support;

use boom::entry::{Act, Actor, Entry};
use support::Place;

const STATUS: &str = "A record never holds its status.";

/// A store with ada its owner, `status-read-from-events` accepted and
/// the end of an elaboration appended: ada's key, the version, the act
/// hash of its proposal and the act hash of the elaboration's end.
struct Approvable {
    ada: String,
    version: String,
    proposal: String,
    ended: String,
}

fn approvable(place: &Place) -> Approvable {
    let ada = place.made();
    let version = place.proposed("status-read-from-events", STATUS);
    place.accepted("status-read-from-events", &version, &ada);
    let ended = place.json(&[
        "append",
        "sounding.elaboration-ended",
        "--provenance",
        "sounding run-7",
    ])["act_hash"]
        .as_str()
        .unwrap()
        .to_owned();
    let proposal = place.log()[1]["act_hash"].as_str().unwrap().to_owned();
    Approvable {
        ada,
        version,
        proposal,
        ended,
    }
}

fn plan(place: &Place) -> String {
    place.file("plan.md", "# The plan\n\nOne unit: contexts.\n")
}

#[test]
fn a_plan_is_approved_with_its_links_and_its_claims() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    let answer = place.json(&[
        "approve",
        &plan,
        "--link",
        &given.ended,
        "--covers",
        "status-read-from-events",
        "--key",
        &given.ada,
    ]);
    assert_eq!(answer["format"], "boom-approve/1");
    let fingerprint = boom::sha256_hex(&std::fs::read(&plan).unwrap());
    assert_eq!(answer["plan"], fingerprint);
    let log = place.log();
    let entry = log.last().unwrap();
    assert_eq!(entry["act"], "approved");
    assert_eq!(entry["actor"], serde_json::json!({"person": "ada"}));
    assert!(entry["signature"].is_string(), "{entry}");
    assert_eq!(entry["counted"], true);
    assert_eq!(
        entry["links"],
        serde_json::json!([given.proposal, given.ended])
    );
    let signed: serde_json::Value =
        serde_json::from_str(entry["signed"].as_str().unwrap()).unwrap();
    assert_eq!(signed["plan"], fingerprint);
    assert_eq!(
        signed["covers"],
        serde_json::json!([format!("status-read-from-events@{}", given.version)])
    );
}

#[test]
fn a_plan_from_no_elaboration_links_only_its_claims() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    place.json(&[
        "approve",
        &plan,
        "--covers",
        "status-read-from-events",
        "--key",
        &given.ada,
    ]);
    let log = place.log();
    assert_eq!(
        log.last().unwrap()["links"],
        serde_json::json!([given.proposal])
    );
}

#[test]
fn nothing_of_the_plan_is_kept() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    place.json(&[
        "approve",
        &plan,
        "--link",
        &given.ended,
        "--key",
        &given.ada,
    ]);
    let fingerprint = boom::sha256_hex(&std::fs::read(&plan).unwrap());
    let kept: i64 = place
        .sqlite()
        .query_row(
            "SELECT count(*) FROM blobs WHERE fingerprint = ?1",
            [&fingerprint],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(kept, 0);
}

#[test]
fn an_output_commit_is_held_as_given() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    let output = format!("git:boom@{}:design/narrative.md", "c".repeat(40));
    let answer = place.json(&[
        "approve",
        &plan,
        "--link",
        &given.ended,
        "--output",
        &output,
        "--key",
        &given.ada,
    ]);
    assert_eq!(answer["outputs"], serde_json::json!([output]));
    let log = place.log();
    let signed: serde_json::Value =
        serde_json::from_str(log.last().unwrap()["signed"].as_str().unwrap()).unwrap();
    assert_eq!(signed["outputs"], serde_json::json!([output]));
}

#[test]
fn a_covered_version_that_is_not_accepted_is_refused() {
    let place = Place::new();
    let given = approvable(&place);
    let other = place.proposed("money-has-currency", "Every amount names its currency.");
    let plan = plan(&place);
    let covered = format!("money-has-currency@{other}");
    let outcome = place.refused(
        &[
            "approve",
            &plan,
            "--link",
            &given.ended,
            "--covers",
            &covered,
            "--key",
            &given.ada,
        ],
        "an approval covers only accepted versions",
    );
    assert!(outcome.stderr.contains(&covered), "{outcome:?}");
}

#[test]
fn a_name_with_no_accepted_version_is_refused() {
    let place = Place::new();
    let given = approvable(&place);
    place.proposed("money-has-currency", "Every amount names its currency.");
    let plan = plan(&place);
    place.refused(
        &[
            "approve",
            &plan,
            "--covers",
            "money-has-currency",
            "--key",
            &given.ada,
        ],
        "money-has-currency has no accepted version",
    );
}

#[test]
fn an_approval_with_no_link_is_refused() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    place.refused(
        &["approve", &plan, "--key", &given.ada],
        "an approval covers a claim or links an entry",
    );
}

#[test]
fn one_plan_is_approved_once() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    let first = place.json(&[
        "approve",
        &plan,
        "--link",
        &given.ended,
        "--key",
        &given.ada,
    ]);
    let entry = first["entry"].as_u64().unwrap();
    place.refused(
        &[
            "approve",
            &plan,
            "--covers",
            "status-read-from-events",
            "--key",
            &given.ada,
        ],
        &format!("is approved already, by entry {entry}"),
    );
}

#[test]
fn an_agent_does_not_approve() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    place.refused(
        &[
            "approve",
            &plan,
            "--link",
            &given.ended,
            "--provenance",
            "run-1",
        ],
        "only a person approves a plan",
    );
    place.refused(
        &["approve", &plan, "--link", &given.ended],
        "only a person approves a plan",
    );
}

#[test]
fn an_approval_appended_by_hand_with_no_signature_counts_for_nothing() {
    let place = Place::new();
    let given = approvable(&place);
    for actor in [
        Actor::Person("ada".to_owned()),
        Actor::Agent {
            provenance: "run-1".to_owned(),
        },
    ] {
        place.append_by_hand(
            &Entry {
                at: 1_763_424_900,
                rules: boom::judge::RULES,
                actor,
                act: Act::Approved {
                    plan: "9".repeat(64),
                    links: vec![given.ended.clone()],
                    outputs: Vec::new(),
                    covers: Vec::new(),
                },
            },
            None,
        );
    }
    let log = place.log();
    for entry in &log[log.len() - 2..] {
        assert_eq!(entry["counted"], false, "{entry}");
        assert!(
            entry["reason"]
                .as_str()
                .unwrap()
                .contains("a person's act needs their signature"),
            "{entry}"
        );
    }
}

#[test]
fn the_line_of_an_approval_names_its_plan_and_its_person() {
    let place = Place::new();
    let given = approvable(&place);
    let plan = plan(&place);
    let answer = place.json(&[
        "approve",
        &plan,
        "--link",
        &given.ended,
        "--key",
        &given.ada,
    ]);
    let fingerprint = answer["plan"].as_str().unwrap();
    let entry = answer["entry"].as_u64().unwrap();
    let outcome = place.run(&["log"]);
    let line = outcome
        .stdout
        .lines()
        .find(|line| line.starts_with(&format!("{entry} ")))
        .unwrap();
    assert_eq!(
        line,
        format!("{entry} approved plan {} by ada", &fingerprint[..12])
    );
}

#[test]
fn no_tool_of_boom_mcp_approves() {
    let place = Place::new();
    place.made();
    let answers = place.mcp(&[
        support::initialize(1),
        support::request(
            2,
            "tools/call",
            serde_json::json!({"name": "approve", "arguments": {}}),
        ),
    ]);
    let message = answers[1]["error"]["message"].as_str().unwrap();
    assert!(message.contains("no tool is named approve"), "{message}");
    assert!(
        message.contains("only a person approves a plan"),
        "{message}"
    );
}
