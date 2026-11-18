//! Persons, owners and agents, and who may propose, accept and reject
//! (persons-acts).

mod support;

use support::Place;

#[test]
fn a_store_made_registers_its_first_owner() {
    let place = Place::new();
    let ada = place.made();
    let log = place.log();
    assert_eq!(log.len(), 1);
    let entry = &log[0];
    assert_eq!(entry["act"], "registered");
    assert_eq!(entry["actor"], serde_json::json!({"person": "ada"}));
    assert_eq!(entry["counted"], true);
    let signed: serde_json::Value =
        serde_json::from_str(entry["signed"].as_str().unwrap()).unwrap();
    assert_eq!(signed["person"], "ada");
    assert_eq!(signed["owner"], true);
    let public = place.keys.public_key(std::path::Path::new(&ada));
    assert_eq!(signed["key"], boom::sign::key_line(&public));
    let signature = entry["signature"].as_str().unwrap();
    assert_eq!(
        boom::sign::verify(
            &public,
            entry["signed"].as_str().unwrap().as_bytes(),
            signature
        ),
        Ok(())
    );
}

#[test]
fn made_twice_with_the_same_key_adds_nothing() {
    let place = Place::new();
    let ada = place.made();
    let first = place.log();
    let again = place.run(&["init", "--person", "ada", "--key", &ada, "--json"]);
    assert_eq!(again.code, 0, "{again:?}");
    let answer: serde_json::Value = serde_json::from_str(&again.stdout).unwrap();
    assert_eq!(answer["format"], "boom-init/1");
    assert_eq!(answer["person"], "ada");
    assert_eq!(answer["entry"], 1);
    assert_eq!(place.log(), first);
}

#[test]
fn made_again_with_another_key_is_refused() {
    let place = Place::new();
    place.made();
    let first = place.log();
    let bo = place.key("bo", 2);
    let refused = place.run(&["init", "--person", "bo", "--key", &bo]);
    assert_eq!(refused.code, 2, "{refused:?}");
    assert!(refused.stderr.contains("exists"), "{refused:?}");
    assert!(
        refused.stderr.contains("its first owner is ada"),
        "{refused:?}"
    );
    assert_eq!(place.log(), first);
}

#[test]
fn a_plain_key_file_is_refused() {
    let place = Place::new();
    let plain = support::path_arg(&place.keys.plain_key("ada", 1));
    let refused = place.run(&["init", "--person", "ada", "--key", &plain]);
    assert_eq!(refused.code, 2, "{refused:?}");
    for named in [
        "ssh-ed25519",
        "sk-ssh-ed25519@openssh.com",
        "sk-ecdsa-sha2-nistp256@openssh.com",
    ] {
        assert!(refused.stderr.contains(named), "{named}: {refused:?}");
    }
    assert!(!place.store.exists());
}

#[test]
fn an_agents_proposal_names_its_provenance() {
    let place = Place::new();
    place.made();
    let outcome = place.run(&[
        "propose",
        "invariant",
        "money-has-currency",
        "Every amount names its currency.",
        "--scenario",
        support::SCENARIO,
        "--provenance",
        "flume:place-7",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let log = place.log();
    let proposed = &log[1];
    assert_eq!(proposed["act"], "proposed");
    assert_eq!(
        proposed["actor"],
        serde_json::json!({"agent": {"provenance": "flume:place-7"}})
    );
    assert_eq!(proposed["signature"], serde_json::Value::Null);
    assert_eq!(proposed["counted"], true);
}

#[test]
fn an_agents_proposal_with_no_provenance_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "invariant",
            "money-has-currency",
            "Every amount names its currency.",
            "--scenario",
            support::SCENARIO,
        ],
        "an agent's proposal names its provenance",
    );
}
