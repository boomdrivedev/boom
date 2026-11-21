//! Terms: a word and its meaning in one repository, the old words it
//! replaces kept in the log alone, and `boom terms check`, run on a file,
//! on the text that an agent writes, and by the gate on the files that a
//! change touches (term-claims, term-check).

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};
use support::{path_arg, Place};

const OWNER: &str = "A person who grants roles and registers keys in a store.";

/// An agent's proposal of a term with `extra`, giving its answer.
fn term(place: &Place, word: &str, meaning: &str, extra: &[&str]) -> boom::cli::Outcome {
    let mut args = vec!["propose", "term", word, meaning, "--provenance", "run-1"];
    args.extend_from_slice(extra);
    place.run(&args)
}

/// The version of a term proposed with `extra`, asserting it answers.
fn proposed(place: &Place, word: &str, meaning: &str, extra: &[&str]) -> String {
    let outcome = term(place, word, meaning, &[extra, &["--json"]].concat());
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    answer["version"].as_str().unwrap().to_owned()
}

/// A store whose owner ada accepted the term `owner` of `boom`, which
/// replaces `keeper` and `warden`; gives ada's key and the version. // boom: allow keeper
fn owner_accepted(place: &Place) -> (String, String) {
    let ada = place.made();
    let version = proposed(
        place,
        "owner",
        OWNER,
        &[
            "--repository",
            "boom",
            "--replaces",
            "keeper", // boom: allow keeper
            "--replaces",
            "Warden",
        ],
    );
    place.accepted("owner", &version, &ada);
    (ada, version)
}

/// `boom terms check` with `args` on the place's store, with `input` as
/// its standard input.
fn check(place: &Place, args: &[&str], input: &str) -> boom::cli::Outcome {
    let store = place.store_arg();
    let mut all = vec!["terms", "check"];
    all.extend_from_slice(args);
    all.extend(["--store", &store]);
    boom::cli::run_with_input(all, &place.keys, &place.env, &mut input.as_bytes())
}

/// The log's entries and the store's view tables, as the file holds
/// them.
fn written(place: &Place) -> Vec<String> {
    let mut rows: Vec<String> = place.log().iter().map(Value::to_string).collect();
    let conn = place.sqlite();
    for table in ["views_head", "claims", "versions", "judgments"] {
        let mut statement = conn.prepare(&format!("SELECT * FROM {table}")).unwrap();
        let columns = statement.column_count();
        let found: Vec<String> = statement
            .query_map([], |row| {
                Ok((0..columns)
                    .map(|at| format!("{:?}", row.get_ref(at).unwrap()))
                    .collect::<Vec<_>>()
                    .join("|"))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows.extend(found);
    }
    rows
}

#[test]
fn a_term_proposed() {
    let place = Place::new();
    place.made();
    let outcome = term(
        &place,
        "owner",
        OWNER,
        &["--repository", "boom", "--replaces", "keeper"], // boom: allow keeper
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let shown = place.show("owner");
    assert_eq!(shown["kind"], "term");
    assert_eq!(shown["repository"], "boom");
    assert_eq!(shown["versions"].as_array().unwrap().len(), 1);
    assert_eq!(shown["newest"]["status"], "proposed");
    assert_eq!(shown["newest"]["repositories"], json!(["boom"]));
    assert_eq!(shown["newest"]["replaces"], json!(["keeper"])); // boom: allow keeper
    let signed = place.last_signed();
    assert_eq!(signed["claim"]["kind"], "term");
    assert_eq!(signed["claim"]["replaces"], json!(["keeper"])); // boom: allow keeper
}

#[test]
fn a_term_binds_one_repository() {
    let place = Place::new();
    place.made();
    // No repository named, in a folder of no registered repository.
    place.refused(
        &["propose", "term", "owner", OWNER, "--provenance", "run-1"],
        "a term binds one repository",
    );
    place.refused(
        &[
            "propose",
            "term",
            "owner",
            OWNER,
            "--repository",
            "boom",
            "--repository",
            "sluice",
            "--provenance",
            "run-1",
        ],
        "a term binds one repository",
    );
    place.refused(
        &[
            "propose",
            "term",
            "owner",
            OWNER,
            "--system",
            "--provenance",
            "run-1",
        ],
        "a term binds one repository",
    );
}

#[test]
fn a_term_with_no_body_and_no_replaces() {
    let place = Place::new();
    place.made();
    proposed(
        &place,
        "operator",
        "The person who runs a host.",
        &["--repository", "boomdrive"],
    );
    let signed = place.last_signed();
    assert!(signed["claim"].get("replaces").is_none(), "{signed}");
    assert_eq!(signed["claim"]["body"], Value::Null);
}

#[test]
fn only_a_term_replaces_words() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "invariant",
            "money-has-currency",
            "Every amount names its currency.",
            "--replaces",
            "keeper", // boom: allow keeper
            "--provenance",
            "run-1",
        ],
        "only a term replaces words",
    );
}

#[test]
fn a_term_does_not_replace_its_own_word() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "term",
            "owner",
            OWNER,
            "--repository",
            "boom",
            "--replaces",
            "Owner",
            "--provenance",
            "run-1",
        ],
        "a term does not replace its own word",
    );
    place.refused(
        &[
            "propose",
            "term",
            "run-log",
            "The log of a run.",
            "--repository",
            "boom",
            "--replaces",
            "Run  Log",
            "--provenance",
            "run-1",
        ],
        "a term does not replace its own word",
    );
}

#[test]
fn the_same_words_in_another_order_give_one_version() {
    let place = Place::new();
    place.made();
    let first = proposed(
        &place,
        "owner",
        OWNER,
        &[
            "--repository",
            "boom",
            "--replaces",
            "keeper", // boom: allow keeper
            "--replaces",
            "Warden",
        ],
    );
    let second = proposed(
        &place,
        "owner",
        OWNER,
        &[
            "--repository",
            "boom",
            "--replaces",
            "warden",
            "--replaces",
            "keeper", // boom: allow keeper
        ],
    );
    assert_eq!(first, second);
    assert_eq!(
        place.show(&format!("owner@{first}"))["replaces"],
        json!(["keeper", "warden"]) // boom: allow keeper
    );
    assert_eq!(place.acts(), ["registered", "proposed"]);
    // One more old word is another version.
    let third = proposed(
        &place,
        "owner",
        OWNER,
        &[
            "--repository",
            "boom",
            "--replaces",
            "keeper", // boom: allow keeper
            "--replaces",
            "warden",
            "--replaces",
            "steward",
        ],
    );
    assert_ne!(first, third);
}

#[test]
fn one_word_in_two_repositories() {
    let place = Place::new();
    let (ada, in_boom) = owner_accepted(&place);
    // The same meaning in another repository is another version.
    let in_sluice = proposed(
        &place,
        "owner",
        OWNER,
        &["--repository", "sluice", "--replaces", "keeper"], // boom: allow keeper
    );
    assert_ne!(in_boom, in_sluice);
    let other = proposed(
        &place,
        "owner",
        "The person who owns a bolt.",
        &["--repository", "sluice"],
    );
    let both = place.show("owner");
    let terms = both["terms"].as_array().unwrap();
    assert_eq!(terms.len(), 2, "{both}");
    assert_eq!(terms[0]["repository"], "boom");
    assert_eq!(terms[0]["accepted"], json!(in_boom));
    assert_eq!(terms[1]["repository"], "sluice");
    assert_eq!(terms[1]["versions"].as_array().unwrap().len(), 2);
    let one = place.json(&["show", "owner", "--repository", "sluice"]);
    assert_eq!(one["repository"], "sluice");
    assert_eq!(one["newest"]["version"], json!(other));
    place.refused(
        &["show", "owner", "--repository", "ledger"],
        "owner has no term for the repository ledger",
    );
    // Each version is found by its name and version, whatever its
    // repository.
    let accepted = place.accepted("owner", &other, &ada);
    assert_eq!(accepted["status"], "accepted");
    assert_eq!(
        place.status("owner", &in_boom),
        "accepted",
        "the term of boom stays"
    );
    // An amendment of the term of sluice binds sluice.
    let amended = place.json(&[
        "propose",
        "term",
        "owner",
        "The person who owns a bolt and its gate.",
        "--amends",
        &other,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(amended["repositories"], json!(["sluice"]));
    // A term of boom does not amend the term of sluice.
    place.refused(
        &[
            "propose",
            "term",
            "owner",
            "Another meaning.",
            "--amends",
            &other,
            "--repository",
            "boom",
            "--provenance",
            "run-3",
        ],
        "a term amends a version of its own repository",
    );
}

#[test]
fn one_name_space_for_the_other_kinds() {
    let place = Place::new();
    place.made();
    place.proposed("billing", "Every bill names its order.");
    place.refused(
        &[
            "propose",
            "term",
            "billing",
            "The act of sending a bill.",
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ],
        "the name billing is taken by the invariant billing",
    );
    proposed(&place, "owner", OWNER, &["--repository", "boom"]);
    place.refused(
        &[
            "propose",
            "invariant",
            "owner",
            "An owner signs.",
            "--provenance",
            "run-1",
        ],
        "the name owner is taken by the term owner",
    );
}

#[test]
fn the_old_words_stay_in_the_log() {
    let place = Place::new();
    let (ada, version) = owner_accepted(&place);
    let money = place.proposed("money-has-currency", "Every amount names its currency.");
    place.accepted("money-has-currency", &money, &ada);
    // Shown to the person who accepts.
    assert_eq!(
        place.show(&format!("owner@{version}"))["replaces"],
        json!(["keeper", "warden"]) // boom: allow keeper
    );
    let words = place.run(&["show", &format!("owner@{version}")]);
    assert!(
        words.stdout.contains("replaces: keeper, warden"), // boom: allow keeper
        "{words:?}"
    );
    // Given in the log.
    assert!(place
        .log()
        .iter()
        .any(|entry| entry["signed"].as_str().unwrap().contains("keeper"))); // boom: allow keeper
                                                                             // Not given to an agent, in JSON or in words.
    let query = place.run(&["query", "--repository", "boom", "--json"]);
    assert_eq!(query.code, 0, "{query:?}");
    assert!(!query.stdout.contains("keeper"), "{query:?}"); // boom: allow keeper
    let answer: Value = serde_json::from_str(&query.stdout).unwrap();
    let names: Vec<&str> = answer["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| claim["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["money-has-currency", "owner"]);
    assert_eq!(answer["claims"][1]["kind"], "term");
    assert_eq!(answer["claims"][1]["statement"], OWNER);
    let lines = place.run(&["query", "--repository", "boom"]);
    assert!(!lines.stdout.contains("keeper"), "{lines:?}"); // boom: allow keeper
    let order: Vec<&str> = lines.stdout.lines().collect();
    assert_eq!(order[0], format!("Boom: 2 claims for boom, at entry {}", 5));
    assert!(order[1].starts_with("invariant money-has-currency@"));
    assert_eq!(order[2], format!("term owner@{version}: {OWNER}"));
    // Not searched.
    let found = place.json(&["search", "keeper", "--repository", "boom"]); // boom: allow keeper
    assert_eq!(found["total"], 0, "{found}");
    let found = place.json(&["search", "grants", "--repository", "boom"]);
    assert_eq!(found["total"], 1, "{found}");
    assert!(!found.to_string().contains("keeper")); // boom: allow keeper
}

#[test]
fn a_term_of_another_repository_is_not_given() {
    let place = Place::new();
    owner_accepted(&place);
    let answer = place.json(&["query", "--repository", "sluice"]);
    assert_eq!(answer["claims"], json!([]));
}

#[test]
fn a_version_from_before_terms_keeps_its_fingerprint() {
    let place = Place::new();
    let ada = place.made();
    let money = place.proposed("money-has-currency", "Every amount names its currency.");
    place.accepted("money-has-currency", &money, &ada);
    // The views are built again from the whole log: each version keeps
    // the fingerprint its entry names.
    place
        .sqlite()
        .execute_batch("DROP TABLE views_head")
        .unwrap();
    let verified = place.json(&["verify"]);
    assert_eq!(verified["chain"], "holds");
    assert_eq!(place.status("money-has-currency", &money), "accepted");
    let signed = place.log()[1]["signed"].as_str().unwrap().to_owned();
    assert!(signed.contains(&format!("\"version\":\"{money}\"")));
    assert!(!signed.contains("replaces"));
}

// boom terms check

#[test]
fn an_old_word_found() {
    let place = Place::new();
    owner_accepted(&place);
    let file = place.file("notes.txt", "one\ntwo\nThe keeper signs.\n"); // boom: allow keeper
    let before = written(&place);
    let outcome = check(&place, &[&file, "--repository", "boom"], "");
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(outcome.stdout, "");
    assert!(outcome.stderr.contains(&format!("{file}:3")), "{outcome:?}");
    assert!(outcome.stderr.contains("\"keeper\""), "{outcome:?}"); // boom: allow keeper
    assert!(outcome.stderr.contains("owner in boom"), "{outcome:?}");
    assert_eq!(written(&place), before, "the check writes nothing");
}

#[test]
fn an_old_word_found_as_json() {
    let place = Place::new();
    let (_, version) = owner_accepted(&place);
    let file = place.file("notes.txt", "one\ntwo\nThe keeper signs.\n"); // boom: allow keeper
    let outcome = check(&place, &[&file, "--repository", "boom", "--json"], "");
    assert_eq!(outcome.code, 2, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-refused/1");
    assert_eq!(
        answer["refused"]["found"],
        json!([{
            "file": file,
            "line": 3,
            "word": "keeper", // boom: allow keeper
            "written": "keeper", // boom: allow keeper
            "term": "owner",
            "version": version,
            "meaning": OWNER,
            "repository": "boom",
        }])
    );
}

#[test]
fn no_old_word() {
    let place = Place::new();
    owner_accepted(&place);
    let file = place.file("notes.txt", "The owner signs.\nThe gatekeeper opens.\n"); // boom: allow keeper
    let before = written(&place);
    let outcome = check(&place, &[&file, "--repository", "boom"], "");
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(
        outcome.stdout,
        "No old word in 1 file, at entry 3 of the store's log.\n"
    );
    let json = check(&place, &[&file, "--repository", "boom", "--json"], "");
    let answer: Value = serde_json::from_str(&json.stdout).unwrap();
    assert_eq!(answer["format"], "boom-terms-check/1");
    assert_eq!(
        answer["files"],
        json!([{"file": file, "repository": "boom", "text": true}])
    );
    assert_eq!(answer["found"], json!([]));
    assert_eq!(answer["head"]["number"], 3);
    assert_eq!(written(&place), before);
}

#[test]
fn a_term_of_another_repository_is_not_checked() {
    let place = Place::new();
    owner_accepted(&place);
    let file = place.file("notes.txt", "The keeper signs.\n"); // boom: allow keeper
    let outcome = check(&place, &[&file, "--repository", "sluice"], "");
    assert_eq!(outcome.code, 0, "{outcome:?}");
}

#[test]
fn the_words_that_the_check_matches() {
    let place = Place::new();
    owner_accepted(&place);
    let file = place.file(
        "code.rs",
        "Keepers and KEEPER\nlet keeper_id = KeeperStore::new();\nThe gatekeeper.\n// boom: allow keeper. The old field keeper_id stays.\nboom: allow warden, and a keeper\n",
    );
    let outcome = check(&place, &[&file, "--repository", "boom", "--json"], "");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    let found: Vec<(u64, &str, &str)> = answer["refused"]["found"]
        .as_array()
        .unwrap()
        .iter()
        .map(|found| {
            (
                found["line"].as_u64().unwrap(),
                found["word"].as_str().unwrap(),
                found["written"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (1, "keeper", "Keepers"), // boom: allow keeper
            (1, "keeper", "KEEPER"),  // boom: allow keeper
            (2, "keeper", "keeper"),  // boom: allow keeper
            (2, "keeper", "Keeper"),  // boom: allow keeper
            (5, "keeper", "keeper"),  // boom: allow keeper
        ]
    );
}

/// The old words that `boom terms check --json` names in `file` of
/// `boom`, each with its line and as the line writes it.
fn found_in(place: &Place, file: &str) -> Vec<(u64, String, String)> {
    let outcome = check(place, &[file, "--repository", "boom", "--json"], "");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    answer
        .pointer("/refused/found")
        .and_then(Value::as_array)
        .map(|found| {
            found
                .iter()
                .map(|found| {
                    (
                        found["line"].as_u64().unwrap(),
                        found["word"].as_str().unwrap().to_owned(),
                        found["written"].as_str().unwrap().to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A store whose owner accepted the term `origin` of `boom`, which
/// replaces `came-from`. // boom: allow came-from
fn origin_accepted(place: &Place) {
    let ada = place.made();
    let version = proposed(
        place,
        "origin",
        "The file or the commit that a claim came from.",
        &["--repository", "boom", "--replaces", "came-from"], // boom: allow came-from
    );
    place.accepted("origin", &version, &ada);
}

#[test]
fn a_hyphenated_old_word_in_a_name_in_code() {
    let place = Place::new();
    origin_accepted(&place);
    let file = place.file(
        "code.rs",
        "--came-from\nlet came_from = CameFrom::new();\nthis.cameFrom\n", // boom: allow came-from
    );
    assert_eq!(
        found_in(&place, &file)
            .iter()
            .map(|(line, word, written)| (*line, word.as_str(), written.as_str()))
            .collect::<Vec<_>>(),
        [
            (1, "came-from", "came-from"), // boom: allow came-from
            (2, "came-from", "came_from"), // boom: allow came-from
            (2, "came-from", "CameFrom"),  // boom: allow came-from
            (3, "came-from", "cameFrom"),  // boom: allow came-from
        ]
    );
}

#[test]
fn a_hyphenated_old_word_never_matches_the_plain_words() {
    let place = Place::new();
    origin_accepted(&place);
    let plain = place.file(
        "plain.md",
        "The file that it came from.\nCame From\nx.came.from\n",
    );
    let outcome = check(&place, &[&plain, "--repository", "boom"], "");
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let both = place.file("both.md", "It came from the came-from flag.\n"); // boom: allow came-from
    assert_eq!(
        found_in(&place, &both),
        [(1, "came-from".to_owned(), "came-from".to_owned())] // boom: allow came-from
    );
}

#[test]
fn two_files_of_two_repositories() {
    let place = Place::new();
    let (ada, _) = owner_accepted(&place);
    let warden = proposed(
        &place,
        "guard",
        "A person who keeps a gate.",
        &["--repository", "sluice", "--replaces", "sentry"],
    );
    place.accepted("guard", &warden, &ada);
    place.repo_added(&ada, "boom", "git@github.com:boomdrivedev/boom.git");
    place.repo_added(&ada, "sluice", "git@github.com:boomdrivedev/sluice.git");
    let a = place.repository("a", Some("https://github.com/boomdrivedev/boom"));
    let b = place.repository("b", Some("https://github.com/boomdrivedev/sluice"));
    // Each file holds the old word of the other's term too.
    std::fs::write(a.join("x.md"), "A keeper. A sentry.\n").unwrap(); // boom: allow keeper
    std::fs::create_dir_all(b.join("docs")).unwrap();
    std::fs::write(b.join("docs/y.md"), "A sentry. A keeper.\n").unwrap(); // boom: allow keeper
    let x = path_arg(&a.join("x.md"));
    let y = path_arg(&b.join("docs/y.md"));
    let outcome = check(&place, &[&x, &y, "--json"], "");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    let found: Vec<(&str, &str, &str)> = answer["refused"]["found"]
        .as_array()
        .unwrap()
        .iter()
        .map(|found| {
            (
                found["file"].as_str().unwrap(),
                found["word"].as_str().unwrap(),
                found["repository"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (x.as_str(), "keeper", "boom"), // boom: allow keeper
            (y.as_str(), "sentry", "sluice")
        ]
    );
    // A file that does not exist yet takes the repository of the
    // nearest folder above it that exists.
    let new = path_arg(&b.join("docs/new/z.md"));
    let mut env = place.env.clone();
    env.cwd = Some(place.dir.path().to_owned());
    let store = place.store_arg();
    let event =
        json!({"tool_name": "Write", "tool_input": {"file_path": new, "content": "A sentry."}});
    let outcome = boom::cli::run_with_input(
        ["terms", "check", "--hook", "--store", &store],
        &place.keys,
        &env,
        &mut event.to_string().as_bytes(),
    );
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(outcome.stderr.contains("guard in sluice"), "{outcome:?}");
}

#[test]
fn no_repository_found() {
    let place = Place::new();
    owner_accepted(&place);
    let folder = place.repository("shop", Some("https://github.com/a/shop"));
    std::fs::write(folder.join("x.md"), "A keeper.\n").unwrap(); // boom: allow keeper
    let x = path_arg(&folder.join("x.md"));
    let outcome = check(&place, &[&x], "");
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome
            .stderr
            .contains("name it with --repository, or register its remote with boom repo add"),
        "{outcome:?}"
    );
    let asked = check(&place, &[&x, "--if-declared"], "");
    assert_eq!(asked.code, 0, "{asked:?}");
    assert_eq!(asked.stderr, "");
}

#[test]
fn no_store_when_asked() {
    let place = Place::new();
    let file = place.file("x.md", "A keeper.\n"); // boom: allow keeper
    let outcome = check(
        &place,
        &[&file, "--if-declared", "--repository", "boom"],
        "",
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!((outcome.stdout.as_str(), outcome.stderr.as_str()), ("", ""));
    assert!(!place.store.exists(), "no store is made");
    let json = check(
        &place,
        &[&file, "--if-declared", "--repository", "boom", "--json"],
        "",
    );
    let answer: Value = serde_json::from_str(&json.stdout).unwrap();
    assert_eq!(answer["head"], Value::Null);
    let not_run = check(&place, &[&file, "--repository", "boom"], "");
    assert_eq!(not_run.code, 3, "{not_run:?}");
}

#[test]
fn the_views_behind_the_log_are_not_written() {
    let place = Place::new();
    owner_accepted(&place);
    let file = place.file("x.md", "A keeper.\n"); // boom: allow keeper
                                                  // Views of an older form: the check replays the log in memory.
    place
        .sqlite()
        .execute("UPDATE views_head SET form = 4", [])
        .unwrap();
    let before = written(&place);
    let outcome = check(&place, &[&file, "--repository", "boom"], "");
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(outcome.stderr.contains("owner in boom"), "{outcome:?}");
    assert_eq!(written(&place), before);
}

#[test]
fn a_store_in_a_folder_that_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let place = Place::new();
    let data = place.dir.path().join("locked");
    std::fs::create_dir(&data).unwrap();
    let store = data.join("boom.sqlite");
    let store_arg = path_arg(&store);
    let ada = place.key("ada", 1);
    let run = |args: &[&str]| {
        let mut all = args.to_vec();
        all.extend(["--store", &store_arg]);
        boom::cli::run(all, &place.keys, &place.env)
    };
    assert_eq!(run(&["init", "--person", "ada", "--key", &ada]).code, 0);
    let proposed = run(&[
        "propose",
        "term",
        "owner",
        OWNER,
        "--repository",
        "boom",
        "--replaces",
        "keeper", // boom: allow keeper
        "--provenance",
        "run-1",
        "--json",
    ]);
    let version: Value = serde_json::from_str(&proposed.stdout).unwrap();
    let version = version["version"].as_str().unwrap().to_owned();
    assert_eq!(
        run(&["accept", &format!("owner@{version}"), "--key", &ada]).code,
        0
    );
    let file = place.file("x.md", "A keeper.\n"); // boom: allow keeper
    std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o555)).unwrap();
    let outcome = run(&["terms", "check", &file, "--repository", "boom"]);
    std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(outcome.stderr.contains("owner in boom"), "{outcome:?}");
}

// The writer's hook

/// `boom terms check --hook --if-declared` on `event`.
fn hooked(place: &Place, event: &Value) -> boom::cli::Outcome {
    check(place, &["--hook", "--if-declared"], &event.to_string())
}

/// A folder of the repository `boom`, registered with its remote.
fn boom_folder(place: &Place, ada: &str) -> PathBuf {
    place.repo_added(ada, "boom", "git@github.com:boomdrivedev/boom.git");
    place.repository("boom", Some("https://github.com/boomdrivedev/boom.git"))
}

#[test]
fn a_write_with_an_old_word() {
    let place = Place::new();
    let (ada, _) = owner_accepted(&place);
    let folder = boom_folder(&place, &ada);
    let file = path_arg(&folder.join("README.md"));
    let outcome = hooked(
        &place,
        &json!({"tool_name": "Write", "tool_input": {"file_path": file, "content": "# boom\n\nThe keeper signs.\n"}}), // boom: allow keeper
    );
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(outcome.stdout, "");
    assert!(outcome.stderr.contains("\"keeper\""), "{outcome:?}"); // boom: allow keeper
    assert!(
        outcome
            .stderr
            .contains(&format!("{file}, line 3 of the content")),
        "{outcome:?}"
    );
    assert!(outcome.stderr.contains(&format!("owner in boom: {OWNER}")));
}

#[test]
fn an_edit_of_a_file_that_holds_an_old_word() {
    let place = Place::new();
    let (ada, _) = owner_accepted(&place);
    let folder = boom_folder(&place, &ada);
    std::fs::write(folder.join("notes.md"), "The keeper signs.\nOne.\n").unwrap(); // boom: allow keeper
    let file = path_arg(&folder.join("notes.md"));
    let clean = hooked(
        &place,
        &json!({"tool_name": "Edit", "tool_input": {"file_path": file, "old_string": "One.", "new_string": "The owner signs."}}),
    );
    assert_eq!(clean.code, 0, "{clean:?}");
    assert_eq!((clean.stdout.as_str(), clean.stderr.as_str()), ("", ""));
    let multi = hooked(
        &place,
        &json!({"tool_name": "MultiEdit", "tool_input": {"file_path": file, "edits": [
            {"old_string": "One.", "new_string": "Two."},
            {"old_string": "Two.", "new_string": "A Warden."},
        ]}}),
    );
    assert_eq!(multi.code, 2, "{multi:?}");
    assert!(
        multi.stderr.contains("line 1 of the new text of edit 2"),
        "{multi:?}"
    );
}

#[test]
fn another_tool() {
    let place = Place::new();
    owner_accepted(&place);
    let outcome = hooked(
        &place,
        &json!({"tool_name": "Bash", "tool_input": {"command": "echo keeper > notes.md"}}), // boom: allow keeper
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!((outcome.stdout.as_str(), outcome.stderr.as_str()), ("", ""));
}

#[test]
fn the_marker_in_a_write() {
    let place = Place::new();
    let (ada, _) = owner_accepted(&place);
    let folder = boom_folder(&place, &ada);
    let file = path_arg(&folder.join("history.md"));
    let outcome = hooked(
        &place,
        &json!({"tool_name": "Write", "tool_input": {"file_path": file, "content": "The keeper was the word of 2026. boom: allow keeper\n"}}),
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
}

#[test]
fn a_write_in_no_registered_repository() {
    let place = Place::new();
    owner_accepted(&place);
    let folder = place.repository("shop", Some("https://github.com/a/shop"));
    let file = path_arg(&folder.join("README.md"));
    let outcome = hooked(
        &place,
        &json!({"tool_name": "Write", "tool_input": {"file_path": file, "content": "The keeper."}}), // boom: allow keeper
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!((outcome.stdout.as_str(), outcome.stderr.as_str()), ("", ""));
}

// The test that proves the turn: an agent writes "keeper" in a file of // boom: allow keeper
// boom, through the plugin's hook as Claude Code runs it, and the boom
// the tree builds stops the write and names "owner".

/// The root of the tree.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The command of the plugin's one `PreToolUse` hook, as Claude Code
/// reads it from the plugin's hooks.json.
fn hook_command() -> (String, String) {
    let hooks: Value = serde_json::from_slice(
        &std::fs::read(root().join("plugins/claude-code/hooks/hooks.json")).unwrap(),
    )
    .unwrap();
    let pre = hooks["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(pre.len(), 1, "{hooks}");
    let command = pre[0]["hooks"].as_array().unwrap();
    assert_eq!(command.len(), 1, "{hooks}");
    assert_eq!(command[0]["type"], "command");
    (
        pre[0]["matcher"].as_str().unwrap().to_owned(),
        command[0]["command"].as_str().unwrap().to_owned(),
    )
}

#[test]
fn an_agent_writes_the_old_word_and_the_hook_names_owner() {
    let (matcher, command) = hook_command();
    for tool in ["Write", "Edit", "MultiEdit"] {
        assert!(matcher.split('|').any(|name| name == tool), "{matcher}");
    }
    assert!(!matcher.split('|').any(|name| name == "Bash"), "{matcher}");
    let place = Place::new();
    let (ada, _) = owner_accepted(&place);
    let folder = boom_folder(&place, &ada);
    // The store at the default path of the environment the hook runs in.
    let data = place.dir.path().join("data/boom");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::copy(&place.store, data.join("boom.sqlite")).unwrap();
    let bin = place.dir.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_boom"), bin.join("boom")).unwrap();
    let run = |content: &str| {
        let event = json!({
            "session_id": "s-1",
            "hook_event_name": "PreToolUse",
            "cwd": path_arg(&folder),
            "tool_name": "Write",
            "tool_input": {"file_path": path_arg(&folder.join("notes.md")), "content": content},
        });
        let mut child = Command::new("sh")
            .args(["-c", &command])
            .current_dir(&folder)
            .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
            .env("XDG_DATA_HOME", place.dir.path().join("data"))
            .env("HOME", place.dir.path().join("home"))
            .env_remove("BOOM_REPOSITORY")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(event.to_string().as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    };
    let stopped = run("The keeper signs.\n"); // boom: allow keeper
    let stderr = String::from_utf8_lossy(&stopped.stderr);
    assert_eq!(stopped.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("\"keeper\""), "{stderr}"); // boom: allow keeper
    assert!(stderr.contains("owner in boom"), "{stderr}");
    let passed = run("The owner signs.\n");
    assert_eq!(passed.status.code(), Some(0));
    assert!(passed.stdout.is_empty() && passed.stderr.is_empty());
}

// The gate task, check-terms.sh

/// A git repository of boom in the place: main holds `docs/old.md` with
/// `keeper` and an archived change with `keeper`; the branch `change` // boom: allow keeper
/// adds `files`. Gives its folder.
fn change(place: &Place, files: &[(&str, &str)]) -> PathBuf {
    let folder = place.dir.path().join("tree");
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(&folder)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "a")
            .env("GIT_AUTHOR_EMAIL", "a@a")
            .env("GIT_COMMITTER_NAME", "a")
            .env("GIT_COMMITTER_EMAIL", "a@a")
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    let write = |path: &str, text: &str| {
        let path = folder.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    std::fs::create_dir_all(&folder).unwrap();
    git(&["init", "-q", "-b", "main"]);
    write("docs/old.md", "The keeper signs.\n"); // boom: allow keeper
    write("README.md", "# boom\n");
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "main"]);
    git(&["checkout", "-q", "-b", "change"]);
    for (path, text) in files {
        write(path, text);
    }
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "change", "--allow-empty"]);
    folder
}

/// check-terms.sh with `args`, in `folder`, with the boom the tree
/// builds and the place's store at its default path.
fn gate(place: &Place, folder: &Path, args: &[&str]) -> std::process::Output {
    let data = place.dir.path().join("data/boom");
    std::fs::create_dir_all(&data).unwrap();
    if place.store.exists() {
        std::fs::copy(&place.store, data.join("boom.sqlite")).unwrap();
    }
    Command::new("sh")
        .arg(root().join("check-terms.sh"))
        .args(args)
        .current_dir(folder)
        .env("BOOM", env!("CARGO_BIN_EXE_boom"))
        .env("XDG_DATA_HOME", place.dir.path().join("data"))
        .env("HOME", place.dir.path().join("home"))
        .env("TMPDIR", place.dir.path())
        .env_remove("BOOM_REPOSITORY")
        .env_remove("SLUICE_TMP")
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_change_that_adds_an_old_word() {
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[("README.md", "# boom\n\nThe keeper signs.\n")]); // boom: allow keeper
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    let stderr = text(&ran.stderr);
    assert!(!ran.status.success(), "{stderr}");
    assert!(stderr.contains("README.md:3"), "{stderr}");
    assert!(stderr.contains("\"keeper\""), "{stderr}"); // boom: allow keeper
    assert!(stderr.contains("owner in boom"), "{stderr}");
}

#[test]
fn an_old_word_that_the_change_does_not_touch() {
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[("notes.md", "The owner signs.\n")]);
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(
        text(&ran.stdout).contains("1 files of the change hold no old word"),
        "{}",
        text(&ran.stdout)
    );
}

#[test]
fn an_archived_change_and_a_marker() {
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(
        &place,
        &[
            (
                "openspec/changes/archive/2026-11-21-terms/proposal.md",
                "The keeper signs.\n", // boom: allow keeper
            ),
            (
                "docs/history.md",
                "The keeper was the old word. boom: allow keeper\n",
            ),
        ],
    );
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
}

#[test]
fn a_change_that_deletes_a_file_that_holds_an_old_word() {
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[]);
    std::fs::remove_file(folder.join("docs/old.md")).unwrap();
    let commit = Command::new("git")
        .args(["commit", "-q", "-a", "-m", "delete"])
        .current_dir(&folder)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "a")
        .env("GIT_AUTHOR_EMAIL", "a@a")
        .env("GIT_COMMITTER_NAME", "a")
        .env("GIT_COMMITTER_EMAIL", "a@a")
        .status()
        .unwrap();
    assert!(commit.success());
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(text(&ran.stdout).contains("adds or modifies no file"));
}

#[test]
fn the_gate_with_no_store() {
    let place = Place::new();
    let folder = change(&place, &[("README.md", "The keeper signs.\n")]); // boom: allow keeper
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(text(&ran.stdout).contains("no store"));
}

#[test]
fn the_task_names_the_entry_it_read() {
    let place = Place::new();
    owner_accepted(&place);
    let folder = change(&place, &[("notes.md", "The owner signs.\n")]);
    let ran = gate(&place, &folder, &["--repository", "boom"]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    assert!(
        text(&ran.stdout).ends_with("at entry 3 of the store's log\n"),
        "{}",
        text(&ran.stdout)
    );
}
