//! A narrative: a claim of the kind `narrative`, with a title and a body
//! of Markdown text, the claims it includes at the version each meant,
//! and the paragraphs of its body; a person accepts it, and no one checks
//! it (narratives; claim-versions).

mod support;

use serde_json::{json, Value};
use support::{Place, NARRATIVE};

const NAME: &str = "boom-end-to-end";
const TITLE: &str = "Boom, end to end";
const STATUS: &str = "status-read-from-events";
const STATUS_WORDS: &str = "A record never holds its status.";

fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// The claim of the log's last entry, as its signed part holds it.
fn last_claim(place: &Place) -> Value {
    let log = place.log();
    let signed: Value =
        serde_json::from_str(log.last().unwrap()["signed"].as_str().unwrap()).unwrap();
    signed["claim"].clone()
}

/// The paragraphs `boom show <name>@<version> --json` gives, as
/// `(number, kind)`.
fn paragraphs(place: &Place, target: &str) -> Vec<(u64, String)> {
    place.show(target)["paragraphs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["number"].as_u64().unwrap(),
                p["kind"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn a_narrative_proposed() {
    let place = Place::new();
    place.made();
    let answer = place.narrative(NAME, TITLE, NARRATIVE, &["--repository", "boom"]);
    assert_eq!(answer["kind"], "narrative");
    let shown = place.show(NAME);
    assert_eq!(shown["kind"], "narrative");
    assert_eq!(shown["versions"].as_array().unwrap().len(), 1);
    assert_eq!(shown["newest"]["status"], "proposed");
    assert_eq!(shown["newest"]["statement"], TITLE);
    assert_eq!(shown["newest"]["body"], NARRATIVE);
    assert_eq!(shown["newest"]["repositories"], json!(["boom"]));
}

#[test]
fn a_narrative_with_its_title_and_body() {
    let place = Place::new();
    place.made();
    place.narrative(NAME, TITLE, NARRATIVE, &[]);
    assert_eq!(place.show(NAME)["kind"], "narrative");
}

#[test]
fn a_narrative_with_no_body() {
    let place = Place::new();
    place.made();
    place.refused(
        &["propose", "narrative", NAME, TITLE, "--provenance", "run-1"],
        "a narrative needs a body",
    );
}

#[test]
fn a_body_that_is_not_text() {
    let place = Place::new();
    place.made();
    let file = place.dir.path().join("narrative.md");
    std::fs::write(&file, [0x23, 0x20, 0xff, 0xfe, 0x0a]).unwrap();
    let file = support::path_arg(&file);
    place.refused(
        &[
            "propose",
            "narrative",
            NAME,
            TITLE,
            "--body",
            &file,
            "--provenance",
            "run-1",
        ],
        "the body of a narrative is UTF-8 text",
    );
}

#[test]
fn a_name_stores_its_accepted_version() {
    let place = Place::new();
    let ada = place.made();
    let status = place.proposed(STATUS, STATUS_WORDS);
    place.accepted(STATUS, &status, &ada);
    place.proposed(STATUS, STATUS_WORDS);
    let answer = place.narrative(NAME, TITLE, NARRATIVE, &["--includes", STATUS]);
    assert_eq!(answer["includes"], json!([format!("{STATUS}@{status}")]));
    assert_eq!(
        last_claim(&place)["includes"],
        json!([format!("{STATUS}@{status}")])
    );
}

#[test]
fn a_claim_that_the_store_does_not_hold() {
    let place = Place::new();
    place.made();
    let file = place.file("narrative.md", NARRATIVE);
    place.refused(
        &[
            "propose",
            "narrative",
            NAME,
            TITLE,
            "--body",
            &file,
            "--includes",
            "record-0005",
            "--provenance",
            "run-1",
        ],
        "record-0005",
    );
}

#[test]
fn includes_on_a_decision() {
    let place = Place::new();
    place.made();
    place.proposed(STATUS, STATUS_WORDS);
    let body = place.decision_body();
    place.refused(
        &[
            "propose",
            "decision",
            "amounts-in-cents",
            "Amounts are in cents.",
            "--body",
            &body,
            "--includes",
            STATUS,
            "--provenance",
            "run-1",
        ],
        "only a narrative includes claims",
    );
}

#[test]
fn a_narrative_included() {
    let place = Place::new();
    place.made();
    place.narrative(NAME, TITLE, NARRATIVE, &[]);
    let file = place.file("other.md", "Another narrative.\n");
    place.refused(
        &[
            "propose",
            "narrative",
            "boom-for-a-reader",
            "Boom, for a reader",
            "--body",
            &file,
            "--includes",
            NAME,
            "--provenance",
            "run-1",
        ],
        "a narrative includes claims of other kinds",
    );
}

#[test]
fn a_heading_text_and_a_list() {
    let place = Place::new();
    place.made();
    let body =
        "# What Boom is\n\nBoom keeps claims.\n\n- An agent proposes.\n- A person accepts.\n";
    let answer = place.narrative(NAME, TITLE, body, &[]);
    let target = format!("{NAME}@{}", version(&answer));
    assert_eq!(
        paragraphs(&place, &target),
        [
            (1, "heading".to_owned()),
            (2, "text".to_owned()),
            (3, "item".to_owned()),
            (4, "item".to_owned())
        ]
    );
}

#[test]
fn a_nested_list() {
    let place = Place::new();
    place.made();
    let body = "- One item\n  - a\n  - b\n  - c\n- Two\n";
    let answer = place.narrative(NAME, TITLE, body, &[]);
    let target = format!("{NAME}@{}", version(&answer));
    assert_eq!(
        paragraphs(&place, &target),
        [(1, "item".to_owned()), (2, "item".to_owned())]
    );
}

#[test]
fn a_thematic_break() {
    let place = Place::new();
    place.made();
    let body = "Boom keeps claims.\n\n---\n\nA person accepts them.\n";
    let answer = place.narrative(NAME, TITLE, body, &[]);
    let target = format!("{NAME}@{}", version(&answer));
    assert_eq!(
        paragraphs(&place, &target),
        [(1, "text".to_owned()), (2, "text".to_owned())]
    );
}

#[test]
fn a_paragraphs_fingerprint() {
    let place = Place::new();
    place.made();
    let answer = place.narrative(NAME, TITLE, NARRATIVE, &[]);
    let shown = place.show(&format!("{NAME}@{}", version(&answer)));
    let second = &shown["paragraphs"][1];
    assert_eq!(second["number"], 2);
    assert_eq!(
        second["fingerprint"],
        boom::sha256_hex(b"Boom keeps claims.")[..12]
    );
}

/// narratives, "A diagram in a body is its source".
#[test]
fn a_diagram_is_a_paragraph() {
    let place = Place::new();
    place.made();
    let block = "```mermaid\nflowchart LR\n  a --> b\n```";
    let body = format!("# The steps\n\n{block}\n");
    let answer = place.narrative(NAME, TITLE, &body, &[]);
    let shown = place.show(&format!("{NAME}@{}", version(&answer)));
    let paragraphs = shown["paragraphs"].as_array().unwrap();
    let kinds: Vec<&str> = paragraphs
        .iter()
        .map(|p| p["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["heading", "code"], "{shown}");
    assert_eq!(
        paragraphs[1]["fingerprint"],
        boom::sha256_hex(block.as_bytes())[..12]
    );
}

#[test]
fn a_design_accepted_with_one_touch() {
    let place = Place::new();
    let ada = place.made();
    let status = place.proposed(STATUS, STATUS_WORDS);
    let appended = place.proposed("events-are-appended", "boom appends an event.");
    let narrative = version(&place.narrative(
        NAME,
        TITLE,
        NARRATIVE,
        &["--includes", STATUS, "--includes", "events-are-appended"],
    ));
    let before = place.keys.signatures();
    let answer = place.json(&[
        "accept",
        &format!("{STATUS}@{status}"),
        &format!("events-are-appended@{appended}"),
        &format!("{NAME}@{narrative}"),
        "--key",
        &ada,
    ]);
    assert_eq!(
        place.keys.signatures(),
        before + 1,
        "the key signs one time"
    );
    assert_eq!(answer["accepted"].as_array().unwrap().len(), 3);
    for (name, version) in [
        (STATUS, &status),
        ("events-are-appended", &appended),
        (NAME, &narrative),
    ] {
        assert_eq!(place.status(name, version), "accepted", "{name}");
    }
}

#[test]
fn an_include_out_of_date() {
    let place = Place::new();
    let ada = place.made();
    let first = place.proposed(STATUS, STATUS_WORDS);
    place.accepted(STATUS, &first, &ada);
    place.narrative(NAME, TITLE, NARRATIVE, &["--includes", STATUS]);
    let second = place.amended(STATUS, "A record never holds its own status.", &first);
    place.accepted(STATUS, &second, &ada);
    let shown = place.show(NAME);
    let include = &shown["newest"]["includes"][0];
    assert_eq!(include["claim"], STATUS);
    assert_eq!(include["version"], first);
    assert_eq!(include["out_of_date"], true);
    assert_eq!(include["accepted"], second);
    let words = place.run(&["show", NAME]).stdout;
    assert!(
        words.contains(&format!(
            "includes: {STATUS}@{first}, out of date: {STATUS}@{second} is accepted"
        )),
        "{words}"
    );
}

#[test]
fn an_open_note_shown() {
    let place = Place::new();
    place.made_by("chuck");
    let answer = place.narrative(NAME, TITLE, NARRATIVE, &[]);
    let target = format!("{NAME}@{}", version(&answer));
    let note = place.noted(&target, 3, "Say what a drive is.");
    let shown = place.show(&target);
    assert_eq!(
        shown["notes"],
        json!([{
            "note": note["note"],
            "version": version(&answer),
            "paragraph": 3,
            "fingerprint": note["fingerprint"],
            "person": "chuck",
            "words": "Say what a drive is.",
            "at": 1_763_424_000u64,
            "entry": note["entry"],
            "status": "open",
            "answer": null,
        }])
    );
    let words = place.run(&["show", &target]).stdout;
    assert!(words.contains("on paragraph 3"), "{words}");
    assert!(words.contains("  open\n"), "{words}");
}

#[test]
fn a_narrative_is_found_by_its_words_and_queried_with_its_body() {
    let place = Place::new();
    let ada = place.made();
    let answer = place.narrative(NAME, TITLE, NARRATIVE, &["--repository", "boom"]);
    place.accepted(NAME, &version(&answer), &ada);
    let found = place.json(&["search", "proposes", "--repository", "boom"]);
    assert_eq!(found["claims"][0]["name"], NAME);
    let query = place.json(&["query", "--repository", "boom"]);
    assert_eq!(query["claims"][0]["kind"], "narrative");
    assert_eq!(
        query["claims"][0]["body"],
        boom::sha256_hex(NARRATIVE.as_bytes())
    );
}

// A narrative binds one level

#[test]
fn a_narrative_of_one_repository() {
    let place = Place::new();
    place.made();
    place.narrative(NAME, TITLE, NARRATIVE, &["--repository", "boom"]);
}

/// The arguments of a proposal of the narrative with `extra`.
fn narrative_args<'a>(file: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["propose", "narrative", NAME, TITLE, "--body", file];
    args.extend(extra);
    args.extend(["--provenance", "run-1"]);
    args
}

#[test]
fn a_narrative_of_two_repositories() {
    let place = Place::new();
    place.made();
    let file = place.file("narrative.md", NARRATIVE);
    place.refused(
        &narrative_args(&file, &["--repository", "boom", "--repository", "sluice"]),
        "a narrative binds one level",
    );
}

#[test]
fn a_narrative_of_a_repository_and_a_context() {
    let place = Place::new();
    place.made();
    place.json(&[
        "propose",
        "context",
        "sluice-host-process",
        "The one process of sluice on a host.",
        "--path",
        "sluice:crates/sluice/src/host_process/**",
        "--provenance",
        "run-1",
    ]);
    let file = place.file("narrative.md", NARRATIVE);
    place.refused(
        &narrative_args(
            &file,
            &["--repository", "sluice", "--context", "sluice-host-process"],
        ),
        "a narrative binds one level",
    );
}

#[test]
fn an_amendment_keeps_one_level() {
    let place = Place::new();
    place.made();
    let v1 = version(&place.narrative(NAME, TITLE, NARRATIVE, &["--repository", "boom"]));
    let file = place.file("again.md", "Again.\n");
    let answer = place.json(&narrative_args(&file, &["--amends", &v1]));
    assert_eq!(answer["repositories"], json!(["boom"]));
}
