//! The author of a proposed version withdraws it, with a reason and no
//! signature; a person rejects several versions with one touch of the
//! key; and `boom search --proposed` lists the versions that wait
//! (claim-versions, "The author withdraws a proposed version" and "A
//! withdrawn version is closed"; persons-acts, "Several versions rejected
//! together"; claim-search, "The versions that wait").

mod support;

use boom::entry::{Act, Actor, Entry, Named};
use serde_json::{json, Value};
use support::{Place, NARRATIVE};

const MONEY: &str = "money-has-currency";
const REFUND: &str = "refund-within-payment";
const STATUS: &str = "status-read-from-events";
const DUPLICATE: &str = "a duplicate";

fn at(name: &str, version: &str) -> String {
    format!("{name}@{version}")
}

/// A store with ada and three versions an agent proposed, giving ada's
/// key and the three targets.
fn three_proposed(place: &Place) -> (String, [String; 3]) {
    let ada = place.made();
    let money = place.proposed(MONEY, "Every amount names its currency.");
    let refund = place.proposed(REFUND, "A refund never exceeds the payment.");
    let status = place.proposed(STATUS, "A record never holds its status.");
    (
        ada,
        [at(MONEY, &money), at(REFUND, &refund), at(STATUS, &status)],
    )
}

/// `boom withdraw` by an agent of `targets`, as the run answers it.
fn agent_withdraws(place: &Place, targets: &[&str]) -> boom::cli::Outcome {
    let mut args = vec!["withdraw"];
    args.extend(targets);
    args.extend(["--reason", DUPLICATE, "--provenance", "run-2"]);
    place.run(&args)
}

fn listed(answer: &Value, key: &str) -> Vec<String> {
    answer[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|named| {
            format!(
                "{}@{}",
                named["claim"].as_str().unwrap(),
                named["version"].as_str().unwrap()
            )
        })
        .collect()
}

fn status_of(place: &Place, target: &str) -> String {
    let (name, version) = target.split_once('@').unwrap();
    place.status(name, version)
}

// The author withdraws a proposed version.

#[test]
fn an_agent_withdraws_its_proposal() {
    let place = Place::new();
    let (_, [money, ..]) = three_proposed(&place);
    let answer = place.json(&[
        "withdraw",
        &money,
        "--reason",
        DUPLICATE,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(answer["format"], "boom-withdraw/1");
    assert_eq!(listed(&answer, "withdrawn"), std::slice::from_ref(&money));
    assert_eq!(answer["withdrawn"][0]["status"], "withdrawn");
    assert_eq!(answer["reason"], DUPLICATE);
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["act"], "withdrawn");
    assert_eq!(last["actor"], json!({"agent": {"provenance": "run-2"}}));
    assert_eq!(last["signature"], Value::Null);
    assert_eq!(last["counted"], true);
    assert_eq!(status_of(&place, &money), "withdrawn");
    let waiting = place.json(&["accept", "--waiting"]).to_string();
    assert!(!waiting.contains(MONEY), "{waiting}");
    assert!(waiting.contains(REFUND), "{waiting}");
}

#[test]
fn a_withdrawn_version_shown() {
    let place = Place::new();
    let (_, [money, ..]) = three_proposed(&place);
    assert_eq!(agent_withdraws(&place, &[&money]).code, 0);
    let entry = place.log().len();
    let shown = place.show(MONEY);
    let version = &shown["versions"][0];
    assert_eq!(version["status"], "withdrawn");
    assert_eq!(version["reason"], DUPLICATE);
    assert_eq!(version["decided_in"], json!(entry));
    assert_eq!(version["accepted_by"], Value::Null);
    assert_eq!(version["rejected_by"], Value::Null);
    let words = place.run(&["show", MONEY]).stdout;
    assert!(
        words.contains(&format!(
            "withdrawn by its author in entry {entry}: {DUPLICATE}"
        )),
        "{words}"
    );
}

#[test]
fn several_withdrawn_together() {
    let place = Place::new();
    let (_, three) = three_proposed(&place);
    let targets: Vec<&str> = three.iter().map(String::as_str).collect();
    let before = place.log().len();
    let outcome = agent_withdraws(&place, &targets);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(place.log().len(), before + 1, "one entry");
    let signed = place.last_signed();
    assert_eq!(signed["versions"].as_array().unwrap().len(), 3);
    for target in &three {
        assert_eq!(status_of(&place, target), "withdrawn");
    }
}

#[test]
fn a_person_withdraws_their_proposal() {
    let place = Place::new();
    let ada = place.made();
    let version = place.json(&[
        "propose",
        "invariant",
        MONEY,
        "Every amount names its currency.",
        "--key",
        &ada,
    ])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let target = at(MONEY, &version);
    let answer = place.json(&[
        "withdraw",
        &target,
        "--reason",
        "no longer needed",
        "--person",
        "ada",
    ]);
    assert_eq!(answer["by"], "ada");
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["actor"], json!({"person": "ada"}));
    assert_eq!(last["signature"], Value::Null);
    assert_eq!(last["counted"], true);
    assert_eq!(status_of(&place, &target), "withdrawn");
}

#[test]
fn another_authors_proposal() {
    let place = Place::new();
    let ada = place.made();
    place.registered(&ada, "bo", 2);
    let version = place.json(&[
        "propose",
        "invariant",
        MONEY,
        "Every amount names its currency.",
        "--key",
        &ada,
    ])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let target = at(MONEY, &version);
    place.refused(
        &[
            "withdraw",
            &target,
            "--reason",
            DUPLICATE,
            "--provenance",
            "run-2",
        ],
        "only its author withdraws",
    );
    place.refused(
        &["withdraw", &target, "--reason", DUPLICATE, "--person", "bo"],
        "only its author withdraws",
    );
    let agents = place.proposed(REFUND, "A refund never exceeds the payment.");
    place.refused(
        &[
            "withdraw",
            &at(REFUND, &agents),
            "--reason",
            DUPLICATE,
            "--person",
            "ada",
        ],
        "only its author withdraws",
    );
}

#[test]
fn a_version_that_is_not_proposed() {
    let place = Place::new();
    let (ada, [money, ..]) = three_proposed(&place);
    let (name, version) = money.split_once('@').unwrap();
    place.accepted(name, version, &ada);
    place.refused(
        &[
            "withdraw",
            &money,
            "--reason",
            DUPLICATE,
            "--provenance",
            "run-2",
        ],
        "only a proposed version is withdrawn",
    );
}

#[test]
fn an_old_draft_withdrawn() {
    let place = Place::new();
    place.made();
    let v1 = place.proposed(MONEY, "Every amount names its currency.");
    let v2 = place.amended(MONEY, "Each amount names its currency.", &v1);
    place.refused(
        &[
            "withdraw",
            &at(MONEY, &v1),
            "--reason",
            DUPLICATE,
            "--provenance",
            "run-2",
        ],
        &format!("superseded by the newer draft {MONEY}@{v2}"),
    );
}

#[test]
fn one_of_several_refused() {
    let place = Place::new();
    let (ada, [money, refund, status]) = three_proposed(&place);
    let (name, version) = status.split_once('@').unwrap();
    place.accepted(name, version, &ada);
    let outcome = place.refused(
        &[
            "withdraw",
            &money,
            &refund,
            &status,
            "--reason",
            DUPLICATE,
            "--provenance",
            "run-2",
        ],
        &status,
    );
    assert!(outcome.stderr.contains("accepted"), "{outcome:?}");
    assert_eq!(status_of(&place, &money), "proposed");
    assert_eq!(status_of(&place, &refund), "proposed");
}

#[test]
fn no_reason() {
    let place = Place::new();
    let (_, [money, ..]) = three_proposed(&place);
    place.refused(
        &["withdraw", &money, "--provenance", "run-2"],
        "a withdrawal gives its reason",
    );
}

#[test]
fn a_key_given() {
    let place = Place::new();
    let (ada, [money, ..]) = three_proposed(&place);
    place.refused(
        &["withdraw", &money, "--reason", DUPLICATE, "--key", &ada],
        "a withdrawal carries no signature",
    );
}

#[test]
fn a_withdrawal_with_no_author() {
    let place = Place::new();
    let (_, [money, ..]) = three_proposed(&place);
    place.refused(
        &["withdraw", &money, "--reason", DUPLICATE],
        "a withdrawal names its author",
    );
}

#[test]
fn withdrawn_again() {
    let place = Place::new();
    let (_, [money, ..]) = three_proposed(&place);
    let first = agent_withdraws(&place, &[&money]);
    assert_eq!(first.code, 0, "{first:?}");
    let log = place.log();
    let again = agent_withdraws(&place, &[&money]);
    assert_eq!(again.code, 0, "{again:?}");
    assert_eq!(place.log(), log, "the log does not change");
}

#[test]
fn a_withdrawal_from_another_program() {
    let place = Place::new();
    let ada = place.made();
    let version = place.json(&[
        "propose",
        "invariant",
        MONEY,
        "Every amount names its currency.",
        "--key",
        &ada,
    ])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    place.append_by_hand(
        &Entry {
            at: 1_763_424_500,
            rules: boom::judge::RULES,
            actor: Actor::Agent {
                provenance: "run-9".to_owned(),
            },
            act: Act::Withdrawn {
                versions: vec![Named {
                    name: MONEY.to_owned(),
                    version: version.clone(),
                }],
                reason: DUPLICATE.to_owned(),
            },
        },
        None,
    );
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["counted"], false);
    assert_eq!(place.status(MONEY, &version), "proposed");
}

#[test]
fn a_signed_withdrawal() {
    let place = Place::new();
    let ada = place.made();
    let version = place.json(&[
        "propose",
        "invariant",
        MONEY,
        "Every amount names its currency.",
        "--key",
        &ada,
    ])["version"]
        .as_str()
        .unwrap()
        .to_owned();
    place.append_by_hand(
        &Entry {
            at: 1_763_424_500,
            rules: boom::judge::RULES,
            actor: Actor::Person("ada".to_owned()),
            act: Act::Withdrawn {
                versions: vec![Named {
                    name: MONEY.to_owned(),
                    version: version.clone(),
                }],
                reason: DUPLICATE.to_owned(),
            },
        },
        Some(std::path::Path::new(&ada)),
    );
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["counted"], false);
    assert_eq!(place.status(MONEY, &version), "proposed");
}

// A withdrawn version is closed.

#[test]
fn a_withdrawn_version_accepted_or_rejected() {
    let place = Place::new();
    let (ada, [money, ..]) = three_proposed(&place);
    assert_eq!(agent_withdraws(&place, &[&money]).code, 0);
    let before = place.keys.signatures();
    place.refused(&["accept", &money, "--key", &ada], "is withdrawn");
    place.refused(
        &["reject", &money, "--reason", "stray", "--key", &ada],
        "is withdrawn",
    );
    assert_eq!(place.keys.signatures(), before, "the key signs nothing");
}

#[test]
fn a_narrative_that_includes_a_withdrawn_version() {
    let place = Place::new();
    let (ada, [_, _, status]) = three_proposed(&place);
    let story = place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        NARRATIVE,
        &["--includes", &status],
    )["version"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(agent_withdraws(&place, &[&status]).code, 0);
    let before = place.keys.signatures();
    place.refused(
        &["accept", &at("boom-end-to-end", &story), "--key", &ada],
        &format!("{status} is withdrawn"),
    );
    assert_eq!(place.keys.signatures(), before, "the key signs nothing");
}

#[test]
fn a_withdrawn_version_amended() {
    let place = Place::new();
    place.made();
    let v1 = place.proposed(MONEY, "Every amount names its currency.");
    assert_eq!(agent_withdraws(&place, &[&at(MONEY, &v1)]).code, 0);
    let v2 = place.amended(MONEY, "Each amount names its currency.", &v1);
    assert_eq!(place.status(MONEY, &v2), "proposed");
    assert_eq!(place.status(MONEY, &v1), "withdrawn");
}

#[test]
fn the_views_and_a_replay_agree() {
    let place = Place::new();
    let (_, three) = three_proposed(&place);
    let targets: Vec<&str> = three.iter().map(String::as_str).collect();
    assert_eq!(agent_withdraws(&place, &targets).code, 0);
    let before = place.show(MONEY);
    let verified = place.json(&["verify"]);
    assert_eq!(verified["rebuilt"], json!([]), "{verified}");
    assert_eq!(place.show(MONEY), before);
}

// Several versions rejected together.

#[test]
fn three_rejected_together() {
    let place = Place::new();
    let (ada, three) = three_proposed(&place);
    let mut args = vec!["reject"];
    args.extend(three.iter().map(String::as_str));
    args.extend(["--reason", "stray drafts", "--key", &ada]);
    let before = place.keys.signatures();
    let log = place.log().len();
    let answer = place.json(&args);
    assert_eq!(answer["format"], "boom-reject/3");
    assert_eq!(listed(&answer, "rejected"), three.to_vec());
    assert_eq!(answer["by"], "ada");
    assert_eq!(
        place.keys.signatures(),
        before + 1,
        "the key signs one time"
    );
    assert_eq!(place.log().len(), log + 1, "one entry");
    assert_eq!(place.last_signed()["versions"].as_array().unwrap().len(), 3);
    for target in &three {
        assert_eq!(status_of(&place, target), "rejected");
    }
    let again = place.json(&args);
    assert_eq!(listed(&again, "rejected"), three.to_vec());
    assert_eq!(place.keys.signatures(), before + 1, "the key signs nothing");
    assert_eq!(place.log().len(), log + 1, "the log does not change");
}

#[test]
fn one_of_three_rejects_refused() {
    let place = Place::new();
    let (ada, [money, refund, status]) = three_proposed(&place);
    let (name, version) = status.split_once('@').unwrap();
    place.accepted(name, version, &ada);
    let before = place.keys.signatures();
    place.refused(
        &[
            "reject", &money, &refund, &status, "--reason", "stray", "--key", &ada,
        ],
        &status,
    );
    assert_eq!(place.keys.signatures(), before, "the key signs nothing");
    assert_eq!(status_of(&place, &money), "proposed");
}

#[test]
fn two_versions_of_one_claim_rejected() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed(MONEY, "Every amount names its currency.");
    let second = place.proposed(MONEY, "Each amount names its currency.");
    place.json(&[
        "reject",
        &at(MONEY, &first),
        &at(MONEY, &second),
        "--reason",
        "stray",
        "--key",
        &ada,
    ]);
    assert_eq!(place.status(MONEY, &first), "rejected");
    assert_eq!(place.status(MONEY, &second), "rejected");
}

// The versions that wait.

/// An agent's proposal of `name` for `shop`, giving its target.
fn for_shop(place: &Place, name: &str, statement: &str) -> String {
    let answer = place.json(&[
        "propose",
        "invariant",
        name,
        statement,
        "--repository",
        "shop",
        "--provenance",
        "run-1",
    ]);
    at(name, answer["version"].as_str().unwrap())
}

fn found(answer: &Value) -> Vec<String> {
    answer["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| {
            format!(
                "{}@{}",
                claim["name"].as_str().unwrap(),
                claim["version"].as_str().unwrap()
            )
        })
        .collect()
}

#[test]
fn each_version_that_waits() {
    let mut place = Place::new();
    let ada = place.made();
    let older = for_shop(&place, MONEY, "Every amount names its currency.");
    place.at(1_763_424_000 + 3_600);
    let newer = for_shop(&place, REFUND, "A refund never exceeds the payment.");
    let accepted = for_shop(&place, STATUS, "A record never holds its status.");
    let (name, version) = accepted.split_once('@').unwrap();
    place.accepted(name, version, &ada);
    let gone = for_shop(
        &place,
        "names-use-terms",
        "A tool names things by its terms.",
    );
    assert_eq!(agent_withdraws(&place, &[&gone]).code, 0);
    let answer = place.json(&["search", "--proposed", "--repository", "shop"]);
    assert_eq!(answer["format"], "boom-search/2");
    assert_eq!(answer["proposed"], true);
    assert_eq!(answer["total"], 2);
    assert_eq!(found(&answer), [older, newer]);
    let first = &answer["claims"][0];
    assert_eq!(first["status"], "proposed");
    assert_eq!(first["proposer"], json!({"agent": {"provenance": "run-1"}}));
    assert_eq!(first["proposed_in"], 2);
}

#[test]
fn the_age_of_a_version() {
    let mut place = Place::new();
    place.made();
    let money = for_shop(&place, MONEY, "Every amount names its currency.");
    place.at(1_763_424_000 + 3 * 86_400 + 60);
    let answer = place.json(&["search", "--proposed", "--all"]);
    assert_eq!(found(&answer), std::slice::from_ref(&money));
    assert_eq!(answer["claims"][0]["age_days"], 3);
    assert_eq!(answer["claims"][0]["proposed_at"], 1_763_424_000);
    let words = place.run(&["search", "--proposed", "--all"]).stdout;
    assert!(
        words.starts_with("1 version waits across every repository"),
        "{words}"
    );
    assert!(
        words.contains(&format!(
            "{money}, for shop, proposed by an agent (run-1) in entry 2, 3 days ago"
        )),
        "{words}"
    );
}

#[test]
fn words_among_the_versions_that_wait() {
    let place = Place::new();
    place.made();
    for_shop(&place, MONEY, "Every amount names its currency.");
    let refund = for_shop(&place, REFUND, "A refund never exceeds the payment.");
    let answer = place.json(&["search", "--proposed", "refund", "--repository", "shop"]);
    assert_eq!(found(&answer), [refund]);
}

#[test]
fn an_old_draft_does_not_wait() {
    let place = Place::new();
    place.made();
    let v1 = place.proposed(MONEY, "Every amount names its currency.");
    let v2 = place.amended(MONEY, "Each amount names its currency.", &v1);
    let answer = place.json(&["search", "--proposed", "--all"]);
    assert_eq!(found(&answer), [at(MONEY, &v2)]);
}

#[test]
fn no_word_with_no_proposed() {
    let place = Place::new();
    place.made();
    let log = place.log();
    let outcome = place.run(&["search", "--repository", "shop"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(place.log(), log);
}
