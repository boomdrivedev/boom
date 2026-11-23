//! A view: the outline of a view, a claim that a person accepts and that
//! holds narratives in order, and the design book, whose parts come from
//! the level that each narrative binds and whose order comes from the
//! outline (views).

mod support;

use serde_json::{json, Value};
use support::{Place, NARRATIVE};

const BOOK: &str = "boomdrive-design-book";
const BOOK_TITLE: &str = "The design book of Boomdrive";
const END_TO_END: &str = "boom-end-to-end";
const RECORDS: &str = "change-records";
const FLOW: &str = "the-merge-queues-flow";
const LOG: &str = "the-drives-log";
const QUEUE: &str = "the-merge-queue";
const HOST: &str = "sluice-host-process";
const STATUS: &str = "status-read-from-events";

fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// Proposes the narrative `name` with `extra`, giving its version.
fn narrative(place: &Place, name: &str, extra: &[&str]) -> String {
    version(&place.narrative(name, &format!("The narrative {name}"), NARRATIVE, extra))
}

/// The arguments of the proposal of the outline `name` of the view
/// `book` with `narratives` and `extra`.
fn outline_args<'a>(name: &'a str, narratives: &[&'a str], extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["propose", "outline", name, BOOK_TITLE, "--view", "book"];
    for held in narratives {
        args.extend(["--narrative", held]);
    }
    args.extend(extra);
    args.extend(["--provenance", "run-1"]);
    args
}

/// Proposes the outline `name` of the view `book` with `narratives` and
/// `extra`, giving its answer.
fn outline(place: &Place, name: &str, narratives: &[&str], extra: &[&str]) -> Value {
    place.json(&outline_args(name, narratives, extra))
}

/// Proposes the context `name` with `globs`, giving its version.
fn context(place: &Place, name: &str, globs: &[&str]) -> String {
    let mut args = vec!["propose", "context", name, "A part of the system."];
    for glob in globs {
        args.extend(["--path", glob]);
    }
    args.extend(["--provenance", "run-1"]);
    version(&place.json(&args))
}

/// The names of the narratives of `entries`, a list of the book.
fn names(entries: &Value) -> Vec<String> {
    entries
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["narrative"].as_str().unwrap().to_owned())
        .collect()
}

/// The names `key` gives of each item of `list`.
fn keys(list: &Value, key: &str) -> Vec<String> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|item| item[key].as_str().unwrap().to_owned())
        .collect()
}

/// The store of "Three levels": `the-drives-log` binds the whole system,
/// `boom-end-to-end` binds `boom`, `the-merge-queue` binds the context
/// `sluice-host-process` of `sluice`, and the outline holds the three.
/// It gives ada's key.
fn store_of_three_levels(place: &Place) -> String {
    let ada = place.made();
    narrative(place, LOG, &["--system"]);
    narrative(place, END_TO_END, &["--repository", "boom"]);
    context(place, HOST, &["sluice:crates/sluice/src/host/**"]);
    narrative(place, QUEUE, &["--context", HOST]);
    outline(place, BOOK, &[LOG, END_TO_END, QUEUE], &[]);
    ada
}

// An outline holds the narratives of a view

#[test]
fn an_outline_proposed() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &["--repository", "boom"]);
    narrative(&place, FLOW, &["--repository", "sluice"]);
    let answer = outline(&place, BOOK, &[END_TO_END, FLOW], &[]);
    assert_eq!(answer["kind"], "outline");
    let shown = place.show(BOOK);
    assert_eq!(shown["kind"], "outline");
    assert_eq!(shown["versions"].as_array().unwrap().len(), 1);
    assert_eq!(shown["newest"]["status"], "proposed");
    assert_eq!(shown["view"], "book");
    assert_eq!(keys(&shown["narratives"], "claim"), [END_TO_END, FLOW]);
}

#[test]
fn no_view() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &[]);
    place.refused(
        &[
            "propose",
            "outline",
            BOOK,
            BOOK_TITLE,
            "--narrative",
            END_TO_END,
            "--provenance",
            "run-1",
        ],
        "an outline names its view",
    );
}

#[test]
fn a_view_that_boom_does_not_compute() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &[]);
    let mut args = outline_args(BOOK, &[END_TO_END], &[]);
    args[5] = "context-map";
    place.refused(&args, "the view of an outline is book");
}

#[test]
fn no_narrative() {
    let place = Place::new();
    place.made();
    place.refused(
        &outline_args(BOOK, &[], &[]),
        "an outline holds one narrative at least",
    );
}

// An outline names each narrative at the version it meant

#[test]
fn a_name_stores_its_accepted_version() {
    let place = Place::new();
    let ada = place.made();
    let v1 = narrative(&place, END_TO_END, &["--repository", "boom"]);
    place.accepted(END_TO_END, &v1, &ada);
    let file = place.file("again.md", "Again.\n");
    place.json(&[
        "propose",
        "narrative",
        END_TO_END,
        "Boom, end to end",
        "--body",
        &file,
        "--amends",
        &v1,
        "--provenance",
        "run-1",
    ]);
    outline(&place, BOOK, &[END_TO_END], &[]);
    assert_eq!(
        place.last_signed()["claim"]["narratives"],
        json!([format!("{END_TO_END}@{v1}")])
    );
}

#[test]
fn a_name_that_does_not_resolve() {
    let place = Place::new();
    place.made();
    place.refused(
        &outline_args(BOOK, &["what-boomdrive-delivers"], &[]),
        "what-boomdrive-delivers",
    );
}

#[test]
fn a_claim_that_is_not_a_narrative() {
    let place = Place::new();
    place.made();
    place.proposed(STATUS, "A record never holds its status.");
    place.refused(
        &outline_args(BOOK, &[STATUS], &[]),
        "an outline holds narratives",
    );
}

#[test]
fn a_narrative_given_two_times() {
    let place = Place::new();
    place.made();
    let v1 = narrative(&place, END_TO_END, &[]);
    let at = format!("{END_TO_END}@{v1}");
    place.refused(
        &outline_args(BOOK, &[END_TO_END, &at], &[]),
        &format!("the outline holds {END_TO_END} two times"),
    );
}

#[test]
fn narratives_on_a_decision() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &[]);
    let body = place.decision_body();
    place.refused(
        &[
            "propose",
            "decision",
            "one-book",
            "The store has one book.",
            "--body",
            &body,
            "--narrative",
            END_TO_END,
            "--provenance",
            "run-1",
        ],
        "only an outline holds narratives",
    );
}

#[test]
fn another_order_is_another_version() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &[]);
    narrative(&place, RECORDS, &[]);
    let first = version(&outline(&place, BOOK, &[END_TO_END, RECORDS], &[]));
    let second = version(&outline(&place, BOOK, &[RECORDS, END_TO_END], &[]));
    assert_ne!(first, second);
    assert_eq!(place.show(BOOK)["versions"].as_array().unwrap().len(), 2);
}

// An outline binds the whole system

#[test]
fn proposed_in_a_worktree_of_a_repository() {
    let place = Place::new();
    let ada = place.made();
    let remote = "git@github.com:boomdrivedev/boom.git";
    place.repo_added(&ada, "boom", remote);
    let folder = place.repository("boom", Some(remote));
    narrative(&place, END_TO_END, &["--repository", "boom"]);
    let answer = place.json_in(&folder, &outline_args(BOOK, &[END_TO_END], &[]));
    assert_eq!(answer["repositories"], json!([]));
    assert!(place.last_signed()["attach"].is_null(), "{answer}");
}

#[test]
fn an_outline_with_a_repository() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &[]);
    place.refused(
        &outline_args(BOOK, &[END_TO_END], &["--repository", "boom"]),
        "an outline binds the whole system",
    );
}

// A store has one design book

#[test]
fn a_second_book() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &[]);
    outline(&place, BOOK, &[END_TO_END], &[]);
    place.refused(&outline_args("another-book", &[END_TO_END], &[]), BOOK);
}

#[test]
fn a_book_after_a_rejected_book() {
    let place = Place::new();
    let ada = place.made();
    narrative(&place, END_TO_END, &[]);
    let first = version(&outline(&place, BOOK, &[END_TO_END], &[]));
    let rejected = place.run(&[
        "reject",
        &format!("{BOOK}@{first}"),
        "--reason",
        "Another book.",
        "--key",
        &ada,
    ]);
    assert_eq!(rejected.code, 0, "{rejected:?}");
    outline(&place, "another-book", &[END_TO_END], &[]);
}

// An outline is judged by a person and never checked

#[test]
fn an_outline_accepted_with_its_narratives() {
    let place = Place::new();
    let ada = place.made();
    let held = narrative(&place, END_TO_END, &[]);
    let book = version(&outline(&place, BOOK, &[END_TO_END], &[]));
    let before = place.keys.signatures();
    place.json(&[
        "accept",
        &format!("{END_TO_END}@{held}"),
        &format!("{BOOK}@{book}"),
        "--key",
        &ada,
    ]);
    assert_eq!(place.keys.signatures(), before + 1);
    assert_eq!(place.status(END_TO_END, &held), "accepted");
    assert_eq!(place.status(BOOK, &book), "accepted");
}

// The parts of the design book

#[test]
fn three_levels() {
    let place = Place::new();
    store_of_three_levels(&place);
    let book = &place.show(BOOK)["book"];
    assert_eq!(names(&book["system"]["narratives"]), [LOG]);
    assert_eq!(book["system"]["sections"], json!([]));
    let chapters = &book["repositories"];
    assert_eq!(keys(chapters, "repository"), ["boom", "sluice"]);
    assert_eq!(names(&chapters[0]["narratives"]), [END_TO_END]);
    assert_eq!(chapters[1]["narratives"], json!([]));
    assert_eq!(keys(&chapters[1]["sections"], "context"), [HOST]);
    assert_eq!(names(&chapters[1]["sections"][0]["narratives"]), [QUEUE]);
}

#[test]
fn a_context_of_two_repositories() {
    let place = Place::new();
    place.made();
    context(
        &place,
        "the-handover",
        &["flume:src/handover.rs", "sluice:src/handover.rs"],
    );
    narrative(
        &place,
        "the-handover-explained",
        &["--context", "the-handover"],
    );
    outline(&place, BOOK, &["the-handover-explained"], &[]);
    let book = &place.show(BOOK)["book"];
    assert_eq!(
        keys(&book["system"]["sections"], "context"),
        ["the-handover"]
    );
    assert_eq!(
        names(&book["system"]["sections"][0]["narratives"]),
        ["the-handover-explained"]
    );
    assert_eq!(book["repositories"], json!([]));
}

#[test]
fn a_repository_with_no_narrative() {
    let place = Place::new();
    let ada = place.made();
    place.repo_added(&ada, "flume", "git@github.com:boomdrivedev/flume.git");
    narrative(&place, END_TO_END, &["--repository", "boom"]);
    outline(&place, BOOK, &[END_TO_END], &[]);
    let book = &place.show(BOOK)["book"];
    assert_eq!(keys(&book["repositories"], "repository"), ["boom"]);
}

// A narrative's level in the book

#[test]
fn a_narrative_that_changes_its_level() {
    let place = Place::new();
    let ada = place.made();
    let model = "drive-architecture-model";
    let v1 = narrative(&place, model, &["--repository", "boom"]);
    place.accepted(model, &v1, &ada);
    outline(&place, BOOK, &[model], &[]);
    let file = place.file("wider.md", "The whole drive.\n");
    let v2 = version(&place.json(&[
        "propose",
        "narrative",
        model,
        "The drive's architecture",
        "--body",
        &file,
        "--amends",
        &v1,
        "--system",
        "--provenance",
        "run-1",
    ]));
    place.accepted(model, &v2, &ada);
    let book = &place.show(BOOK)["book"];
    assert_eq!(names(&book["system"]["narratives"]), [model]);
    assert_eq!(book["system"]["narratives"][0]["version"], json!(v2));
    assert_eq!(book["repositories"], json!([]));
}

// The order of the design book

#[test]
fn the_outlines_order() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &["--repository", "boom"]);
    narrative(&place, RECORDS, &["--repository", "boom"]);
    outline(&place, BOOK, &[RECORDS, END_TO_END], &[]);
    let book = &place.show(BOOK)["book"];
    assert_eq!(
        names(&book["repositories"][0]["narratives"]),
        [RECORDS, END_TO_END]
    );
}

#[test]
fn the_order_of_the_chapters() {
    let place = Place::new();
    place.made();
    narrative(&place, END_TO_END, &["--repository", "boom"]);
    narrative(&place, FLOW, &["--repository", "sluice"]);
    outline(&place, BOOK, &[FLOW, END_TO_END], &[]);
    let book = &place.show(BOOK)["book"];
    assert_eq!(
        keys(&book["repositories"], "repository"),
        ["sluice", "boom"]
    );
}

#[test]
fn the_same_book_from_two_runs() {
    let place = Place::new();
    store_of_three_levels(&place);
    let first = place.run(&["show", BOOK, "--json"]);
    assert_eq!(first.code, 0, "{first:?}");
    let verified = place.run(&["verify"]);
    assert_eq!(verified.code, 0, "{verified:?}");
    let second = place.run(&["show", BOOK, "--json"]);
    assert_eq!(second.code, 0, "{second:?}");
    assert_eq!(first.stdout, second.stdout);
}

// A narrative that the outline does not hold

#[test]
fn a_narrative_not_in_the_outline() {
    let place = Place::new();
    store_of_three_levels(&place);
    narrative(&place, "the-design-book", &["--system"]);
    let shown = place.show(BOOK);
    assert_eq!(
        names(&shown["book"]["system"]["narratives"]),
        [LOG, "the-design-book"]
    );
    assert_eq!(shown["book"]["system"]["narratives"][1]["held"], false);
    assert_eq!(shown["unheld"], json!(["the-design-book"]));
}

#[test]
fn a_rejected_narrative() {
    let place = Place::new();
    let ada = place.made();
    narrative(&place, LOG, &["--system"]);
    let overview = narrative(&place, "the-design-book", &["--system"]);
    let rejected = place.run(&[
        "reject",
        &format!("the-design-book@{overview}"),
        "--reason",
        "A narrative names no book.",
        "--key",
        &ada,
    ]);
    assert_eq!(rejected.code, 0, "{rejected:?}");
    outline(&place, BOOK, &[LOG], &[]);
    let shown = place.show(BOOK);
    assert_eq!(names(&shown["book"]["system"]["narratives"]), [LOG]);
    assert_eq!(shown["unheld"], json!([]));
}

// boom show gives an outline

#[test]
fn a_narrative_out_of_date() {
    let place = Place::new();
    let ada = place.made();
    let v1 = narrative(&place, END_TO_END, &["--repository", "boom"]);
    place.accepted(END_TO_END, &v1, &ada);
    let book = version(&outline(&place, BOOK, &[END_TO_END], &[]));
    let file = place.file("again.md", "Again.\n");
    let v2 = version(&place.json(&[
        "propose",
        "narrative",
        END_TO_END,
        "Boom, end to end",
        "--body",
        &file,
        "--amends",
        &v1,
        "--provenance",
        "run-1",
    ]));
    place.accepted(END_TO_END, &v2, &ada);
    let outcome = place.run(&["show", BOOK]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert!(
        outcome.stdout.contains(&format!(
            "narratives: {END_TO_END}@{v1}, out of date: {END_TO_END}@{v2} is accepted"
        )),
        "{}",
        outcome.stdout
    );
    let at = place.show(&format!("{BOOK}@{book}"));
    assert_eq!(at["view"], "book");
    assert_eq!(at["narratives"][0]["out_of_date"], true);
}

#[test]
fn the_book_in_words() {
    let place = Place::new();
    store_of_three_levels(&place);
    let outcome = place.run(&["show", BOOK]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let words = &outcome.stdout;
    let version_of = |name: &str| {
        place.show(name)["newest"]["version"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let expected = format!(
        "Part I. The system\n  {LOG}@{} proposed\nPart II. The repositories\n  boom\n    {END_TO_END}@{} proposed\n  sluice\n    {HOST}\n      {QUEUE}@{} proposed\n",
        version_of(LOG),
        version_of(END_TO_END),
        version_of(QUEUE)
    );
    assert!(words.contains(&expected), "{words}");
}
