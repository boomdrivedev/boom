//! The act hash of each entry, the links between entries, and an entry
//! of a type that another tool owns (record-log, "Each entry has an act
//! hash" and "A link names an entry that the log holds"; lineage).

mod support;

use boom::entry::{Act, Actor, Entry};
use support::Place;

const MONEY: &str = "Every amount names its currency.";

/// A store with ada its owner and one proposal, giving the act hash of
/// the proposal.
fn with_a_proposal(place: &Place) -> String {
    place.made();
    place.proposed("money-has-currency", MONEY);
    place.log()[1]["act_hash"].as_str().unwrap().to_owned()
}

fn landing(links: &[&str], actor: Actor) -> Entry {
    Entry {
        at: 1_763_424_300,
        rules: boom::judge::RULES,
        actor,
        act: Act::Other {
            act: "sluice.landed".to_owned(),
            links: links.iter().map(|&link| link.to_owned()).collect(),
            content: "{}".to_owned(),
        },
    }
}

fn agent() -> Actor {
    Actor::Agent {
        provenance: "sluice host deneb".to_owned(),
    }
}

#[test]
fn each_entry_gives_the_sha256_of_its_signed_part() {
    let place = Place::new();
    place.three_entries();
    let log = place.log();
    assert_eq!(log.len(), 3);
    for entry in &log {
        let signed = entry["signed"].as_str().unwrap();
        assert_eq!(
            entry["act_hash"].as_str().unwrap(),
            boom::sha256_hex(signed.as_bytes()),
            "{entry}"
        );
        assert_eq!(entry["links"], serde_json::json!([]), "{entry}");
    }
    assert_ne!(log[0]["act_hash"], log[1]["act_hash"]);
    assert_ne!(log[1]["act_hash"], log[2]["act_hash"]);
}

#[test]
fn an_entry_of_rule_set_0_gives_the_hash_of_its_signed_part_as_stored() {
    let place = Place::new();
    place.made();
    let mut old = support::accept_entry("ada", "money-has-currency", "000000000000");
    old.rules = 0;
    let signed = place.append_by_hand(&old, None);
    assert!(!signed.contains("\"rules\""), "{signed}");
    let log = place.log();
    assert_eq!(
        log[1]["act_hash"].as_str().unwrap(),
        boom::sha256_hex(signed.as_bytes())
    );
}

#[test]
fn a_landing_is_appended_with_its_links_and_its_content() {
    let place = Place::new();
    let proposal = with_a_proposal(&place);
    let content = place.file(
        "landing.json",
        "{\"revision\": \"abc\", \"verdicts\": []}\n",
    );
    let answer = place.json(&[
        "append",
        "sluice.landed",
        "--link",
        &proposal,
        "--content",
        &content,
        "--provenance",
        "sluice host deneb",
    ]);
    assert_eq!(answer["format"], "boom-append/1");
    assert_eq!(answer["entry"], 3);
    let log = place.log();
    let entry = &log[2];
    assert_eq!(entry["act"], "sluice.landed");
    assert_eq!(entry["links"], serde_json::json!([proposal]));
    assert_eq!(
        entry["actor"],
        serde_json::json!({"agent": {"provenance": "sluice host deneb"}})
    );
    assert_eq!(entry["counted"], true);
    assert_eq!(answer["act_hash"], entry["act_hash"]);
    let signed = entry["signed"].as_str().unwrap();
    assert!(
        signed.ends_with(&format!(
            "\"links\":[\"{proposal}\"],\"content\":{{\"revision\": \"abc\", \"verdicts\": []}}}}"
        )),
        "{signed}"
    );
}

#[test]
fn content_that_is_absent_is_an_empty_object() {
    let place = Place::new();
    place.made();
    place.json(&["append", "millpond.capture", "--provenance", "run-1"]);
    let signed = place.log()[1]["signed"].as_str().unwrap().to_owned();
    assert!(signed.ends_with("\"links\":[],\"content\":{}}"), "{signed}");
}

#[test]
fn the_same_entry_appended_again_repeats_it() {
    let place = Place::new();
    let proposal = with_a_proposal(&place);
    let args = [
        "append",
        "sluice.landed",
        "--link",
        &proposal,
        "--provenance",
        "run-1",
    ];
    place.json(&args);
    let again = place.json(&args);
    assert_eq!(again["repeats"], true);
    assert_eq!(again["entry"], 3);
    assert_eq!(place.log().len(), 3);
}

#[test]
fn a_type_of_boom_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &["append", "boom.approved", "--provenance", "run-1"],
        "boom names no other tool",
    );
}

#[test]
fn a_type_with_no_tool_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &["append", "landed", "--provenance", "run-1"],
        "is <tool>.<type>",
    );
}

#[test]
fn content_that_is_not_an_object_is_refused() {
    let place = Place::new();
    place.made();
    let list = place.file("list.json", "[1, 2]");
    place.refused(
        &[
            "append",
            "sluice.landed",
            "--content",
            &list,
            "--provenance",
            "run-1",
        ],
        "the content is one JSON object",
    );
}

#[test]
fn a_registered_persons_hold_carries_no_signature() {
    let place = Place::new();
    let proposal = with_a_proposal(&place);
    place.json(&[
        "append",
        "planning.held",
        "--link",
        &proposal,
        "--person",
        "ada",
    ]);
    let log = place.log();
    assert_eq!(log[2]["actor"], serde_json::json!({"person": "ada"}));
    assert_eq!(log[2]["signature"], serde_json::Value::Null);
    assert_eq!(log[2]["counted"], true);
}

#[test]
fn a_person_the_store_does_not_register_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &["append", "planning.held", "--person", "bo"],
        "no person bo is registered",
    );
}

#[test]
fn an_entry_has_one_actor() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "append",
            "planning.held",
            "--person",
            "ada",
            "--provenance",
            "run-1",
        ],
        "an entry has one actor",
    );
    place.refused(&["append", "planning.held"], "an entry names its actor");
}

#[test]
fn a_signed_entry_of_another_tool_counts_for_nothing() {
    let place = Place::new();
    place.made();
    let key = place.keys.security_key("ada", 1);
    place.append_by_hand(&landing(&[], Actor::Person("ada".to_owned())), Some(&key));
    let log = place.log();
    assert_eq!(log[1]["counted"], false);
    assert!(
        log[1]["reason"]
            .as_str()
            .unwrap()
            .contains("the key signs no entry of another tool"),
        "{}",
        log[1]
    );
}

#[test]
fn a_link_to_an_entry_the_log_does_not_hold_is_refused() {
    let place = Place::new();
    place.made();
    let missing = "0".repeat(64);
    place.refused(
        &[
            "append",
            "sluice.landed",
            "--link",
            &missing,
            "--provenance",
            "run-1",
        ],
        &missing,
    );
}

#[test]
fn a_short_link_is_refused() {
    let place = Place::new();
    let proposal = with_a_proposal(&place);
    place.refused(
        &[
            "append",
            "sluice.landed",
            "--link",
            &proposal[..12],
            "--provenance",
            "run-1",
        ],
        "a link is the 64 lowercase hexadecimal characters of an act hash",
    );
}

#[test]
fn a_link_given_two_times_is_named_once() {
    let place = Place::new();
    let proposal = with_a_proposal(&place);
    place.json(&[
        "append",
        "sluice.landed",
        "--link",
        &proposal,
        "--link",
        &proposal,
        "--provenance",
        "run-1",
    ]);
    assert_eq!(place.log()[2]["links"], serde_json::json!([proposal]));
}

#[test]
fn a_link_to_a_later_entry_appended_by_hand_counts_for_nothing() {
    let place = Place::new();
    place.made();
    let later = landing(&[], agent());
    let later_hash = boom::entry::act_hash(&later.signed_part());
    place.append_by_hand(&landing(&[&later_hash], agent()), None);
    place.append_by_hand(&later, None);
    place.json(&["verify"]);
    let log = place.log();
    assert_eq!(log[1]["counted"], false);
    assert!(
        log[1]["reason"].as_str().unwrap().contains(&later_hash),
        "{}",
        log[1]
    );
    assert_eq!(log[2]["counted"], true);
    assert_eq!(log[2]["act_hash"].as_str().unwrap(), later_hash);
}

#[test]
fn the_line_of_a_landing_names_each_link_by_12_characters() {
    let place = Place::new();
    let proposal = with_a_proposal(&place);
    place.json(&[
        "append",
        "sluice.landed",
        "--link",
        &proposal,
        "--provenance",
        "run-1",
    ]);
    let outcome = place.run(&["log"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let line = outcome.stdout.lines().nth(2).unwrap();
    assert!(
        line.starts_with(&format!("3 sluice.landed {} by an agent", &proposal[..12])),
        "{line}"
    );
}

#[test]
fn the_views_of_the_links_are_built_again() {
    let place = Place::new();
    let proposal = with_a_proposal(&place);
    place.json(&[
        "append",
        "sluice.landed",
        "--link",
        &proposal,
        "--provenance",
        "run-1",
    ]);
    let links = |place: &Place| -> Vec<(i64, String)> {
        let conn = place.sqlite();
        let mut statement = conn
            .prepare("SELECT seq, target FROM links ORDER BY seq, position")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let before = links(&place);
    assert_eq!(before, [(3, proposal)]);
    let acts: i64 = place
        .sqlite()
        .query_row("SELECT count(*) FROM acts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(acts, 3);
    place
        .sqlite()
        .execute_batch("DROP TABLE acts; DROP TABLE links;")
        .unwrap();
    let verified = place.json(&["verify"]);
    let rebuilt = verified["rebuilt"].as_array().unwrap();
    assert!(rebuilt.contains(&serde_json::json!("acts")), "{verified}");
    assert!(rebuilt.contains(&serde_json::json!("links")), "{verified}");
    assert_eq!(links(&place), before);
}

#[test]
fn a_link_counts_where_its_entry_came_in_a_catch_up() {
    let place = Place::new();
    place.made();
    let first = landing(&[], agent());
    let first_hash = boom::entry::act_hash(&first.signed_part());
    place.append_by_hand(&first, None);
    // The views stand at entry 1; the next command catches up, and the
    // link to entry 2 counts.
    place.json(&[
        "append",
        "timber.published",
        "--link",
        &first_hash,
        "--provenance",
        "run-1",
    ]);
    let log = place.log();
    assert_eq!(log[2]["counted"], true, "{}", log[2]);
}
