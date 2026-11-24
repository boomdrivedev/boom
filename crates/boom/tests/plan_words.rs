//! The plan words in a proposal's statement and body: a warning that
//! names each one, and never a refusal (claim-versions, "The plan words"
//! and "A proposal warns of plan words").

mod support;

use serde_json::json;
use support::Place;

fn proposal<'a>(statement: &'a str, more: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "propose",
        "invariant",
        "each-entry-kept",
        statement,
        "--provenance",
        "run-1",
    ];
    args.extend_from_slice(more);
    args
}

#[test]
fn a_plan_word_in_the_statement() {
    let place = Place::new();
    place.made();
    let before = place.log().len();
    let outcome = place.run(&proposal("In the first wave, boom keeps each entry.", &[]));
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(place.log().len(), before + 1);
    assert!(
        outcome
            .stderr
            .contains("warning: \"wave\" names a plan, in the statement."),
        "{outcome:?}"
    );
    assert!(outcome.stdout.contains("each-entry-kept@"), "{outcome:?}");
}

#[test]
fn a_plan_word_in_the_body() {
    let place = Place::new();
    place.made();
    let mut text = String::new();
    for line in 1..12 {
        text.push_str(&format!("Line {line} of the context.\n"));
    }
    text.push_str("When the store is locked, boom takes the happy path.\n");
    let body = place.file("body.md", &text);
    let outcome = place.run(&[
        "propose",
        "decision",
        "one-way",
        "The shop takes one way.",
        "--body",
        &body,
        "--provenance",
        "run-1",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(
        outcome
            .stderr
            .contains("warning: \"happy path\" names a plan, in line 12 of the body."),
        "{outcome:?}"
    );
}

#[test]
fn the_warnings_as_json() {
    let place = Place::new();
    place.made();
    let body = place.file(
        "body.md",
        "The shop.\n\nIts parts.\n\nPhases of the work.\n",
    );
    let answer = place.json(&proposal("The MVP keeps each entry.", &["--body", &body]));
    assert_eq!(answer["format"], "boom-propose/2");
    assert_eq!(
        answer["plan_words"],
        json!([
            {"word": "MVP", "in": "statement", "line": null},
            {"word": "phase", "in": "body", "line": 5},
        ])
    );
}

#[test]
fn another_form() {
    let place = Place::new();
    place.made();
    let body = place.file("body.md", "Phases\n");
    let outcome = place.run(&proposal("boom keeps each entry.", &["--body", &body]));
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(
        outcome.stderr.contains("warning: \"phase\" names a plan"),
        "{outcome:?}"
    );
}

#[test]
fn no_plan_word() {
    let place = Place::new();
    place.made();
    let body = place.file("body.md", "Each entry stays in the log.\n");
    let mut args = proposal("boom keeps each entry.", &["--body", &body]);
    args.push("--json");
    let outcome = place.run(&args);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(!outcome.stderr.contains("warning"), "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["plan_words"], json!([]));
}

#[test]
fn a_plan_word_inside_a_longer_word() {
    let place = Place::new();
    place.made();
    let outcome = place.run(&proposal("A microwave keeps each entry warm.", &[]));
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(!outcome.stderr.contains("warning"), "{outcome:?}");
}

#[test]
fn the_same_version_again() {
    let place = Place::new();
    place.made();
    let before = place.log().len();
    let args = proposal("The roadmap keeps each entry.", &[]);
    let first = place.run(&args);
    let second = place.run(&args);
    for outcome in [&first, &second] {
        assert_eq!(outcome.code, 0, "{outcome:?}");
        assert!(
            outcome.stderr.contains("warning: \"roadmap\" names a plan"),
            "{outcome:?}"
        );
    }
    assert_eq!(place.log().len(), before + 1);
}

#[test]
fn a_marker_on_the_line() {
    let place = Place::new();
    place.made();
    let body = place.file("body.md", "the user's words: \"MVP\" boom: allow MVP\n");
    let outcome = place.run(&proposal("boom keeps each entry.", &["--body", &body]));
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(!outcome.stderr.contains("warning"), "{outcome:?}");
}
