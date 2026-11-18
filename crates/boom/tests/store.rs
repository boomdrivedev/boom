//! The store's file, its views, its readers and writers, and its blobs
//! (record-store).

mod support;

use support::Place;

#[test]
fn the_default_store_is_under_xdg_data_home() {
    let place = Place::new();
    let ada = place.key("ada", 1);
    let outcome = place.run_bare(&["init", "--person", "ada", "--key", &ada, "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let expected = place.dir.path().join("data/boom/boom.sqlite");
    assert!(expected.is_file());
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["store"], expected.to_str().unwrap());
    assert!(!place.dir.path().join("home").exists());
    assert!(!place.store.exists());
}

#[test]
fn a_store_named_outright_is_used_and_nothing_else_made() {
    let place = Place::new();
    place.made();
    assert!(place.store.is_file());
    let log = place.run(&["log"]);
    assert_eq!(log.code, 0, "{log:?}");
    assert!(!place.dir.path().join("data").exists());
    assert!(!place.dir.path().join("home").exists());
    let mut made: Vec<String> = std::fs::read_dir(place.dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| !name.starts_with("boom.sqlite-"))
        .collect();
    made.sort();
    assert_eq!(made, ["boom.sqlite", "keys"]);
}

#[test]
fn no_store_yet_exits_3_and_makes_nothing() {
    let place = Place::new();
    let outcome = place.run(&[
        "propose",
        "invariant",
        "money-has-currency",
        "Every amount names its currency.",
        "--scenario",
        support::SCENARIO,
        "--provenance",
        "run-1",
    ]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(outcome.stderr.contains(&place.store_arg()), "{outcome:?}");
    assert!(
        outcome.stderr.contains("boom init makes one"),
        "{outcome:?}"
    );
    assert!(!place.store.exists());
    let made: Vec<_> = std::fs::read_dir(place.dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(made, ["keys"]);
}

fn fingerprint(text: &str) -> String {
    boom::sha256_hex(text.as_bytes())
}

#[test]
fn one_body_two_versions_one_blob() {
    let place = Place::new();
    place.made();
    let body = place.file("money.md", support::BODY);
    let first = place.proposed_with_body(
        "money-has-currency",
        "Every amount names its currency.",
        &body,
    );
    let second = place.proposed_with_body(
        "money-has-currency",
        "Every amount names one currency.",
        &body,
    );
    assert_ne!(first["version"], second["version"]);
    let blobs: i64 = place
        .sqlite()
        .query_row("SELECT count(*) FROM blobs", [], |row| row.get(0))
        .unwrap();
    assert_eq!(blobs, 1);
    for answer in [&first, &second] {
        let target = format!("money-has-currency@{}", answer["version"].as_str().unwrap());
        assert_eq!(
            place.show(&target)["body_fingerprint"],
            fingerprint(support::BODY)
        );
    }
}

#[test]
fn a_body_read_back_is_its_bytes() {
    let place = Place::new();
    place.made();
    let body = place.file("money.md", support::BODY);
    let answer = place.proposed_with_body(
        "money-has-currency",
        "Every amount names its currency.",
        &body,
    );
    let target = format!("money-has-currency@{}", answer["version"].as_str().unwrap());
    assert_eq!(place.show(&target)["body"], support::BODY);
    let words = place.run(&["show", &target]);
    assert_eq!(words.code, 0, "{words:?}");
    assert!(words.stdout.ends_with(support::BODY), "{words:?}");
}

#[test]
fn an_altered_blob_is_unreadable() {
    let place = Place::new();
    place.made();
    let body = place.file("money.md", support::BODY);
    let answer = place.proposed_with_body(
        "money-has-currency",
        "Every amount names its currency.",
        &body,
    );
    let target = format!("money-has-currency@{}", answer["version"].as_str().unwrap());
    let altered = place
        .sqlite()
        .execute(
            "UPDATE blobs SET bytes = ?1 WHERE fingerprint = ?2",
            rusqlite::params![
                b"An amount is a number.".to_vec(),
                fingerprint(support::BODY)
            ],
        )
        .unwrap();
    assert_eq!(altered, 1);
    let outcome = place.run(&["show", &target, "--json"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-not-run/1");
    let reason = answer["not_run"]["reason"].as_str().unwrap();
    assert!(reason.contains(&fingerprint(support::BODY)), "{reason}");
    assert!(!outcome.stdout.contains("An amount is"), "{outcome:?}");
    assert!(!outcome.stderr.contains("An amount is"), "{outcome:?}");
}

const VIEWS: [&str; 5] = ["views_head", "persons", "claims", "versions", "judgments"];

fn tables(place: &Place) -> Vec<String> {
    let conn = place.sqlite();
    let mut statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap();
    statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn views_deleted_change_no_answer() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed("money-has-currency", "Every amount names its currency.");
    place.accepted("money-has-currency", &first, &ada);
    place.amended(
        "money-has-currency",
        "Every amount names one currency.",
        &first,
    );
    let before = place.show("money-has-currency");
    let log = place.log();
    let made = tables(&place);
    let conn = place.sqlite();
    for table in VIEWS {
        conn.execute_batch(&format!("DROP TABLE {table}")).unwrap();
    }
    drop(conn);
    assert!(VIEWS
        .iter()
        .all(|view| !tables(&place).iter().any(|table| table == view)));
    assert_eq!(place.show("money-has-currency"), before);
    assert_eq!(tables(&place), made);
    assert_eq!(place.log(), log);
}

#[test]
fn views_behind_the_log_are_rebuilt_before_an_answer() {
    let place = Place::new();
    place.made();
    place.proposed("money-has-currency", "Every amount names its currency.");
    let proposal = boom::claim::Proposal::new(
        boom::claim::Kind::Assumption,
        "orders-are-paid-once",
        "An order is paid once.",
        vec![support::SCENARIO.to_owned()],
        None,
        None,
        None,
    );
    let entry = boom::entry::Entry {
        at: 1_763_424_600,
        actor: boom::entry::Actor::Agent {
            provenance: "another-program".to_owned(),
        },
        act: boom::entry::Act::Proposed {
            claim: proposal.clone(),
            imported: None,
            attach: None,
        },
    };
    place.append_by_hand(&entry, None);
    let head: i64 = place
        .sqlite()
        .query_row("SELECT seq FROM views_head", [], |row| row.get(0))
        .unwrap();
    assert_eq!(head, 2, "the views were left behind");
    let shown = place.show("orders-are-paid-once");
    assert_eq!(shown["kind"], "assumption");
    assert_eq!(shown["versions"][0]["version"], proposal.version.as_str());
    assert_eq!(shown["versions"][0]["status"], "proposed");
    let head: i64 = place
        .sqlite()
        .query_row("SELECT seq FROM views_head", [], |row| row.get(0))
        .unwrap();
    assert_eq!(head, 3);

    // The same proposal again, by hand, repeats it and counts for nothing.
    place.append_by_hand(&entry, None);
    let repeat = place.log().pop().unwrap();
    assert_eq!(repeat["counted"], false);
    assert_eq!(repeat["reason"], "repeats entry 3");
    assert_eq!(
        place.show("orders-are-paid-once")["versions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

fn agents_proposal_command(place: &Place, name: &str) -> std::process::Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_boom"));
    command
        .args([
            "propose",
            "invariant",
            name,
            "Every amount names its currency.",
            "--scenario",
            support::SCENARIO,
            "--provenance",
            name,
            "--store",
        ])
        .arg(&place.store)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null());
    command
}

#[test]
fn two_proposals_at_once_both_land_chained() {
    let place = Place::new();
    place.made();
    let mut one = agents_proposal_command(&place, "money-has-currency")
        .spawn()
        .unwrap();
    let mut two = agents_proposal_command(&place, "refund-within-payment")
        .spawn()
        .unwrap();
    assert_eq!(one.wait().unwrap().code(), Some(0));
    assert_eq!(two.wait().unwrap().code(), Some(0));
    let log = place.log();
    assert_eq!(log.len(), 3);
    let mut proposed: Vec<String> = log[1..]
        .iter()
        .map(|entry| {
            entry["actor"]["agent"]["provenance"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    proposed.sort();
    assert_eq!(proposed, ["money-has-currency", "refund-within-payment"]);
    for at in 1..log.len() {
        assert_eq!(log[at]["prev"], log[at - 1]["hash"]);
        assert_eq!(log[at]["counted"], true);
    }
}

#[test]
fn a_read_during_a_write_sees_none_of_it() {
    let place = Place::new();
    place.made();
    place.proposed("money-has-currency", "Every amount names its currency.");
    let before = place.show("money-has-currency");
    let writer = place.sqlite();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    writer
        .execute(
            "INSERT INTO log (seq, prev, hash, signed, signature) VALUES (3, 'p', 'h', '{}', NULL)",
            [],
        )
        .unwrap();
    let started = std::time::Instant::now();
    let during = place.show("money-has-currency");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "the read waited on the write: {:?}",
        started.elapsed()
    );
    assert_eq!(during, before);
    writer.execute_batch("ROLLBACK").unwrap();
}

#[test]
fn an_append_and_its_views_are_one_transaction() {
    let place = Place::new();
    place.made();
    place
        .sqlite()
        .execute_batch(
            "CREATE TRIGGER no_versions BEFORE INSERT ON versions \
             BEGIN SELECT RAISE(ABORT, 'versions refused'); END;",
        )
        .unwrap();
    let rows = place.rows();
    let outcome = place.run(&[
        "propose",
        "invariant",
        "money-has-currency",
        "Every amount names its currency.",
        "--scenario",
        support::SCENARIO,
        "--provenance",
        "run-1",
    ]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert!(outcome.stderr.contains("versions refused"), "{outcome:?}");
    assert_eq!(place.rows(), rows);
}
