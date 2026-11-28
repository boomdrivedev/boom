//! The repositories a store registers, each one's remote and its
//! delivery at session start, set by an owner (repository-settings).

#[path = "../src/tests/support.rs"]
mod support;

use serde_json::json;
use support::Place;

#[test]
fn an_owner_registers_a_repository() {
    let place = Place::new();
    let ada = place.made();
    place.repo_added(&ada, "sluice", "git@github.com:boomdrivedev/sluice.git");
    let shown = place.json(&["repo", "show", "sluice"]);
    assert_eq!(shown["format"], "boom-repo/1");
    assert_eq!(
        shown["repositories"],
        json!([{
            "name": "sluice",
            "remote": "github.com/boomdrivedev/sluice",
            "session": true,
            "set_by": "ada",
            "set_in": 2,
        }])
    );
    let last = place.last_signed();
    assert_eq!(last["act"], "repository");
    assert_eq!(last["remote"], "github.com/boomdrivedev/sluice");
}

#[test]
fn a_person_who_is_not_an_owner_is_refused() {
    let place = Place::new();
    let ada = place.made();
    let bo = place.registered(&ada, "bo", 2);
    let before = place.log().len();
    place.refused(
        &[
            "repo",
            "add",
            "sluice",
            "--remote",
            "https://github.com/a/sluice",
            "--key",
            &bo,
        ],
        "only an owner sets a repository",
    );
    assert_eq!(place.log().len(), before);
}

#[test]
fn an_agent_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "repo",
            "add",
            "sluice",
            "--remote",
            "https://github.com/a/sluice",
        ],
        "only an owner sets a repository",
    );
}

#[test]
fn a_remote_taken_is_refused() {
    let place = Place::new();
    let ada = place.made();
    place.repo_added(&ada, "sluice", "https://github.com/a/sluice");
    place.refused(
        &[
            "repo",
            "add",
            "other",
            "--remote",
            "git@github.com:a/sluice.git",
            "--key",
            &ada,
        ],
        "sluice",
    );
}

#[test]
fn a_set_changes_the_delivery_and_a_repeat_appends_nothing() {
    let place = Place::new();
    let ada = place.made();
    place.repo_added(&ada, "sluice", "https://github.com/a/sluice");
    let off = ["repo", "set", "sluice", "--session", "off", "--key", &ada];
    let set = place.json(&off);
    assert_eq!(set["repositories"][0]["session"], false);
    let before = place.log().len();
    let again = place.run(&off);
    assert_eq!(again.code, 0, "{again:?}");
    assert_eq!(place.log().len(), before);
}

#[test]
fn a_remote_in_no_known_form_is_refused() {
    let place = Place::new();
    let ada = place.made();
    place.refused(
        &[
            "repo",
            "add",
            "sluice",
            "--remote",
            "not a remote",
            "--key",
            &ada,
        ],
        "a remote is",
    );
}

#[test]
fn the_views_and_a_replay_agree() {
    let place = Place::new();
    let ada = place.made();
    place.repo_added(&ada, "sluice", "https://github.com/a/sluice");
    place.repo_added(&ada, "flume", "https://github.com/a/flume");
    let before = place.json(&["repo", "show"]);
    let verified = place.run(&["verify"]);
    assert_eq!(verified.code, 0, "{verified:?}");
    assert_eq!(place.json(&["repo", "show"]), before);
}
