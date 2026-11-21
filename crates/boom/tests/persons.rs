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

    let ada = place.made();
    let bo = support::path_arg(&place.keys.plain_key("bo", 2));
    let refused = place.refused(
        &["register", "bo", &format!("{bo}.pub"), "--key", &ada],
        "ssh-ed25519",
    );
    for named in [
        "sk-ssh-ed25519@openssh.com",
        "sk-ecdsa-sha2-nistp256@openssh.com",
    ] {
        assert!(refused.stderr.contains(named), "{named}: {refused:?}");
    }
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
        ],
        "an agent's proposal names its provenance",
    );
}

#[test]
fn an_owner_registers_a_second_person() {
    let place = Place::new();
    let ada = place.made();
    let bo = place.key("bo", 2);
    let answer = place.json(&["register", "bo", &format!("{bo}.pub"), "--key", &ada]);
    assert_eq!(answer["format"], "boom-register/1");
    assert_eq!(answer["person"], "bo");
    assert_eq!(answer["owner"], false);
    let log = place.log();
    assert_eq!(log.len(), 2);
    let entry = &log[1];
    assert_eq!(entry["act"], "registered");
    assert_eq!(entry["actor"], serde_json::json!({"person": "ada"}));
    let signed = entry["signed"].as_str().unwrap();
    let fields: serde_json::Value = serde_json::from_str(signed).unwrap();
    assert_eq!(fields["person"], "bo");
    let bo_public = place.keys.public_key(std::path::Path::new(&bo));
    assert_eq!(fields["key"], boom::sign::key_line(&bo_public));
    let ada_public = place.keys.public_key(std::path::Path::new(&ada));
    assert_eq!(
        boom::sign::verify(
            &ada_public,
            signed.as_bytes(),
            entry["signature"].as_str().unwrap()
        ),
        Ok(())
    );
}

#[test]
fn a_register_by_someone_not_an_owner_is_refused() {
    let place = Place::new();
    let ada = place.made();
    let bo = place.registered(&ada, "bo", 2);
    let cy = place.key("cy", 3);
    place.refused(
        &["register", "cy", &format!("{cy}.pub"), "--key", &bo],
        "only an owner registers a person",
    );
}

#[test]
fn a_key_registered_to_another_is_refused() {
    let place = Place::new();
    let ada = place.made();
    let bo = place.registered(&ada, "bo", 2);
    place.refused(
        &["register", "cy", &format!("{bo}.pub"), "--key", &ada],
        "registered to bo",
    );
}

const MONEY: &str = "Every amount names its currency.";

#[test]
fn signed_without_presence_is_refused() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place
        .keys
        .set_flags(std::path::Path::new(&ada), support::ABSENT);
    place.refused(
        &[
            "accept",
            &format!("money-has-currency@{version}"),
            "--key",
            &ada,
        ],
        "the signature does not show its person present",
    );
    assert_eq!(place.status("money-has-currency", &version), "proposed");
}

#[test]
fn signed_by_a_key_not_registered_is_refused() {
    let place = Place::new();
    place.made();
    let version = place.proposed("money-has-currency", MONEY);
    let cy = place.key("cy", 3);
    place.refused(
        &[
            "accept",
            &format!("money-has-currency@{version}"),
            "--key",
            &cy,
        ],
        "is not registered",
    );
}

#[test]
fn an_agents_accept_is_refused() {
    let place = Place::new();
    place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.refused(
        &["accept", &format!("money-has-currency@{version}")],
        "only a person accepts",
    );
    assert_eq!(place.status("money-has-currency", &version), "proposed");
}

#[test]
fn a_person_accepts_an_agents_proposal() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    let answer = place.accepted("money-has-currency", &version, &ada);
    assert_eq!(answer["format"], "boom-accept/2");
    assert_eq!(answer["status"], "accepted");
    assert_eq!(answer["by"], "ada");
    assert_eq!(place.status("money-has-currency", &version), "accepted");
    let accepted = place.log().pop().unwrap();
    assert_eq!(accepted["act"], "accepted");
    assert_eq!(accepted["actor"], serde_json::json!({"person": "ada"}));
}

#[test]
fn no_one_accepts_their_own_proposal() {
    let place = Place::new();
    let ada = place.made();
    let proposed = place.json(&[
        "propose",
        "invariant",
        "money-has-currency",
        MONEY,
        "--key",
        &ada,
    ]);
    let version = proposed["version"].as_str().unwrap();
    assert_eq!(
        place.log()[1]["actor"],
        serde_json::json!({"person": "ada"})
    );
    place.refused(
        &[
            "accept",
            &format!("money-has-currency@{version}"),
            "--key",
            &ada,
        ],
        "waits on another person",
    );
    assert_eq!(place.status("money-has-currency", version), "proposed");
}

#[test]
fn another_person_accepts_it() {
    let place = Place::new();
    let ada = place.made();
    let bo = place.registered(&ada, "bo", 2);
    let proposed = place.json(&[
        "propose",
        "invariant",
        "money-has-currency",
        MONEY,
        "--key",
        &ada,
    ]);
    let version = proposed["version"].as_str().unwrap();
    let answer = place.accepted("money-has-currency", version, &bo);
    assert_eq!(answer["status"], "accepted");
    assert_eq!(answer["by"], "bo");
    let accepted = place.log().pop().unwrap();
    assert_eq!(accepted["act"], "accepted");
    assert_eq!(accepted["actor"], serde_json::json!({"person": "bo"}));
}

#[test]
fn a_reject_with_no_reason_is_refused() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.refused(
        &[
            "reject",
            &format!("money-has-currency@{version}"),
            "--key",
            &ada,
        ],
        "a reject gives its reason",
    );
    assert_eq!(place.status("money-has-currency", &version), "proposed");
}

#[test]
fn a_signature_verifies_with_no_boom() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", MONEY);
    place.accepted("money-has-currency", &version, &ada);
    let accepted = place.log().pop().unwrap();
    assert_eq!(accepted["act"], "accepted");
    let signed = accepted["signed"].as_str().unwrap();
    let signature =
        ssh_key::SshSig::from_pem(accepted["signature"].as_str().unwrap().trim_end()).unwrap();
    let registration: serde_json::Value =
        serde_json::from_str(place.log()[0]["signed"].as_str().unwrap()).unwrap();
    let registered =
        ssh_key::PublicKey::from_openssh(registration["key"].as_str().unwrap()).unwrap();
    assert_eq!(
        registered.algorithm().as_str(),
        "sk-ssh-ed25519@openssh.com"
    );
    registered
        .verify("boom", signed.as_bytes(), &signature)
        .expect("the accept's signature verifies by ada's registered key");
    assert!(registered
        .verify("boom", b"other words", &signature)
        .is_err());
}

/// Three agents' proposals, giving each as `<name>@<version>`.
fn three_proposed(place: &Place) -> Vec<String> {
    [
        ("money-has-currency", MONEY),
        (
            "refund-within-payment",
            "A refund never exceeds the payment.",
        ),
        ("orders-are-paid-once", "An order is paid once."),
    ]
    .iter()
    .map(|(name, statement)| format!("{name}@{}", place.proposed(name, statement)))
    .collect()
}

#[test]
fn three_accepted_together() {
    let place = Place::new();
    let ada = place.made();
    let targets = three_proposed(&place);
    let before = place.log().len();
    let signed_before = place.keys.signatures();
    let mut args = vec!["accept"];
    args.extend(targets.iter().map(String::as_str));
    args.extend(["--key", &ada]);
    let answer = place.json(&args);
    assert_eq!(answer["format"], "boom-accept/2");
    let accepted = answer["accepted"].as_array().unwrap();
    assert_eq!(accepted.len(), 3);
    assert_eq!(
        place.keys.signatures(),
        signed_before + 1,
        "the key signs one time"
    );
    let ada_public = place.keys.public_key(std::path::Path::new(&ada));
    let log = place.log();
    assert_eq!(log.len(), before + 1, "one entry accepts the three");
    let entry = &log[before];
    assert_eq!(entry["targets"], serde_json::json!(targets));
    for (target, listed) in targets.iter().zip(accepted.iter()) {
        let (name, version) = target.split_once('@').unwrap();
        assert_eq!(listed["claim"], name);
        assert_eq!(listed["version"], version);
        assert_eq!(listed["status"], "accepted");
        assert_eq!(listed["by"], "ada");
        assert_eq!(listed["entry"], entry["number"]);
        assert_eq!(place.status(name, version), "accepted");
        assert_eq!(entry["act"], "accepted");
        assert_eq!(entry["actor"], serde_json::json!({"person": "ada"}));
        assert_eq!(entry["counted"], true);
        assert_eq!(
            boom::sign::verify(
                &ada_public,
                entry["signed"].as_str().unwrap().as_bytes(),
                entry["signature"].as_str().unwrap()
            ),
            Ok(())
        );
    }
}

#[test]
fn one_of_them_refused_signs_and_appends_nothing() {
    let place = Place::new();
    let ada = place.made();
    let mut targets = three_proposed(&place);
    let own = place.json(&[
        "propose",
        "assumption",
        "invoices-are-monthly",
        "Invoices are sent monthly.",
        "--key",
        &ada,
    ]);
    let own = format!("invoices-are-monthly@{}", own["version"].as_str().unwrap());
    targets[1] = own.clone();
    let signatures = place.keys.signatures();
    let mut args = vec!["accept"];
    args.extend(targets.iter().map(String::as_str));
    args.extend(["--key", &ada]);
    let refused = place.refused(&args, &own);
    assert!(
        refused.stderr.contains("waits on another person"),
        "{refused:?}"
    );
    assert_eq!(place.keys.signatures(), signatures, "ada's key signed");
    for target in [&targets[0], &targets[2]] {
        let (name, version) = target.split_once('@').unwrap();
        assert_eq!(place.status(name, version), "proposed");
    }
}

/// Ada's accept of `targets` as one act, written by hand and signed by
/// `key`, the way `boom accept` writes it.
fn one_accept_of(targets: &[&str]) -> boom::entry::Entry {
    boom::entry::Entry {
        at: 1_763_424_500,
        actor: boom::entry::Actor::Person("ada".to_owned()),
        act: boom::entry::Act::Accepted {
            versions: targets
                .iter()
                .map(|target| {
                    let (name, version) = target.split_once('@').unwrap();
                    boom::entry::Named {
                        name: name.to_owned(),
                        version: version.to_owned(),
                    }
                })
                .collect(),
        },
    }
}

#[test]
fn an_accept_again_appends_nothing() {
    let place = Place::new();
    let ada = place.made();
    let targets = three_proposed(&place);
    let mut args = vec!["accept"];
    args.extend(targets.iter().map(String::as_str));
    args.extend(["--key", &ada]);
    place.json(&args);
    let before = place.log().len();
    let signed_before = place.keys.signatures();
    let again = place.json(&args);
    assert_eq!(again["accepted"].as_array().unwrap().len(), 3);
    assert_eq!(place.log().len(), before);
    assert_eq!(place.keys.signatures(), signed_before);
}

#[test]
fn a_claim_named_twice_is_refused() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", MONEY);
    let second = place.proposed("money-has-currency", "Each amount names one currency.");
    let before = place.log().len();
    place.refused(
        &[
            "accept",
            &format!("money-has-currency@{first}"),
            &format!("money-has-currency@{second}"),
            "--key",
            &ada,
        ],
        "names money-has-currency twice",
    );
    assert_eq!(place.log().len(), before);
}

#[test]
fn one_accept_with_one_bad_version_counts_for_nothing() {
    let place = Place::new();
    let ada = place.made();
    let targets = three_proposed(&place);
    let mut named: Vec<&str> = targets.iter().map(String::as_str).collect();
    named.push("no-such-claim@000000000000");
    place.append_by_hand(&one_accept_of(&named), Some(std::path::Path::new(&ada)));
    let entry = place.log().pop().unwrap();
    assert_eq!(entry["counted"], false);
    for target in &targets {
        let (name, version) = target.split_once('@').unwrap();
        assert_eq!(place.status(name, version), "proposed", "{target}");
    }
}

#[test]
fn one_accept_reads_as_an_accept_of_each_in_the_views_and_a_replay() {
    let place = Place::new();
    let ada = place.made();
    let targets = three_proposed(&place);
    let named: Vec<&str> = targets.iter().map(String::as_str).collect();
    place.append_by_hand(&one_accept_of(&named), Some(std::path::Path::new(&ada)));
    let entry = place.log().pop().unwrap();
    assert_eq!(entry["counted"], true, "{entry}");
    let shown: Vec<_> = targets.iter().map(|target| place.show(target)).collect();
    for (target, shown) in targets.iter().zip(&shown) {
        assert_eq!(shown["status"], "accepted", "{target}");
        assert_eq!(shown["decided_in"], entry["number"], "{target}");
    }
    let verified = place.run(&["verify"]);
    assert_eq!(verified.code, 0, "{verified:?}");
    for (target, before) in targets.iter().zip(&shown) {
        assert_eq!(&place.show(target), before, "{target}");
    }
    let words = place.run(&["log"]).stdout;
    let line = format!("{} accepted {} by ada", entry["number"], targets.join(", "));
    assert!(words.contains(&line), "{words}");
    place.refused(
        &["show", &entry["number"].to_string()],
        "accepts 3 versions",
    );
}
