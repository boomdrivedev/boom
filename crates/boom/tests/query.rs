//! What an agent working in a repository gets from boom: every claim
//! that stands for it, as the person accepted it, replayed from the log
//! and written nowhere in the repository (repository-rules).

mod support;

use serde_json::{json, Value};
use support::{Place, SCENARIO};

const MONEY: &str = "Every amount names its currency.";
const CENTS: &str = "Amounts are kept in integer cents.";
const PAID: &str = "An order is paid once.";
const REFUND: &str = "A refund never exceeds the payment.";

/// An agent's proposal of a first version, with `extra`, giving its
/// version.
fn proposed(place: &Place, kind: &str, name: &str, statement: &str, extra: &[&str]) -> String {
    let mut args = vec![
        "propose",
        kind,
        name,
        statement,
        "--scenario",
        SCENARIO,
        "--provenance",
        "run-1",
    ];
    args.extend_from_slice(extra);
    place.json(&args)["version"].as_str().unwrap().to_owned()
}

/// A first version proposed with `extra` and accepted by `key`, giving
/// its version.
fn standing(
    place: &Place,
    key: &str,
    kind: &str,
    name: &str,
    statement: &str,
    extra: &[&str],
) -> String {
    let version = proposed(place, kind, name, statement, extra);
    place.accepted(name, &version, key);
    version
}

/// `boom query --repository <repository> --json`'s answer.
fn query(place: &Place, repository: &str) -> Value {
    place.json(&["query", "--repository", repository])
}

/// The names of the claims an answer gives, in its order.
fn names(answer: &Value) -> Vec<String> {
    answer["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| claim["name"].as_str().unwrap().to_owned())
        .collect()
}

/// The log as the file holds it, every column.
fn log_rows(place: &Place) -> Vec<(i64, String, String, String, Option<String>)> {
    let conn = place.sqlite();
    let mut statement = conn
        .prepare("SELECT seq, prev, hash, signed, signature FROM log ORDER BY seq")
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

/// `money-has-currency` attached to `shop`, standing, accepted by ada;
/// gives ada's key and its version.
fn money_in_shop(place: &Place) -> (String, String) {
    let ada = place.made();
    let version = standing(
        place,
        &ada,
        "invariant",
        "money-has-currency",
        MONEY,
        &["--repository", "shop"],
    );
    (ada, version)
}

#[test]
fn what_stands_for_a_repository() {
    let place = Place::new();
    let ada = place.made();
    let money = proposed(
        &place,
        "invariant",
        "money-has-currency",
        MONEY,
        &["--repository", "shop"],
    );
    let accepted = place.accepted("money-has-currency", &money, &ada);
    standing(
        &place,
        &ada,
        "decision",
        "amounts-in-cents",
        CENTS,
        &["--system"],
    );
    standing(
        &place,
        &ada,
        "assumption",
        "orders-are-paid-once",
        PAID,
        &["--repository", "ledger"],
    );
    proposed(
        &place,
        "invariant",
        "refund-within-payment",
        REFUND,
        &["--repository", "shop"],
    );
    let answer = query(&place, "shop");
    assert_eq!(names(&answer), ["money-has-currency", "amounts-in-cents"]);
    assert_eq!(
        answer["claims"][0],
        json!({
            "name": "money-has-currency",
            "kind": "invariant",
            "version": money,
            "statement": MONEY,
            "scenarios": [SCENARIO],
            "body": null,
            "repositories": ["shop"],
            "accepted": {"by": "ada", "entry": accepted["entry"]},
        })
    );
    assert_eq!(answer["claims"][1]["kind"], "decision");
    assert_eq!(answer["claims"][1]["repositories"], json!([]));
    assert_eq!(
        names(&query(&place, "ledger")),
        ["amounts-in-cents", "orders-are-paid-once"]
    );
}

#[test]
fn an_amendment_waiting_gives_the_standing_words() {
    let place = Place::new();
    let (_, base) = money_in_shop(&place);
    place.amended(
        "money-has-currency",
        "Every amount names exactly one currency.",
        &base,
    );
    let answer = query(&place, "shop");
    assert_eq!(names(&answer), ["money-has-currency"]);
    assert_eq!(answer["claims"][0]["version"], base.as_str());
    assert_eq!(answer["claims"][0]["statement"], MONEY);
}

#[test]
fn an_amendment_accepted_replaces_its_base() {
    let place = Place::new();
    let (ada, base) = money_in_shop(&place);
    let words = "Every amount names exactly one currency.";
    let amendment = place.amended("money-has-currency", words, &base);
    place.accepted("money-has-currency", &amendment, &ada);
    let answer = query(&place, "shop");
    assert_eq!(names(&answer), ["money-has-currency"]);
    assert_eq!(answer["claims"][0]["version"], amendment.as_str());
    assert_eq!(answer["claims"][0]["statement"], words);
    assert_eq!(answer["claims"][0]["repositories"], json!(["shop"]));
    assert!(!answer.to_string().contains(&base), "{answer}");
}

#[test]
fn a_rejected_version_is_not_given() {
    let place = Place::new();
    let ada = place.made();
    let version = proposed(
        &place,
        "invariant",
        "money-has-currency",
        MONEY,
        &["--repository", "shop"],
    );
    place.json(&[
        "reject",
        &format!("money-has-currency@{version}"),
        "--reason",
        "Amounts are in one currency.",
        "--key",
        &ada,
    ]);
    assert_eq!(query(&place, "shop")["claims"], json!([]));
}

#[test]
fn nothing_stands() {
    let place = Place::new();
    place.made();
    let outcome = place.run(&["query", "--repository", "shop", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["claims"], json!([]));
    let words = place.run(&["query", "--repository", "shop"]);
    assert_eq!(words.code, 0, "{words:?}");
    assert_eq!(
        words.stdout,
        "No claim stands for shop, at entry 1 of the store's log.\n"
    );
}

#[test]
fn answered_as_json() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    standing(
        &place,
        &ada,
        "decision",
        "amounts-in-cents",
        CENTS,
        &["--repository", "shop"],
    );
    let outcome = place.run(&["query", "--repository", "shop", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-query/1");
    assert_eq!(answer["repository"], "shop");
    let last = place.log().pop().unwrap();
    assert_eq!(
        answer["head"],
        json!({"number": last["number"], "hash": last["hash"]})
    );
    assert_eq!(names(&answer), ["money-has-currency", "amounts-in-cents"]);
    assert_eq!(answer["claims"][0]["kind"], "invariant");
    assert_eq!(answer["claims"][1]["kind"], "decision");
}

#[test]
fn answered_in_words() {
    let place = Place::new();
    let (ada, money) = money_in_shop(&place);
    let body = place.file("cents.md", support::BODY);
    let cents = proposed(
        &place,
        "decision",
        "amounts-in-cents",
        CENTS,
        &["--body", &body, "--system"],
    );
    place.accepted("amounts-in-cents", &cents, &ada);
    let before = log_rows(&place);
    let outcome = place.run(&["query", "--repository", "shop"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stderr, "", "{outcome:?}");
    let last = before.last().unwrap().0;
    assert_eq!(
        outcome.stdout,
        format!(
            "2 claims stand for shop, at entry {last} of the store's log.\n\
             invariant money-has-currency@{money}: {MONEY}\n  - {SCENARIO}\n\
             decision amounts-in-cents@{cents}: {CENTS}\n  - {SCENARIO}\n\
             \x20 body: boom show amounts-in-cents@{cents}\n"
        )
    );
    assert!(!outcome.stdout.contains("An amount is"), "{outcome:?}");
    assert_eq!(log_rows(&place), before);
}

#[test]
fn the_same_log_the_same_answer() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    standing(
        &place,
        &ada,
        "decision",
        "amounts-in-cents",
        CENTS,
        &["--system"],
    );
    let before = log_rows(&place);
    let args = ["query", "--repository", "shop", "--json"];
    let first = place.run(&args);
    assert_eq!(first.code, 0, "{first:?}");
    place
        .sqlite()
        .execute_batch(
            "DROP TABLE views_head; DROP TABLE persons; DROP TABLE claims; \
             DROP TABLE versions; DROP TABLE judgments;",
        )
        .unwrap();
    let second = place.run(&args);
    assert_eq!(second.code, 0, "{second:?}");
    assert_eq!(second.stdout.as_bytes(), first.stdout.as_bytes());
    assert_eq!(log_rows(&place), before);
}

/// Runs `boom query` with no `--repository` in `cwd`, asserting it
/// refuses, writing nothing and appending nothing, and gives its words.
fn refused_in(place: &Place, cwd: &std::path::Path) -> String {
    let before = log_rows(place);
    let outcome = place.run_in(cwd, &["query"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(outcome.stdout, "", "{outcome:?}");
    assert_eq!(log_rows(place), before);
    outcome.stderr
}

#[test]
fn declared_by_the_repository() {
    let place = Place::new();
    money_in_shop(&place);
    place.repository("shop-repo", Some("repository = \"shop\"\n"));
    let inside = place.empty_folder("shop-repo/src/money");
    let answer = place.json_in(&inside, &["query"]);
    assert_eq!(answer["repository"], "shop");
    assert_eq!(names(&answer), ["money-has-currency"]);
    let words = place.run_in(&inside, &["query"]);
    assert!(
        words
            .stdout
            .starts_with("1 claim stands for shop, at entry "),
        "{words:?}"
    );
}

#[test]
fn named_outright() {
    let place = Place::new();
    money_in_shop(&place);
    let repo = place.repository("shop-repo", Some("repository = \"shop\"\n"));
    let answer = place.json_in(&repo, &["query", "--repository", "ledger"]);
    assert_eq!(answer["repository"], "ledger");
    assert_eq!(answer["claims"], json!([]));
}

#[test]
fn neither_named_nor_declared_is_refused() {
    let place = Place::new();
    money_in_shop(&place);
    let repo = place.repository("undeclared", None);
    let words = refused_in(&place, &repo);
    assert!(words.contains("--repository"), "{words}");
    assert!(words.contains(".config/boom.toml"), "{words}");
}

#[test]
fn a_declaration_above_the_repository_is_not_read() {
    let place = Place::new();
    money_in_shop(&place);
    let above = place.empty_folder("above");
    place.declare(&above, "repository = \"shop\"\n");
    let repo = place.repository("above/repo", None);
    let words = refused_in(&place, &repo);
    assert!(words.contains("--repository"), "{words}");
}

#[test]
fn outside_a_repository_no_declaration_is_read() {
    let place = Place::new();
    money_in_shop(&place);
    let outside = place.empty_folder("outside");
    place.declare(&outside, "repository = \"shop\"\n");
    let words = refused_in(&place, &outside);
    assert!(words.contains("--repository"), "{words}");
}

#[test]
fn a_declaration_it_cannot_read_is_refused() {
    let place = Place::new();
    money_in_shop(&place);
    let named = place.repository("named", Some("repository = \"Shop\"\n"));
    let file = named.join(".config/boom.toml");
    let words = refused_in(&place, &named);
    assert!(words.contains(file.to_str().unwrap()), "{words}");
    assert!(
        words.contains("a repository's name is kebab case"),
        "{words}"
    );
    let other = place.repository(
        "other",
        Some("repository = \"shop\"\ncontext = \"billing\"\n"),
    );
    let file = other.join(".config/boom.toml");
    let words = refused_in(&place, &other);
    assert!(words.contains(file.to_str().unwrap()), "{words}");
    assert!(words.contains("context"), "{words}");
}

#[test]
fn a_statement_edited_in_the_views_changes_nothing() {
    let place = Place::new();
    let (_, version) = money_in_shop(&place);
    let target = format!("money-has-currency@{version}");
    let edited = place
        .sqlite()
        .execute(
            "UPDATE versions SET statement = 'Any amount will do.' \
             WHERE name = 'money-has-currency' AND version = ?1",
            [&version],
        )
        .unwrap();
    assert_eq!(edited, 1);
    assert_eq!(
        place.show(&target)["statement"],
        "Any amount will do.",
        "the views were edited and left as fresh"
    );
    let answer = query(&place, "shop");
    assert_eq!(answer["claims"][0]["version"], version.as_str());
    assert_eq!(answer["claims"][0]["statement"], MONEY);
    let words = place.run(&["query", "--repository", "shop"]);
    assert!(words.stdout.contains(MONEY), "{words:?}");
    assert!(!words.stdout.contains("Any amount"), "{words:?}");
}

#[test]
fn a_status_edited_in_the_views_changes_nothing() {
    let place = Place::new();
    place.made();
    let version = proposed(
        &place,
        "invariant",
        "money-has-currency",
        MONEY,
        &["--repository", "shop"],
    );
    let conn = place.sqlite();
    let marked = conn
        .execute(
            "UPDATE versions SET status = 'standing', decided_by = 'ada', decided_in = 2 \
             WHERE name = 'money-has-currency' AND version = ?1",
            [&version],
        )
        .unwrap();
    assert_eq!(marked, 1);
    let standing = conn
        .execute(
            "UPDATE claims SET standing = ?1 WHERE name = 'money-has-currency'",
            [&version],
        )
        .unwrap();
    assert_eq!(standing, 1);
    drop(conn);
    assert_eq!(
        place.status("money-has-currency", &version),
        "standing",
        "the views were edited and left as fresh"
    );
    assert_eq!(query(&place, "shop")["claims"], json!([]));
}

#[test]
fn an_accept_written_by_hand_binds_nothing() {
    let place = Place::new();
    place.made();
    let version = proposed(
        &place,
        "invariant",
        "money-has-currency",
        MONEY,
        &["--repository", "shop"],
    );
    place.append_by_hand(
        &support::accept_entry("ada", "money-has-currency", &version),
        None,
    );
    let accepted = place.log().pop().unwrap();
    assert_eq!(accepted["act"], "accepted");
    assert_eq!(accepted["counted"], false);
    let answer = query(&place, "shop");
    assert_eq!(answer["claims"], json!([]));
    assert_eq!(answer["head"]["number"], accepted["number"]);
}
