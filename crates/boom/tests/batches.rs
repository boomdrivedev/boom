//! A person accepts a batch with one touch of the key: a narrative with
//! the proposed versions it includes, a revision with its cause, and each
//! batch that waits, listed by `boom accept --waiting` (persons-acts;
//! narrative-page, "The batches that wait on a person").

#[path = "../src/tests/support.rs"]
mod support;

use serde_json::Value;
use support::{Place, NARRATIVE};

const NAME: &str = "boom-end-to-end";
const OTHER: &str = "change-records";
const STATUS: &str = "status-read-from-events";
const MONEY: &str = "money-has-currency";
const CAUSE: &str = "names-no-plan";
const AMOUNTS: &str = "amounts-in-cents";

fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// The versions the log's last entry accepts, each `<name>@<version>`:
/// an accept of one names it, an accept of several each in `versions`.
fn last_accepted(place: &Place) -> Vec<String> {
    let signed = place.last_signed();
    match signed["versions"].as_array() {
        Some(versions) => versions
            .iter()
            .map(|named| {
                format!(
                    "{}@{}",
                    named["name"].as_str().unwrap(),
                    named["version"].as_str().unwrap()
                )
            })
            .collect(),
        None => vec![format!(
            "{}@{}",
            signed["name"].as_str().unwrap(),
            signed["version"].as_str().unwrap()
        )],
    }
}

/// The versions an answer lists under `accepted`, each
/// `<name>@<version>`.
fn listed(answer: &Value) -> Vec<String> {
    answer["accepted"]
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

/// A store with ada, `status-read-from-events` proposed and
/// `money-has-currency` accepted, giving ada's key and the two versions.
fn status_and_money(place: &Place) -> (String, String, String) {
    let ada = place.made();
    let status = place.proposed(STATUS, "A record never holds its status.");
    let money = place.proposed(MONEY, "Every amount names its currency.");
    place.accepted(MONEY, &money, &ada);
    (ada, status, money)
}

/// The narrative `name`, including each of `includes`, giving its
/// version.
fn narrative(place: &Place, name: &str, includes: &[&str]) -> String {
    let mut extra = Vec::new();
    for include in includes {
        extra.extend(["--includes", include]);
    }
    version(&place.narrative(name, "Boom, end to end", NARRATIVE, &extra))
}

/// A new version of the invariant `name` that amends `base` and names the
/// cause `cause`, giving its version.
fn revised(place: &Place, name: &str, words: &str, base: &str, cause: &str) -> String {
    version(&place.json(&[
        "propose",
        "invariant",
        name,
        words,
        "--amends",
        base,
        "--cause",
        cause,
        "--provenance",
        "run-2",
    ]))
}

// The accept of a narrative takes the claims that it includes.

#[test]
fn a_narrative_and_its_proposed_claim() {
    let place = Place::new();
    let (ada, status, money) = status_and_money(&place);
    let story = narrative(&place, NAME, &[STATUS, MONEY]);
    let before = place.keys.signatures();
    let answer = place.json(&["accept", &format!("{NAME}@{story}"), "--key", &ada]);
    assert_eq!(
        place.keys.signatures(),
        before + 1,
        "the key signs one time"
    );
    let both = vec![format!("{NAME}@{story}"), format!("{STATUS}@{status}")];
    assert_eq!(last_accepted(&place), both);
    assert_eq!(listed(&answer), both);
    assert_eq!(place.status(STATUS, &status), "accepted");
    assert_eq!(place.status(NAME, &story), "accepted");
    assert_eq!(place.status(MONEY, &money), "accepted");
}

#[test]
fn a_narrative_that_includes_a_rejected_version() {
    let place = Place::new();
    let (ada, status, _) = status_and_money(&place);
    let story = narrative(&place, NAME, &[STATUS, MONEY]);
    place.json(&[
        "reject",
        &format!("{STATUS}@{status}"),
        "--reason",
        "Too narrow.",
        "--key",
        &ada,
    ]);
    let before = place.keys.signatures();
    place.refused(
        &["accept", &format!("{NAME}@{story}"), "--key", &ada],
        &format!("{STATUS}@{status} is rejected"),
    );
    assert_eq!(place.keys.signatures(), before, "the key signs nothing");
}

#[test]
fn a_narrative_that_includes_a_superseded_draft() {
    let place = Place::new();
    let (ada, status, _) = status_and_money(&place);
    let story = narrative(&place, NAME, &[STATUS]);
    place.amended(STATUS, "No record holds its status.", &status);
    place.refused(
        &["accept", &format!("{NAME}@{story}"), "--key", &ada],
        &format!("{STATUS}@{status} is superseded"),
    );
}

#[test]
fn two_narratives_that_include_one_claim() {
    let place = Place::new();
    let (ada, status, _) = status_and_money(&place);
    let story = narrative(&place, NAME, &[STATUS]);
    let other = narrative(&place, OTHER, &[STATUS]);
    place.json(&[
        "accept",
        &format!("{NAME}@{story}"),
        &format!("{OTHER}@{other}"),
        "--key",
        &ada,
    ]);
    assert_eq!(
        last_accepted(&place),
        [
            format!("{NAME}@{story}"),
            format!("{STATUS}@{status}"),
            format!("{OTHER}@{other}"),
        ]
    );
}

#[test]
fn a_narrative_accepted_again() {
    let place = Place::new();
    let (ada, status, _) = status_and_money(&place);
    let story = narrative(&place, NAME, &[STATUS, MONEY]);
    let target = format!("{NAME}@{story}");
    place.json(&["accept", &target, "--key", &ada]);
    let log = place.log();
    let before = place.keys.signatures();
    let answer = place.json(&["accept", &target, "--key", &ada]);
    assert_eq!(
        listed(&answer),
        [target.clone(), format!("{STATUS}@{status}")]
    );
    assert_eq!(place.keys.signatures(), before, "the key signs nothing");
    assert_eq!(place.log(), log);
}

// The accept of a cause takes its revision.

/// A store with `money-has-currency` and `status-read-from-events`
/// accepted, the cause `names-no-plan` proposed, and a new version of each
/// of the two that names it: ada's key, the cause, and the two new
/// versions.
fn a_revision(place: &Place) -> (String, String, String, String) {
    let ada = place.made();
    let money = place.proposed(MONEY, "Every amount names its currency.");
    let status = place.proposed(STATUS, "A record never holds its status.");
    place.json(&[
        "accept",
        &format!("{MONEY}@{money}"),
        &format!("{STATUS}@{status}"),
        "--key",
        &ada,
    ]);
    let cause = place.proposed(CAUSE, "A claim names no plan.");
    let money = revised(
        place,
        MONEY,
        "Each amount names its currency.",
        &money,
        CAUSE,
    );
    let status = revised(place, STATUS, "No record holds its status.", &status, CAUSE);
    (ada, cause, money, status)
}

#[test]
fn a_cause_and_its_two_versions() {
    let place = Place::new();
    let (ada, cause, money, status) = a_revision(&place);
    let before = place.keys.signatures();
    let answer = place.json(&[
        "accept",
        "--cause",
        &format!("{CAUSE}@{cause}"),
        "--key",
        &ada,
    ]);
    assert_eq!(
        place.keys.signatures(),
        before + 1,
        "the key signs one time"
    );
    let all = vec![
        format!("{CAUSE}@{cause}"),
        format!("{MONEY}@{money}"),
        format!("{STATUS}@{status}"),
    ];
    assert_eq!(last_accepted(&place), all);
    assert_eq!(listed(&answer), all);
    assert_eq!(place.status(MONEY, &money), "accepted");
    assert_eq!(place.status(STATUS, &status), "accepted");
    assert_eq!(place.status(CAUSE, &cause), "accepted");
}

#[test]
fn a_superseded_draft_is_left_out() {
    let place = Place::new();
    let (ada, cause, money, status) = a_revision(&place);
    let newer = revised(
        &place,
        MONEY,
        "Each amount names one currency.",
        &money,
        CAUSE,
    );
    place.json(&[
        "accept",
        "--cause",
        &format!("{CAUSE}@{cause}"),
        "--key",
        &ada,
    ]);
    assert_eq!(
        last_accepted(&place),
        [
            format!("{CAUSE}@{cause}"),
            format!("{STATUS}@{status}"),
            format!("{MONEY}@{newer}"),
        ]
    );
    assert_eq!(place.status(MONEY, &money), "superseded");
}

#[test]
fn a_cause_that_no_version_names() {
    let place = Place::new();
    let (ada, _, money) = status_and_money(&place);
    place.refused(
        &[
            "accept",
            "--cause",
            &format!("{MONEY}@{money}"),
            "--key",
            &ada,
        ],
        &format!("no version names {MONEY}@{money} as its cause"),
    );
}

// An accept lists what it takes.

#[test]
fn the_list_before_the_key() {
    let place = Place::new();
    let (ada, status, _) = status_and_money(&place);
    let story = narrative(&place, NAME, &[STATUS]);
    place.json(&["accept", &format!("{NAME}@{story}"), "--key", &ada]);
    let told = place.keys.told();
    assert!(
        told.ends_with(&format!(
            "This accept takes 2 versions in one entry:\n  narrative {NAME}@{story}\n  invariant {STATUS}@{status}\n"
        )),
        "{told}"
    );
}

// boom accept --waiting.

/// A narrative with one proposed claim, a cause with one version that
/// names it, and one other proposed claim: ada's key, and each of the
/// five versions, `<name>@<version>`, in the order of their batches.
fn three_batches(place: &Place) -> (String, [String; 5]) {
    let ada = place.made();
    let status = place.proposed(STATUS, "A record never holds its status.");
    let story = narrative(place, NAME, &[STATUS]);
    let money = place.proposed(MONEY, "Every amount names its currency.");
    place.accepted(MONEY, &money, &ada);
    let cause = place.proposed(CAUSE, "A claim names no plan.");
    let money = revised(
        place,
        MONEY,
        "Each amount names its currency.",
        &money,
        CAUSE,
    );
    let amounts = place.proposed(AMOUNTS, "Amounts are kept in cents.");
    (
        ada,
        [
            format!("{NAME}@{story}"),
            format!("{STATUS}@{status}"),
            format!("{CAUSE}@{cause}"),
            format!("{MONEY}@{money}"),
            format!("{AMOUNTS}@{amounts}"),
        ],
    )
}

/// The batches `boom accept --waiting --json` gives, each as its shape,
/// its head and the versions it takes.
fn batches(place: &Place) -> Vec<(String, String, Vec<String>)> {
    place.json(&["accept", "--waiting"])["batches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|batch| {
            (
                batch["shape"].as_str().unwrap().to_owned(),
                format!(
                    "{}@{}",
                    batch["claim"].as_str().unwrap(),
                    batch["version"].as_str().unwrap()
                ),
                batch["versions"]
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
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn the_three_shapes() {
    let place = Place::new();
    let (_, [story, status, cause, money, amounts]) = three_batches(&place);
    assert_eq!(
        batches(&place),
        [
            ("narrative".to_owned(), story.clone(), vec![story, status]),
            ("revision".to_owned(), cause.clone(), vec![cause, money]),
            ("ruling".to_owned(), amounts.clone(), vec![amounts]),
        ]
    );
}

#[test]
fn a_version_that_a_narrative_includes_is_no_ruling() {
    let place = Place::new();
    place.made();
    let status = place.proposed(STATUS, "A record never holds its status.");
    let story = narrative(&place, NAME, &[STATUS]);
    let shapes = batches(&place);
    assert_eq!(shapes.len(), 1, "{shapes:?}");
    assert_eq!(shapes[0].1, format!("{NAME}@{story}"));
    assert_eq!(shapes[0].2[1], format!("{STATUS}@{status}"));
}

#[test]
fn the_batches_listed() {
    let place = Place::new();
    let (_, [story, _, cause, _, amounts]) = three_batches(&place);
    let log = place.log();
    let answer = place.json(&["accept", "--waiting"]);
    let commands: Vec<&str> = answer["batches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|batch| batch["command"].as_str().unwrap())
        .collect();
    assert_eq!(
        commands,
        [
            format!("boom accept {story} --key <key>"),
            format!("boom accept --cause {cause} --key <key>"),
            format!("boom accept {amounts} --key <key>"),
        ]
    );
    assert_eq!(answer["versions"], 5);
    // Every batch in one command: the heads, then each cause.
    let every = format!("boom accept {story} {amounts} --cause {cause} --key <key>");
    assert_eq!(answer["command"], every.as_str());
    assert_eq!(place.log(), log);
    let words = place.run(&["accept", "--waiting"]);
    assert_eq!(words.code, 0, "{words:?}");
    assert!(
        words.stdout.starts_with(&format!(
            "3 batches wait on a person, 5 versions.\nEach one that Boom does not refuse, in one sign: {every}\n"
        )),
        "{}",
        words.stdout
    );
}

#[test]
fn nothing_waits() {
    let place = Place::new();
    place.made();
    let outcome = place.run(&["accept", "--waiting"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout, "Nothing waits on a person.\n");
    assert_eq!(
        place.json(&["accept", "--waiting"])["batches"],
        serde_json::json!([])
    );
}

#[test]
fn each_batch_in_one_sign() {
    let place = Place::new();
    let (ada, five) = three_batches(&place);
    let before = place.keys.signatures();
    let answer = place.json(&["accept", "--waiting", "--key", &ada]);
    assert_eq!(
        place.keys.signatures(),
        before + 1,
        "the key signs one time"
    );
    let mut accepted = last_accepted(&place);
    accepted.sort();
    let mut all = five.to_vec();
    all.sort();
    assert_eq!(accepted, all);
    assert_eq!(listed(&answer).len(), 5);
    assert!(batches(&place).is_empty());
}

#[test]
fn a_refused_batch_is_left_out() {
    let place = Place::new();
    let (ada, status, _) = status_and_money(&place);
    let story = narrative(&place, NAME, &[STATUS]);
    place.json(&[
        "reject",
        &format!("{STATUS}@{status}"),
        "--reason",
        "Too narrow.",
        "--key",
        &ada,
    ]);
    let amounts = place.proposed(AMOUNTS, "Amounts are kept in cents.");
    let answer = place.json(&["accept", "--waiting", "--key", &ada]);
    assert_eq!(last_accepted(&place), [format!("{AMOUNTS}@{amounts}")]);
    assert_eq!(place.status(NAME, &story), "proposed");
    let left = &answer["left_out"][0];
    assert_eq!(left["claim"], NAME);
    assert_eq!(left["closed"][0]["claim"], STATUS);
    assert_eq!(left["closed"][0]["status"], "rejected");
}

// A person's key in Boom's settings, and one accept of several batches.

/// Writes Boom's settings of the place, `<config>/boom/config.toml`.
fn settings(place: &Place, text: &str) {
    let folder = place.env.xdg_config_home.as_ref().unwrap().join("boom");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("config.toml"), text).unwrap();
}

#[test]
fn an_accept_signs_with_the_key_of_the_settings() {
    let place = Place::new();
    let (ada, [story, status, cause, _, amounts]) = three_batches(&place);
    settings(
        &place,
        &format!("key = {{ path = \"{ada}\", askpass = \"/opt/askpass-of-the-test\" }}\n"),
    );
    // The batches give their commands with no --key.
    let answer = place.json(&["accept", "--waiting"]);
    assert_eq!(
        answer["command"],
        format!("boom accept {story} {amounts} --cause {cause}").as_str()
    );
    assert_eq!(
        answer["batches"][0]["command"],
        format!("boom accept {story}").as_str()
    );
    let before = place.keys.signatures();
    let accepted = place.json(&["accept", &story]);
    assert_eq!(place.keys.signatures(), before + 1);
    assert_eq!(last_accepted(&place), [story.clone(), status]);
    assert_eq!(accepted["accepted"].as_array().unwrap().len(), 2);
    let asked = place.keys.asked();
    assert_eq!(
        asked.last().unwrap(),
        &boom::sign::Key {
            path: ada.clone().into(),
            askpass: Some("/opt/askpass-of-the-test".into()),
        }
    );
}

#[test]
fn a_key_named_by_key_takes_the_passphrase_program_of_its_own_settings() {
    let place = Place::new();
    let (ada, [_, _, _, _, amounts]) = three_batches(&place);
    let bo = place.registered(&ada, "bo", 2);
    settings(
        &place,
        &format!("key = {{ path = \"{ada}\", askpass = \"/opt/askpass-of-the-test\" }}\n"),
    );
    place.json(&["accept", &amounts, "--key", &bo]);
    assert_eq!(
        place.keys.asked().last().unwrap(),
        &boom::sign::Key {
            path: bo.into(),
            askpass: None
        }
    );
}

#[test]
fn an_accept_with_no_key_and_no_settings_is_refused() {
    let place = Place::new();
    let (_, [story, ..]) = three_batches(&place);
    place.refused(&["accept", &story], "or the key in Boom's settings");
}

#[test]
fn settings_that_boom_does_not_read_refuse_the_accept() {
    let place = Place::new();
    let (_, [story, ..]) = three_batches(&place);
    let log = place.log();
    settings(&place, "key = { path = \"keys/ada\" }\n");
    place.refused(
        &["accept", &story],
        "key.path is a path that is absolute or starts with ~/",
    );
    settings(
        &place,
        "key = { path = \"/keys/ada\", passphrase = \"x\" }\n",
    );
    place.refused(&["accept", &story], "are not read");
    assert_eq!(place.log(), log);
}

#[test]
fn a_path_of_the_settings_under_the_home() {
    let place = Place::new();
    settings(
        &place,
        "key = { path = \"~/.ssh/id_ecdsa_sk\", askpass = \"~/bin/askpass\" }\n",
    );
    let home = place.env.home.clone().unwrap();
    assert_eq!(
        boom::settings::key(&place.env).unwrap(),
        Some(boom::sign::Key {
            path: home.join(".ssh/id_ecdsa_sk"),
            askpass: Some(home.join("bin/askpass")),
        })
    );
    settings(&place, "# no key yet\n");
    assert_eq!(boom::settings::key(&place.env).unwrap(), None);
}

#[test]
fn one_accept_takes_two_causes_with_their_revisions() {
    let place = Place::new();
    let (ada, [_, _, cause, money, _]) = three_batches(&place);
    let words = place.proposed(OTHER, "A change record names its approval.");
    place.accepted(OTHER, &words, &ada);
    let second = place.proposed("names-no-wave", "A claim names no wave.");
    let revision = revised(
        &place,
        OTHER,
        "Each change record names its approval.",
        &words,
        "names-no-wave",
    );
    let second = format!("names-no-wave@{second}");
    let before = place.keys.signatures();
    place.json(&[
        "accept", "--cause", &cause, "--cause", &second, "--key", &ada,
    ]);
    assert_eq!(place.keys.signatures(), before + 1);
    assert_eq!(
        last_accepted(&place),
        [cause, money, second, format!("{OTHER}@{revision}")]
    );
}
