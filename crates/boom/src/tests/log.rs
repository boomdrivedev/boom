//! The log: entries appended and never edited, numbered in order,
//! chained, and each judged whenever it is read (record-log).

use super::support;

use support::Place;

const MONEY: &str = "Every amount names its currency.";

#[test]
fn a_proposal_and_its_accept_are_two_entries() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    let before = place.log();
    place.accepted("money-has-currency", &version, &ada);
    let after = place.log();
    assert_eq!(after.len(), 3);
    assert_eq!(after[1]["act"], "proposed");
    assert_eq!(after[2]["act"], "accepted");
    assert_eq!(after[1], before[1], "the proposal is as it was");
}

#[test]
fn a_clock_set_back_changes_no_status() {
    let mut place = Place::new();
    let ada = place.made();
    place.at(2_000_000_000);
    let version = place.proposed("money-has-currency", MONEY);
    place.at(1_000_000_000);
    place.accepted("money-has-currency", &version, &ada);
    let log = place.log();
    assert!(log[2]["at"].as_u64() < log[1]["at"].as_u64());
    assert_eq!(place.status("money-has-currency", &version), "accepted");
}

#[test]
fn every_entry_names_its_format() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &version, &ada);
    let other = place.proposed(
        "refund-within-payment",
        "A refund never exceeds the payment.",
    );
    place.json(&[
        "reject",
        &format!("refund-within-payment@{other}"),
        "--reason",
        "Partial refunds are out of scope.",
        "--key",
        &ada,
    ]);
    let log = place.log();
    assert_eq!(log.len(), 5);
    for entry in &log {
        assert_eq!(entry["format"], "boom-entry/1", "{entry}");
    }
    assert_eq!(place.json(&["log"])["format"], "boom-log/1");
}

#[test]
fn entries_are_numbered_from_one_in_order() {
    let place = Place::new();
    place.made();
    let first = place.proposed("money-has-currency", MONEY);
    let second = place.proposed(
        "refund-within-payment",
        "A refund never exceeds the payment.",
    );
    let log = place.log();
    let numbers: Vec<u64> = log
        .iter()
        .map(|entry| entry["number"].as_u64().unwrap())
        .collect();
    assert_eq!(numbers, [1, 2, 3]);
    assert_eq!(log[0]["act"], "registered");
    let version = |entry: &serde_json::Value| {
        let signed: serde_json::Value =
            serde_json::from_str(entry["signed"].as_str().unwrap()).unwrap();
        signed["claim"]["version"].as_str().unwrap().to_owned()
    };
    assert_eq!(version(&log[1]), first);
    assert_eq!(version(&log[2]), second);
}

#[test]
fn each_entry_gives_its_hash_and_the_one_before() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &version, &ada);
    let log = place.log();
    assert_eq!(log.len(), 3);
    assert_eq!(log[0]["prev"], boom::entry::GENESIS);
    for (at, entry) in log.iter().enumerate() {
        if at > 0 {
            assert_eq!(entry["prev"], log[at - 1]["hash"]);
        }
        assert_eq!(
            entry["hash"],
            boom::entry::hash(
                entry["prev"].as_str().unwrap(),
                entry["signed"].as_str().unwrap(),
                entry["signature"].as_str()
            )
        );
    }
}

#[test]
fn an_entrys_words_changed_is_found_by_verify() {
    let place = Place::new();
    place.three_entries();
    let changed = place
        .sqlite()
        .execute(
            "UPDATE log SET signed = replace(signed, 'its currency', 'any currency') WHERE seq = 2",
            [],
        )
        .unwrap();
    assert_eq!(changed, 1);
    let rows = place.rows();
    // A command reads the views' head and what follows it, not entry 2.
    let shown = place.run(&["show", "money-has-currency"]);
    assert_eq!(shown.code, 0, "{shown:?}");
    let outcome = place.run(&["verify"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome.stderr.contains("is altered: entry 2 "),
        "{outcome:?}"
    );
    assert!(outcome.stdout.is_empty(), "{outcome:?}");
    assert_eq!(place.rows(), rows);
}

#[test]
fn the_head_entrys_words_changed_is_found_by_every_command() {
    let place = Place::new();
    place.three_entries();
    let changed = place
        .sqlite()
        .execute(
            "UPDATE log SET signed = replace(signed, 'the payment', 'any payment') WHERE seq = 3",
            [],
        )
        .unwrap();
    assert_eq!(changed, 1);
    let rows = place.rows();
    let outcome = place.run(&["show", "money-has-currency"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome.stderr.contains("is altered: entry 3 "),
        "{outcome:?}"
    );
    assert!(outcome.stdout.is_empty(), "{outcome:?}");
    assert_eq!(place.rows(), rows);
}

#[test]
fn an_entry_appended_by_hand_that_does_not_follow_is_found() {
    let place = Place::new();
    place.three_entries();
    let conn = place.sqlite();
    let signed: String = conn
        .query_row("SELECT signed FROM log WHERE seq = 3", [], |row| row.get(0))
        .unwrap();
    let signed = signed.replace("refund-within-payment", "refund-within-order");
    let prev = boom::entry::GENESIS;
    conn.execute(
        "INSERT INTO log (seq, prev, hash, signed, signature) VALUES (4, ?1, ?2, ?3, NULL)",
        rusqlite::params![prev, boom::entry::hash(prev, &signed, None), signed],
    )
    .unwrap();
    let head: i64 = conn
        .query_row("SELECT seq FROM views_head", [], |row| row.get(0))
        .unwrap();
    drop(conn);
    let outcome = place.run(&["query", "--repository", "shop"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome.stderr.contains("is altered: entry 4 "),
        "{outcome:?}"
    );
    assert!(
        outcome.stderr.contains("previous hash does not match"),
        "{outcome:?}"
    );
    let after: i64 = place
        .sqlite()
        .query_row("SELECT seq FROM views_head", [], |row| row.get(0))
        .unwrap();
    assert_eq!(after, head, "the views are as they were");
}

#[test]
fn an_entry_removed_is_found_by_verify() {
    let place = Place::new();
    place.three_entries();
    let removed = place
        .sqlite()
        .execute("DELETE FROM log WHERE seq = 2", [])
        .unwrap();
    assert_eq!(removed, 1);
    let rows = place.rows();
    let outcome = place.run(&["verify"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome.stderr.contains("is altered: entry 3 "),
        "{outcome:?}"
    );
    assert!(
        outcome.stderr.contains("previous hash does not match"),
        "{outcome:?}"
    );
    assert_eq!(place.rows(), rows);
}

#[test]
fn boom_log_checks_the_whole_chain() {
    let place = Place::new();
    place.three_entries();
    place
        .sqlite()
        .execute(
            "UPDATE log SET signed = replace(signed, 'its currency', 'any currency') WHERE seq = 2",
            [],
        )
        .unwrap();
    let outcome = place.run(&["log"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome.stderr.contains("is altered: entry 2 "),
        "{outcome:?}"
    );
}

#[test]
fn an_accept_written_by_hand_counts_for_nothing() {
    let place = Place::new();
    place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.append_by_hand(
        &support::accept_entry("ada", "money-has-currency", &version),
        None,
    );
    assert_eq!(place.status("money-has-currency", &version), "proposed");
    let accepted = place.log().pop().unwrap();
    assert_eq!(accepted["act"], "accepted");
    assert_eq!(accepted["counted"], false);
    assert_eq!(accepted["reason"], "a person's act needs their signature");
    let words = place.run(&["log"]);
    assert!(
        words
            .stdout
            .contains(&format!(
                "3 accepted money-has-currency@{version} by ada, not counted: a person's act needs their signature"
            )),
        "{words:?}"
    );
}

#[test]
fn an_accept_by_a_key_no_one_registered_counts_for_nothing() {
    let place = Place::new();
    place.made();
    let version = place.proposed("money-has-currency", MONEY);
    let cy = place.keys.security_key("cy", 3);
    place.append_by_hand(
        &support::accept_entry("ada", "money-has-currency", &version),
        Some(&cy),
    );
    assert_eq!(place.status("money-has-currency", &version), "proposed");
    let accepted = place.log().pop().unwrap();
    assert_eq!(accepted["counted"], false);
    assert_eq!(accepted["reason"], "its key is not registered");
}

#[test]
fn each_line_names_the_version_of_its_act() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &version, &ada);
    let words = place.run(&["log"]).stdout;
    let lines: Vec<&str> = words.lines().collect();
    assert_eq!(lines[0], "1 registered by ada", "{words}");
    assert!(
        lines[1].starts_with(&format!(
            "2 proposed money-has-currency@{version} by an agent"
        )),
        "{words}"
    );
    assert_eq!(
        lines[2],
        format!("3 accepted money-has-currency@{version} by ada")
    );
    let entries = place.log();
    assert_eq!(entries[0]["target"], serde_json::Value::Null);
    assert_eq!(
        entries[1]["target"],
        format!("money-has-currency@{version}")
    );
    assert_eq!(
        entries[2]["target"],
        format!("money-has-currency@{version}")
    );
}

#[test]
fn an_entry_number_shows_the_version_it_names() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &version, &ada);
    let by_name = place.show(&format!("money-has-currency@{version}"));
    assert_eq!(place.show("2"), by_name);
    assert_eq!(place.show("3"), by_name);
    place.refused(&["show", "1"], "entry 1 names no version");
    place.refused(&["show", "9"], "the log has no entry 9");
}

#[test]
fn a_note_is_an_entry() {
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
    let proposed = place.log().last().unwrap().clone();
    place.noted(
        &format!("boom-end-to-end@{version}"),
        2,
        "Say what a claim is.",
    );
    let log = place.log();
    let at = log.len() - 1;
    assert_eq!(log[at]["act"], "noted");
    assert_eq!(log[at - 1]["act"], "proposed");
    assert_eq!(log[at - 1], proposed, "the proposal is as it was");
    let words = place.run(&["log"]).stdout;
    assert!(
        words.contains(&format!("noted boom-end-to-end@{version} by chuck")),
        "{words}"
    );
    // The views agree with a replay of the whole log.
    assert_eq!(place.json(&["verify"])["rebuilt"], serde_json::json!([]));
}
