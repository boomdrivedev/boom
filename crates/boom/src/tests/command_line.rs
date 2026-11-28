//! The command line: its commands, exit codes, JSON answers, no
//! prompts, and repeated writes (command-line).

use super::support;

use support::Place;

#[test]
fn the_version() {
    let outcome = Place::new().run_bare(&["--version"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(
        outcome.stdout.trim_end(),
        format!("boom {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn the_log_of_a_new_store() {
    let place = Place::new();
    place.made();
    let log = place.run(&["log"]);
    assert_eq!(log.code, 0, "{log:?}");
    let lines: Vec<&str> = log.stdout.lines().collect();
    assert_eq!(lines.len(), 1, "{log:?}");
    assert!(lines[0].starts_with("1 registered by ada"), "{log:?}");
}

const PROPOSAL: [&str; 6] = [
    "propose",
    "invariant",
    "money-has-currency",
    "Every amount names its currency.",
    "--provenance",
    "run-1",
];

#[test]
fn a_proposal_answered_as_json() {
    let place = Place::new();
    place.made();
    let mut args = PROPOSAL.to_vec();
    args.push("--json");
    let outcome = place.run(&args);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-propose/2");
    assert_eq!(answer["claim"], "money-has-currency");
    assert_eq!(answer["version"].as_str().unwrap().len(), 12);
    assert_eq!(answer["status"], "proposed");
    assert!(!outcome.stderr.is_empty(), "the words go to standard error");
}

#[test]
fn an_unknown_kind_is_refused_naming_the_kinds() {
    let place = Place::new();
    place.made();
    let mut args = PROPOSAL.to_vec();
    args[1] = "rule";
    let outcome = place.run(&args);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    for kind in ["invariant", "decision", "assumption"] {
        assert!(outcome.stderr.contains(kind), "{kind}: {outcome:?}");
    }
}

#[test]
fn a_registration_repeated_adds_nothing() {
    let place = Place::new();
    let ada = place.made();
    let bo = place.key("bo", 2);
    let public = format!("{bo}.pub");
    let first = place.json(&["register", "bo", &public, "--key", &ada]);
    let second = place.run(&["register", "bo", &public, "--key", &ada, "--json"]);
    assert_eq!(second.code, 0, "{second:?}");
    let second: serde_json::Value = serde_json::from_str(&second.stdout).unwrap();
    assert_eq!(second, first);
    let registrations = place
        .log()
        .into_iter()
        .filter(|entry| entry["act"] == "registered")
        .count();
    assert_eq!(registrations, 2, "ada's and bo's");
}

fn agents_proposal(place: &Place) -> String {
    place.proposed("money-has-currency", "Every amount names its currency.")
}

#[test]
fn a_refusal_writes_nothing() {
    let place = Place::new();
    place.made();
    let version = agents_proposal(&place);
    let before = std::fs::read(&place.store).unwrap();
    let log = place.log();
    let refused = place.run(&["accept", &format!("money-has-currency@{version}")]);
    assert_eq!(refused.code, 2, "{refused:?}");
    assert_eq!(place.log(), log);
    assert_eq!(std::fs::read(&place.store).unwrap(), before);
}

#[test]
fn a_refusal_answered_as_json() {
    let place = Place::new();
    place.made();
    let version = agents_proposal(&place);
    let outcome = place.run(&["accept", &format!("money-has-currency@{version}"), "--json"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-refused/1");
    let reason = answer["refused"]["reason"].as_str().unwrap();
    assert!(reason.starts_with("only a person accepts"), "{reason}");
}

#[test]
fn a_key_whose_program_cannot_sign_could_not_run() {
    let place = Place::new();
    let ada = place.made();
    let version = agents_proposal(&place);
    let log = place.log();
    let keygen = boom::sign::SshKeygen {
        program: place.dir.path().join("no-such-ssh-keygen"),
    };
    let outcome = place.run_with(
        &keygen,
        &[
            "accept",
            &format!("money-has-currency@{version}"),
            "--key",
            &ada,
        ],
    );
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(
        outcome.stderr.contains("the key's program did not sign"),
        "{outcome:?}"
    );
    assert_eq!(place.log(), log);
}

#[test]
fn not_run_answered_as_json() {
    let place = Place::new();
    place.three_entries();
    place
        .sqlite()
        .execute(
            "UPDATE log SET signed = replace(signed, 'its currency', 'any currency') WHERE seq = 2",
            [],
        )
        .unwrap();
    // An older entry changed is found by boom verify (record-log).
    let outcome = place.run(&["verify", "--json"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-not-run/1");
    let reason = answer["not_run"]["reason"].as_str().unwrap();
    assert!(reason.contains("entry 2"), "{reason}");
}

#[test]
fn an_import_answered_as_json() {
    let place = Place::new();
    place.made();
    let dir = place.folder(
        "adr",
        &[(
            "0001-record-architecture-decisions.md",
            "---\nstatus: accepted\n---\n\n# Record architecture decisions\n",
        )],
    );
    let outcome = place.run(&[
        "import",
        "adr",
        &dir,
        "--repository",
        "shop",
        "--dry-run",
        "--json",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-import/2");
    assert!(!outcome.stderr.is_empty(), "the words go to standard error");
}

#[test]
fn a_kind_it_does_not_take() {
    let place = Place::new();
    place.made();
    let outcome = place.refused(
        &[
            "propose",
            "widget",
            "money-has-currency",
            "Every amount names its currency.",
            "--provenance",
            "run-1",
        ],
        "invalid value 'widget'",
    );
    assert!(
        outcome
            .stderr
            .contains("invariant, decision, assumption, term, context"),
        "{outcome:?}"
    );
}

#[test]
fn a_note_answered_as_json() {
    let place = Place::new();
    place.made_by("chuck");
    let version = place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &[],
    )["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let target = format!("boom-end-to-end@{version}");
    let outcome = place.run(&["note", &target, "--paragraph", "1", "A word.", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-note/1");
    assert_eq!(answer["note"].as_str().unwrap().len(), 12);
}

#[test]
fn the_commands() {
    let place = Place::new();
    let help = place.run_bare(&["--help"]).stdout;
    for command in [
        "init", "register", "propose", "accept", "reject", "show", "log", "verify", "import",
        "repo", "terms", "note", "page", "hook", "mcp",
    ] {
        assert!(
            help.lines()
                .any(|line| line.trim_start().starts_with(&format!("{command} "))),
            "{command}: {help}"
        );
    }
    place.made();
    place.refused(&["page", "--json"], "boom page takes no --json");
}
