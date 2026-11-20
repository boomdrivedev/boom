//! Claims by name, their versions and amendments, one name space, and
//! each version's status read from the log (claim-versions).

mod support;

use serde_json::json;
use support::Place;

const MONEY: &str = "Every amount names its currency.";

#[test]
fn a_claim_proposed_waits_on_a_person() {
    let place = Place::new();
    place.made();
    let outcome = place.run(&[
        "propose",
        "invariant",
        "refund-within-payment",
        "A refund never exceeds what the order was paid.",
        "--provenance",
        "run-1",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let shown = place.show("refund-within-payment");
    assert_eq!(shown["format"], "boom-show/2");
    assert_eq!(shown["kind"], "invariant");
    assert_eq!(shown["accepted"], json!(null));
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0]["status"], "proposed");
    assert_eq!(versions[0]["base"], json!(null));
    assert_eq!(
        versions[0]["proposer"],
        json!({"agent": {"provenance": "run-1"}})
    );
}

#[test]
fn a_name_not_in_kebab_case_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "invariant",
            "Refund_Within",
            MONEY,
            "--provenance",
            "run-1",
        ],
        "a name is kebab case",
    );
}

#[test]
fn an_invariant_needs_a_statement_alone() {
    let place = Place::new();
    place.made();
    let version = place.proposed("money-has-currency", MONEY);
    let shown = place.show(&format!("money-has-currency@{version}"));
    assert_eq!(shown["statement"], MONEY);
    assert_eq!(shown["scenarios"], json!([]));
    assert_eq!(shown["body"], json!(null));
    let signed: serde_json::Value =
        serde_json::from_str(place.log()[1]["signed"].as_str().unwrap()).unwrap();
    assert_eq!(
        signed["claim"].get("scenarios"),
        None,
        "a new entry writes no scenarios"
    );
}

#[test]
fn a_decision_with_no_body_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "decision",
            "amounts-in-cents",
            "Amounts are kept in integer cents.",
            "--provenance",
            "run-1",
        ],
        "a decision needs a body",
    );
    let body = place.decision_body();
    let answer = place.json(&[
        "propose",
        "decision",
        "amounts-in-cents",
        "Amounts are kept in integer cents.",
        "--body",
        &body,
        "--provenance",
        "run-1",
    ]);
    assert_eq!(answer["status"], "proposed");
}

#[test]
fn a_statement_of_two_lines_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "invariant",
            "money-has-currency",
            "Every amount names its currency.\nAnd only one.",
            "--provenance",
            "run-1",
        ],
        "a statement is one line",
    );
}

#[test]
fn the_same_proposal_twice_is_one_version() {
    let place = Place::new();
    place.made();
    let first = place.proposed("money-has-currency", MONEY);
    let second = place.proposed("money-has-currency", MONEY);
    assert_eq!(first, second);
    assert_eq!(place.acts(), ["registered", "proposed"]);
}

#[test]
fn one_word_changed_is_another_version() {
    let place = Place::new();
    place.made();
    let first = place.proposed("money-has-currency", MONEY);
    let second = place.proposed("money-has-currency", "Every amount names one currency.");
    assert_ne!(first, second);
    assert_eq!(place.acts(), ["registered", "proposed", "proposed"]);
}

#[test]
fn a_name_taken_by_another_kind_is_refused() {
    let place = Place::new();
    place.made();
    place.proposed("billing", "Every invoice is billed once.");
    let body = place.decision_body();
    place.refused(
        &[
            "propose",
            "decision",
            "billing",
            "We bill monthly.",
            "--body",
            &body,
            "--provenance",
            "run-1",
        ],
        "the invariant billing",
    );
}

#[test]
fn two_first_versions_wait_side_by_side() {
    let place = Place::new();
    place.made();
    let first = place.proposed("money-has-currency", MONEY);
    let second = place.proposed("money-has-currency", "Every amount names one currency.");
    let shown = place.show("money-has-currency");
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0]["version"].as_str(), Some(first.as_str()));
    assert_eq!(versions[1]["version"].as_str(), Some(second.as_str()));
    assert!(versions
        .iter()
        .all(|version| version["status"] == "proposed"));
    assert_eq!(shown["accepted"], json!(null));
}

#[test]
fn a_base_the_claim_does_not_have_is_refused() {
    let place = Place::new();
    place.made();
    place.proposed("money-has-currency", MONEY);
    let refused = place.refused(
        &[
            "propose",
            "invariant",
            "money-has-currency",
            "Every amount names one currency.",
            "--amends",
            "ffffffffffff",
            "--provenance",
            "run-2",
        ],
        "money-has-currency",
    );
    assert!(refused.stderr.contains("ffffffffffff"), "{refused:?}");
}

#[test]
fn an_unknown_name_is_refused() {
    let place = Place::new();
    place.made();
    let outcome = place.run(&["show", "no-such-claim"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome
            .stderr
            .contains("no claim has the name no-such-claim"),
        "{outcome:?}"
    );
}

#[test]
fn a_version_shown_gives_its_body() {
    let place = Place::new();
    place.made();
    let body = place.file("money.md", support::BODY);
    let answer = place.proposed_with_body("money-has-currency", MONEY, &body);
    let version = answer["version"].as_str().unwrap();
    let shown = place.show(&format!("money-has-currency@{version}"));
    assert_eq!(shown["format"], "boom-show/2");
    assert_eq!(shown["version"], version);
    assert_eq!(shown["statement"], MONEY);
    assert_eq!(shown["scenarios"], json!([]));
    assert_eq!(shown["status"], "proposed");
    assert_eq!(shown["body"], support::BODY);
}

const ONE: &str = "Every amount names one currency.";
const ISO: &str = "Every amount names its currency by its ISO 4217 code.";

#[test]
fn a_name_whose_claim_stands_is_refused() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &first, &ada);
    let refused = place.refused(
        &[
            "propose",
            "invariant",
            "money-has-currency",
            ONE,
            "--provenance",
            "run-2",
        ],
        &format!("money-has-currency@{first} stands"),
    );
    assert!(refused.stderr.contains("amend it"), "{refused:?}");
}

#[test]
fn an_amendment_names_its_base() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &first, &ada);
    let amendment = place.amended("money-has-currency", ONE, &first);
    let shown = place.show(&format!("money-has-currency@{amendment}"));
    assert_eq!(shown["status"], "proposed");
    assert_eq!(shown["base"], first.as_str());
    assert_eq!(place.status("money-has-currency", &first), "accepted");
    assert_eq!(place.show("money-has-currency")["accepted"], first.as_str());
}

#[test]
fn two_amendments_of_one_base() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &first, &ada);
    let one = place.amended("money-has-currency", ONE, &first);
    let iso = place.amended("money-has-currency", ISO, &first);
    place.accepted("money-has-currency", &one, &ada);
    place.refused(
        &[
            "accept",
            &format!("money-has-currency@{iso}"),
            "--key",
            &ada,
        ],
        &format!("money-has-currency@{one} stands"),
    );
    assert_eq!(place.status("money-has-currency", &iso), "proposed");
}

#[test]
fn the_same_words_on_the_new_base_stand() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &first, &ada);
    let one = place.amended("money-has-currency", ONE, &first);
    let iso = place.amended("money-has-currency", ISO, &first);
    place.accepted("money-has-currency", &one, &ada);
    let iso_on_one = place.amended("money-has-currency", ISO, &one);
    assert_ne!(iso_on_one, iso);
    place.accepted("money-has-currency", &iso_on_one, &ada);
    assert_eq!(place.status("money-has-currency", &iso_on_one), "accepted");
    assert_eq!(place.status("money-has-currency", &iso), "proposed");
    assert_eq!(place.status("money-has-currency", &one), "superseded");
}

#[test]
fn proposed_standing_replaced() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", MONEY);
    assert_eq!(place.status("money-has-currency", &first), "proposed");
    place.accepted("money-has-currency", &first, &ada);
    let amendment = place.amended("money-has-currency", ONE, &first);
    place.accepted("money-has-currency", &amendment, &ada);
    let shown = place.show("money-has-currency");
    assert_eq!(shown["accepted"], amendment.as_str());
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions[0]["version"], first.as_str());
    assert_eq!(versions[0]["status"], "superseded");
    assert_eq!(versions[0]["accepted_by"], "ada");
    assert_eq!(versions[1]["version"], amendment.as_str());
    assert_eq!(versions[1]["status"], "accepted");
    assert_eq!(versions[1]["base"], first.as_str());
}

#[test]
fn a_rejected_version_gives_who_and_why() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    let target = format!("money-has-currency@{version}");
    let reason = "An amount in a ledger of one currency names none.";
    let answer = place.json(&["reject", &target, "--reason", reason, "--key", &ada]);
    assert_eq!(answer["format"], "boom-reject/2");
    assert_eq!(answer["status"], "rejected");
    let again = place.json(&["reject", &target, "--reason", reason, "--key", &ada]);
    assert_eq!(again, answer);
    assert_eq!(place.acts(), ["registered", "proposed", "rejected"]);
    let shown = place.show("money-has-currency");
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions[0]["status"], "rejected");
    assert_eq!(versions[0]["rejected_by"], "ada");
    assert_eq!(versions[0]["accepted_by"], json!(null));
    assert_eq!(versions[0]["reason"], reason);
    assert_eq!(shown["accepted"], json!(null));
}

const AMOUNTS_AS_FLOATS: &str = "---
status: superseded by 0003
---

# Amounts are floats

Amounts are kept as floats.
";

const AMOUNTS_IN_CENTS: &str = "---
status: accepted
---

# Amounts are kept in cents

Every amount is an integer of cents.
";

/// Imports the shop's superseded `0002` with `0003`, drafting the
/// scenarios given for `0002`, giving `amounts-as-floats`'s version.
fn floats_imported(place: &Place) -> String {
    let dir = place.folder(
        "adr",
        &[
            ("0002-amounts-as-floats.md", AMOUNTS_AS_FLOATS),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
        ],
    );
    let drafts = json!({
        "format": "boom-import-drafts/2",
        "records": {
            "0002-amounts-as-floats.md": {},
            "0003-amounts-in-cents.md": {},
        },
    });
    let drafts = place.file("drafts.json", &drafts.to_string());
    let answer = place.json(&[
        "import",
        "adr",
        &dir,
        "--repository",
        "shop",
        "--drafts",
        &drafts,
        "--provenance",
        "run-1",
    ]);
    let floats = answer["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["source"] == "0002-amounts-as-floats.md")
        .unwrap();
    floats["version"].as_str().unwrap().to_owned()
}

#[test]
fn history_with_no_scenario() {
    let place = Place::new();
    place.made();
    let version = floats_imported(&place);
    let shown = place.show(&format!("amounts-as-floats@{version}"));
    assert_eq!(shown["status"], "proposed");
    assert_eq!(shown["scenarios"], json!([]));
    assert_eq!(shown["history"]["status"], "superseded");
    assert_eq!(shown["history"]["by"], "amounts-in-cents");
}

#[test]
fn the_same_words_as_history_are_another_version() {
    let place = Place::new();
    place.made();
    let history = floats_imported(&place);
    let body = place.file("floats.md", "Amounts are kept as floats.\n");
    let proposed = place.json(&[
        "propose",
        "decision",
        "amounts-as-floats",
        "Amounts are floats",
        "--body",
        &body,
        "--provenance",
        "run-2",
    ]);
    let standing = proposed["version"].as_str().unwrap();
    assert_ne!(standing, history);
    let shown = place.show(&format!("amounts-as-floats@{history}"));
    let plain = place.show(&format!("amounts-as-floats@{standing}"));
    for field in ["statement", "scenarios", "body_fingerprint"] {
        assert_eq!(shown[field], plain[field], "{field}");
    }
    assert_eq!(plain["history"], json!(null));
    assert_eq!(
        place.show("amounts-as-floats")["versions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn history_accepted_keeps_its_files_status() {
    let place = Place::new();
    let ada = place.made();
    let version = floats_imported(&place);
    let answer = place.accepted("amounts-as-floats", &version, &ada);
    assert_eq!(answer["status"], "superseded");
    let shown = place.show("amounts-as-floats");
    assert_eq!(shown["accepted"], json!(null));
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0]["status"], "superseded");
    assert_eq!(versions[0]["accepted_by"], "ada");
    let again = place.accepted("amounts-as-floats", &version, &ada);
    assert_eq!(again, answer);
    let first = place.proposed_decision("amounts-as-floats", "Amounts are floats again");
    place.accepted("amounts-as-floats", &first, &ada);
    assert_eq!(place.status("amounts-as-floats", &first), "accepted");
    assert_eq!(place.status("amounts-as-floats", &version), "superseded");
}
