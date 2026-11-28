//! The walk of the links, back to what an entry took in and forward to
//! what took it in (lineage, "boom lineage walks the links" and "The
//! target of a walk").

#[path = "../src/tests/support.rs"]
mod support;

use support::Place;

const STATUS: &str = "A record never holds its status.";
const RECORD: &str = "boom:contexts-own-paths";

/// The act hashes of a chain from an elaboration's end to a landing.
struct Chain {
    ada: String,
    version: String,
    proposal: String,
    ended: String,
    approval: String,
    recorded: String,
    landed: String,
}

fn act_hash(answer: &serde_json::Value) -> String {
    answer["act_hash"].as_str().unwrap().to_owned()
}

fn chain(place: &Place) -> Chain {
    let ada = place.made();
    let version = place.proposed("status-read-from-events", STATUS);
    place.accepted("status-read-from-events", &version, &ada);
    let proposal = place.log()[1]["act_hash"].as_str().unwrap().to_owned();
    let ended = act_hash(&place.json(&[
        "append",
        "sounding.elaboration-ended",
        "--provenance",
        "sounding run-7",
    ]));
    let plan = place.file("plan.md", "# The plan\n");
    place.json(&[
        "approve",
        &plan,
        "--link",
        &ended,
        "--covers",
        "status-read-from-events",
        "--key",
        &ada,
    ]);
    let approval = place.log().last().unwrap()["act_hash"]
        .as_str()
        .unwrap()
        .to_owned();
    let root = place.dir.path().join("openspec/changes/contexts-own-paths");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("proposal.md"), "## Why\n").unwrap();
    place.json(&[
        "record",
        "change",
        RECORD,
        "--approval",
        &approval,
        "--unit",
        "contexts",
        "--bolt",
        "b-12",
        "--covers",
        "status-read-from-events",
        "--provenance",
        "run-1",
    ]);
    let recorded = place.log().last().unwrap()["act_hash"]
        .as_str()
        .unwrap()
        .to_owned();
    let landed = act_hash(&place.json(&[
        "append",
        "sluice.landed",
        "--link",
        &recorded,
        "--provenance",
        "sluice host deneb",
    ]));
    Chain {
        ada,
        version,
        proposal,
        ended,
        approval,
        recorded,
        landed,
    }
}

fn hashes(found: &serde_json::Value) -> Vec<String> {
    found
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["act_hash"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn from_a_change_record_back_and_forward() {
    let place = Place::new();
    let chain = chain(&place);
    let walked = place.json(&["lineage", RECORD]);
    assert_eq!(walked["format"], "boom-lineage/1");
    assert_eq!(walked["entry"]["act_hash"], chain.recorded);
    assert_eq!(walked["entry"]["act"], "recorded");
    // The approval one hop back; its claim's proposal and the
    // elaboration's end two hops back, by number.
    assert_eq!(
        hashes(&walked["back"]),
        [
            chain.approval.clone(),
            chain.proposal.clone(),
            chain.ended.clone()
        ]
    );
    assert_eq!(walked["back"][0]["hops"], 1);
    assert_eq!(walked["back"][2]["hops"], 2);
    assert_eq!(
        hashes(&walked["forward"]),
        std::slice::from_ref(&chain.landed)
    );
}

#[test]
fn two_entries_that_one_step_took_in_are_each_given_once() {
    let place = Place::new();
    let chain = chain(&place);
    let walked = place.json(&["lineage", &chain.approval]);
    let back = hashes(&walked["back"]);
    assert_eq!(back, [chain.proposal.clone(), chain.ended.clone()]);
    assert_eq!(
        hashes(&walked["forward"]),
        [chain.recorded.clone(), chain.landed.clone()]
    );
}

#[test]
fn the_claims_of_an_approval_out_of_date() {
    let place = Place::new();
    let chain = chain(&place);
    let v2 = place.amended(
        "status-read-from-events",
        "A record never holds its status; boom reads it.",
        &chain.version,
    );
    place.accepted("status-read-from-events", &v2, &chain.ada);
    let walked = place.json(&["lineage", &chain.approval[..12]]);
    let covered = &walked["entry"]["covers"][0];
    assert_eq!(covered["claim"], "status-read-from-events");
    assert_eq!(covered["version"], chain.version);
    assert_eq!(covered["out_of_date"], true);
    assert_eq!(covered["accepted"], v2);
    let words = place.run(&["lineage", &chain.approval]).stdout;
    assert!(
        words.contains(&format!(
            "covers status-read-from-events@{}, out of date: status-read-from-events@{v2} is accepted",
            chain.version
        )),
        "{words}"
    );
}

#[test]
fn a_version_of_a_change_record_is_a_target() {
    let place = Place::new();
    let chain = chain(&place);
    let version = place.json(&["show", RECORD])["newest"]
        .as_str()
        .unwrap()
        .to_owned();
    let walked = place.json(&["lineage", &format!("{RECORD}@{version}")]);
    assert_eq!(walked["entry"]["act_hash"], chain.recorded);
}

#[test]
fn a_prefix_that_starts_two_act_hashes_is_refused() {
    let place = Place::new();
    place.made();
    place.json(&["append", "millpond.capture", "--provenance", "run-1"]);
    place.json(&["append", "millpond.capture", "--provenance", "run-2"]);
    place.refused(&["lineage", "abcdef"], "12 characters or more");
    // Two act hashes rarely share 12 characters, so the views are made
    // to hold two that do.
    place
        .sqlite()
        .execute(
            "UPDATE acts SET act_hash = ?1 || substr(act_hash, 13) WHERE seq IN (2, 3)",
            ["abcdefabcdef"],
        )
        .unwrap();
    let outcome = place.run(&["lineage", "abcdefabcdef"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome.stderr.contains("several entries, 2, 3"),
        "{outcome:?}"
    );
}

#[test]
fn a_target_that_names_nothing_is_refused() {
    let place = Place::new();
    chain(&place);
    place.refused(
        &["lineage", "boom:no-such-change"],
        "no change record has the name",
    );
    place.refused(
        &["lineage", &"0".repeat(12)],
        "no entry has an act hash that starts with",
    );
}

#[test]
fn the_views_built_again_give_the_same_walk() {
    let place = Place::new();
    chain(&place);
    let before = place.json(&["lineage", RECORD]);
    place
        .sqlite()
        .execute_batch("DROP TABLE acts; DROP TABLE links;")
        .unwrap();
    place.json(&["verify"]);
    assert_eq!(place.json(&["lineage", RECORD]), before);
}

#[test]
fn an_agent_walks_the_links_through_its_tool() {
    let place = Place::new();
    let chain = chain(&place);
    let result = place.call("lineage", serde_json::json!({"target": RECORD}));
    assert_ne!(result["isError"], true, "{result}");
    let walked = support::text(&result);
    assert_eq!(walked["format"], "boom-lineage/1");
    assert_eq!(walked["entry"]["act_hash"], chain.recorded);
}
