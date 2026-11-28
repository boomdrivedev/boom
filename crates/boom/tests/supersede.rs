//! `boom supersede` proposes a version that supersedes the accepted
//! version of another claim, by a new claim or by an accepted claim of
//! the same kind, and a person's accept takes that claim out of force
//! (claim-versions, "boom supersede proposes a version that supersedes
//! an accepted claim", "A supersession by a new claim", "A supersession
//! by an accepted claim of the same kind", "A version names the accepted
//! version that it supersedes", "The accept of a supersession takes the
//! claim it names out of force" and "The person reads what a
//! supersession takes out of force").

mod support;

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
