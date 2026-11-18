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
