//! The first edges of the graph: the claims a version rests on, the
//! claims a decision changes, and the files and commits it came from,
//! each stored in the entry of the version with the version it meant
//! (claim-edges; claim-versions).

mod support;

use serde_json::{json, Value};
use support::Place;

const RESIDENT: &str = "flume-has-a-resident";
const CLIENT: &str = "one-github-client-per-host";
const SOCKET: &str = "the-runner-serves-a-socket";
const SOCKET_WORDS: &str = "The runner serves a socket for status and tokens";
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

/// The claim of the log's last entry, as its signed part holds it.
fn last_claim(place: &Place) -> Value {
    let log = place.log();
    let signed: Value =
        serde_json::from_str(log.last().unwrap()["signed"].as_str().unwrap()).unwrap();
    signed["claim"].clone()
}

/// A store with `flume-has-a-resident` and `one-github-client-per-host`,
/// each accepted by ada, giving ada's key and their two versions.
fn two_claims(place: &Place) -> (String, String, String) {
    let ada = place.made();
    let resident = place.proposed(RESIDENT, "flume has one resident on each host.");
    place.accepted(RESIDENT, &resident, &ada);
    let client = place.proposed(CLIENT, "One GitHub client serves each host.");
    place.accepted(CLIENT, &client, &ada);
    (ada, resident, client)
}

/// The decision `the-runner-serves-a-socket`, proposed with `extra`.
fn socket(place: &Place, extra: &[&str]) -> Value {
    let body = place.decision_body();
    let mut args = vec![
        "propose",
        "decision",
        SOCKET,
        SOCKET_WORDS,
        "--body",
        &body,
        "--provenance",
        "run-1",
    ];
    args.extend(extra);
    place.json(&args)
}

#[test]
fn a_decision_stores_its_edges() {
    let place = Place::new();
    let (_, resident, client) = two_claims(&place);
    let mockup = place.file("mockup.svg", "<svg/>\n");
    let answer = socket(
        &place,
        &[
            "--source",
            RESIDENT,
            "--changes",
            CLIENT,
            "--origin",
            &mockup,
        ],
    );
    assert_eq!(answer["source"], json!([format!("{RESIDENT}@{resident}")]));
    assert_eq!(answer["changes"], json!([format!("{CLIENT}@{client}")]));
    let fingerprint = boom::sha256_hex(b"<svg/>\n");
    assert_eq!(
        answer["origin"],
        json!([{"file": "mockup.svg", "fingerprint": fingerprint}])
    );
    let claim = last_claim(&place);
    assert_eq!(claim["source"], json!([format!("{RESIDENT}@{resident}")]));
    assert_eq!(claim["changes"], json!([format!("{CLIENT}@{client}")]));
    assert_eq!(
        claim["origin"],
        json!([{"file": {"name": "mockup.svg", "fingerprint": fingerprint}}])
    );
}

#[test]
fn only_a_decision_changes_claims() {
    let place = Place::new();
    two_claims(&place);
    place.refused(
        &[
            "propose",
            "invariant",
            "hosts-share-a-client",
            "Hosts share one client.",
            "--changes",
            CLIENT,
            "--provenance",
            "run-1",
        ],
        "only a decision changes claims",
    );
}

#[test]
fn one_source_given_two_times_is_one_edge() {
    let place = Place::new();
    let (_, resident, _) = two_claims(&place);
    let answer = socket(&place, &["--source", RESIDENT, "--source", RESIDENT]);
    assert_eq!(answer["source"], json!([format!("{RESIDENT}@{resident}")]));
}

#[test]
fn a_name_stores_its_accepted_version_or_its_newest() {
    let place = Place::new();
    let (ada, resident, _) = two_claims(&place);
    let newer = place.amended(RESIDENT, "flume has one resident.", &resident);
    // A name stores the accepted version, not the newest.
    let answer = socket(&place, &["--source", RESIDENT]);
    assert_eq!(answer["source"], json!([format!("{RESIDENT}@{resident}")]));
    // A version named stores that version.
    let named = socket(&place, &["--source", &format!("{RESIDENT}@{newer}")]);
    assert_eq!(named["source"], json!([format!("{RESIDENT}@{newer}")]));
    // With no accepted version, the newest.
    let first = place.proposed("money-has-currency", "Every amount names its currency.");
    let second = place.proposed("money-has-currency", "Every amount names one currency.");
    assert_ne!(first, second);
    let answer = socket(&place, &["--source", "money-has-currency"]);
    assert_eq!(
        answer["source"],
        json!([format!("money-has-currency@{second}")])
    );
    let _ = ada;
}

#[test]
fn an_edge_names_a_claim_of_the_store() {
    let place = Place::new();
    two_claims(&place);
    let body = place.decision_body();
    let refused = |edge: &str, words: &str| {
        place.refused(
            &[
                "propose",
                "decision",
                SOCKET,
                SOCKET_WORDS,
                "--body",
                &body,
                "--source",
                edge,
                "--provenance",
                "run-1",
            ],
            words,
        )
    };
    let outcome = refused("record-0001", "no claim has the name record-0001");
    assert!(outcome.stderr.contains("record-0001"), "{outcome:?}");
    refused(
        &format!("{RESIDENT}@000000000000"),
        &format!("the claim {RESIDENT} has no version 000000000000"),
    );
    let outcome = refused(
        "https://example.com/model",
        "an edge names a claim of the store",
    );
    assert!(
        outcome.stderr.contains("https://example.com/model"),
        "{outcome:?}"
    );
    refused("0001", "no claim has the name 0001");
}

#[test]
fn a_version_names_its_own_claim_by_amends() {
    let place = Place::new();
    let (_, resident, _) = two_claims(&place);
    place.refused(
        &[
            "propose",
            "invariant",
            RESIDENT,
            "flume has one resident.",
            "--amends",
            &resident,
            "--source",
            RESIDENT,
            "--provenance",
            "run-1",
        ],
        "with --amends, not with --source",
    );
}

#[test]
fn a_file_is_kept_by_its_fingerprint_and_read_back() {
    let place = Place::new();
    place.made();
    let mockup = place.file("mockup.svg", "<svg>overview</svg>\n");
    let answer = place.json(&[
        "propose",
        "invariant",
        "status-on-a-socket",
        "Status is read from a socket.",
        "--origin",
        &mockup,
        "--provenance",
        "run-1",
    ]);
    let fingerprint = answer["origin"][0]["fingerprint"].as_str().unwrap();
    assert_eq!(fingerprint, boom::sha256_hex(b"<svg>overview</svg>\n"));
    let outcome = place.run(&["show", "--artifact", fingerprint]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout, "<svg>overview</svg>\n");
    assert_eq!(outcome.bytes, None);
    let shown = place.json(&["show", "--artifact", fingerprint]);
    assert_eq!(shown["format"], "boom-show/2");
    assert_eq!(shown["name"], "mockup.svg");
    assert_eq!(shown["text"], "<svg>overview</svg>\n");
    assert_eq!(shown["size"], 20);
}

#[test]
fn an_artifact_that_is_not_text_is_written_as_its_bytes() {
    let place = Place::new();
    place.made();
    let path = place.dir.path().join("mockup.png");
    let bytes = [0x89, b'P', b'N', b'G', 0xff, 0x00, 0x01];
    std::fs::write(&path, bytes).unwrap();
    let path = support::path_arg(&path);
    let answer = place.json(&[
        "propose",
        "invariant",
        "status-on-a-socket",
        "Status is read from a socket.",
        "--origin",
        &path,
        "--provenance",
        "run-1",
    ]);
    let fingerprint = answer["origin"][0]["fingerprint"].as_str().unwrap();
    let outcome = place.run(&["show", "--artifact", fingerprint]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.bytes.as_deref(), Some(&bytes[..]));
    let shown = place.json(&["show", "--artifact", fingerprint]);
    assert_eq!(shown["text"], json!(null));
    assert_eq!(shown["base64"], "iVBOR/8AAQ==");
}

#[test]
fn a_file_in_a_commit() {
    let place = Place::new();
    place.made();
    let reference = format!("git:sluice@{COMMIT}:design/overview.svg");
    let answer = place.json(&[
        "propose",
        "invariant",
        "status-on-a-socket",
        "Status is read from a socket.",
        "--origin",
        &reference,
        "--provenance",
        "run-1",
    ]);
    assert_eq!(
        answer["origin"],
        json!([{
            "repository": "sluice",
            "commit": COMMIT,
            "path": "design/overview.svg",
            "reference": reference,
        }])
    );
    let version = answer["version"].as_str().unwrap();
    let shown = place.show(&format!("status-on-a-socket@{version}"));
    assert_eq!(shown["origin"][0]["repository"], "sluice");
    assert_eq!(shown["origin"][0]["commit"], COMMIT);
    assert_eq!(shown["origin"][0]["path"], "design/overview.svg");
    let words = place.run(&["show", &format!("status-on-a-socket@{version}")]);
    assert!(
        words.stdout.contains(&format!("origin: {reference}\n")),
        "{words:?}"
    );
}

#[test]
fn a_reference_to_a_commit_in_its_form() {
    let place = Place::new();
    place.made();
    let refused = |origin: &str, words: &str| {
        place.refused(
            &[
                "propose",
                "invariant",
                "status-on-a-socket",
                "Status is read from a socket.",
                "--origin",
                origin,
                "--provenance",
                "run-1",
            ],
            words,
        );
    };
    refused(
        "git:sluice@1a2b3c:design/overview.svg",
        "a commit has 40 hexadecimal characters",
    );
    refused(
        &format!("git:sluice@{COMMIT}"),
        "a reference to a commit names the path of a file in it",
    );
    refused(
        &format!("git:sluice@{COMMIT}:../secret.txt"),
        "the path of a file in a commit is relative, with no `..`",
    );
    refused(
        &format!("git:sluice@{COMMIT}:/etc/passwd"),
        "the path of a file in a commit is relative",
    );
    refused(
        "no-such-file.svg",
        "the origin no-such-file.svg cannot be read",
    );
}

#[test]
fn boom_show_takes_a_target_or_an_artifact() {
    let place = Place::new();
    place.made();
    let fingerprint = boom::sha256_hex(b"nothing kept");
    let outcome = place.run(&["show", "--artifact", &fingerprint]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome.stderr.contains("no artifact has the fingerprint"),
        "{outcome:?}"
    );
    let outcome = place.run(&["show", "money-has-currency", "--artifact", &fingerprint]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome
            .stderr
            .contains("boom show takes a target or an artifact, and not both"),
        "{outcome:?}"
    );
    let outcome = place.run(&["show"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome.stderr.contains("boom show takes a target"),
        "{outcome:?}"
    );
}

#[test]
fn a_body_is_no_artifact() {
    let place = Place::new();
    place.made();
    let body = place.file("body.md", support::BODY);
    place.proposed_with_body(
        "money-has-currency",
        "Every amount names its currency.",
        &body,
    );
    let outcome = place.run(&[
        "show",
        "--artifact",
        &boom::sha256_hex(support::BODY.as_bytes()),
    ]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
}

#[test]
fn an_edge_out_of_date() {
    let place = Place::new();
    let (ada, resident, _) = two_claims(&place);
    let decision = socket(&place, &["--source", RESIDENT]);
    let version = decision["version"].as_str().unwrap().to_owned();
    let newer = place.amended(RESIDENT, "flume has one resident.", &resident);
    place.accepted(RESIDENT, &newer, &ada);
    let shown = place.show(&format!("{SOCKET}@{version}"));
    assert_eq!(
        shown["source"],
        json!([{
            "claim": RESIDENT,
            "version": resident,
            "statement": "flume has one resident on each host.",
            "accepted": newer,
            "out_of_date": true,
        }])
    );
    let words = place.run(&["show", &format!("{SOCKET}@{version}")]).stdout;
    assert!(
        words.contains(&format!(
            "source: {RESIDENT}@{resident}, out of date: {RESIDENT}@{newer} is accepted\n  flume has one resident on each host.\n"
        )),
        "{words}"
    );
}

#[test]
fn a_change_before_and_now() {
    let place = Place::new();
    let (_, _, client) = two_claims(&place);
    let decision = socket(&place, &["--changes", CLIENT]);
    let version = decision["version"].as_str().unwrap().to_owned();
    let shown = place.show(&format!("{SOCKET}@{version}"));
    assert_eq!(shown["changes"][0]["version"], client);
    assert_eq!(shown["changes"][0]["out_of_date"], false);
    assert_eq!(
        shown["changes"][0]["before"],
        "One GitHub client serves each host."
    );
    assert_eq!(shown["changes"][0]["now"], SOCKET_WORDS);
    let words = place.run(&["show", &format!("{SOCKET}@{version}")]).stdout;
    assert!(
        words.contains(&format!(
            "changes: {CLIENT}@{client}\n  Before: One GitHub client serves each host.\n  Now: {SOCKET_WORDS}\n"
        )),
        "{words}"
    );
}

#[test]
fn the_edges_answered_as_json_in_their_order() {
    let place = Place::new();
    two_claims(&place);
    let reference = format!("git:sluice@{COMMIT}:design/overview.svg");
    let decision = socket(
        &place,
        &[
            "--source", CLIENT, "--source", RESIDENT, "--origin", &reference,
        ],
    );
    let version = decision["version"].as_str().unwrap();
    let shown = place.show(&format!("{SOCKET}@{version}"));
    let named: Vec<&str> = shown["source"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edge| edge["claim"].as_str().unwrap())
        .collect();
    assert_eq!(named, [CLIENT, RESIDENT]);
    assert_eq!(shown["changes"], json!([]));
    assert_eq!(shown["origin"][0]["reference"], reference);
}

#[test]
fn one_more_edge_is_another_version() {
    let place = Place::new();
    two_claims(&place);
    let first = socket(&place, &["--source", RESIDENT]);
    let again = socket(&place, &["--source", RESIDENT]);
    assert_eq!(first["version"], again["version"]);
    assert_eq!(first["entry"], again["entry"]);
    let more = socket(&place, &["--source", RESIDENT, "--source", CLIENT]);
    assert_ne!(first["version"], more["version"]);
    let none = socket(&place, &[]);
    assert_ne!(first["version"], none["version"]);
}

#[test]
fn an_amendment_under_its_base() {
    let place = Place::new();
    let ada = place.made();
    let money = "money-has-currency";
    let v1 = place.proposed(money, "Every amount names its currency.");
    place.accepted(money, &v1, &ada);
    let v2 = place.amended(money, "Every amount names one currency.", &v1);
    place.json(&[
        "reject",
        &format!("{money}@{v2}"),
        "--reason",
        "one is not the point",
        "--key",
        &ada,
    ]);
    let v3 = place.amended(money, "Every amount names its currency, always.", &v1);
    let words = place.run(&["show", money]).stdout;
    let others = words.split("other versions:\n").nth(1).unwrap();
    let at_v1 = others.find(&format!("  {v1} ")).unwrap();
    let at_v2 = others.find(&format!("    {v2} ")).unwrap();
    assert!(at_v1 < at_v2, "{words}");
    assert!(!others.contains(&v3), "{words}");
    let shown = place.show(money);
    let bases: Vec<(&str, Value)> = shown["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|version| {
            (
                version["version"].as_str().unwrap(),
                version["base"].clone(),
            )
        })
        .collect();
    assert_eq!(
        bases,
        [
            (v1.as_str(), json!(null)),
            (v2.as_str(), json!(v1)),
            (v3.as_str(), json!(v1)),
        ]
    );
}

#[test]
fn a_claim_named_as_a_source_and_changed() {
    let place = Place::new();
    let (ada, resident, client) = two_claims(&place);
    let decision = socket(&place, &["--source", RESIDENT, "--changes", CLIENT]);
    let version = decision["version"].as_str().unwrap().to_owned();
    // A proposed version counts while none is accepted.
    let shown = place.show(RESIDENT);
    assert_eq!(
        shown["source_of"],
        json!([{"claim": SOCKET, "version": version, "meant": resident}])
    );
    assert_eq!(shown["changed_by"], json!([]));
    let shown = place.show(CLIENT);
    assert_eq!(
        shown["changed_by"],
        json!([{"claim": SOCKET, "version": version, "meant": client}])
    );
    place.accepted(SOCKET, &version, &ada);
    let words = place.run(&["show", RESIDENT]).stdout;
    assert!(
        words.contains(&format!(
            "source of:\n  {SOCKET}@{version}, on {RESIDENT}@{resident}\n"
        )),
        "{words}"
    );
    // A newer proposal that names no source does not count while the
    // accepted version stands.
    let body = place.decision_body();
    place.json(&[
        "propose",
        "decision",
        SOCKET,
        "The runner serves a socket.",
        "--body",
        &body,
        "--amends",
        &version,
        "--provenance",
        "run-2",
    ]);
    assert_eq!(place.show(RESIDENT)["source_of"][0]["version"], version);
}

#[test]
fn a_name_in_a_body_is_no_edge() {
    let place = Place::new();
    two_claims(&place);
    let body = place.file("body.md", &format!("This rests on {RESIDENT}.\n"));
    place.proposed_with_body("status-on-a-socket", "Status is read from a socket.", &body);
    assert_eq!(place.show(RESIDENT)["source_of"], json!([]));
}

#[test]
fn the_same_answer_after_the_views_are_built_again() {
    let place = Place::new();
    two_claims(&place);
    let mockup = place.file("mockup.svg", "<svg/>\n");
    socket(
        &place,
        &[
            "--source",
            RESIDENT,
            "--changes",
            CLIENT,
            "--origin",
            &mockup,
        ],
    );
    let before = place.run(&["show", RESIDENT, "--json"]).stdout;
    let rows: i64 = place
        .sqlite()
        .query_row("SELECT count(*) FROM edges", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 3);
    let verified = place.json(&["verify"]);
    assert_eq!(verified["rebuilt"], json!([]), "{verified}");
    place
        .sqlite()
        .execute_batch("DROP TABLE edges; DROP TABLE views_head;")
        .unwrap();
    let after = place.run(&["show", RESIDENT, "--json"]).stdout;
    assert_eq!(before, after);
}

#[test]
fn a_version_from_before_the_edges_keeps_its_fingerprint() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", "Every amount names its currency.");
    place.accepted("money-has-currency", &version, &ada);
    let claim = last_claim_of(&place, 2);
    for edge in ["source", "changes", "origin"] {
        assert_eq!(claim.get(edge), None, "{claim}");
    }
    place
        .sqlite()
        .execute_batch("DROP TABLE views_head;")
        .unwrap();
    assert_eq!(
        place.show(&format!("money-has-currency@{version}"))["status"],
        "accepted"
    );
}

/// The claim of the entry numbered `seq`.
fn last_claim_of(place: &Place, seq: usize) -> Value {
    let signed: Value =
        serde_json::from_str(place.log()[seq - 1]["signed"].as_str().unwrap()).unwrap();
    signed["claim"].clone()
}

#[test]
fn views_of_the_form_before_the_edges_are_built_again() {
    let place = Place::new();
    two_claims(&place);
    let decision = socket(&place, &["--source", RESIDENT]);
    let before = place.run(&["show", RESIDENT, "--json"]).stdout;
    // Views as the release before the edges wrote them: form 5, with no
    // table of edges.
    place
        .sqlite()
        .execute_batch("DROP TABLE edges; UPDATE views_head SET form = 5;")
        .unwrap();
    let after = place.run(&["show", RESIDENT, "--json"]).stdout;
    assert_eq!(before, after);
    let shown: Value = serde_json::from_str(&after).unwrap();
    assert_eq!(shown["source_of"][0]["version"], decision["version"]);
}

#[test]
fn a_narrative_with_its_includes() {
    let place = Place::new();
    place.made();
    let status = place.proposed(
        "status-read-from-events",
        "A record never holds its status.",
    );
    let narrative = place.file("narrative.md", support::NARRATIVE);
    let answer = place.json(&[
        "propose",
        "narrative",
        "boom-end-to-end",
        "Boom, end to end",
        "--body",
        &narrative,
        "--includes",
        "status-read-from-events",
        "--provenance",
        "run-1",
    ]);
    let edge = format!("status-read-from-events@{status}");
    assert_eq!(answer["includes"], json!([edge]));
    assert_eq!(last_claim(&place)["includes"], json!([edge]));
}

#[test]
fn an_outline_with_its_narratives() {
    let place = Place::new();
    place.made();
    let end_to_end = place.narrative(
        "boom-end-to-end",
        "Boom, end to end",
        support::NARRATIVE,
        &[],
    )["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let records = place.narrative(
        "change-records",
        "Boom's change record",
        support::NARRATIVE,
        &[],
    )["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let answer = place.json(&[
        "propose",
        "outline",
        "boomdrive-design-book",
        "The design book of Boomdrive",
        "--view",
        "book",
        "--narrative",
        "change-records",
        "--narrative",
        "boom-end-to-end",
        "--provenance",
        "run-1",
    ]);
    let edges = json!([
        format!("change-records@{records}"),
        format!("boom-end-to-end@{end_to_end}")
    ]);
    assert_eq!(answer["narratives"], edges);
    assert_eq!(last_claim(&place)["narratives"], edges);
}

// A version names its cause (claim-edges, "A version names its cause").

const CAUSE: &str = "names-no-plan";
const MONEY: &str = "money-has-currency";

/// A store with `money-has-currency` accepted by ada and `names-no-plan`
/// proposed, giving ada's key, money's version and the cause's version.
fn money_and_cause(place: &Place) -> (String, String, String) {
    let ada = place.made();
    let money = place.proposed(MONEY, "Every amount names its currency.");
    place.accepted(MONEY, &money, &ada);
    let cause = place.proposed(CAUSE, "A claim names no plan.");
    (ada, money, cause)
}

#[test]
fn a_revision_with_its_cause() {
    let place = Place::new();
    let (_, money, cause) = money_and_cause(&place);
    let answer = place.json(&[
        "propose",
        "invariant",
        MONEY,
        "Each amount names its currency.",
        "--amends",
        &money,
        "--cause",
        CAUSE,
        "--provenance",
        "run-1",
    ]);
    let stored = format!("{CAUSE}@{cause}");
    assert_eq!(answer["cause"], json!(stored));
    assert_eq!(last_claim(&place)["cause"], json!(stored));
    let version = answer["version"].as_str().unwrap();
    let shown = place.run(&["show", &format!("{MONEY}@{version}")]);
    assert_eq!(shown.code, 0, "{shown:?}");
    assert!(
        shown
            .stdout
            .contains(&format!("cause: {stored}\n  A claim names no plan.\n")),
        "{}",
        shown.stdout
    );
    assert_eq!(
        place.show(&format!("{MONEY}@{version}"))["cause"][0]["claim"],
        CAUSE
    );
}

#[test]
fn a_cause_that_the_store_does_not_hold() {
    let place = Place::new();
    let (_, money, _) = money_and_cause(&place);
    let before = place.log().len();
    place.refused(
        &[
            "propose",
            "invariant",
            MONEY,
            "Each amount names its currency.",
            "--amends",
            &money,
            "--cause",
            "no-such-claim",
            "--provenance",
            "run-1",
        ],
        "no-such-claim",
    );
    assert_eq!(place.log().len(), before);
}

#[test]
fn the_cause_is_part_of_the_content() {
    let mut plain = boom::claim::Proposal::new(
        boom::claim::Kind::Invariant,
        MONEY,
        "Each amount names its currency.",
        Vec::new(),
        None,
        None,
        None,
    );
    let without = plain.version.clone();
    plain.cause = Some(format!("{CAUSE}@aaaaaaaaaaaa"));
    let with = plain.fingerprint(None);
    assert_ne!(without, with);
    plain.cause = Some(format!("{CAUSE}@bbbbbbbbbbbb"));
    assert_ne!(plain.fingerprint(None), with);
}

#[test]
fn a_cause_names_another_claim() {
    let place = Place::new();
    let (_, money, _) = money_and_cause(&place);
    place.refused(
        &[
            "propose",
            "invariant",
            MONEY,
            "Each amount names its currency.",
            "--amends",
            &money,
            "--cause",
            &format!("{MONEY}@{money}"),
            "--provenance",
            "run-1",
        ],
        "--amends",
    );
}
