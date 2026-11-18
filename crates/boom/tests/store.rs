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
