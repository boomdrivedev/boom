//! Contexts: a claim that owns path globs, each glob in one repository; a
//! claim that binds a context; and the claims for a file, from the globs
//! of the accepted contexts (context-claims; claim-attachments;
//! repository-rules).

mod support;

use serde_json::{json, Value};
use support::{text, Place};

const OVERVIEW: &str = "sluice-overview";
const OVERVIEW_WORDS: &str = "The overview that a reader of the queue sees.";
const OVERVIEW_GLOB: &str = "sluice:crates/sluice/src/overview*.rs";
const CORE_GLOB: &str = "sluice:crates/sluice-core/src/overview.rs";
const BRANCHES: &str = "the-overview-names-each-branch";
const BRANCHES_WORDS: &str = "The overview names each branch that the queue holds.";
const IN_OVERVIEW: &str = "sluice:crates/sluice/src/overview.rs";

/// Proposes the context `name` with `globs` and `extra`, giving its answer.
fn context(place: &Place, name: &str, globs: &[&str], extra: &[&str]) -> Value {
    let mut args = vec!["propose", "context", name, OVERVIEW_WORDS];
    for glob in globs {
        args.extend(["--path", glob]);
    }
    args.extend(extra);
    args.extend(["--provenance", "run-1"]);
    place.json(&args)
}

/// Proposes the invariant `name` with `extra`, giving its answer.
fn invariant(place: &Place, name: &str, words: &str, extra: &[&str]) -> Value {
    let mut args = vec!["propose", "invariant", name, words];
    args.extend(extra);
    args.extend(["--provenance", "run-1"]);
    place.json(&args)
}

fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// The attachment of the log's last entry.
fn last_attach(place: &Place) -> Value {
    place.last_signed()["attach"].clone()
}

/// The names of the claims of an answer.
fn names(answer: &Value) -> Vec<String> {
    answer["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| claim["name"].as_str().unwrap().to_owned())
        .collect()
}

/// The first store of the spec: ada's store with `sluice-overview`
/// accepted with one glob, `the-overview-names-each-branch` accepted
/// bound to it alone, an invariant of `sluice` and one of the whole
/// system, each accepted. It gives ada's key and the context's version.
fn first_store(place: &Place) -> (String, String) {
    let ada = place.made();
    let overview = version(&context(place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &overview, &ada);
    let branches = version(&invariant(
        place,
        BRANCHES,
        BRANCHES_WORDS,
        &["--context", OVERVIEW],
    ));
    place.accepted(BRANCHES, &branches, &ada);
    let queue = version(&invariant(
        place,
        "the-queue-lands-one-at-a-time",
        "The queue lands one branch at a time.",
        &["--repository", "sluice"],
    ));
    place.accepted("the-queue-lands-one-at-a-time", &queue, &ada);
    let system = version(&invariant(
        place,
        "money-has-currency",
        "Every amount names its currency.",
        &["--system"],
    ));
    place.accepted("money-has-currency", &system, &ada);
    (ada, overview)
}

/// The claim `name` of an answer.
fn claim_of<'a>(answer: &'a Value, name: &str) -> &'a Value {
    answer["claims"]
        .as_array()
        .unwrap()
        .iter()
        .find(|claim| claim["name"] == name)
        .unwrap()
}

fn query_path(place: &Place, paths: &[&str]) -> Value {
    let mut args = vec!["query"];
    for path in paths {
        args.extend(["--path", path]);
    }
    place.json(&args)
}

#[test]
fn a_context_proposed() {
    let place = Place::new();
    place.made();
    let answer = context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]);
    assert_eq!(answer["kind"], "context");
    assert_eq!(answer["status"], "proposed");
    assert_eq!(answer["paths"], json!([OVERVIEW_GLOB]));
    assert_eq!(answer["repositories"], json!(["sluice"]));
    assert_eq!(answer["contexts"], json!([]));
    let shown = place.show(OVERVIEW);
    assert_eq!(shown["kind"], "context");
    assert_eq!(shown["versions"].as_array().unwrap().len(), 1);
    assert_eq!(shown["newest"]["status"], "proposed");
    assert_eq!(shown["newest"]["paths"], json!([OVERVIEW_GLOB]));
    let signed = place.last_signed();
    assert_eq!(signed["claim"]["paths"], json!([OVERVIEW_GLOB]));
    assert_eq!(signed["attach"], json!({"repositories": ["sluice"]}));
    let words = place.run(&["show", &format!("{OVERVIEW}@{}", version(&answer))]);
    assert!(
        words.stdout.contains(&format!("path: {OVERVIEW_GLOB}\n")),
        "{words:?}"
    );
}

#[test]
fn a_context_with_no_body_and_one_glob() {
    let place = Place::new();
    place.made();
    let answer = context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]);
    assert_eq!(answer["status"], "proposed");
    assert!(place.last_signed()["claim"]["body"].is_null());
}

#[test]
fn a_context_with_no_glob() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "context",
            OVERVIEW,
            OVERVIEW_WORDS,
            "--provenance",
            "run-1",
        ],
        "a context owns one glob at least",
    );
}

#[test]
fn a_glob_out_of_its_form() {
    let place = Place::new();
    place.made();
    for (glob, words) in [
        (
            "crates/sluice/src/overview.rs",
            "a glob is `<repository>:<pattern>`",
        ),
        ("sluice:../flume/src/status.rs", "a pattern is relative"),
        ("sluice:/src/a.rs", "a pattern is relative"),
        ("sluice:src\\a.rs", "a pattern is relative"),
        ("sluice:src/[a.rs", "is not a glob"),
    ] {
        place.refused(
            &[
                "propose",
                "context",
                OVERVIEW,
                OVERVIEW_WORDS,
                "--path",
                glob,
                "--provenance",
                "run-1",
            ],
            words,
        );
    }
}

#[test]
fn the_same_globs_in_another_order() {
    let place = Place::new();
    place.made();
    let first = context(
        &place,
        OVERVIEW,
        &["flume:src/status.rs", "sluice:src/a.rs"],
        &[],
    );
    let second = context(
        &place,
        OVERVIEW,
        &["sluice:src/a.rs", "flume:src/status.rs"],
        &[],
    );
    assert_eq!(first["version"], second["version"]);
    assert_eq!(place.acts(), ["registered", "proposed"]);
}

#[test]
fn a_context_in_two_repositories() {
    let place = Place::new();
    place.made();
    let answer = context(
        &place,
        OVERVIEW,
        &[
            "sluice:crates/sluice/src/overview.rs",
            "flume:src/status.rs",
        ],
        &[],
    );
    assert_eq!(answer["repositories"], json!(["flume", "sluice"]));
    assert_eq!(
        last_attach(&place),
        json!({"repositories": ["flume", "sluice"]})
    );
    assert_eq!(
        place.last_signed()["claim"]["paths"],
        json!([
            "flume:src/status.rs",
            "sluice:crates/sluice/src/overview.rs"
        ])
    );
}

#[test]
fn a_context_binds_the_repositories_of_its_globs_alone() {
    let place = Place::new();
    let ada = place.made();
    let other = version(&context(
        &place,
        "flume-status",
        &["flume:src/status.rs"],
        &[],
    ));
    place.accepted("flume-status", &other, &ada);
    for extra in [
        &["--repository", "sluice"][..],
        &["--system"],
        &["--context", "flume-status"],
    ] {
        let mut args = vec![
            "propose",
            "context",
            OVERVIEW,
            OVERVIEW_WORDS,
            "--path",
            "sluice:src/a.rs",
            "--provenance",
            "run-1",
        ];
        args.extend(extra);
        place.refused(&args, "a context binds the repositories of its globs");
    }
}

#[test]
fn only_a_context_owns_paths() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "propose",
            "invariant",
            BRANCHES,
            BRANCHES_WORDS,
            "--path",
            "sluice:src/a.rs",
            "--provenance",
            "run-1",
        ],
        "only a context owns paths",
    );
}

#[test]
fn one_more_glob_is_another_version() {
    let place = Place::new();
    place.made();
    let first = context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]);
    let second = context(
        &place,
        "sluice-overview-2",
        &[OVERVIEW_GLOB, CORE_GLOB],
        &[],
    );
    let again = context(&place, "sluice-overview-2", &[OVERVIEW_GLOB], &[]);
    assert_ne!(second["version"], again["version"]);
    assert_ne!(first["version"], again["version"], "the name is content");
}

#[test]
fn bound_to_a_context() {
    let place = Place::new();
    let ada = place.made();
    let overview = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &overview, &ada);
    let answer = invariant(&place, BRANCHES, BRANCHES_WORDS, &["--context", OVERVIEW]);
    let bound = format!("{OVERVIEW}@{overview}");
    assert_eq!(answer["contexts"], json!([bound]));
    assert_eq!(answer["repositories"], json!([]));
    assert_eq!(last_attach(&place), json!({"contexts": [bound]}));
    let words = place.run(&[
        "propose",
        "invariant",
        BRANCHES,
        BRANCHES_WORDS,
        "--context",
        OVERVIEW,
        "--provenance",
        "run-1",
    ]);
    assert!(
        words
            .stdout
            .contains(&format!("it binds the context {bound}\n")),
        "{words:?}"
    );
}

#[test]
fn bound_to_a_context_and_a_repository() {
    let place = Place::new();
    let ada = place.made();
    let overview = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &overview, &ada);
    invariant(
        &place,
        BRANCHES,
        BRANCHES_WORDS,
        &["--context", OVERVIEW, "--repository", "flume"],
    );
    assert_eq!(
        last_attach(&place),
        json!({"repositories": ["flume"], "contexts": [format!("{OVERVIEW}@{overview}")]})
    );
}

#[test]
fn bound_to_the_newest_version_where_none_is_accepted() {
    let place = Place::new();
    place.made();
    let overview = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    invariant(&place, BRANCHES, BRANCHES_WORDS, &["--context", OVERVIEW]);
    assert_eq!(
        last_attach(&place),
        json!({"contexts": [format!("{OVERVIEW}@{overview}")]})
    );
}

#[test]
fn bound_to_the_version_it_names() {
    let place = Place::new();
    let ada = place.made();
    let first = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &first, &ada);
    let second = version(&context(
        &place,
        OVERVIEW,
        &[OVERVIEW_GLOB, CORE_GLOB],
        &["--amends", &first],
    ));
    let named = format!("{OVERVIEW}@{second}");
    invariant(&place, BRANCHES, BRANCHES_WORDS, &["--context", &named]);
    assert_eq!(last_attach(&place), json!({"contexts": [named]}));
    place.refused(
        &[
            "propose",
            "invariant",
            "another-claim",
            "Another claim.",
            "--context",
            &format!("{OVERVIEW}@000000000000"),
            "--provenance",
            "run-1",
        ],
        "the context sluice-overview has no version 000000000000",
    );
}

#[test]
fn a_name_that_is_not_a_context() {
    let place = Place::new();
    place.made();
    place.proposed("money-has-currency", "Every amount names its currency.");
    place.refused(
        &[
            "propose",
            "invariant",
            BRANCHES,
            BRANCHES_WORDS,
            "--context",
            "money-has-currency",
            "--provenance",
            "run-1",
        ],
        "money-has-currency is not a context",
    );
    place.refused(
        &[
            "propose",
            "invariant",
            BRANCHES,
            BRANCHES_WORDS,
            "--context",
            "no-such-context",
            "--provenance",
            "run-1",
        ],
        "no claim has the name no-such-context",
    );
}

#[test]
fn a_term_bound_to_a_context() {
    let place = Place::new();
    place.made();
    context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]);
    place.refused(
        &[
            "propose",
            "term",
            "refund",
            "Money that goes back to the payer.",
            "--repository",
            "sluice",
            "--context",
            OVERVIEW,
            "--provenance",
            "run-1",
        ],
        "a term binds one repository",
    );
}

#[test]
fn a_context_and_the_whole_system() {
    let place = Place::new();
    place.made();
    context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]);
    let before = place.log();
    let outcome = place.run(&[
        "propose",
        "invariant",
        BRANCHES,
        BRANCHES_WORDS,
        "--context",
        OVERVIEW,
        "--system",
        "--provenance",
        "run-1",
    ]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(place.log(), before);
}

#[test]
fn proposed_again_after_the_context_changed() {
    let place = Place::new();
    let ada = place.made();
    let first = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &first, &ada);
    let proposed = invariant(&place, BRANCHES, BRANCHES_WORDS, &["--context", OVERVIEW]);
    let second = version(&context(
        &place,
        OVERVIEW,
        &[OVERVIEW_GLOB, CORE_GLOB],
        &["--amends", &first],
    ));
    place.accepted(OVERVIEW, &second, &ada);
    let entries = place.log().len();
    let again = invariant(&place, BRANCHES, BRANCHES_WORDS, &["--context", OVERVIEW]);
    assert_eq!(again["version"], proposed["version"]);
    assert_eq!(again["entry"], proposed["entry"]);
    assert_eq!(again["contexts"], json!([format!("{OVERVIEW}@{first}")]));
    assert_eq!(place.log().len(), entries);
}

#[test]
fn the_same_words_bound_to_a_context_are_the_same_version() {
    let shop = Place::new();
    shop.made();
    let bound_to_shop = invariant(
        &shop,
        "money-has-currency",
        "Every amount names its currency.",
        &["--repository", "shop"],
    );
    let billing = Place::new();
    billing.made();
    context(&billing, "shop-billing", &["shop:src/billing/**"], &[]);
    let bound_to_billing = invariant(
        &billing,
        "money-has-currency",
        "Every amount names its currency.",
        &["--context", "shop-billing"],
    );
    assert_eq!(bound_to_shop["version"], bound_to_billing["version"]);
}

#[test]
fn amended_after_its_context_changed() {
    let place = Place::new();
    let ada = place.made();
    let first = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &first, &ada);
    let branches = version(&invariant(
        &place,
        BRANCHES,
        BRANCHES_WORDS,
        &["--context", OVERVIEW],
    ));
    place.accepted(BRANCHES, &branches, &ada);
    let second = version(&context(
        &place,
        OVERVIEW,
        &[OVERVIEW_GLOB, CORE_GLOB],
        &["--amends", &first],
    ));
    place.accepted(OVERVIEW, &second, &ada);
    let cwd = place.repository("ledger", None);
    let amended = place.json_in(
        &cwd,
        &[
            "propose",
            "invariant",
            BRANCHES,
            "The overview names each branch the queue holds, by its name.",
            "--amends",
            &branches,
            "--provenance",
            "run-2",
        ],
    );
    assert_eq!(amended["contexts"], json!([format!("{OVERVIEW}@{second}")]));
    assert_eq!(amended["repositories"], json!([]));
}

#[test]
fn a_context_amended_binds_the_repositories_of_its_own_globs() {
    let place = Place::new();
    let ada = place.made();
    let first = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &first, &ada);
    let second = context(
        &place,
        OVERVIEW,
        &[OVERVIEW_GLOB, "flume:src/status.rs"],
        &["--amends", &first],
    );
    assert_eq!(second["repositories"], json!(["flume", "sluice"]));
}

#[test]
fn moved_into_a_context() {
    let place = Place::new();
    let ada = place.made();
    let overview = version(&context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &overview, &ada);
    let in_sluice = version(&invariant(
        &place,
        BRANCHES,
        BRANCHES_WORDS,
        &["--repository", "sluice"],
    ));
    place.accepted(BRANCHES, &in_sluice, &ada);
    let moved = place.json(&[
        "propose",
        "invariant",
        BRANCHES,
        BRANCHES_WORDS,
        "--amends",
        &in_sluice,
        "--context",
        OVERVIEW,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(moved["repositories"], json!([]));
    assert_eq!(moved["contexts"], json!([format!("{OVERVIEW}@{overview}")]));
    // The accepted version stays attached to sluice until a person
    // accepts the amendment.
    let submit = query_path(&place, &["sluice:crates/sluice/src/submit.rs"]);
    assert!(names(&submit).contains(&BRANCHES.to_owned()), "{submit}");
    place.accepted(BRANCHES, &version(&moved), &ada);
    let submit = query_path(&place, &["sluice:crates/sluice/src/submit.rs"]);
    assert!(!names(&submit).contains(&BRANCHES.to_owned()), "{submit}");
    assert!(names(&query_path(&place, &[IN_OVERVIEW])).contains(&BRANCHES.to_owned()));
}

#[test]
fn a_file_in_a_context() {
    let place = Place::new();
    let (_, overview) = first_store(&place);
    let answer = query_path(&place, &[IN_OVERVIEW]);
    assert_eq!(answer["repository"], "sluice");
    assert_eq!(
        names(&answer),
        [
            "money-has-currency",
            BRANCHES,
            "the-queue-lands-one-at-a-time",
            OVERVIEW
        ]
    );
    let bound = format!("{OVERVIEW}@{overview}");
    assert_eq!(
        answer["paths"],
        json!([{"path": IN_OVERVIEW, "contexts": [bound]}])
    );
    assert_eq!(claim_of(&answer, BRANCHES)["contexts"], json!([bound]));
    assert_eq!(claim_of(&answer, BRANCHES)["repositories"], json!([]));
}

#[test]
fn a_file_in_no_context() {
    let place = Place::new();
    first_store(&place);
    let answer = query_path(&place, &["sluice:crates/sluice/src/submit.rs"]);
    assert_eq!(
        names(&answer),
        [
            "money-has-currency",
            "the-queue-lands-one-at-a-time",
            OVERVIEW
        ]
    );
    assert_eq!(answer["paths"][0]["contexts"], json!([]));
    let words = place.run(&["query", "--path", "sluice:crates/sluice/src/submit.rs"]);
    assert_eq!(words.code, 0, "{words:?}");
    assert_eq!(
        words.stdout.lines().nth(1),
        Some("path sluice:crates/sluice/src/submit.rs: no context")
    );
}

#[test]
fn the_contexts_of_a_path_in_words() {
    let place = Place::new();
    let (_, overview) = first_store(&place);
    let words = place.run(&["query", "--path", IN_OVERVIEW]);
    let lines: Vec<&str> = words.stdout.lines().collect();
    assert!(
        lines[0].starts_with("Boom: 4 claims for sluice, at entry "),
        "{words:?}"
    );
    assert_eq!(
        lines[1],
        format!("path {IN_OVERVIEW}: {OVERVIEW}@{overview}")
    );
}

#[test]
fn a_glob_does_not_cross_a_folder() {
    let place = Place::new();
    first_store(&place);
    let answer = query_path(&place, &["sluice:crates/sluice/src/overview/watch.rs"]);
    assert!(!names(&answer).contains(&BRANCHES.to_owned()), "{answer}");
}

#[test]
fn a_file_that_does_not_exist() {
    let place = Place::new();
    first_store(&place);
    let answer = query_path(&place, &["sluice:crates/sluice/src/overview_new.rs"]);
    assert!(names(&answer).contains(&BRANCHES.to_owned()), "{answer}");
}

#[test]
fn a_repository_with_no_context() {
    let place = Place::new();
    let ada = place.made();
    let status = version(&invariant(
        &place,
        "status-is-read-from-a-socket",
        "Status is read from a socket.",
        &["--repository", "flume"],
    ));
    place.accepted("status-is-read-from-a-socket", &status, &ada);
    first_store_after(&place, &ada);
    let by_path = query_path(&place, &["flume:src/status.rs"]);
    let by_repository = place.json(&["query", "--repository", "flume"]);
    assert_eq!(by_path["claims"], by_repository["claims"]);
    assert_eq!(
        names(&by_path),
        ["money-has-currency", "status-is-read-from-a-socket"]
    );
}

/// The context and the claims of the first store, in a store that ada
/// made already.
fn first_store_after(place: &Place, ada: &str) {
    let overview = version(&context(place, OVERVIEW, &[OVERVIEW_GLOB], &[]));
    place.accepted(OVERVIEW, &overview, ada);
    let branches = version(&invariant(
        place,
        BRANCHES,
        BRANCHES_WORDS,
        &["--context", OVERVIEW],
    ));
    place.accepted(BRANCHES, &branches, ada);
    let system = version(&invariant(
        place,
        "money-has-currency",
        "Every amount names its currency.",
        &["--system"],
    ));
    place.accepted("money-has-currency", &system, ada);
}

#[test]
fn only_the_repository() {
    let place = Place::new();
    first_store(&place);
    let answer = place.json(&["query", "--repository", "sluice"]);
    let found = names(&answer);
    assert!(found.contains(&OVERVIEW.to_owned()), "{answer}");
    assert!(!found.contains(&BRANCHES.to_owned()), "{answer}");
    assert_eq!(answer.get("paths"), None);
    // The context comes after the terms, last.
    assert_eq!(found.last().map(String::as_str), Some(OVERVIEW));
    let words = place.run(&["query", "--repository", "sluice"]);
    assert!(
        words
            .stdout
            .lines()
            .last()
            .unwrap()
            .starts_with(&format!("context {OVERVIEW}@")),
        "{words:?}"
    );
}

#[test]
fn contexts_after_the_terms() {
    let place = Place::new();
    let ada = place.made();
    let term = place.json(&[
        "propose",
        "term",
        "refund",
        "Money that goes back to the payer.",
        "--repository",
        "shop",
        "--provenance",
        "run-1",
    ]);
    place.accepted("refund", &version(&term), &ada);
    let billing = version(&context(
        &place,
        "shop-billing",
        &["shop:src/billing/**"],
        &[],
    ));
    place.accepted("shop-billing", &billing, &ada);
    let words = place.run(&["query", "--repository", "shop"]);
    let lines: Vec<&str> = words.stdout.lines().collect();
    assert!(lines[1].starts_with("term refund@"), "{words:?}");
    assert_eq!(
        lines[2],
        format!("context shop-billing@{billing}: {OVERVIEW_WORDS}")
    );
}

#[test]
fn a_context_that_waits() {
    let place = Place::new();
    let ada = place.made();
    context(&place, OVERVIEW, &[OVERVIEW_GLOB], &[]);
    let branches = version(&invariant(
        &place,
        BRANCHES,
        BRANCHES_WORDS,
        &["--context", OVERVIEW],
    ));
    place.accepted(BRANCHES, &branches, &ada);
    let answer = query_path(&place, &[IN_OVERVIEW]);
    assert!(!names(&answer).contains(&BRANCHES.to_owned()), "{answer}");
    assert_eq!(answer["paths"][0]["contexts"], json!([]));
}

#[test]
fn a_context_with_a_new_glob() {
    let place = Place::new();
    let (ada, first) = first_store(&place);
    let core = "sluice:crates/sluice-core/src/overview.rs";
    assert!(!names(&query_path(&place, &[core])).contains(&BRANCHES.to_owned()));
    let second = version(&context(
        &place,
        OVERVIEW,
        &[OVERVIEW_GLOB, CORE_GLOB],
        &["--amends", &first],
    ));
    // The binding follows the accepted context, not the version it meant.
    assert!(!names(&query_path(&place, &[core])).contains(&BRANCHES.to_owned()));
    place.accepted(OVERVIEW, &second, &ada);
    let answer = query_path(&place, &[core]);
    assert!(names(&answer).contains(&BRANCHES.to_owned()), "{answer}");
    assert_eq!(
        answer["paths"][0]["contexts"],
        json!([format!("{OVERVIEW}@{second}")])
    );
    assert_eq!(
        claim_of(&answer, BRANCHES)["contexts"],
        json!([format!("{OVERVIEW}@{first}")])
    );
}

#[test]
fn two_contexts_own_one_file() {
    let place = Place::new();
    let (ada, overview) = first_store(&place);
    let all = version(&context(&place, "sluice-all", &["sluice:**"], &[]));
    place.accepted("sluice-all", &all, &ada);
    let answer = query_path(&place, &[IN_OVERVIEW]);
    assert_eq!(
        answer["paths"][0]["contexts"],
        json!([
            format!("sluice-all@{all}"),
            format!("{OVERVIEW}@{overview}")
        ])
    );
}

#[test]
fn two_paths_of_one_repository() {
    let place = Place::new();
    first_store(&place);
    let answer = query_path(&place, &[IN_OVERVIEW, "sluice:crates/sluice/src/submit.rs"]);
    let found = names(&answer);
    let mut once = found.clone();
    once.sort();
    once.dedup();
    assert_eq!(found.len(), once.len(), "{answer}");
    assert!(found.contains(&BRANCHES.to_owned()));
    assert_eq!(answer["paths"].as_array().unwrap().len(), 2);
    assert_eq!(
        answer["paths"][1]["path"],
        "sluice:crates/sluice/src/submit.rs"
    );
}

#[test]
fn the_paths_of_one_query_are_refused_where_they_disagree() {
    let place = Place::new();
    place.made();
    place.refused(
        &[
            "query",
            "--path",
            "sluice:src/a.rs",
            "--path",
            "flume:src/b.rs",
        ],
        "the paths of one query name one repository",
    );
    place.refused(
        &["query", "--path", "sluice:src/a.rs", "--if-declared"],
        "the two flags cannot be given together",
    );
    place.refused(
        &[
            "query",
            "--path",
            "sluice:src/a.rs",
            "--repository",
            "sluice",
        ],
        "a path names its repository",
    );
    place.refused(&["query", "--path", "src/a.rs"], "names no repository");
    place.refused(&["query", "--path", "sluice:../a.rs"], "relative");
}

#[test]
fn named_by_a_path() {
    let place = Place::new();
    place.made();
    let cwd = place.repository("sluice", None);
    let answer = place.json_in(&cwd, &["query", "--path", "ledger:src/post.rs"]);
    assert_eq!(answer["repository"], "ledger");
}

#[test]
fn the_same_answer_after_the_views_are_built_again() {
    let place = Place::new();
    first_store(&place);
    let args = ["query", "--path", IN_OVERVIEW, "--json"];
    let before = place.run(&args).stdout;
    let verified = place.json(&["verify"]);
    assert_eq!(verified["rebuilt"], json!([]));
    place
        .sqlite()
        .execute_batch("DROP TABLE paths; DROP TABLE bindings; DROP TABLE views_head;")
        .unwrap();
    assert_eq!(place.run(&args).stdout, before);
    place
        .sqlite()
        .execute_batch("DELETE FROM paths; DELETE FROM bindings;")
        .unwrap();
    let verified = place.json(&["verify"]);
    assert_eq!(verified["rebuilt"], json!(["paths", "bindings"]));
    assert_eq!(place.run(&args).stdout, before);
}

#[test]
fn views_of_the_form_before_contexts_are_built_again() {
    let place = Place::new();
    first_store(&place);
    let args = ["query", "--path", IN_OVERVIEW, "--json"];
    let before = place.run(&args).stdout;
    // Views as the release before contexts wrote them: form 6, with no
    // table of globs or bindings.
    place
        .sqlite()
        .execute_batch("DROP TABLE paths; DROP TABLE bindings; UPDATE views_head SET form = 6;")
        .unwrap();
    assert_eq!(place.run(&args).stdout, before);
}

#[test]
fn a_search_of_a_repository_finds_the_claims_of_its_contexts() {
    let place = Place::new();
    let ada = place.made();
    let billing = version(&context(
        &place,
        "shop-billing",
        &["shop:src/billing/**"],
        &[],
    ));
    place.accepted("shop-billing", &billing, &ada);
    let refunds = version(&invariant(
        &place,
        "a-refund-names-its-order",
        "A refund names the order that it pays back.",
        &["--context", "shop-billing"],
    ));
    place.accepted("a-refund-names-its-order", &refunds, &ada);
    let in_shop = place.json(&["search", "refund", "--repository", "shop"]);
    assert_eq!(names(&in_shop), ["a-refund-names-its-order"]);
    let in_ledger = place.json(&["search", "refund", "--repository", "ledger"]);
    assert_eq!(names(&in_ledger), Vec::<String>::new());
    let words = place.run(&["search", "refund", "--repository", "shop"]);
    assert!(
        words
            .stdout
            .contains(&format!("for the context shop-billing@{billing}:")),
        "{words:?}"
    );
    // A query of the repository alone does not give it; a path does.
    let query = place.json(&["query", "--repository", "shop"]);
    assert!(!names(&query).contains(&"a-refund-names-its-order".to_owned()));
    let query = query_path(&place, &["shop:src/billing/refund.rs"]);
    assert!(names(&query).contains(&"a-refund-names-its-order".to_owned()));
}

#[test]
fn a_binding_out_of_date() {
    let place = Place::new();
    let (ada, first) = first_store(&place);
    let second = version(&context(
        &place,
        OVERVIEW,
        &[OVERVIEW_GLOB, CORE_GLOB],
        &["--amends", &first],
    ));
    place.accepted(OVERVIEW, &second, &ada);
    let branches = place.show(BRANCHES)["accepted"]
        .as_str()
        .unwrap()
        .to_owned();
    let target = format!("{BRANCHES}@{branches}");
    let shown = place.show(&target);
    assert_eq!(shown["contexts"], json!([format!("{OVERVIEW}@{first}")]));
    assert_eq!(shown["repositories"], json!([]));
    assert_eq!(
        shown["bindings"],
        json!([{
            "context": OVERVIEW,
            "version": first,
            "accepted": second,
            "out_of_date": true,
        }])
    );
    let words = place.run(&["show", &target]);
    assert!(
        words.stdout.contains(&format!(
            "context: {OVERVIEW}@{first}, out of date: {OVERVIEW}@{second} is accepted\n"
        )),
        "{words:?}"
    );
    assert!(
        words
            .stdout
            .contains(&format!("binds: the context {OVERVIEW}@{first}\n")),
        "{words:?}"
    );
}

#[test]
fn the_claims_bound_to_a_context() {
    let place = Place::new();
    let (_, overview) = first_store(&place);
    let branches = place.show(BRANCHES)["accepted"]
        .as_str()
        .unwrap()
        .to_owned();
    let shown = place.show(OVERVIEW);
    assert_eq!(
        shown["bound"],
        json!([{"claim": BRANCHES, "version": branches, "meant": overview}])
    );
    assert_eq!(shown["newest"]["paths"], json!([OVERVIEW_GLOB]));
    let words = place.run(&["show", OVERVIEW]);
    assert!(
        words.stdout.contains(&format!(
            "\nbound:\n  {BRANCHES}@{branches}, on {OVERVIEW}@{overview}\n"
        )),
        "{words:?}"
    );
    assert_eq!(place.show(BRANCHES).get("bound"), None);
}

#[test]
fn a_context_the_log_does_not_hold() {
    let place = Place::new();
    place.made();
    let proposal = |contexts: Vec<String>, kind: boom::claim::Kind| boom::entry::Entry {
        at: 1_763_424_100,
        actor: boom::entry::Actor::Agent {
            provenance: "run-1".to_owned(),
        },
        act: boom::entry::Act::Proposed {
            claim: Box::new(boom::claim::Proposal::new(
                kind,
                "money-has-currency",
                "Every amount names its currency.",
                Vec::new(),
                None,
                None,
                None,
            )),
            imported: None,
            attach: Some(boom::entry::Attach {
                repositories: Vec::new(),
                contexts,
            }),
        },
    };
    place.append_by_hand(
        &proposal(
            vec![format!("{OVERVIEW}@000000000000")],
            boom::claim::Kind::Invariant,
        ),
        None,
    );
    let log = place.log();
    let last = log.last().unwrap();
    assert_eq!(last["counted"], false, "{last}");
    assert!(
        last["reason"]
            .as_str()
            .unwrap()
            .contains("no claim has the name sluice-overview"),
        "{last}"
    );
    let outcome = place.run(&["show", "money-has-currency"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    for (contexts, words) in [
        (Vec::new(), "a repository or a context at least"),
        (vec!["sluice-overview".to_owned()], "<name>@<version>"),
        (
            vec![
                "b-context@aaaaaaaaaaaa".to_owned(),
                "a-context@aaaaaaaaaaaa".to_owned(),
            ],
            "in the order of their names",
        ),
        (
            vec![
                "a-context@aaaaaaaaaaaa".to_owned(),
                "a-context@bbbbbbbbbbbb".to_owned(),
            ],
            "each context once",
        ),
    ] {
        place.append_by_hand(&proposal(contexts, boom::claim::Kind::Invariant), None);
        let log = place.log();
        let last = log.last().unwrap();
        assert_eq!(last["counted"], false, "{last}");
        assert!(last["reason"].as_str().unwrap().contains(words), "{last}");
    }
}

#[test]
fn a_binding_to_a_claim_that_is_not_a_context_counts_for_nothing() {
    let place = Place::new();
    place.made();
    let money = place.proposed("money-has-currency", "Every amount names its currency.");
    let entry = boom::entry::Entry {
        at: 1_763_424_100,
        actor: boom::entry::Actor::Agent {
            provenance: "run-1".to_owned(),
        },
        act: boom::entry::Act::Proposed {
            claim: Box::new(boom::claim::Proposal::new(
                boom::claim::Kind::Invariant,
                BRANCHES,
                BRANCHES_WORDS,
                Vec::new(),
                None,
                None,
                None,
            )),
            imported: None,
            attach: Some(boom::entry::Attach {
                repositories: Vec::new(),
                contexts: vec![format!("money-has-currency@{money}")],
            }),
        },
    };
    place.append_by_hand(&entry, None);
    let log = place.log();
    let last = log.last().unwrap();
    assert_eq!(last["counted"], false, "{last}");
    assert!(
        last["reason"]
            .as_str()
            .unwrap()
            .contains("money-has-currency is not a context"),
        "{last}"
    );
}

#[test]
fn a_context_whose_attachment_is_not_its_globs_counts_for_nothing() {
    let place = Place::new();
    place.made();
    let entry = |repositories: Vec<String>, contexts: Vec<String>| boom::entry::Entry {
        at: 1_763_424_100,
        actor: boom::entry::Actor::Agent {
            provenance: "run-1".to_owned(),
        },
        act: boom::entry::Act::Proposed {
            claim: Box::new(boom::claim::Proposal::context(
                OVERVIEW,
                OVERVIEW_WORDS,
                None,
                None,
                &[OVERVIEW_GLOB.to_owned()],
            )),
            imported: None,
            attach: Some(boom::entry::Attach {
                repositories,
                contexts,
            }),
        },
    };
    for (repositories, contexts, words) in [
        (
            vec!["flume".to_owned()],
            Vec::new(),
            "a context binds the repositories of its globs",
        ),
        (
            vec!["sluice".to_owned()],
            vec!["other@aaaaaaaaaaaa".to_owned()],
            "and no context",
        ),
    ] {
        place.append_by_hand(&entry(repositories, contexts), None);
        let log = place.log();
        let last = log.last().unwrap();
        assert_eq!(last["counted"], false, "{last}");
        assert!(last["reason"].as_str().unwrap().contains(words), "{last}");
    }
}

#[test]
fn a_version_from_before_contexts_keeps_its_fingerprint_and_its_attachment() {
    let place = Place::new();
    let ada = place.made();
    let money = version(&invariant(
        &place,
        "money-has-currency",
        "Every amount names its currency.",
        &["--repository", "shop"],
    ));
    place.accepted("money-has-currency", &money, &ada);
    let term = version(&place.json(&[
        "propose",
        "term",
        "refund",
        "Money that goes back to the payer.",
        "--repository",
        "shop",
        "--replaces",
        "chargeback",
        "--provenance",
        "run-1",
    ]));
    let edged = version(&invariant(
        &place,
        "refunds-name-their-currency",
        "A refund names its currency.",
        &["--source", "money-has-currency", "--system"],
    ));
    // Each entry written as a boom before contexts wrote it: no `paths`
    // and no `contexts`.
    for entry in place.log() {
        let signed = entry["signed"].as_str().unwrap();
        assert!(!signed.contains("\"paths\""), "{signed}");
        assert!(!signed.contains("\"contexts\""), "{signed}");
    }
    place
        .sqlite()
        .execute_batch("DROP TABLE versions; DROP TABLE views_head;")
        .unwrap();
    for (name, version, repositories) in [
        ("money-has-currency", &money, json!(["shop"])),
        ("refund", &term, json!(["shop"])),
        ("refunds-name-their-currency", &edged, json!([])),
    ] {
        let shown = place.show(&format!("{name}@{version}"));
        assert_eq!(shown["version"], *version);
        assert_eq!(shown["repositories"], repositories);
        assert_eq!(shown["contexts"], json!([]));
    }
    let verified = place.json(&["verify"]);
    assert_eq!(verified["rebuilt"], json!([]));
}

#[test]
fn a_context_proposed_through_its_tool() {
    let place = Place::new();
    place.made();
    let result = place.call(
        "propose",
        json!({
            "kind": "context",
            "name": OVERVIEW,
            "statement": OVERVIEW_WORDS,
            "paths": [OVERVIEW_GLOB],
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    assert_eq!(text(&result)["repositories"], json!(["sluice"]));
    assert_eq!(place.acts(), ["registered", "proposed"]);
    assert_eq!(last_attach(&place), json!({"repositories": ["sluice"]}));
}

#[test]
fn the_claims_for_a_path_through_its_tool() {
    let place = Place::new();
    first_store(&place);
    let result = place.call("query", json!({"paths": [IN_OVERVIEW]}));
    assert_eq!(result["isError"], false, "{result}");
    assert_eq!(text(&result), query_path(&place, &[IN_OVERVIEW]));
    let result = place.call(
        "propose",
        json!({
            "kind": "invariant",
            "name": "another-overview-claim",
            "statement": "The overview is read only.",
            "contexts": [OVERVIEW],
            "provenance": "run-1",
        }),
    );
    assert_eq!(result["isError"], false, "{result}");
    assert_eq!(
        last_attach(&place)["contexts"][0]
            .as_str()
            .unwrap()
            .split('@')
            .next(),
        Some(OVERVIEW)
    );
}
