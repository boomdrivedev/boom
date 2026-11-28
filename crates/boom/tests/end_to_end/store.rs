//! Two runs of the binary that write one store at one time: each one
//! lands, chained (record-store). The other tests of the store run in
//! the crate, in src/tests/store.rs.

use super::support;

use support::Place;

fn agents_proposal_command(place: &Place, name: &str) -> std::process::Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_boom"));
    command
        .args([
            "propose",
            "invariant",
            name,
            "Every amount names its currency.",
            "--provenance",
            name,
            "--store",
        ])
        .arg(&place.store)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null());
    command
}

#[test]
fn two_proposals_at_once_both_land_chained() {
    let _slot = super::slot();
    let place = Place::new();
    place.made();
    let mut one = agents_proposal_command(&place, "money-has-currency")
        .spawn()
        .unwrap();
    let mut two = agents_proposal_command(&place, "refund-within-payment")
        .spawn()
        .unwrap();
    assert_eq!(one.wait().unwrap().code(), Some(0));
    assert_eq!(two.wait().unwrap().code(), Some(0));
    let log = place.log();
    assert_eq!(log.len(), 3);
    let mut proposed: Vec<String> = log[1..]
        .iter()
        .map(|entry| {
            entry["actor"]["agent"]["provenance"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    proposed.sort();
    assert_eq!(proposed, ["money-has-currency", "refund-within-payment"]);
    for at in 1..log.len() {
        assert_eq!(log[at]["prev"], log[at - 1]["hash"]);
        assert_eq!(log[at]["counted"], true);
    }
}
