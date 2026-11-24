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
        &format!("money-has-currency@{first} is accepted"),
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
        &format!("money-has-currency@{one} is accepted"),
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

/// A store of ada's with `money-has-currency@<v1>` accepted, `<v2>`
/// amending `<v1>` and `<v3>` amending the draft `<v2>`, giving ada's
/// key and the three versions.
fn a_draft_of_a_draft(place: &Place) -> (String, [String; 3]) {
    let ada = place.made();
    let v1 = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &v1, &ada);
    let v2 = place.amended("money-has-currency", ONE, &v1);
    let v3 = place.amended("money-has-currency", ISO, &v2);
    (ada, [v1, v2, v3])
}

#[test]
fn a_draft_amended() {
    let place = Place::new();
    let (_, [v1, v2, v3]) = a_draft_of_a_draft(&place);
    let shown = place.show("money-has-currency");
    assert_eq!(shown["accepted"], v1.as_str());
    assert_eq!(shown["newest"]["version"], v3.as_str());
    assert_eq!(place.status("money-has-currency", &v2), "superseded");
    assert_eq!(place.status("money-has-currency", &v3), "proposed");
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions[1]["status"], "superseded");
    assert_eq!(versions[1]["accepted_by"], json!(null));
}

#[test]
fn the_newest_draft_accepted() {
    let place = Place::new();
    let (ada, [v1, v2, v3]) = a_draft_of_a_draft(&place);
    place.accepted("money-has-currency", &v3, &ada);
    assert_eq!(place.status("money-has-currency", &v3), "accepted");
    assert_eq!(place.status("money-has-currency", &v1), "superseded");
    assert_eq!(place.status("money-has-currency", &v2), "superseded");
    assert_eq!(place.show("money-has-currency")["accepted"], v3.as_str());
}

#[test]
fn a_first_draft_amended() {
    let place = Place::new();
    let ada = place.made();
    let v1 = place.proposed("money-has-currency", MONEY);
    let v2 = place.amended("money-has-currency", ONE, &v1);
    assert_eq!(place.status("money-has-currency", &v1), "superseded");
    place.accepted("money-has-currency", &v2, &ada);
    assert_eq!(place.status("money-has-currency", &v2), "accepted");
    assert_eq!(place.status("money-has-currency", &v1), "superseded");
}

#[test]
fn an_old_draft_accepted() {
    let place = Place::new();
    let (ada, [_, v2, v3]) = a_draft_of_a_draft(&place);
    let before = place.log();
    place.refused(
        &["accept", &format!("money-has-currency@{v2}"), "--key", &ada],
        &format!("superseded by the newer draft money-has-currency@{v3}"),
    );
    assert_eq!(place.log(), before);
}

#[test]
fn an_old_draft_rejected() {
    let place = Place::new();
    let (ada, [_, v2, v3]) = a_draft_of_a_draft(&place);
    let before = place.log();
    place.refused(
        &[
            "reject",
            &format!("money-has-currency@{v2}"),
            "--reason",
            "Too narrow.",
            "--key",
            &ada,
        ],
        &format!("superseded by the newer draft money-has-currency@{v3}"),
    );
    assert_eq!(place.log(), before);
}

#[test]
fn an_old_draft_amended() {
    let place = Place::new();
    let (_, [_, v2, v3]) = a_draft_of_a_draft(&place);
    let before = place.log();
    place.refused(
        &[
            "propose",
            "invariant",
            "money-has-currency",
            "Every amount names its currency, always.",
            "--amends",
            &v2,
            "--provenance",
            "run-3",
        ],
        &format!("amend that draft, with --amends {v3}"),
    );
    assert_eq!(place.log(), before);
}

#[test]
fn the_newer_draft_rejected() {
    let place = Place::new();
    let (ada, [_, v2, v3]) = a_draft_of_a_draft(&place);
    let target = format!("money-has-currency@{v3}");
    place.json(&["reject", &target, "--reason", "Too narrow.", "--key", &ada]);
    assert_eq!(place.status("money-has-currency", &v3), "rejected");
    assert_eq!(place.status("money-has-currency", &v2), "superseded");
}

#[test]
fn a_log_from_before_this_rule() {
    let place = Place::new();
    let (_, [_, v2, _]) = a_draft_of_a_draft(&place);
    // Views as the release before this rule wrote them: form 8, with the
    // old draft still proposed.
    place
        .sqlite()
        .execute_batch(&format!(
            "UPDATE versions SET status = 'proposed' WHERE version = '{v2}'; \
             UPDATE views_head SET form = 8;"
        ))
        .unwrap();
    assert_eq!(place.status("money-has-currency", &v2), "superseded");
    assert_eq!(place.json(&["verify"])["rebuilt"], json!([]));
}

#[test]
fn an_amendment_of_a_rejected_version() {
    let place = Place::new();
    let ada = place.made();
    let v1 = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &v1, &ada);
    let v2 = place.amended("money-has-currency", ONE, &v1);
    let target = format!("money-has-currency@{v2}");
    place.json(&["reject", &target, "--reason", "Too narrow.", "--key", &ada]);
    let v3 = place.amended("money-has-currency", ISO, &v2);
    place.accepted("money-has-currency", &v3, &ada);
    assert_eq!(place.status("money-has-currency", &v3), "accepted");
    assert_eq!(place.status("money-has-currency", &v2), "rejected");
}

#[test]
fn a_draft_of_a_draft_on_an_old_base() {
    let place = Place::new();
    let ada = place.made();
    let v1 = place.proposed("money-has-currency", MONEY);
    let v2 = place.amended("money-has-currency", ONE, &v1);
    let w1 = place.proposed("money-has-currency", ISO);
    place.accepted("money-has-currency", &w1, &ada);
    place.refused(
        &["accept", &format!("money-has-currency@{v2}"), "--key", &ada],
        &format!("money-has-currency@{w1} is accepted"),
    );
    assert_eq!(place.status("money-has-currency", &v2), "proposed");
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

#[test]
fn a_name_shows_its_newest_version_in_full_then_the_others() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &first, &ada);
    let amendment = place.amended("money-has-currency", ONE, &first);
    let shown = place.show("money-has-currency");
    assert_eq!(shown["accepted"], first.as_str());
    assert_eq!(shown["newest"]["version"], amendment.as_str());
    assert_eq!(shown["newest"]["statement"], ONE);
    let words = place.run(&["show", "money-has-currency"]).stdout;
    let newest = words.find(ONE).expect(&words);
    let others = words.find("other versions:").expect(&words);
    let older = words.rfind(MONEY).expect(&words);
    assert!(newest < others && others < older, "{words}");
    assert!(
        words.starts_with(&format!("invariant money-has-currency, accepted: {first}")),
        "{words}"
    );
}

#[test]
fn one_more_include() {
    let place = Place::new();
    place.made();
    place.proposed(
        "status-read-from-events",
        "A record never holds its status.",
    );
    place.proposed("events-are-appended", "boom appends an event.");
    let first = place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &["--includes", "status-read-from-events"],
    );
    let second = place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &[
            "--includes",
            "status-read-from-events",
            "--includes",
            "events-are-appended",
        ],
    );
    assert_ne!(first["version"], second["version"]);
}

#[test]
fn a_version_from_before_narratives() {
    let place = Place::new();
    let ada = place.made();
    let money = place.proposed("money-has-currency", "Every amount names its currency.");
    place.accepted("money-has-currency", &money, &ada);
    let body = place.decision_body();
    let decision = place.json(&[
        "propose",
        "decision",
        "amounts-in-cents",
        "Amounts are in cents.",
        "--body",
        &body,
        "--source",
        "money-has-currency",
        "--provenance",
        "run-1",
    ])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    // Each entry written as a boom before narratives wrote it: no
    // `includes` and no `answers`.
    for entry in place.log() {
        let signed = entry["signed"].as_str().unwrap();
        assert!(!signed.contains("\"includes\""), "{signed}");
        assert!(!signed.contains("\"answers\""), "{signed}");
    }
    // Views as the release before narratives wrote them: form 7, with no
    // table of notes; then none at all.
    let before = place.run(&["show", "amounts-in-cents", "--json"]).stdout;
    place
        .sqlite()
        .execute_batch("DROP TABLE notes; UPDATE views_head SET form = 7;")
        .unwrap();
    assert_eq!(
        place.run(&["show", "amounts-in-cents", "--json"]).stdout,
        before
    );
    place
        .sqlite()
        .execute_batch("DROP TABLE versions; DROP TABLE views_head;")
        .unwrap();
    for (name, version) in [
        ("money-has-currency", &money),
        ("amounts-in-cents", &decision),
    ] {
        assert_eq!(
            place.show(&format!("{name}@{version}"))["version"],
            *version
        );
    }
    assert_eq!(place.json(&["verify"])["rebuilt"], json!([]));
}

#[test]
fn an_outline_with_its_title() {
    let place = Place::new();
    place.made();
    place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &[],
    );
    place.json(&[
        "propose",
        "outline",
        "boomdrive-design-book",
        "The design book of Boomdrive",
        "--view",
        "book",
        "--narrative",
        "boom-end-to-end",
        "--provenance",
        "run-1",
    ]);
    let shown = place.show("boomdrive-design-book");
    assert_eq!(shown["kind"], "outline");
    assert_eq!(shown["newest"]["body"], serde_json::Value::Null);
}
