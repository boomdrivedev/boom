//! An entry is judged by the rules it was written under: each entry names
//! its rule set, and a rule that came into force with a later rule set
//! does not hold for it (record-log).

#[path = "../src/tests/support.rs"]
mod support;

use std::path::Path;

use boom::claim::{Kind, Proposal};
use boom::entry::{Act, Actor, Attach, Entry, Named};
use boom::judge::RULES;
use support::{Place, NARRATIVE};

/// An agent's proposal of the narrative `name`, with the body that the
/// store holds already, bound to the repositories `boom` and `sluice`,
/// written under the rule set `rules`.
fn two_level_narrative(name: &str, rules: u32) -> (Entry, String) {
    let mut proposal = Proposal::new(
        Kind::Narrative,
        name,
        "Boom across two repositories",
        Vec::new(),
        Some(boom::sha256_hex(NARRATIVE.as_bytes())),
        None,
        None,
    );
    proposal.version = proposal.fingerprint(None);
    let version = proposal.version.clone();
    let entry = Entry {
        at: 1_763_424_600,
        rules,
        actor: Actor::Agent {
            provenance: "run-1".to_owned(),
        },
        act: Act::Proposed {
            claim: Box::new(proposal),
            imported: None,
            attach: Some(Attach {
                repositories: vec!["boom".to_owned(), "sluice".to_owned()],
                contexts: Vec::new(),
            }),
        },
    };
    (entry, version)
}

/// Ada's act on `name@version`, signed, written under the rule set
/// `rules`.
fn ada(act: Act, rules: u32) -> Entry {
    Entry {
        at: 1_763_424_700,
        rules,
        actor: Actor::Person("ada".to_owned()),
        act,
    }
}

fn accept(name: &str, version: &str) -> Act {
    Act::Accepted {
        versions: vec![Named {
            name: name.to_owned(),
            version: version.to_owned(),
        }],
    }
}

fn reject(name: &str, version: &str) -> Act {
    Act::Rejected {
        versions: vec![Named {
            name: name.to_owned(),
            version: version.to_owned(),
        }],
        reason: "The newer draft says it better.".to_owned(),
    }
}

/// A store whose narrative `boom-end-to-end` binds `boom`, so that the
/// store holds the body of [`NARRATIVE`].
fn with_a_narrative() -> (Place, String) {
    let place = Place::new();
    let key = place.made();
    place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        NARRATIVE,
        &["--repository", "boom"],
    );
    (place, key)
}

#[test]
fn a_proposal_and_its_accept_written_before_a_narrative_bound_one_level_count() {
    let (place, key) = with_a_narrative();
    let (proposed, version) = two_level_narrative("boom-across-repositories", 0);
    place.append_by_hand(&proposed, None);
    place.append_by_hand(
        &ada(accept("boom-across-repositories", &version), 0),
        Some(Path::new(&key)),
    );
    let log = place.log();
    let [.., proposal, accepted] = log.as_slice() else {
        panic!("{log:?}")
    };
    assert_eq!(proposal["counted"], true, "{proposal}");
    assert_eq!(proposal["rules"], 0);
    assert_eq!(accepted["counted"], true, "{accepted}");
    assert_eq!(
        place.status("boom-across-repositories", &version),
        "accepted"
    );
    let verified = place.run(&["verify"]);
    assert_eq!(verified.code, 0, "{verified:?}");
    assert_eq!(
        place.status("boom-across-repositories", &version),
        "accepted"
    );
}

#[test]
fn a_proposal_written_under_the_rule_that_a_narrative_binds_one_level_is_refused() {
    let (place, _) = with_a_narrative();
    let (proposed, _) = two_level_narrative("boom-across-repositories", RULES);
    place.append_by_hand(&proposed, None);
    let entry = place.log().pop().unwrap();
    assert_eq!(entry["counted"], false, "{entry}");
    assert_eq!(entry["rules"], RULES);
    let reason = entry["reason"].as_str().unwrap();
    assert!(reason.contains("a narrative binds one level"), "{reason}");
}

/// A store with `money-has-currency` proposed, then a newer draft that
/// amends it, giving ada's key, the first draft and the newer one.
fn with_two_drafts() -> (Place, String, String, String) {
    let place = Place::new();
    let key = place.made();
    let first = place.proposed("money-has-currency", "Every amount names its currency.");
    let newer = place.amended(
        "money-has-currency",
        "Every amount names its currency, in ISO 4217.",
        &first,
    );
    assert_eq!(place.status("money-has-currency", &first), "superseded");
    (place, key, first, newer)
}

#[test]
fn an_accept_of_a_draft_signed_before_a_newer_draft_superseded_it_counts() {
    let (place, key, first, newer) = with_two_drafts();
    place.append_by_hand(
        &ada(accept("money-has-currency", &first), 0),
        Some(Path::new(&key)),
    );
    let entry = place.log().pop().unwrap();
    assert_eq!(entry["counted"], true, "{entry}");
    assert_eq!(place.status("money-has-currency", &first), "accepted");
    assert_eq!(place.status("money-has-currency", &newer), "proposed");
    let verified = place.run(&["verify"]);
    assert_eq!(verified.code, 0, "{verified:?}");
    assert_eq!(place.status("money-has-currency", &first), "accepted");
}

#[test]
fn a_reject_of_a_draft_signed_before_a_newer_draft_superseded_it_counts() {
    let (place, key, first, _) = with_two_drafts();
    place.append_by_hand(
        &ada(reject("money-has-currency", &first), 0),
        Some(Path::new(&key)),
    );
    let entry = place.log().pop().unwrap();
    assert_eq!(entry["counted"], true, "{entry}");
    assert_eq!(place.status("money-has-currency", &first), "rejected");
}

#[test]
fn an_accept_and_a_reject_of_a_superseded_draft_under_the_rule_are_refused() {
    let (place, key, first, newer) = with_two_drafts();
    place.append_by_hand(
        &ada(accept("money-has-currency", &first), RULES),
        Some(Path::new(&key)),
    );
    place.append_by_hand(
        &ada(reject("money-has-currency", &first), RULES),
        Some(Path::new(&key)),
    );
    let log = place.log();
    let [.., accepted, rejected] = log.as_slice() else {
        panic!("{log:?}")
    };
    for entry in [accepted, rejected] {
        assert_eq!(entry["counted"], false, "{entry}");
        let reason = entry["reason"].as_str().unwrap();
        assert!(
            reason.contains(&format!(
                "superseded by the newer draft money-has-currency@{newer}"
            )),
            "{reason}"
        );
    }
    assert_eq!(place.status("money-has-currency", &first), "superseded");
    place.refused(
        &[
            "accept",
            &format!("money-has-currency@{first}"),
            "--key",
            &key,
        ],
        "superseded by the newer draft",
    );
}

#[test]
fn an_entry_boom_writes_names_the_rule_set_it_was_written_under() {
    let place = Place::new();
    place.made();
    place.proposed("money-has-currency", "Every amount names its currency.");
    for entry in place.log() {
        assert_eq!(entry["rules"], RULES, "{entry}");
        let signed = entry["signed"].as_str().unwrap();
        assert!(
            signed.contains(&format!(r#""rules":{RULES},"actor""#)),
            "{signed}"
        );
    }
}

#[test]
fn an_entry_of_a_rule_set_this_boom_does_not_hold_is_refused() {
    let (place, _) = with_a_narrative();
    let (proposed, _) = two_level_narrative("boom-across-repositories", RULES + 1);
    place.append_by_hand(&proposed, None);
    let entry = place.log().pop().unwrap();
    assert_eq!(entry["counted"], false, "{entry}");
    let reason = entry["reason"].as_str().unwrap();
    assert!(reason.contains("a newer boom judges it"), "{reason}");
}
