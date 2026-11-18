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
