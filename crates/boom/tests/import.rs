//! Importing a repository's decision records and rules: each read from
//! its file, proposed in its own words with what its agent drafts, held
//! to one name and one number, read once, and accepted together
//! (record-import). Every file a test imports is written by the test,
//! from a string here, into its own folder (core.5; design 10).

mod support;

use serde_json::{json, Value};
use support::Place;

const RECORD_ARCHITECTURE_DECISIONS: &str = "---
number: 1
title: Record architecture decisions
status: accepted
date: 2026-01-02
decision-makers:
- ada
- bo
---

# Record architecture decisions

We record each decision that shapes the shop as a MADR file.
";

const AMOUNTS_IN_CENTS: &str = "---
status: accepted
date: 2026-01-04
decision-makers: ada
---

# Amounts are kept in cents

## Context

Floats round: 0.1 + 0.2 is not 0.3.

## Decision

Every amount is an integer of cents.
";

const AMOUNTS_AS_FLOATS: &str = "---
status: superseded by [ADR-0003](0003-amounts-in-cents.md)
date: 2026-01-03
decision-makers:
- ada
---

# Amounts are floats

Amounts are kept as floats.
";

const ONE_DATABASE: &str = "# One database

The shop keeps one database.
";

const README: &str = "# Decisions

The shop's decision records.
";

/// The folder `adr` of the place, holding these files.
fn adr(place: &Place, files: &[(&str, &str)]) -> String {
    place.folder("adr", files)
}

/// `boom import adr <dir> --repository shop --dry-run --json`'s answer.
fn dry_run(place: &Place, dir: &str) -> Value {
    place.json(&["import", "adr", dir, "--repository", "shop", "--dry-run"])
}

/// The record of an import's answer whose source is `source`.
fn record<'a>(answer: &'a Value, source: &str) -> &'a Value {
    answer["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["source"] == source)
        .unwrap_or_else(|| panic!("no record {source}: {answer}"))
}

#[test]
fn a_folder_of_madr_files_read() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[
            (
                "0001-record-architecture-decisions.md",
                RECORD_ARCHITECTURE_DECISIONS,
            ),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
            ("README.md", README),
        ],
    );
    let before = place.log();
    let answer = dry_run(&place, &dir);
    assert_eq!(answer["format"], "boom-import/1");
    assert_eq!(answer["repository"], "shop");
    assert_eq!(answer["records"].as_array().unwrap().len(), 2);
    let first = record(&answer, "0001-record-architecture-decisions.md");
    assert_eq!(first["kind"], "decision");
    assert_eq!(first["name"], "record-architecture-decisions");
    assert_eq!(first["statement"], "Record architecture decisions");
    assert_eq!(
        first["body"],
        "We record each decision that shapes the shop as a MADR file.\n"
    );
    assert_eq!(first["version"], Value::Null);
    assert_eq!(
        first["imported"],
        json!({
            "repository": "shop",
            "from": "0001-record-architecture-decisions.md",
            "form": "madr",
            "legacy": "0001",
            "status": "accepted",
            "date": "2026-01-02",
            "deciders": ["ada", "bo"],
            "section": null,
            "drafted": true,
        })
    );
    assert_eq!(first["history"], Value::Null);
    assert_eq!(first["needs"], json!(["a scenario"]));
    let cents = record(&answer, "0003-amounts-in-cents.md");
    assert_eq!(cents["name"], "amounts-in-cents");
    assert_eq!(cents["statement"], "Amounts are kept in cents");
    assert_eq!(cents["imported"]["legacy"], "0003");
    assert_eq!(cents["imported"]["status"], "accepted");
    assert_eq!(cents["imported"]["date"], "2026-01-04");
    assert_eq!(cents["imported"]["deciders"], json!(["ada"]));
    assert!(cents["body"]
        .as_str()
        .unwrap()
        .starts_with("## Context\n\nFloats round"));
    let skipped = answer["skipped"].as_array().unwrap();
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0]["file"], "README.md");
    assert_eq!(place.log(), before);
}

#[test]
fn a_record_named_in_its_frontmatter() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[(
            "the-rename.md",
            "---\nname: the-rename\nstatus: accepted\n---\n\n# The shop is renamed\n\nIt was the store.\n",
        )],
    );
    let answer = dry_run(&place, &dir);
    let rename = record(&answer, "the-rename.md");
    assert_eq!(rename["kind"], "decision");
    assert_eq!(rename["name"], "the-rename");
    assert_eq!(rename["imported"]["legacy"], Value::Null);
    assert_eq!(rename["statement"], "The shop is renamed");
    assert!(answer["skipped"].as_array().unwrap().is_empty());
}

#[test]
fn a_record_with_no_title_is_refused() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[
            ("0001-record-architecture-decisions.md", RECORD_ARCHITECTURE_DECISIONS),
            (
                "0003-amounts-in-cents.md",
                "---\nstatus: accepted\n---\n\nAmounts are kept in cents.\n\n```\n# not a title\n```\n",
            ),
        ],
    );
    place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ],
        "0003-amounts-in-cents.md: holds no title",
    );
}

#[test]
fn two_numbers_that_differ_are_refused() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[(
            "0003-amounts-in-cents.md",
            "---\nnumber: 4\nstatus: accepted\n---\n\n# Amounts are kept in cents\n",
        )],
    );
    let refused = place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ],
        "0003-amounts-in-cents.md",
    );
    for number in ["0003", "0004"] {
        assert!(refused.stderr.contains(number), "{number}: {refused:?}");
    }
}

#[test]
fn a_successor_by_its_number() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[
            (
                "0002-amounts-as-floats.md",
                "---\nstatus: superseded by 0003\n---\n\n# Amounts are floats\n",
            ),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
        ],
    );
    let answer = dry_run(&place, &dir);
    let floats = record(&answer, "0002-amounts-as-floats.md");
    assert_eq!(
        floats["history"],
        json!({"status": "superseded", "by": "amounts-in-cents"})
    );
    assert_eq!(floats["imported"]["status"], "superseded by 0003");
    assert_eq!(floats["needs"], json!([]));
    let linked = adr(&place, &[("0002-amounts-as-floats.md", AMOUNTS_AS_FLOATS)]);
    let answer = dry_run(&place, &linked);
    assert_eq!(
        record(&answer, "0002-amounts-as-floats.md")["history"]["by"],
        "amounts-in-cents"
    );
}

#[test]
fn a_successor_not_in_the_folder_is_refused() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[
            (
                "0002-amounts-as-floats.md",
                "---\nstatus: superseded by 0009\n---\n\n# Amounts are floats\n",
            ),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
        ],
    );
    let refused = place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ],
        "0002-amounts-as-floats.md",
    );
    assert!(refused.stderr.contains("superseded by 0009"), "{refused:?}");
}

#[test]
fn a_status_boom_does_not_know_is_refused() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[(
            "0003-amounts-in-cents.md",
            "---\nstatus: draft\n---\n\n# Amounts are kept in cents\n",
        )],
    );
    let refused = place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ],
        "0003-amounts-in-cents.md",
    );
    for named in [
        "draft",
        "accepted",
        "proposed",
        "superseded",
        "deprecated",
        "rejected",
    ] {
        assert!(refused.stderr.contains(named), "{named}: {refused:?}");
    }
}

#[test]
fn a_record_with_no_status_is_read_as_accepted() {
    let place = Place::new();
    place.made();
    let dir = adr(&place, &[("0004-one-database.md", ONE_DATABASE)]);
    let answer = dry_run(&place, &dir);
    let one = record(&answer, "0004-one-database.md");
    assert_eq!(one["kind"], "decision");
    assert_eq!(one["name"], "one-database");
    assert_eq!(one["statement"], "One database");
    assert_eq!(one["imported"]["status"], Value::Null);
    assert_eq!(one["imported"]["legacy"], "0004");
    assert_eq!(one["imported"]["deciders"], json!([]));
    assert_eq!(one["history"], Value::Null);
    assert_eq!(one["needs"], json!(["a scenario"]));
}

#[test]
fn an_import_with_no_repository_is_refused() {
    let place = Place::new();
    place.made();
    let dir = adr(&place, &[("0004-one-database.md", ONE_DATABASE)]);
    place.refused(
        &["import", "adr", &dir, "--provenance", "run-1"],
        "an import names its repository",
    );
}
