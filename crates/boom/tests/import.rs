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

const RULES: &str = "# shop rules

The rules that bind wherever the shop changes.

## Rules

### The ledger

- **[core.1]** A record's status is read from its events, never stored
  on the record.
- **[core.2]** A view is computed from the ledger and never stored as
  the truth.

## Details

- **core.1** Rules out: a status field on a record.
  Source: record 0001.
- **core.9** Rules out: nothing, since no rule is core.9.
";

const RULES_ALONE: &str = "# shop rules

## Rules

### The ledger

- **[core.1]** A record's status is read from its events, never stored
  on the record.
- **[core.2]** A view is computed from the ledger and never stored as
  the truth.
";

const CORE_1: &str = "A record's status is read from its events, never stored on the record.";
const CORE_2: &str = "A view is computed from the ledger and never stored as the truth.";

/// The page `core.md` in the folder `rules` of the place, with the
/// details page beside it where one is given.
fn rules_page(place: &Place, page: &str, details: Option<&str>) -> String {
    let mut files = vec![("core.md", page)];
    if let Some(details) = details {
        files.push(("core-details.md", details));
    }
    let dir = place.folder("rules", &files);
    format!("{dir}/core.md")
}

/// `boom import rules <page> --repository shop --dry-run --json`'s
/// answer.
fn rules_dry_run(place: &Place, page: &str) -> Value {
    place.json(&["import", "rules", page, "--repository", "shop", "--dry-run"])
}

#[test]
fn a_rules_page_read() {
    let place = Place::new();
    place.made();
    let page = rules_page(&place, RULES, None);
    let before = place.log();
    let answer = rules_dry_run(&place, &page);
    assert_eq!(answer["format"], "boom-import/1");
    let records = answer["records"].as_array().unwrap();
    assert_eq!(records.len(), 2);
    let first = record(&answer, "core.1");
    assert_eq!(first["kind"], "invariant");
    assert_eq!(first["name"], Value::Null);
    assert_eq!(first["statement"], CORE_1);
    assert_eq!(
        first["body"],
        "- **core.1** Rules out: a status field on a record.\n  Source: record 0001.\n"
    );
    assert_eq!(
        first["imported"],
        json!({
            "repository": "shop",
            "from": "core.md",
            "form": "rules",
            "legacy": "core.1",
            "status": null,
            "date": null,
            "deciders": [],
            "section": "The ledger",
            "drafted": true,
        })
    );
    let second = record(&answer, "core.2");
    assert_eq!(second["statement"], CORE_2);
    assert_eq!(second["imported"]["legacy"], "core.2");
    assert_eq!(second["imported"]["section"], "The ledger");
    assert_eq!(second["body"], Value::Null);
    assert_eq!(place.log(), before);
}

#[test]
fn details_on_the_page_beside_it() {
    let place = Place::new();
    place.made();
    let page = rules_page(
        &place,
        RULES_ALONE,
        Some(
            "# shop rules, their details\n\n- **core.1** Rules out: a status field on a record.\n",
        ),
    );
    let answer = rules_dry_run(&place, &page);
    assert_eq!(
        record(&answer, "core.1")["body"],
        "- **core.1** Rules out: a status field on a record.\n"
    );
    assert_eq!(record(&answer, "core.1")["statement"], CORE_1);
    assert_eq!(record(&answer, "core.2")["body"], Value::Null);
}

#[test]
fn a_page_with_no_rules_is_refused() {
    let place = Place::new();
    place.made();
    let page = rules_page(
        &place,
        "# shop rules\n\n## Details\n\n- **core.1** Rules out: x.\n",
        None,
    );
    place.refused(
        &[
            "import",
            "rules",
            &page,
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ],
        "core.md: has no `## Rules` section",
    );
}

#[test]
fn one_id_twice_is_refused() {
    let place = Place::new();
    place.made();
    let page = rules_page(
        &place,
        "# shop rules\n\n## Rules\n\n- **[core.1]** One rule.\n- **[core.1]** Another rule.\n",
        None,
    );
    let refused = place.refused(
        &[
            "import",
            "rules",
            &page,
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ],
        "core.md",
    );
    assert!(refused.stderr.contains("core.1"), "{refused:?}");
}

#[test]
fn what_a_draft_still_needs() {
    let place = Place::new();
    place.made();
    let page = rules_page(&place, RULES, None);
    let before = place.log();
    let answer = rules_dry_run(&place, &page);
    for source in ["core.1", "core.2"] {
        assert_eq!(
            record(&answer, source)["needs"],
            json!(["a name", "a scenario"]),
            "{source}"
        );
    }
    assert_eq!(place.log(), before);
}

const CENTS_SCENARIO: &str =
    "Given an amount of 0.30, when it is kept, then it is kept as 30 cents.";
const RECORDS_SCENARIO: &str =
    "Given a decision that shapes the shop, when it is made, then a MADR file records it.";

/// A drafts file in the place holding `records`, giving its path.
fn drafts(place: &Place, file: &str, records: Value) -> String {
    let text = json!({"format": "boom-import-drafts/1", "records": records}).to_string();
    place.file(file, &text)
}

/// The two records of the shop's folder, `0001` and `0003`.
fn two_records(place: &Place) -> String {
    adr(
        place,
        &[
            (
                "0001-record-architecture-decisions.md",
                RECORD_ARCHITECTURE_DECISIONS,
            ),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
        ],
    )
}

/// Drafts giving each of the two records a scenario.
fn two_drafts(place: &Place) -> String {
    drafts(
        place,
        "drafts.json",
        json!({
            "0001-record-architecture-decisions.md": {"scenarios": [RECORDS_SCENARIO]},
            "0003-amounts-in-cents.md": {"scenarios": [CENTS_SCENARIO]},
        }),
    )
}

/// `boom import adr <dir> --repository <repository> --drafts <drafts>
/// --provenance run-1 --json`'s answer.
fn imported(place: &Place, dir: &str, repository: &str, drafts: &str) -> Value {
    place.json(&[
        "import",
        "adr",
        dir,
        "--repository",
        repository,
        "--drafts",
        drafts,
        "--provenance",
        "run-1",
    ])
}

#[test]
fn imported_with_drafts() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = two_drafts(&place);
    let answer = imported(&place, &dir, "shop", &drafts);
    assert_eq!(answer["format"], "boom-import/1");
    let cents = record(&answer, "0003-amounts-in-cents.md");
    assert_eq!(cents["status"], "proposed");
    let version = cents["version"].as_str().unwrap();
    assert_eq!(version.len(), 12);
    let shown = place.show(&format!("amounts-in-cents@{version}"));
    assert_eq!(shown["kind"], "decision");
    assert_eq!(shown["statement"], "Amounts are kept in cents");
    assert_eq!(
        shown["body"],
        "## Context\n\nFloats round: 0.1 + 0.2 is not 0.3.\n\n## Decision\n\nEvery amount is an integer of cents.\n"
    );
    assert_eq!(shown["scenarios"], json!([CENTS_SCENARIO]));
    assert_eq!(
        shown["imported"],
        json!({
            "repository": "shop",
            "from": "0003-amounts-in-cents.md",
            "form": "madr",
            "legacy": "0003",
            "status": "accepted",
            "date": "2026-01-04",
            "deciders": ["ada"],
            "section": null,
            "drafted": true,
        })
    );
    assert_eq!(shown["history"], Value::Null);
    assert_eq!(shown["status"], "proposed");
}

#[test]
fn a_rule_named_by_its_draft() {
    let place = Place::new();
    place.made();
    let page = rules_page(&place, RULES, None);
    let drafts = drafts(
        &place,
        "drafts.json",
        json!({
            "core.1": {"name": "status-from-events", "scenarios": ["Given a record accepted, then its status is read from its events."]},
            "core.2": {"name": "views-computed", "scenarios": ["Given a view, then it is computed from the ledger."]},
        }),
    );
    let answer = place.json(&[
        "import",
        "rules",
        &page,
        "--repository",
        "shop",
        "--drafts",
        &drafts,
        "--provenance",
        "run-1",
    ]);
    let rule = record(&answer, "core.1");
    assert_eq!(rule["name"], "status-from-events");
    assert_eq!(rule["kind"], "invariant");
    assert_eq!(rule["status"], "proposed");
    let version = rule["version"].as_str().unwrap();
    let shown = place.show(&format!("status-from-events@{version}"));
    assert_eq!(shown["kind"], "invariant");
    assert_eq!(shown["statement"], CORE_1);
    assert_eq!(shown["imported"]["legacy"], "core.1");
    assert_eq!(shown["imported"]["from"], "core.md");
    assert_eq!(shown["imported"]["section"], "The ledger");
}

#[test]
fn a_draft_missing_is_refused() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = drafts(
        &place,
        "drafts.json",
        json!({"0001-record-architecture-decisions.md": {"scenarios": [RECORDS_SCENARIO]}}),
    );
    place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--drafts",
            &drafts,
            "--provenance",
            "run-1",
        ],
        "0003-amounts-in-cents.md needs a scenario",
    );
}

#[test]
fn a_draft_for_no_record_is_refused() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = drafts(
        &place,
        "drafts.json",
        json!({
            "0001-record-architecture-decisions.md": {"scenarios": [RECORDS_SCENARIO]},
            "0003-amounts-in-cents.md": {"scenarios": [CENTS_SCENARIO]},
            "0009-gone.md": {"scenarios": ["Given nothing, then nothing."]},
        }),
    );
    place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--drafts",
            &drafts,
            "--provenance",
            "run-1",
        ],
        "0009-gone.md",
    );
}

#[test]
fn an_import_proposes_and_accepts_nothing() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = two_drafts(&place);
    let before = place.log().len();
    let answer = imported(&place, &dir, "shop", &drafts);
    let log = place.log();
    assert_eq!(log.len(), before + 2);
    for entry in &log[before..] {
        assert_eq!(entry["act"], "proposed");
        assert_eq!(entry["actor"], json!({"agent": {"provenance": "run-1"}}));
        assert_eq!(entry["signature"], Value::Null);
        assert_eq!(entry["counted"], true);
    }
    assert!(log.iter().all(|entry| entry["act"] != "accepted"));
    for source in [
        "0001-record-architecture-decisions.md",
        "0003-amounts-in-cents.md",
    ] {
        let proposed = record(&answer, source);
        let name = proposed["name"].as_str().unwrap();
        let version = proposed["version"].as_str().unwrap();
        assert_eq!(place.status(name, version), "proposed");
    }
}

#[test]
fn one_record_refused_appends_none() {
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
            ("0004-one-database.md", ONE_DATABASE),
        ],
    );
    let drafts = drafts(
        &place,
        "drafts.json",
        json!({
            "0001-record-architecture-decisions.md": {"scenarios": [RECORDS_SCENARIO]},
            "0003-amounts-in-cents.md": {"scenarios": [CENTS_SCENARIO], "name": "Bad_Name"},
            "0004-one-database.md": {"scenarios": ["Given an order, then it is kept in the one database."]},
        }),
    );
    place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--drafts",
            &drafts,
            "--provenance",
            "run-1",
        ],
        "Bad_Name",
    );
}

#[test]
fn an_import_with_no_provenance_is_refused() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = two_drafts(&place);
    place.refused(
        &[
            "import",
            "adr",
            &dir,
            "--repository",
            "shop",
            "--drafts",
            &drafts,
        ],
        "an agent's proposal names its provenance",
    );
}

#[test]
fn a_superseded_record_comes_in_as_history() {
    let place = Place::new();
    place.made();
    let dir = adr(
        &place,
        &[
            ("0002-amounts-as-floats.md", AMOUNTS_AS_FLOATS),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
        ],
    );
    let drafts = drafts(
        &place,
        "drafts.json",
        json!({"0003-amounts-in-cents.md": {"scenarios": [CENTS_SCENARIO]}}),
    );
    let answer = imported(&place, &dir, "shop", &drafts);
    let floats = record(&answer, "0002-amounts-as-floats.md");
    assert_eq!(floats["name"], "amounts-as-floats");
    assert_eq!(floats["status"], "proposed");
    let version = floats["version"].as_str().unwrap();
    let shown = place.show(&format!("amounts-as-floats@{version}"));
    assert_eq!(shown["kind"], "decision");
    assert_eq!(shown["status"], "proposed");
    assert_eq!(shown["scenarios"], json!([]));
    assert_eq!(
        shown["history"],
        json!({"status": "superseded", "by": "amounts-in-cents"})
    );
    assert_eq!(
        shown["imported"]["status"],
        "superseded by [ADR-0003](0003-amounts-in-cents.md)"
    );
}

const MONEY_HAS_CURRENCY: &str = "---
status: accepted
---

# Every amount names its currency
";

/// The folder holding `0002-money-has-currency.md`.
fn money_folder(place: &Place) -> String {
    adr(place, &[("0002-money-has-currency.md", MONEY_HAS_CURRENCY)])
}

/// Drafts for `0002-money-has-currency.md`, naming it `name` where one
/// is given.
fn money_drafts(place: &Place, name: Option<&str>) -> String {
    let mut draft = json!({"scenarios": ["Given an amount, then it names its currency."]});
    if let Some(name) = name {
        draft["name"] = json!(name);
    }
    drafts(
        place,
        "money-drafts.json",
        json!({"0002-money-has-currency.md": draft}),
    )
}

/// The arguments of an import of `dir` for `repository` with `drafts`.
fn import_args<'a>(dir: &'a str, repository: &'a str, drafts: &'a str) -> Vec<&'a str> {
    vec![
        "import",
        "adr",
        dir,
        "--repository",
        repository,
        "--drafts",
        drafts,
        "--provenance",
        "run-1",
    ]
}

#[test]
fn a_name_the_store_holds_is_refused() {
    let place = Place::new();
    place.made();
    place.proposed("money-has-currency", "Every amount names its currency.");
    let dir = money_folder(&place);
    let drafts = money_drafts(&place, None);
    let refused = place.refused(
        &import_args(&dir, "shop", &drafts),
        "the invariant money-has-currency",
    );
    assert!(
        refused.stderr.contains("otherwise in its draft"),
        "{refused:?}"
    );
}

#[test]
fn named_otherwise_by_its_draft() {
    let place = Place::new();
    place.made();
    place.proposed("money-has-currency", "Every amount names its currency.");
    let dir = money_folder(&place);
    let drafts = money_drafts(&place, Some("money-carries-currency"));
    let answer = imported(&place, &dir, "shop", &drafts);
    let money = record(&answer, "0002-money-has-currency.md");
    assert_eq!(money["name"], "money-carries-currency");
    assert_eq!(money["kind"], "decision");
    let version = money["version"].as_str().unwrap();
    let shown = place.show(&format!("money-carries-currency@{version}"));
    assert_eq!(shown["kind"], "decision");
    assert_eq!(shown["status"], "proposed");
    assert_eq!(shown["imported"]["legacy"], "0002");
}

#[test]
fn a_number_the_store_holds_is_refused() {
    let place = Place::new();
    place.made();
    let shop = adr(
        &place,
        &[(
            "0001-record-architecture-decisions.md",
            RECORD_ARCHITECTURE_DECISIONS,
        )],
    );
    let shop_drafts = drafts(
        &place,
        "shop-drafts.json",
        json!({"0001-record-architecture-decisions.md": {"scenarios": [RECORDS_SCENARIO]}}),
    );
    imported(&place, &shop, "shop", &shop_drafts);
    let billing = place.folder(
        "billing-adr",
        &[(
            "0001-use-postgres.md",
            "---\nstatus: accepted\n---\n\n# Use Postgres\n",
        )],
    );
    let billing_drafts = drafts(
        &place,
        "billing-drafts.json",
        json!({"0001-use-postgres.md": {"scenarios": ["Given an invoice, then Postgres keeps it."]}}),
    );
    let refused = place.refused(
        &import_args(&billing, "billing", &billing_drafts),
        "the number 0001",
    );
    assert!(
        refused.stderr.contains("record-architecture-decisions"),
        "{refused:?}"
    );
}

#[test]
fn two_records_under_one_name_are_refused() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = drafts(
        &place,
        "drafts.json",
        json!({
            "0001-record-architecture-decisions.md": {"scenarios": [RECORDS_SCENARIO], "name": "money"},
            "0003-amounts-in-cents.md": {"scenarios": [CENTS_SCENARIO], "name": "money"},
        }),
    );
    let refused = place.refused(&import_args(&dir, "shop", &drafts), "money");
    for file in [
        "0001-record-architecture-decisions.md",
        "0003-amounts-in-cents.md",
    ] {
        assert!(refused.stderr.contains(file), "{file}: {refused:?}");
    }
}

#[test]
fn the_same_import_twice_adds_nothing() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = two_drafts(&place);
    let first = imported(&place, &dir, "shop", &drafts);
    let log = place.log();
    let second = imported(&place, &dir, "shop", &drafts);
    assert_eq!(second, first);
    assert_eq!(place.log(), log);
}

#[test]
fn a_file_edited_after_its_import_is_refused() {
    let place = Place::new();
    place.made();
    let dir = two_records(&place);
    let drafts = two_drafts(&place);
    let first = imported(&place, &dir, "shop", &drafts);
    let version = record(&first, "0003-amounts-in-cents.md")["version"]
        .as_str()
        .unwrap()
        .to_owned();
    adr(
        &place,
        &[(
            "0003-amounts-in-cents.md",
            &AMOUNTS_IN_CENTS.replace(
                "# Amounts are kept in cents",
                "# Amounts are kept in whole cents",
            ),
        )],
    );
    let refused = place.refused(
        &import_args(&dir, "shop", &drafts),
        "0003-amounts-in-cents.md",
    );
    assert!(
        refused
            .stderr
            .contains(&format!("amounts-in-cents@{version}")),
        "{refused:?}"
    );
    assert!(
        refused.stderr.contains("boom propose --amends"),
        "{refused:?}"
    );
}

#[test]
fn another_repositorys_page_of_the_same_name() {
    let place = Place::new();
    place.made();
    let shop = rules_page(&place, RULES, None);
    let shop_drafts = drafts(
        &place,
        "shop-drafts.json",
        json!({
            "core.1": {"name": "status-from-events", "scenarios": ["Given a record accepted, then its status is read from its events."]},
            "core.2": {"name": "views-computed", "scenarios": ["Given a view, then it is computed from the ledger."]},
        }),
    );
    place.json(&[
        "import",
        "rules",
        &shop,
        "--repository",
        "shop",
        "--drafts",
        &shop_drafts,
        "--provenance",
        "run-1",
    ]);
    let billing_dir = place.folder(
        "billing-rules",
        &[(
            "core.md",
            "# billing rules\n\n## Rules\n\n- **[core.1]** Every invoice is billed once.\n",
        )],
    );
    let billing = format!("{billing_dir}/core.md");
    let billing_drafts = drafts(
        &place,
        "billing-drafts.json",
        json!({"core.1": {"name": "invoices-billed-once", "scenarios": ["Given an invoice billed, then it is not billed again."]}}),
    );
    let answer = place.json(&[
        "import",
        "rules",
        &billing,
        "--repository",
        "billing",
        "--drafts",
        &billing_drafts,
        "--provenance",
        "run-2",
    ]);
    let rule = record(&answer, "core.1");
    assert_eq!(rule["name"], "invoices-billed-once");
    assert_eq!(rule["status"], "proposed");
    let version = rule["version"].as_str().unwrap();
    let shown = place.show(&format!("invoices-billed-once@{version}"));
    assert_eq!(shown["imported"]["repository"], "billing");
    assert_eq!(shown["imported"]["legacy"], "core.1");
    assert_eq!(shown["statement"], "Every invoice is billed once.");
}

/// Runs the accept an import's answer gives, with `key`, giving its
/// answer.
fn accept_given(place: &Place, answer: &Value, key: &str) -> Value {
    let versions: Vec<&str> = answer["accept"]["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|version| version.as_str().unwrap())
        .collect();
    let mut args = vec!["accept"];
    args.extend(&versions);
    args.extend(["--key", key]);
    place.json(&args)
}

#[test]
fn an_import_accepted_together() {
    let place = Place::new();
    let ada = place.made();
    let dir = adr(
        &place,
        &[
            (
                "0001-record-architecture-decisions.md",
                RECORD_ARCHITECTURE_DECISIONS,
            ),
            ("0002-amounts-as-floats.md", AMOUNTS_AS_FLOATS),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
        ],
    );
    let drafts = two_drafts(&place);
    let mut args = import_args(&dir, "shop", &drafts);
    let words = place.run(&args);
    assert_eq!(words.code, 0, "{words:?}");
    args.push("--json");
    let outcome = place.run(&args);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    let versions = answer["accept"]["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 3);
    let command = answer["accept"]["command"].as_str().unwrap();
    assert!(command.starts_with("boom accept "), "{command}");
    assert!(command.ends_with(" --key <key>"), "{command}");
    for version in versions {
        assert!(command.contains(version.as_str().unwrap()), "{command}");
    }
    assert!(
        words.stdout.trim_end().ends_with(command),
        "the command is printed last: {words:?}"
    );
    let before = place.log().len();
    accept_given(&place, &answer, &ada);
    for (source, status) in [
        ("0001-record-architecture-decisions.md", "standing"),
        ("0002-amounts-as-floats.md", "history"),
        ("0003-amounts-in-cents.md", "standing"),
    ] {
        let found = record(&answer, source);
        let name = found["name"].as_str().unwrap();
        let version = found["version"].as_str().unwrap();
        assert_eq!(place.status(name, version), status, "{source}");
    }
    assert_eq!(place.show("amounts-as-floats")["standing"], Value::Null);
    let log = place.log();
    assert_eq!(log.len(), before + 3);
    let ada_public = place.keys.public_key(std::path::Path::new(&ada));
    for entry in &log[before..] {
        assert_eq!(entry["act"], "accepted");
        assert_eq!(entry["actor"], json!({"person": "ada"}));
        assert_eq!(
            boom::sign::verify(
                &ada_public,
                entry["signed"].as_str().unwrap().as_bytes(),
                entry["signature"].as_str().unwrap()
            ),
            Ok(())
        );
    }
}

#[test]
fn an_imported_record_stands_for_its_repository_alone() {
    let place = Place::new();
    let ada = place.made();
    let dir = two_records(&place);
    let drafts = two_drafts(&place);
    let answer = imported(&place, &dir, "shop", &drafts);
    let version = record(&answer, "0003-amounts-in-cents.md")["version"]
        .as_str()
        .unwrap()
        .to_owned();
    accept_given(&place, &answer, &ada);
    assert_eq!(place.last_signed()["act"], "accepted");
    let shown = place.show(&format!("amounts-in-cents@{version}"));
    assert_eq!(shown["status"], "standing");
    assert_eq!(shown["repositories"], json!(["shop"]));
    let names = |repository: &str| -> Vec<String> {
        place.json(&["query", "--repository", repository])["claims"]
            .as_array()
            .unwrap()
            .iter()
            .map(|claim| claim["name"].as_str().unwrap().to_owned())
            .collect()
    };
    let shop = names("shop");
    assert!(shop.contains(&"amounts-in-cents".to_owned()), "{shop:?}");
    let billing = names("billing");
    assert!(billing.is_empty(), "{billing:?}");
}

#[test]
fn a_record_left_proposed_waits() {
    let place = Place::new();
    let ada = place.made();
    let dir = adr(
        &place,
        &[
            (
                "0001-record-architecture-decisions.md",
                RECORD_ARCHITECTURE_DECISIONS,
            ),
            (
                "0004-one-database.md",
                "---\nstatus: proposed\n---\n\n# One database\n\nThe shop keeps one database.\n",
            ),
        ],
    );
    let drafts = drafts(
        &place,
        "drafts.json",
        json!({
            "0001-record-architecture-decisions.md": {"scenarios": [RECORDS_SCENARIO]},
            "0004-one-database.md": {"scenarios": ["Given an order, then it is kept in the one database."]},
        }),
    );
    let answer = imported(&place, &dir, "shop", &drafts);
    let one = record(&answer, "0004-one-database.md");
    let waiting = format!("one-database@{}", one["version"].as_str().unwrap());
    let versions = answer["accept"]["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 1);
    assert!(versions.iter().all(|version| version != &json!(waiting)));
    accept_given(&place, &answer, &ada);
    assert_eq!(
        place.status("one-database", one["version"].as_str().unwrap()),
        "proposed"
    );
    let first = record(&answer, "0001-record-architecture-decisions.md");
    assert_eq!(
        place.status(
            "record-architecture-decisions",
            first["version"].as_str().unwrap()
        ),
        "standing"
    );
}
