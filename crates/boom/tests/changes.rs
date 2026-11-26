//! Each version of a change record, which an agent records and no person
//! accepts (change-records).

mod support;

use boom::entry::{Act, Actor, Change, ChangeFile, Entry};
use support::Place;

const STATUS: &str = "A record never holds its status.";
const RECORD: &str = "boom:contexts-own-paths";

/// A store with `status-read-from-events` accepted and a plan approved
/// that covers it, and a change's folder in the working folder.
struct Recordable {
    ada: String,
    version: String,
    approval: String,
}

fn recordable(place: &Place) -> Recordable {
    let ada = place.made();
    let version = place.proposed("status-read-from-events", STATUS);
    place.accepted("status-read-from-events", &version, &ada);
    let plan = place.file("plan.md", "# The plan\n");
    place.json(&[
        "approve",
        &plan,
        "--covers",
        "status-read-from-events",
        "--key",
        &ada,
    ]);
    let approval = place.log().last().unwrap()["act_hash"]
        .as_str()
        .unwrap()
        .to_owned();
    folder(place, "Contexts own paths.\n");
    Recordable {
        ada,
        version,
        approval,
    }
}

/// Writes the change's folder, its design holding `design`.
fn folder(place: &Place, design: &str) {
    let root = place.dir.path().join("openspec/changes/contexts-own-paths");
    std::fs::create_dir_all(root.join("specs/change-records")).unwrap();
    std::fs::write(root.join("proposal.md"), "## Why\n").unwrap();
    std::fs::write(root.join("design.md"), design).unwrap();
    std::fs::write(root.join("tasks.md"), "- [ ] 1.1 Build it\n").unwrap();
    std::fs::write(
        root.join("specs/change-records/spec.md"),
        "## ADDED Requirements\n",
    )
    .unwrap();
}

fn recorded(place: &Place, given: &Recordable, more: &[&str]) -> serde_json::Value {
    let mut args = vec![
        "record",
        "change",
        RECORD,
        "--approval",
        &given.approval,
        "--unit",
        "contexts",
        "--bolt",
        "b-12",
        "--provenance",
        "run-1",
    ];
    args.extend(more);
    place.json(&args)
}

#[test]
fn a_first_version_is_recorded_and_written() {
    let place = Place::new();
    let given = recordable(&place);
    let answer = recorded(&place, &given, &[]);
    assert_eq!(answer["format"], "boom-record-change/1");
    assert_eq!(answer["base"], serde_json::Value::Null);
    let log = place.log();
    let entry = log.last().unwrap();
    assert_eq!(entry["act"], "recorded");
    assert_eq!(entry["links"], serde_json::json!([given.approval]));
    assert_eq!(
        entry["actor"],
        serde_json::json!({"agent": {"provenance": "run-1"}})
    );
    let shown = place.json(&["show", RECORD]);
    assert_eq!(shown["format"], "boom-change/1");
    assert_eq!(shown["status"], "written");
    assert_eq!(shown["versions"][0]["version"], answer["version"]);
    assert_eq!(shown["versions"][0]["unit"], "contexts");
    assert_eq!(shown["versions"][0]["bolt"], "b-12");
}

#[test]
fn no_person_accepts_a_change_record() {
    let place = Place::new();
    let given = recordable(&place);
    recorded(&place, &given, &[]);
    place.refused(
        &["accept", RECORD, "--key", &given.ada],
        "no person accepts a change record",
    );
}

#[test]
fn a_person_entry_of_a_change_record_counts_for_nothing() {
    let place = Place::new();
    let given = recordable(&place);
    let answer = recorded(&place, &given, &[]);
    let log = place.log();
    let mut entry: Entry = Entry::parse(log.last().unwrap()["signed"].as_str().unwrap()).unwrap();
    entry.actor = Actor::Person("ada".to_owned());
    entry.at += 1;
    if let Act::Recorded { change, .. } = &mut entry.act {
        change.unit = "other".to_owned();
        change.base = Some(answer["version"].as_str().unwrap().to_owned());
        change.version = change.fingerprint();
    }
    place.append_by_hand(&entry, None);
    let log = place.log();
    assert_eq!(log.last().unwrap()["counted"], false);
    assert!(
        log.last().unwrap()["reason"]
            .as_str()
            .unwrap()
            .contains("no person records or accepts a change record"),
        "{}",
        log.last().unwrap()
    );
}

#[test]
fn a_change_record_names_its_provenance() {
    let place = Place::new();
    let given = recordable(&place);
    place.refused(
        &[
            "record",
            "change",
            RECORD,
            "--approval",
            &given.approval,
            "--unit",
            "contexts",
            "--bolt",
            "b-12",
        ],
        "a change record names its provenance",
    );
}

#[test]
fn a_version_needs_its_approval_its_unit_and_its_bolt() {
    let place = Place::new();
    let given = recordable(&place);
    for (left_out, words) in [
        ("--approval", "names its approval"),
        ("--unit", "names its unit"),
        ("--bolt", "names its bolt"),
    ] {
        let mut args = vec!["record", "change", RECORD];
        for (flag, value) in [
            ("--approval", given.approval.as_str()),
            ("--unit", "contexts"),
            ("--bolt", "b-12"),
        ] {
            if flag != left_out {
                args.extend([flag, value]);
            }
        }
        args.extend(["--provenance", "run-1"]);
        place.refused(&args, words);
    }
}

#[test]
fn each_file_is_kept_by_its_fingerprint() {
    let place = Place::new();
    let given = recordable(&place);
    recorded(&place, &given, &[]);
    let shown = place.json(&["show", RECORD]);
    let files = shown["versions"][0]["files"].as_array().unwrap();
    let paths: Vec<&str> = files
        .iter()
        .map(|file| file["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "design.md",
            "proposal.md",
            "specs/change-records/spec.md",
            "tasks.md"
        ]
    );
    let tasks = files
        .iter()
        .find(|file| file["path"] == "tasks.md")
        .unwrap();
    assert_eq!(
        tasks["sha256"].as_str().unwrap(),
        boom::sha256_hex(b"- [ ] 1.1 Build it\n")
    );
    let outcome = place.run(&["show", "--artifact", tasks["sha256"].as_str().unwrap()]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout, "- [ ] 1.1 Build it\n");
}

#[test]
fn an_empty_folder_is_refused() {
    let place = Place::new();
    let given = recordable(&place);
    let empty = place.empty_folder("empty");
    let empty = support::path_arg(&empty);
    place.refused(
        &[
            "record",
            "change",
            RECORD,
            "--approval",
            &given.approval,
            "--unit",
            "contexts",
            "--bolt",
            "b-12",
            "--folder",
            &empty,
            "--provenance",
            "run-1",
        ],
        "holds no file",
    );
}

#[test]
fn a_symbolic_link_and_a_large_file_are_refused() {
    let place = Place::new();
    let given = recordable(&place);
    let root = place.dir.path().join("openspec/changes/contexts-own-paths");
    std::os::unix::fs::symlink(root.join("design.md"), root.join("linked.md")).unwrap();
    let args = [
        "record",
        "change",
        RECORD,
        "--approval",
        &given.approval,
        "--unit",
        "contexts",
        "--bolt",
        "b-12",
        "--provenance",
        "run-1",
    ];
    place.refused(&args, "symbolic link linked.md");
    std::fs::remove_file(root.join("linked.md")).unwrap();
    std::fs::write(root.join("big.bin"), vec![0_u8; 1024 * 1024 + 1]).unwrap();
    place.refused(&args, "big.bin is more than 1 MiB");
}

#[test]
fn a_rebase_keeps_the_version() {
    let place = Place::new();
    let given = recordable(&place);
    let first = recorded(
        &place,
        &given,
        &[
            "--origin",
            &format!(
                "git:boom@{}:openspec/changes/contexts-own-paths",
                "1".repeat(40)
            ),
        ],
    );
    let entries = place.log().len();
    let second = recorded(
        &place,
        &given,
        &[
            "--origin",
            &format!(
                "git:boom@{}:openspec/changes/contexts-own-paths",
                "2".repeat(40)
            ),
        ],
    );
    assert_eq!(first["version"], second["version"]);
    assert_eq!(second["repeats"], true);
    assert_eq!(place.log().len(), entries);
}

#[test]
fn a_file_changed_gives_a_new_version_that_amends_the_one_before() {
    let place = Place::new();
    let given = recordable(&place);
    let first = recorded(&place, &given, &[]);
    let v1 = first["version"].as_str().unwrap();
    folder(&place, "Contexts own paths, and globs.\n");
    let second = recorded(&place, &given, &["--amends", v1]);
    assert_ne!(second["version"], first["version"]);
    assert_eq!(second["base"], v1);
    let shown = place.json(&["show", RECORD]);
    assert_eq!(shown["versions"].as_array().unwrap().len(), 2);
    assert_eq!(shown["newest"], second["version"]);
}

#[test]
fn a_second_version_with_no_base_is_refused() {
    let place = Place::new();
    let given = recordable(&place);
    let first = recorded(&place, &given, &[]);
    folder(&place, "Contexts own paths, and globs.\n");
    place.refused(
        &[
            "record",
            "change",
            RECORD,
            "--approval",
            &given.approval,
            "--unit",
            "contexts",
            "--bolt",
            "b-12",
            "--provenance",
            "run-1",
        ],
        &format!("--amends {}", first["version"].as_str().unwrap()),
    );
}

#[test]
fn a_first_version_with_a_base_is_refused() {
    let place = Place::new();
    let given = recordable(&place);
    place.refused(
        &[
            "record",
            "change",
            RECORD,
            "--approval",
            &given.approval,
            "--unit",
            "contexts",
            "--bolt",
            "b-12",
            "--amends",
            "000000000000",
            "--provenance",
            "run-1",
        ],
        "has no version to amend",
    );
}

/// Appends a landing that links the `recorded` entry of `version`.
fn landed(place: &Place, version: &str) -> serde_json::Value {
    let shown = place.json(&["show", &format!("{RECORD}@{version}")]);
    let hash = shown["versions"][0]["act_hash"]
        .as_str()
        .unwrap()
        .to_owned();
    place.json(&[
        "append",
        "sluice.landed",
        "--link",
        &hash,
        "--provenance",
        "sluice host deneb",
    ])
}

#[test]
fn a_landing_marks_the_version_it_links() {
    let place = Place::new();
    let given = recordable(&place);
    let first = recorded(&place, &given, &[]);
    let v1 = first["version"].as_str().unwrap();
    folder(&place, "Contexts own paths, and globs.\n");
    let second = recorded(&place, &given, &["--amends", v1]);
    let v2 = second["version"].as_str().unwrap();
    landed(&place, v2);
    let shown = place.json(&["show", RECORD]);
    assert_eq!(shown["status"], "landed");
    assert_eq!(shown["landed"], v2);
}

#[test]
fn a_landed_change_never_changes() {
    let place = Place::new();
    let given = recordable(&place);
    let first = recorded(&place, &given, &[]);
    let v1 = first["version"].as_str().unwrap();
    landed(&place, v1);
    folder(&place, "Contexts own paths, and globs.\n");
    place.refused(
        &[
            "record",
            "change",
            RECORD,
            "--approval",
            &given.approval,
            "--unit",
            "contexts",
            "--bolt",
            "b-12",
            "--amends",
            v1,
            "--provenance",
            "run-1",
        ],
        "a landed change never changes",
    );
}

#[test]
fn an_approval_that_is_not_an_approval_is_refused() {
    let place = Place::new();
    let given = recordable(&place);
    let landing = place.json(&["append", "sluice.landed", "--provenance", "run-1"]);
    let hash = landing["act_hash"].as_str().unwrap();
    place.refused(
        &[
            "record",
            "change",
            RECORD,
            "--approval",
            hash,
            "--unit",
            "contexts",
            "--bolt",
            "b-12",
            "--provenance",
            "run-1",
        ],
        &format!("{hash} is not an approved entry"),
    );
    let _ = given;
}

#[test]
fn a_claim_the_approval_does_not_cover_is_refused() {
    let place = Place::new();
    let given = recordable(&place);
    let other = place.proposed("money-has-currency", "Every amount names its currency.");
    place.accepted("money-has-currency", &other, &given.ada);
    for covered in [
        "money-has-currency".to_owned(),
        format!("money-has-currency@{other}"),
    ] {
        place.refused(
            &[
                "record",
                "change",
                RECORD,
                "--approval",
                &given.approval,
                "--unit",
                "contexts",
                "--bolt",
                "b-12",
                "--covers",
                &covered,
                "--provenance",
                "run-1",
            ],
            "is a claim that the approval",
        );
    }
}

#[test]
fn a_name_covers_the_version_the_approval_covers_and_shows_it_out_of_date() {
    let place = Place::new();
    let given = recordable(&place);
    let v2 = place.amended(
        "status-read-from-events",
        "A record never holds its status; boom reads it.",
        &given.version,
    );
    place.accepted("status-read-from-events", &v2, &given.ada);
    recorded(&place, &given, &["--covers", "status-read-from-events"]);
    let shown = place.json(&["show", RECORD]);
    let covered = &shown["versions"][0]["covers"][0];
    assert_eq!(covered["claim"], "status-read-from-events");
    assert_eq!(covered["version"], given.version);
    assert_eq!(covered["out_of_date"], true);
    assert_eq!(covered["accepted"], v2);
    let words = place.run(&["show", RECORD]).stdout;
    assert!(
        words.contains(&format!(
            "covers: status-read-from-events@{}, out of date: status-read-from-events@{v2} is accepted",
            given.version
        )),
        "{words}"
    );
}

#[test]
fn a_change_record_the_store_does_not_hold_is_refused() {
    let place = Place::new();
    recordable(&place);
    let outcome = place.run(&["show", "boom:no-such-change"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome
            .stderr
            .contains("no change record has the name boom:no-such-change"),
        "{outcome:?}"
    );
}

#[test]
fn the_line_of_a_version_names_the_change_record() {
    let place = Place::new();
    let given = recordable(&place);
    let answer = recorded(&place, &given, &[]);
    let entry = answer["entry"].as_u64().unwrap();
    let outcome = place.run(&["log"]);
    let line = outcome
        .stdout
        .lines()
        .find(|line| line.starts_with(&format!("{entry} ")))
        .unwrap();
    assert!(
        line.starts_with(&format!(
            "{entry} recorded {RECORD}@{} by an agent",
            answer["version"].as_str().unwrap()
        )),
        "{line}"
    );
}

#[test]
fn a_version_whose_name_is_not_its_fingerprint_counts_for_nothing() {
    let place = Place::new();
    let given = recordable(&place);
    let mut change = Change {
        repository: "boom".to_owned(),
        name: "contexts-own-paths".to_owned(),
        version: "000000000000".to_owned(),
        base: None,
        approval: given.approval.clone(),
        unit: "contexts".to_owned(),
        bolt: "b-12".to_owned(),
        covers: Vec::new(),
        files: vec![ChangeFile {
            path: "proposal.md".to_owned(),
            sha256: "b".repeat(64),
        }],
    };
    place.append_by_hand(
        &Entry {
            at: 1_763_424_900,
            rules: boom::judge::RULES,
            actor: Actor::Agent {
                provenance: "run-1".to_owned(),
            },
            act: Act::Recorded {
                change: Box::new(change.clone()),
                origin: None,
            },
        },
        None,
    );
    change.version = change.fingerprint();
    let log = place.log();
    assert_eq!(log.last().unwrap()["counted"], false);
    assert!(
        log.last().unwrap()["reason"]
            .as_str()
            .unwrap()
            .contains(&change.version),
        "{}",
        log.last().unwrap()
    );
}

#[test]
fn the_views_of_the_changes_are_built_again() {
    let place = Place::new();
    let given = recordable(&place);
    let first = recorded(&place, &given, &[]);
    landed(&place, first["version"].as_str().unwrap());
    let before = place.json(&["show", RECORD]);
    place.sqlite().execute_batch("DROP TABLE changes;").unwrap();
    place.json(&["verify"]);
    assert_eq!(place.json(&["show", RECORD]), before);
}

#[test]
fn an_agent_records_a_version_through_its_tool() {
    let place = Place::new();
    let given = recordable(&place);
    let result = place.call(
        "record_change",
        serde_json::json!({
            "target": RECORD,
            "approval": given.approval,
            "unit": "contexts",
            "bolt": "b-12",
            "provenance": "run-1",
        }),
    );
    assert_ne!(result["isError"], true, "{result}");
    let answer = support::text(&result);
    assert_eq!(answer["format"], "boom-record-change/1");
    assert_eq!(answer["change"], RECORD);
    assert_eq!(place.json(&["show", RECORD])["status"], "written");
}
