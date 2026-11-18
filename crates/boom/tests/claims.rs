//! Claims by name, their versions and amendments, one name space, and
//! each version's status read from the log (claim-versions).

mod support;

use serde_json::json;
use support::{Place, SCENARIO};

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
        "--scenario",
        "Given an order paid 40.00, when a refund of 45.00 is asked, then it is refused.",
        "--provenance",
        "run-1",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let shown = place.show("refund-within-payment");
    assert_eq!(shown["format"], "boom-show/1");
    assert_eq!(shown["kind"], "invariant");
    assert_eq!(shown["standing"], json!(null));
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
            "--scenario",
            SCENARIO,
            "--provenance",
            "run-1",
        ],
        "a name is kebab case",
    );
}

#[test]
fn a_claim_with_no_scenario_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "invariant",
            "money-has-currency",
            MONEY,
            "--provenance",
            "run-1",
        ],
        "a claim needs a scenario",
    );
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
            "--scenario",
            SCENARIO,
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
    place.refused(
        &[
            "propose",
            "decision",
            "billing",
            "We bill monthly.",
            "--scenario",
            SCENARIO,
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
    assert_eq!(shown["standing"], json!(null));
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
            "--scenario",
            SCENARIO,
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
    assert_eq!(shown["format"], "boom-show/1");
    assert_eq!(shown["version"], version);
    assert_eq!(shown["statement"], MONEY);
    assert_eq!(shown["scenarios"], json!([SCENARIO]));
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
            "--scenario",
            SCENARIO,
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
    assert_eq!(place.status("money-has-currency", &first), "standing");
    assert_eq!(place.show("money-has-currency")["standing"], first.as_str());
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
    assert_eq!(place.status("money-has-currency", &iso_on_one), "standing");
    assert_eq!(place.status("money-has-currency", &iso), "proposed");
    assert_eq!(place.status("money-has-currency", &one), "replaced");
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
    assert_eq!(shown["standing"], amendment.as_str());
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions[0]["version"], first.as_str());
    assert_eq!(versions[0]["status"], "replaced");
    assert_eq!(versions[0]["accepted_by"], "ada");
    assert_eq!(versions[1]["version"], amendment.as_str());
    assert_eq!(versions[1]["status"], "standing");
    assert_eq!(versions[1]["base"], first.as_str());
}
