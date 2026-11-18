//! The log: entries appended and never edited, numbered in order,
//! chained, and each judged whenever it is read (record-log).

mod support;

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
    assert_eq!(place.status("money-has-currency", &version), "standing");
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
fn an_entrys_words_changed_is_found() {
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
    let outcome = place.run(&["show", "money-has-currency"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome.stderr.contains("is altered: entry 2 "),
        "{outcome:?}"
    );
    assert!(outcome.stdout.is_empty(), "{outcome:?}");
    assert_eq!(place.rows(), rows);
}

#[test]
fn an_entry_removed_is_found() {
    let place = Place::new();
    place.three_entries();
    let removed = place
        .sqlite()
        .execute("DELETE FROM log WHERE seq = 2", [])
        .unwrap();
    assert_eq!(removed, 1);
    let rows = place.rows();
    let outcome = place.run(&[
        "propose",
        "assumption",
        "orders-are-paid-once",
        "An order is paid once.",
        "--scenario",
        support::SCENARIO,
        "--provenance",
        "run-3",
    ]);
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
