//! `boom supersede` proposes a version that supersedes the accepted
//! version of another claim, by a new claim or by an accepted claim of
//! the same kind, and a person's accept takes that claim out of force
//! (claim-versions, "boom supersede proposes a version that supersedes
//! an accepted claim", "A supersession by a new claim", "A supersession
//! by an accepted claim of the same kind", "A version names the accepted
//! version that it supersedes", "The accept of a supersession takes the
//! claim it names out of force" and "The person reads what a
//! supersession takes out of force").

use super::support;

use support::Place;

const BILLING: &str = "billing-owns-invoices";
const WRITERS: &str = "invoice-writers";
const BILLING_WORDS: &str = "Only billing writes invoices.";
const WRITERS_WORDS: &str = "Only billing and refunds write invoices.";
const REASON: &str = "Refunds issue their own credit notes.";

fn at(name: &str, version: &str) -> String {
    format!("{name}@{version}")
}

/// An agent's proposal of a first version of the invariant `name` with
/// `extra`, giving its version.
fn proposed(place: &Place, name: &str, statement: &str, extra: &[&str]) -> String {
    let mut args = vec!["propose", "invariant", name, statement];
    args.extend(extra);
    args.extend(["--provenance", "run-1"]);
    place.json(&args)["version"].as_str().unwrap().to_owned()
}

/// A store with ada, and `billing-owns-invoices@<b1>`, an invariant that
/// binds `shop`, accepted by ada; gives ada's key and `<b1>`.
fn billing_in_shop(place: &Place) -> (String, String) {
    let ada = place.made();
    let b1 = proposed(place, BILLING, BILLING_WORDS, &["--repository", "shop"]);
    place.accepted(BILLING, &b1, &ada);
    (ada, b1)
}

// A version names the accepted version that it supersedes.

#[test]
fn a_version_no_longer_accepted_from_another_program() {
    let place = Place::new();
    let (ada, b1) = billing_in_shop(&place);
    let b2 = place.amended(BILLING, "Billing alone writes invoices.", &b1);
    place.accepted(BILLING, &b2, &ada);
    let mut claim = boom::claim::Proposal::new(
        boom::claim::Kind::Invariant,
        WRITERS,
        WRITERS_WORDS,
        Vec::new(),
        None,
        None,
        None,
    );
    claim.supersedes = Some(boom::claim::Supersession {
        claim: at(BILLING, &b1),
        reason: Some(REASON.to_owned()),
    });
    claim.version = claim.fingerprint(None);
    let entry = boom::entry::Entry {
        at: 1_763_424_600,
        rules: boom::judge::RULES,
        actor: boom::entry::Actor::Agent {
            provenance: "another-program".to_owned(),
        },
        act: boom::entry::Act::Proposed {
            claim: Box::new(claim),
            imported: None,
            attach: None,
        },
    };
    place.append_by_hand(&entry, None);
    let proposed = place.log().pop().unwrap();
    assert_eq!(proposed["counted"], false, "{proposed}");
    let reason = proposed["reason"].as_str().unwrap();
    assert!(
        reason.contains(&format!("{} is accepted", at(BILLING, &b2))),
        "{reason}"
    );
    let shown = place.run(&["show", WRITERS]);
    assert_eq!(shown.code, 2, "{shown:?}");
    assert_eq!(place.status(BILLING, &b2), "accepted");
}

/// The names of the claims `boom query --repository <repository>` gives.
fn queried(place: &Place, repository: &str) -> Vec<String> {
    place.json(&["query", "--repository", repository])["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| claim["name"].as_str().unwrap().to_owned())
        .collect()
}

/// `boom supersede billing-owns-invoices invoice-writers` with the
/// statement of invoice-writers, the reason and `extra`, by an agent.
fn superseded_by_writers(place: &Place, extra: &[&str]) -> serde_json::Value {
    let mut args = vec![
        "supersede",
        BILLING,
        WRITERS,
        WRITERS_WORDS,
        "--reason",
        REASON,
        "--provenance",
        "run-2",
    ];
    args.extend(extra);
    place.json(&args)
}

// boom supersede proposes a version that supersedes an accepted claim.

#[test]
fn a_proposal_that_waits() {
    let place = Place::new();
    let (_, b1) = billing_in_shop(&place);
    let entries = place.log().len();
    let outcome = place.run(&[
        "supersede",
        BILLING,
        WRITERS,
        WRITERS_WORDS,
        "--reason",
        REASON,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(
        outcome
            .stdout
            .contains(&format!("supersedes: {}", at(BILLING, &b1))),
        "{outcome:?}"
    );
    assert_eq!(place.acts().len(), entries + 1);
    assert_eq!(place.acts().last().unwrap(), "proposed");
    let shown = place.show(WRITERS);
    let w1 = shown["versions"][0]["version"].as_str().unwrap().to_owned();
    assert_eq!(place.status(WRITERS, &w1), "proposed");
    assert_eq!(place.status(BILLING, &b1), "accepted");
    assert_eq!(queried(&place, "shop"), [BILLING]);
}

#[test]
fn a_supersession_answered_as_json() {
    let place = Place::new();
    let (_, b1) = billing_in_shop(&place);
    let answer = superseded_by_writers(&place, &[]);
    assert_eq!(answer["format"], "boom-supersede/1");
    assert_eq!(answer["claim"], WRITERS);
    let w1 = answer["version"].as_str().unwrap();
    assert_eq!(w1.len(), 12);
    assert_eq!(answer["status"], "proposed");
    assert_eq!(answer["supersedes"], BILLING);
    assert_eq!(answer["supersedes_version"], b1.as_str());
    assert_eq!(answer["reason"], REASON);
    assert_eq!(
        answer["accept"],
        format!("boom accept {} --key <key>", at(WRITERS, w1))
    );
}

#[test]
fn a_claim_with_no_accepted_version() {
    let place = Place::new();
    place.made();
    proposed(&place, "cash-only", "We take cash only.", &["--system"]);
    place.refused(
        &[
            "supersede",
            "cash-only",
            "bank-transfers",
            "We take bank transfers.",
            "--provenance",
            "run-2",
        ],
        "cash-only has no accepted version to supersede",
    );
}

#[test]
fn a_claim_that_supersedes_itself() {
    let place = Place::new();
    billing_in_shop(&place);
    let outcome = place.refused(
        &["supersede", BILLING, BILLING, "--provenance", "run-2"],
        "a claim does not supersede itself",
    );
    assert!(
        outcome.stderr.contains("boom propose --amends"),
        "{outcome:?}"
    );
}

// A supersession by a new claim.

#[test]
fn a_new_claim_replaces_an_invariant() {
    let place = Place::new();
    let (_, b1) = billing_in_shop(&place);
    let answer = place.json(&[
        "supersede",
        BILLING,
        WRITERS,
        WRITERS_WORDS,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(answer["claim"], WRITERS);
    assert_eq!(answer["kind"], "invariant");
    assert_eq!(answer["base"], serde_json::Value::Null);
    assert_eq!(answer["repositories"], serde_json::json!(["shop"]));
    assert_eq!(answer["supersedes"], BILLING);
    assert_eq!(answer["supersedes_version"], b1.as_str());
    assert_eq!(answer["reason"], serde_json::Value::Null);
}

#[test]
fn a_new_claim_of_another_kind() {
    let place = Place::new();
    let (_, b1) = billing_in_shop(&place);
    let body = place.decision_body();
    let answer = place.json(&[
        "supersede",
        BILLING,
        WRITERS,
        "Refunds write their own credit notes.",
        "--kind",
        "decision",
        "--body",
        &body,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(answer["kind"], "decision");
    assert_eq!(answer["supersedes_version"], b1.as_str());
    let version = answer["version"].as_str().unwrap();
    let shown = place.show(&at(WRITERS, version));
    assert_eq!(shown["kind"], "decision");
    assert_eq!(shown["body"], support::DECISION, "{shown}");
}

#[test]
fn a_new_claim_with_no_statement() {
    let place = Place::new();
    billing_in_shop(&place);
    place.refused(
        &["supersede", BILLING, WRITERS, "--provenance", "run-2"],
        "a new claim gives its statement",
    );
}

// A version names the accepted version that it supersedes.

#[test]
fn a_term_superseded() {
    let place = Place::new();
    let ada = place.made();
    let answer = place.json(&[
        "propose",
        "term",
        "owner",
        "A person who registers persons.",
        "--repository",
        "shop",
        "--provenance",
        "run-1",
    ]);
    place.accepted("owner", answer["version"].as_str().unwrap(), &ada);
    place.refused(
        &[
            "supersede",
            "owner",
            "registrar",
            "A person who grants roles.",
            "--provenance",
            "run-2",
        ],
        "only an invariant, a decision or an assumption is superseded",
    );
}

// A supersession by an accepted claim of the same kind.

const KEEP: &str = "tests-as-units";
const KEEP_WORDS: &str = "A tool's logic is tested as units.";
const DUPLICATE: &str = "tests-as-units-too";
const DUPLICATE_WORDS: &str = "A tool's logic is tested as units, too.";

/// A store with ada, and the invariants `tests-as-units@<k1>`, with a body
/// and two sources, and `tests-as-units-too@<d1>`, each accepted and
/// binding the whole system; gives ada's key, `<k1>`, `<d1>` and the two
/// sources.
fn duplicates(place: &Place) -> (String, String, String, [String; 2]) {
    let ada = place.made();
    let sources = [
        ("money-has-currency", "Every amount names its currency."),
        (
            "status-read-from-events",
            "A record never holds its status.",
        ),
    ]
    .map(|(name, words)| at(name, &proposed(place, name, words, &["--system"])));
    let body = place.file("units.md", support::BODY);
    let k1 = proposed(
        place,
        KEEP,
        KEEP_WORDS,
        &[
            "--system",
            "--body",
            &body,
            "--source",
            &sources[0],
            "--source",
            &sources[1],
        ],
    );
    place.accepted(KEEP, &k1, &ada);
    let d1 = proposed(place, DUPLICATE, DUPLICATE_WORDS, &["--system"]);
    place.accepted(DUPLICATE, &d1, &ada);
    (ada, k1, d1, sources)
}

#[test]
fn a_duplicate_retired_by_the_claim_to_keep() {
    let place = Place::new();
    let (_, k1, d1, sources) = duplicates(&place);
    let answer = place.json(&[
        "supersede",
        DUPLICATE,
        KEEP,
        "--reason",
        "a duplicate",
        "--provenance",
        "run-2",
    ]);
    assert_eq!(answer["format"], "boom-supersede/1");
    assert_eq!(answer["claim"], KEEP);
    assert_eq!(answer["base"], k1.as_str());
    assert_eq!(answer["source"], serde_json::json!(sources));
    assert_eq!(answer["repositories"], serde_json::json!([]));
    assert_eq!(answer["contexts"], serde_json::json!([]));
    assert_eq!(answer["supersedes"], DUPLICATE);
    assert_eq!(answer["supersedes_version"], d1.as_str());
    assert_eq!(answer["reason"], "a duplicate");
    let k2 = answer["version"].as_str().unwrap();
    assert_ne!(k2, k1);
    let kept = place.show(&at(KEEP, &k1));
    let shown = place.show(&at(KEEP, k2));
    assert_eq!(shown["statement"], KEEP_WORDS);
    assert_eq!(shown["body_fingerprint"], kept["body_fingerprint"]);
    assert_eq!(
        shown["body_fingerprint"],
        boom::sha256_hex(support::BODY.as_bytes()).as_str()
    );
    assert_eq!(shown["body"], support::BODY);
    assert_eq!(place.status(KEEP, &k1), "accepted");
    assert_eq!(place.status(DUPLICATE, &d1), "accepted");
}

#[test]
fn new_words_for_the_claim_to_keep() {
    let place = Place::new();
    let (_, k1, d1, sources) = duplicates(&place);
    let words = "Unit tests prove a tool's logic.";
    let answer = place.json(&["supersede", DUPLICATE, KEEP, words, "--provenance", "run-2"]);
    assert_eq!(answer["base"], k1.as_str());
    assert_eq!(answer["source"], serde_json::json!(sources));
    assert_eq!(answer["supersedes_version"], d1.as_str());
    let shown = place.show(&at(KEEP, answer["version"].as_str().unwrap()));
    assert_eq!(shown["statement"], words);
    assert_eq!(shown["body"], support::BODY);
}

#[test]
fn an_accepted_claim_of_another_kind() {
    let place = Place::new();
    let (ada, ..) = duplicates(&place);
    let cents = place.proposed_decision("amounts-in-cents", "Amounts are in cents.");
    place.accepted("amounts-in-cents", &cents, &ada);
    place.refused(
        &[
            "supersede",
            DUPLICATE,
            "amounts-in-cents",
            "--provenance",
            "run-2",
        ],
        "amounts-in-cents is a decision and tests-as-units-too an invariant",
    );
    place.refused(
        &[
            "supersede",
            DUPLICATE,
            KEEP,
            "--kind",
            "decision",
            "--provenance",
            "run-2",
        ],
        "--kind decision: tests-as-units is an invariant",
    );
}

// The accept of a supersession takes the claim it names out of force.

/// The names of the claims `boom search <words> --all` gives.
fn searched(place: &Place, words: &[&str]) -> Vec<String> {
    let mut args = vec!["search"];
    args.extend(words);
    args.push("--all");
    place.json(&args)["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| claim["name"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn a_new_claim_accepted() {
    let place = Place::new();
    let (ada, b1) = billing_in_shop(&place);
    let w1 = superseded_by_writers(&place, &[])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = place.log();
    let answer = place.accepted(WRITERS, &w1, &ada);
    assert_eq!(answer["status"], "accepted");
    assert_eq!(place.status(WRITERS, &w1), "accepted");
    assert_eq!(place.status(BILLING, &b1), "superseded");
    assert_eq!(place.show(BILLING)["accepted"], serde_json::Value::Null);
    assert_eq!(queried(&place, "shop"), [WRITERS]);
    let log = place.log();
    assert_eq!(log.len(), before.len() + 1);
    assert_eq!(log[..before.len()], before[..]);
    assert!(log.iter().all(|entry| entry["counted"] == true), "{log:?}");
    assert_eq!(
        place.json(&["verify"])["rebuilt"],
        serde_json::json!([]),
        "the views an accept wrote are those a replay builds"
    );
}

#[test]
fn the_duplicate_leaves_force() {
    let place = Place::new();
    let (ada, k1, d1, _) = duplicates(&place);
    assert_eq!(searched(&place, &["units,", "too"]), [DUPLICATE]);
    let k2 = place.json(&[
        "supersede",
        DUPLICATE,
        KEEP,
        "--reason",
        "a duplicate",
        "--provenance",
        "run-2",
    ])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let answer = place.accepted(KEEP, &k2, &ada);
    assert_eq!(answer["status"], "accepted");
    assert_eq!(place.status(KEEP, &k2), "accepted");
    assert_eq!(place.status(KEEP, &k1), "superseded");
    assert_eq!(place.status(DUPLICATE, &d1), "superseded");
    let queried: Vec<(String, String)> = place.json(&["query", "--repository", "shop"])["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| {
            (
                claim["name"].as_str().unwrap().to_owned(),
                claim["version"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert!(
        queried.contains(&(KEEP.to_owned(), k2.clone())),
        "{queried:?}"
    );
    assert!(
        queried.iter().all(|(name, _)| name != DUPLICATE),
        "{queried:?}"
    );
    assert!(searched(&place, &["units,", "too"]).is_empty());
    assert_eq!(place.json(&["verify"])["rebuilt"], serde_json::json!([]));
}

#[test]
fn the_claim_named_moved_on() {
    let place = Place::new();
    let (ada, b1) = billing_in_shop(&place);
    let w1 = superseded_by_writers(&place, &[])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let b2 = place.amended(BILLING, "Billing alone writes invoices.", &b1);
    place.accepted(BILLING, &b2, &ada);
    let signatures = place.keys.signatures();
    let outcome = place.refused(
        &["accept", &at(WRITERS, &w1), "--key", &ada],
        &format!("{} is accepted", at(BILLING, &b2)),
    );
    assert!(outcome.stderr.contains(&at(WRITERS, &w1)), "{outcome:?}");
    assert_eq!(place.keys.signatures(), signatures);
    assert_eq!(place.status(WRITERS, &w1), "proposed");
}

#[test]
fn one_accept_names_both_claims() {
    let place = Place::new();
    let (ada, b1) = billing_in_shop(&place);
    let w1 = superseded_by_writers(&place, &[])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let b2 = place.amended(BILLING, "Billing alone writes invoices.", &b1);
    let signatures = place.keys.signatures();
    let outcome = place.refused(
        &[
            "accept",
            &at(WRITERS, &w1),
            &at(BILLING, &b2),
            "--key",
            &ada,
        ],
        "not both",
    );
    for named in [at(WRITERS, &w1), at(BILLING, &b2)] {
        assert!(outcome.stderr.contains(&named), "{named}: {outcome:?}");
    }
    assert_eq!(place.keys.signatures(), signatures);
    assert_eq!(place.status(WRITERS, &w1), "proposed");
    assert_eq!(place.status(BILLING, &b2), "proposed");
}

// The person reads what a supersession takes out of force.

#[test]
fn the_accept_lists_the_claim_it_supersedes() {
    let place = Place::new();
    let (ada, b1) = billing_in_shop(&place);
    let w1 = superseded_by_writers(&place, &[])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let told = place.keys.told().len();
    let outcome = place.run(&["accept", &at(WRITERS, &w1), "--key", &ada, "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let listed = place.keys.told()[told..].to_owned();
    assert!(
        listed.contains(&format!(
            "invariant {}, which supersedes {}\n",
            at(WRITERS, &w1),
            at(BILLING, &b1)
        )),
        "{listed}"
    );
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-accept/2");
    assert_eq!(answer["supersedes"], at(BILLING, &b1));
    assert_eq!(answer["accepted"][0]["supersedes"], at(BILLING, &b1));
    assert!(
        outcome
            .stderr
            .contains(&format!("and it superseded {}", at(BILLING, &b1))),
        "{outcome:?}"
    );
}

#[test]
fn a_superseded_claim_shown() {
    let place = Place::new();
    let (ada, b1) = billing_in_shop(&place);
    let w1 = superseded_by_writers(&place, &[])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    place.accepted(WRITERS, &w1, &ada);
    let shown = place.show(BILLING);
    assert_eq!(shown["accepted"], serde_json::Value::Null);
    assert_eq!(shown["versions"][0]["version"], b1.as_str());
    assert_eq!(shown["versions"][0]["status"], "superseded");
    assert_eq!(
        shown["superseded_by"],
        serde_json::json!([{
            "claim": WRITERS,
            "version": w1,
            "status": "accepted",
            "meant": b1,
        }])
    );
    let words = place.run(&["show", BILLING]);
    assert!(
        words.stdout.contains(&format!(
            "superseded by:\n  {} accepted, on {}\n",
            at(WRITERS, &w1),
            at(BILLING, &b1)
        )),
        "{words:?}"
    );
}

#[test]
fn a_version_that_supersedes_shown() {
    let place = Place::new();
    let (_, b1) = billing_in_shop(&place);
    let w1 = superseded_by_writers(&place, &[])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let shown = place.show(&at(WRITERS, &w1));
    assert_eq!(shown["supersedes"]["claim"], BILLING);
    assert_eq!(shown["supersedes"]["version"], b1.as_str());
    assert_eq!(shown["supersedes"]["statement"], BILLING_WORDS);
    assert_eq!(shown["supersedes"]["reason"], REASON);
    let words = place.run(&["show", &at(WRITERS, &w1)]);
    assert!(
        words.stdout.contains(&format!(
            "supersedes: {}\n  {BILLING_WORDS}\n  reason: {REASON}\n",
            at(BILLING, &b1)
        )),
        "{words:?}"
    );
    assert_eq!(
        place.show(&at(BILLING, &b1))["supersedes"],
        serde_json::Value::Null
    );
}
