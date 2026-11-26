//! A person's note on a paragraph of a version of a narrative: an entry
//! with the person's name and no signature, open until a later version
//! of the narrative answers it (narrative-notes).

mod support;

use boom::entry::{Act, Actor, Entry};
use serde_json::{json, Value};
use support::{Place, NARRATIVE};

const NAME: &str = "boom-end-to-end";
const TITLE: &str = "Boom, end to end";
const DRIVE: &str = "Say what a drive is.";

fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// A store whose one person is chuck, with the narrative `boom-end-to-end`
/// proposed, giving chuck's key and its version.
fn one_narrative(place: &Place) -> (String, String) {
    let chuck = place.made_by("chuck");
    let answer = place.narrative(NAME, TITLE, NARRATIVE, &[]);
    (chuck, version(&answer))
}

/// The amendment of `base` with `extra`, as `boom propose` answers it.
fn amend(place: &Place, base: &str, extra: &[&str]) -> boom::cli::Outcome {
    let file = place.file(
        "second.md",
        &format!("{NARRATIVE}\nA drive is a store with its runner.\n"),
    );
    let mut args = vec![
        "propose",
        "narrative",
        NAME,
        TITLE,
        "--body",
        &file,
        "--amends",
        base,
        "--provenance",
        "run-2",
    ];
    args.extend(extra);
    place.run(&args)
}

#[test]
fn a_note_written() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let target = format!("{NAME}@{v1}");
    let outcome = place.run(&["note", &target, "--paragraph", "3", DRIVE]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["act"], "noted");
    assert_eq!(last["actor"], json!({"person": "chuck"}));
    assert_eq!(last["signature"], Value::Null);
    assert_eq!(last["counted"], true);
    let signed: Value = serde_json::from_str(last["signed"].as_str().unwrap()).unwrap();
    assert_eq!(signed["name"], NAME);
    assert_eq!(signed["version"], v1);
    assert_eq!(signed["paragraph"], 3);
    assert_eq!(signed["words"], DRIVE);
}

#[test]
fn two_persons_and_no_person() {
    let place = Place::new();
    let ada = place.made();
    place.registered(&ada, "bo", 2);
    let v1 = version(&place.narrative(NAME, TITLE, NARRATIVE, &[]));
    place.refused(
        &["note", &format!("{NAME}@{v1}"), "--paragraph", "1", DRIVE],
        "--person names the person",
    );
    let outcome = place.run(&[
        "note",
        &format!("{NAME}@{v1}"),
        "--paragraph",
        "1",
        DRIVE,
        "--person",
        "bo",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
}

#[test]
fn a_person_not_registered() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    place.refused(
        &[
            "note",
            &format!("{NAME}@{v1}"),
            "--paragraph",
            "1",
            DRIVE,
            "--person",
            "cy",
        ],
        "cy is not registered",
    );
}

#[test]
fn a_note_on_an_invariant() {
    let place = Place::new();
    place.made_by("chuck");
    let status = place.proposed(
        "status-read-from-events",
        "A record never holds its status.",
    );
    place.refused(
        &[
            "note",
            &format!("status-read-from-events@{status}"),
            "--paragraph",
            "1",
            DRIVE,
        ],
        "a note is on a narrative",
    );
}

#[test]
fn a_paragraph_the_version_does_not_have() {
    let place = Place::new();
    place.made_by("chuck");
    let body: String = (1..=12).map(|n| format!("Paragraph {n}.\n\n")).collect();
    let v1 = version(&place.narrative(NAME, TITLE, &body, &[]));
    place.refused(
        &["note", &format!("{NAME}@{v1}"), "--paragraph", "40", DRIVE],
        "has 12 paragraphs",
    );
}

#[test]
fn an_agents_note() {
    let place = Place::new();
    let (chuck, v1) = one_narrative(&place);
    let target = format!("{NAME}@{v1}");
    place.refused(
        &[
            "note",
            &target,
            "--paragraph",
            "1",
            DRIVE,
            "--provenance",
            "run-1",
        ],
        "only a person writes a note",
    );
    place.refused(
        &["note", &target, "--paragraph", "1", DRIVE, "--key", &chuck],
        "no signature",
    );
    place.refused(
        &["note", &target, "--paragraph", "1", "  "],
        "a note holds words",
    );
}

#[test]
fn the_same_note_twice() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let target = format!("{NAME}@{v1}");
    let first = place.noted(&target, 3, DRIVE);
    let second = place.noted(&target, 3, DRIVE);
    assert_eq!(first["note"], second["note"]);
    assert_eq!(first["entry"], second["entry"]);
    assert_eq!(place.acts().iter().filter(|act| *act == "noted").count(), 1);
}

#[test]
fn an_unsigned_accept() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    place.append_by_hand(&support::accept_entry("chuck", NAME, &v1), None);
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["counted"], false);
    assert_eq!(place.status(NAME, &v1), "proposed");
}

#[test]
fn a_note_from_an_agent() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let fingerprint = boom::sha256_hex(b"Boom keeps claims.")[..12].to_owned();
    place.append_by_hand(
        &Entry {
            at: 1_763_424_500,
            rules: boom::judge::RULES,
            actor: Actor::Agent {
                provenance: "run-1".to_owned(),
            },
            act: Act::Noted {
                name: NAME.to_owned(),
                version: v1.clone(),
                paragraph: 2,
                fingerprint,
                words: DRIVE.to_owned(),
            },
        },
        None,
    );
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["counted"], false);
    assert!(
        last["reason"]
            .as_str()
            .unwrap()
            .contains("only a person writes a note"),
        "{last}"
    );
    assert_eq!(place.show(NAME)["notes"], json!([]));
}

#[test]
fn a_note_on_a_paragraph_of_another_fingerprint() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    place.append_by_hand(
        &Entry {
            at: 1_763_424_500,
            rules: boom::judge::RULES,
            actor: Actor::Person("chuck".to_owned()),
            act: Act::Noted {
                name: NAME.to_owned(),
                version: v1,
                paragraph: 2,
                fingerprint: "000000000000".to_owned(),
                words: DRIVE.to_owned(),
            },
        },
        None,
    );
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["counted"], false, "{last}");
    // A replay of the whole log judges it the same way.
    place.json(&["verify"]);
    assert_eq!(place.show(NAME)["notes"], json!([]));
}

#[test]
fn an_amendment_answers_the_note() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let note = place.noted(&format!("{NAME}@{v1}"), 3, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let answer = format!("{note}=Section 10 now says it.");
    let outcome = amend(&place, &v1, &["--answers", &answer, "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let v2 = version(&serde_json::from_str(&outcome.stdout).unwrap());
    let shown = place.show(NAME);
    assert_eq!(shown["notes"][0]["note"], note);
    assert_eq!(shown["notes"][0]["status"], "answered");
    assert_eq!(
        shown["notes"][0]["answer"],
        json!({"version": v2, "words": "Section 10 now says it."})
    );
    let words = place.run(&["show", NAME]).stdout;
    assert!(
        words.contains(&format!("answered by {NAME}@{v2}: Section 10 now says it.")),
        "{words}"
    );
    // The answer is part of the version's content.
    assert_eq!(
        place.show(&format!("{NAME}@{v2}"))["answers"],
        json!([{"note": note, "words": "Section 10 now says it."}])
    );
}

#[test]
fn the_notes_on_an_old_draft() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let note = place.noted(&format!("{NAME}@{v1}"), 3, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let answer = format!("{note}=Section 10 now says it.");
    let outcome = amend(&place, &v1, &["--answers", &answer, "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let v2 = version(&serde_json::from_str(&outcome.stdout).unwrap());
    // The draft that the note is on is superseded, and the new draft
    // answers its note.
    assert_eq!(place.show(&format!("{NAME}@{v1}"))["status"], "superseded");
    let shown = place.show(NAME);
    assert_eq!(shown["notes"][0]["status"], "answered");
    assert_eq!(shown["notes"][0]["answer"]["version"], v2.as_str());
}

#[test]
fn an_amendment_that_leaves_a_note_open() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let note = place.noted(&format!("{NAME}@{v1}"), 3, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = place.log();
    let outcome = amend(&place, &v1, &[]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    for words in [note.as_str(), "paragraph 3", DRIVE] {
        assert!(outcome.stderr.contains(words), "{words}: {outcome:?}");
    }
    assert_eq!(place.log(), before);
}

#[test]
fn the_answer_rejected() {
    let place = Place::new();
    let (chuck, v1) = one_narrative(&place);
    let note = place.noted(&format!("{NAME}@{v1}"), 3, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let outcome = amend(
        &place,
        &v1,
        &["--answers", &format!("{note}=It does now."), "--json"],
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let v2 = version(&serde_json::from_str(&outcome.stdout).unwrap());
    assert_eq!(place.show(NAME)["notes"][0]["status"], "answered");
    place.json(&[
        "reject",
        &format!("{NAME}@{v2}"),
        "--reason",
        "The answer does not say it.",
        "--key",
        &chuck,
    ]);
    let shown = place.show(NAME);
    assert_eq!(shown["notes"][0]["status"], "open");
    assert_eq!(shown["notes"][0]["answer"], Value::Null);
}

#[test]
fn the_answer_withdrawn() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let note = place.noted(&format!("{NAME}@{v1}"), 3, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let outcome = amend(
        &place,
        &v1,
        &["--answers", &format!("{note}=It does now."), "--json"],
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let v2 = version(&serde_json::from_str(&outcome.stdout).unwrap());
    assert_eq!(place.show(NAME)["notes"][0]["status"], "answered");
    place.json(&[
        "withdraw",
        &format!("{NAME}@{v2}"),
        "--reason",
        "The answer does not say it.",
        "--provenance",
        "run-2",
    ]);
    let shown = place.show(NAME);
    assert_eq!(shown["notes"][0]["status"], "open");
    assert_eq!(shown["notes"][0]["answer"], Value::Null);
}

#[test]
fn a_note_of_another_narrative() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let other =
        version(&place.narrative("boom-for-a-reader", "Boom, for a reader", NARRATIVE, &[]));
    let note = place.noted(&format!("boom-for-a-reader@{other}"), 2, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = place.log();
    let outcome = amend(&place, &v1, &["--answers", &format!("{note}=Not here.")]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(outcome.stderr.contains(&note), "{outcome:?}");
    assert!(outcome.stderr.contains("boom-for-a-reader"), "{outcome:?}");
    assert_eq!(place.log(), before);
}

#[test]
fn an_answer_to_a_note_that_does_not_exist() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let outcome = amend(&place, &v1, &["--answers", "0123456789ab=No such note."]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome.stderr.contains("no note has the name 0123456789ab"),
        "{outcome:?}"
    );
}

#[test]
fn a_note_answered_as_json() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    let answer = place.noted(&format!("{NAME}@{v1}"), 1, DRIVE);
    assert_eq!(answer["format"], "boom-note/1");
    assert_eq!(answer["note"].as_str().unwrap().len(), 12);
    assert_eq!(
        answer["note"],
        boom::narrative::note_name(NAME, &v1, 1, DRIVE, "chuck")
    );
}

const BOOK: &str = "elaboration:boom/the-design-book";

/// The elaboration `boom/<name>` as a note gives it.
fn elaboration(name: &str) -> Value {
    json!({
        "repository": "boom",
        "name": name,
        "reference": format!("elaboration:boom/{name}"),
    })
}

/// The narrative `name`, proposed from `origin`, giving its version.
fn from(place: &Place, name: &str, origin: &str) -> String {
    version(&place.narrative(name, TITLE, NARRATIVE, &["--origin", origin]))
}

#[test]
fn a_note_reaches_the_elaboration_of_its_version() {
    let place = Place::new();
    place.made_by("chuck");
    let v1 = from(&place, NAME, BOOK);
    place.noted(&format!("{NAME}@{v1}"), 3, DRIVE);
    let shown = place.show(NAME);
    assert_eq!(
        shown["notes"][0]["elaboration"],
        elaboration("the-design-book")
    );
    assert_eq!(shown["notes"][0]["narrative"], NAME);
    let at_version = place.show(&format!("{NAME}@{v1}"));
    assert_eq!(
        at_version["notes"][0]["elaboration"],
        elaboration("the-design-book")
    );
    let words = place.run(&["show", NAME]);
    assert!(
        words
            .stdout
            .contains(&format!("{DRIVE}\n  elaboration: {BOOK}\n  open\n")),
        "{words:?}"
    );
}

#[test]
fn a_note_on_a_version_of_no_elaboration() {
    let place = Place::new();
    let (_, v1) = one_narrative(&place);
    place.noted(&format!("{NAME}@{v1}"), 3, DRIVE);
    let shown = place.show(NAME);
    assert_eq!(shown["notes"][0]["elaboration"], Value::Null);
    let words = place.run(&["show", NAME]);
    assert!(!words.stdout.contains("elaboration:"), "{words:?}");
}

#[test]
fn each_note_keeps_the_elaboration_of_its_version() {
    let place = Place::new();
    place.made_by("chuck");
    let v1 = from(&place, NAME, "elaboration:boom/one");
    let first = place.noted(&format!("{NAME}@{v1}"), 3, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let answers = format!("{first}=Section 10 now says it.");
    let v2 = amend(
        &place,
        &v1,
        &["--origin", "elaboration:boom/two", "--answers", &answers],
    );
    assert_eq!(v2.code, 0, "{v2:?}");
    let v2 = version(&place.show(NAME)["newest"]);
    place.noted(&format!("{NAME}@{v2}"), 2, "Say what a store is.");
    let notes = place.show(NAME)["notes"].clone();
    assert_eq!(notes[0]["version"], v1);
    assert_eq!(notes[0]["elaboration"], elaboration("one"));
    assert_eq!(notes[1]["version"], v2);
    assert_eq!(notes[1]["elaboration"], elaboration("two"));
}

#[test]
fn the_open_notes_of_an_elaboration() {
    let place = Place::new();
    place.made_by("chuck");
    let open = from(&place, NAME, BOOK);
    let open_note = place.noted(&format!("{NAME}@{open}"), 3, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let answered = from(&place, "change-records", BOOK);
    let answered_note = place.noted(&format!("change-records@{answered}"), 2, "Say more.")["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let file = place.file("change-records-2.md", &format!("{NARRATIVE}\nMore.\n"));
    let answers = format!("{answered_note}=It says more.");
    let outcome = place.run(&[
        "propose",
        "narrative",
        "change-records",
        TITLE,
        "--body",
        &file,
        "--amends",
        &answered,
        "--origin",
        BOOK,
        "--answers",
        &answers,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let other = from(&place, "the-lineage", "elaboration:boom/other");
    place.noted(&format!("the-lineage@{other}"), 2, "Not this one.");
    let entries = place.log().len();
    let answer = place.json(&["query", "--elaboration", "boom/the-design-book"]);
    assert_eq!(answer["format"], "boom-query/2");
    assert_eq!(answer["claims"], json!([]));
    assert_eq!(answer["elaboration"], elaboration("the-design-book"));
    let notes = answer["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 1, "{answer}");
    assert_eq!(notes[0]["note"], open_note.as_str());
    assert_eq!(notes[0]["narrative"], NAME);
    assert_eq!(notes[0]["version"], open);
    assert_eq!(notes[0]["paragraph"], 3);
    assert_eq!(notes[0]["person"], "chuck");
    assert_eq!(notes[0]["words"], DRIVE);
    assert_eq!(notes[0]["status"], "open");
    assert_eq!(notes[0]["elaboration"], elaboration("the-design-book"));
    let words = place.run(&["query", "--elaboration", "boom/the-design-book"]);
    assert!(
        words
            .stdout
            .starts_with(&format!("Boom: 1 open note of {BOOK}, at entry ")),
        "{words:?}"
    );
    assert_eq!(place.log().len(), entries);
}

#[test]
fn an_elaboration_with_no_open_note() {
    let place = Place::new();
    place.made_by("chuck");
    let answer = place.json(&["query", "--elaboration", "boom/nothing-yet"]);
    assert_eq!(answer["notes"], json!([]));
    assert_eq!(answer["repository"], Value::Null);
}

#[test]
fn an_elaboration_query_in_its_form() {
    let place = Place::new();
    place.made_by("chuck");
    for (extra, words) in [
        (
            vec!["--repository", "boom"],
            "boom query takes --elaboration or --repository, and not both",
        ),
        (
            vec!["--path", "boom:README.md"],
            "boom query takes --elaboration or --path, and not both",
        ),
        (
            vec!["--if-declared"],
            "boom query takes --elaboration or --if-declared, and not both",
        ),
    ] {
        let mut args = vec!["query", "--elaboration", "boom/the-design-book"];
        args.extend(extra);
        place.refused(&args, words);
    }
    place.refused(
        &["query", "--elaboration", "the-design-book"],
        "an elaboration is <repository>/<name>",
    );
}
