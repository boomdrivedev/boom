//! What an agent finds by searching the words of the claims that stand,
//! in the repository it works in or every repository: only what stands,
//! as the person accepted it, replayed from the log for each answer
//! with no index kept, and ranked (claim-search).
//!
//! Each claim a test proposes takes a scenario and a body written so
//! that only the fields its scenario names hold the words it searches
//! (design 8): the tests' `SCENARIO` holds `amount` and `currency`, so
//! these take `HOLDS` unless a test names another.

mod support;

use serde_json::{json, Value};
use support::Place;

/// A scenario holding none of the words the tests search.
const HOLDS: &str = "Given it, then it holds.";
const MONEY: &str = "Every amount names its currency.";
const CENTS: &str = "Amounts are kept in integer cents.";
const SHOP: &[&str] = &["--repository", "shop"];

/// A first version of a claim, as a test proposes it.
#[derive(Clone, Copy)]
struct Claim<'a> {
    kind: &'a str,
    name: &'a str,
    statement: &'a str,
    body: Option<&'a str>,
    /// What it binds, as `boom propose` takes it.
    binds: &'a [&'a str],
}

impl<'a> Claim<'a> {
    /// A claim of `kind` for `shop`, with no body.
    fn new(kind: &'a str, name: &'a str, statement: &'a str) -> Self {
        Claim {
            kind,
            name,
            statement,
            body: None,
            binds: SHOP,
        }
    }

    fn invariant(name: &'a str, statement: &'a str) -> Self {
        Claim::new("invariant", name, statement)
    }

    fn decision(name: &'a str, statement: &'a str) -> Self {
        Claim::new("decision", name, statement)
    }

    fn body(self, body: &'a str) -> Self {
        Claim {
            body: Some(body),
            ..self
        }
    }

    fn binds(self, binds: &'a [&'a str]) -> Self {
        Claim { binds, ..self }
    }

    /// An agent's proposal of it, giving its version.
    fn proposed(self, place: &Place) -> String {
        // A decision needs a body: a builder that gives none gives it
        // the shop's decision.
        let body = self
            .body
            .or((self.kind == "decision").then_some(support::DECISION))
            .map(|text| place.file(&format!("{}.md", self.name), text));
        let mut args = vec![
            "propose",
            self.kind,
            self.name,
            self.statement,
            "--provenance",
            "run-1",
        ];
        if let Some(body) = &body {
            args.extend(["--body", body]);
        }
        args.extend_from_slice(self.binds);
        place.json(&args)["version"].as_str().unwrap().to_owned()
    }

    /// Its proposal accepted by `key`, giving its version.
    fn standing(self, place: &Place, key: &str) -> String {
        let version = self.proposed(place);
        place.accepted(self.name, &version, key);
        version
    }
}

/// `boom search <words> --repository shop --json`'s answer.
fn search(place: &Place, words: &[&str]) -> Value {
    let mut args = vec!["search"];
    args.extend_from_slice(words);
    args.extend_from_slice(SHOP);
    place.json(&args)
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

/// The invariant `money-has-currency`, "Every amount names its
/// currency.", standing for `shop`; gives ada's key and its version.
fn money_in_shop(place: &Place) -> (String, String) {
    let ada = place.made();
    let version = Claim::invariant("money-has-currency", MONEY).standing(place, &ada);
    (ada, version)
}

const ROUNDING: &str = "# Amounts in cents\n\nAmounts are kept as integers of the smallest unit, \
                        so no rounding ever loses a cent between two ledgers.\n";

#[test]
fn found_by_its_statement() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    Claim::decision("amounts-in-cents", CENTS).standing(&place, &ada);
    assert_eq!(
        names(&search(&place, &["currency"])),
        ["money-has-currency"]
    );
    assert_eq!(names(&search(&place, &["names"])), ["money-has-currency"]);
}

#[test]
fn found_by_its_name() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    Claim::decision("amounts-in-cents", CENTS).standing(&place, &ada);
    assert_eq!(names(&search(&place, &["money"])), ["money-has-currency"]);
}

#[test]
fn found_by_an_old_versions_scenarios() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    // An old entry, written before claims lost their scenarios, keeps
    // them, and a search finds its words there.
    let old = boom::claim::Proposal::new(
        boom::claim::Kind::Invariant,
        "orders-are-paid-once",
        "An order is paid once.",
        vec!["Given a payment, then the ledger records it once.".to_owned()],
        None,
        None,
        None,
    );
    place.append_by_hand(
        &boom::entry::Entry {
            at: 1_763_424_600,
            rules: boom::judge::RULES,
            actor: boom::entry::Actor::Agent {
                provenance: "an-old-boom".to_owned(),
            },
            act: boom::entry::Act::Proposed {
                claim: Box::new(old.clone()),
                imported: None,
                attach: Some(boom::entry::Attach {
                    repositories: vec!["shop".to_owned()],
                    contexts: Vec::new(),
                }),
            },
        },
        None,
    );
    place.accepted("orders-are-paid-once", &old.version, &ada);
    assert_eq!(
        names(&search(&place, &["ledger"])),
        ["orders-are-paid-once"]
    );
}

#[test]
fn found_in_its_body() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    Claim::decision("amounts-in-cents", CENTS)
        .body(ROUNDING)
        .standing(&place, &ada);
    let answer = search(&place, &["rounding"]);
    assert_eq!(names(&answer), ["amounts-in-cents"]);
    let excerpt = answer["claims"][0]["excerpt"].as_str().unwrap();
    assert!(excerpt.contains("rounding"), "{answer}");
}

#[test]
fn every_word_is_found() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    Claim::decision("amounts-in-cents", CENTS).standing(&place, &ada);
    assert_eq!(
        names(&search(&place, &["amount", "currency"])),
        ["money-has-currency"]
    );
    let mut both = names(&search(&place, &["amount"]));
    both.sort();
    assert_eq!(both, ["amounts-in-cents", "money-has-currency"]);
}

#[test]
fn another_form_of_a_word() {
    let place = Place::new();
    money_in_shop(&place);
    assert_eq!(
        names(&search(&place, &["AMOUNTS", "Currencies"])),
        ["money-has-currency"]
    );
}

#[test]
fn a_word_written_with_accents() {
    let place = Place::new();
    let ada = place.made();
    Claim::decision("dates-name-their-zone", "A naive date names its zone.").standing(&place, &ada);
    assert_eq!(
        names(&search(&place, &["naïve"])),
        ["dates-name-their-zone"]
    );
}

#[test]
fn words_not_a_query_language() {
    let place = Place::new();
    money_in_shop(&place);
    let answer = search(&place, &["currency*", "\"amount", "OR"]);
    assert_eq!(answer["claims"], json!([]));
    assert_eq!(answer["total"], 0);
    assert_eq!(
        names(&search(&place, &["currency*", "\"amount"])),
        ["money-has-currency"]
    );
    for words in [&["(amount", "currency)"][..], &["amount:", "^currency"]] {
        assert_eq!(names(&search(&place, words)), ["money-has-currency"]);
    }
}

#[test]
fn no_word_to_search_is_refused() {
    let place = Place::new();
    money_in_shop(&place);
    let outcome = place.refused(
        &["search", "***", "--repository", "shop"],
        "a word holding a letter or a digit",
    );
    assert_eq!(outcome.stdout, "", "{outcome:?}");
}

/// A rules page for `shop` whose two rules stand under `### The
/// ledger`, neither rule's other words holding `core` or `ledger`.
const RULES: &str = "# shop rules

## Rules

### The ledger

- **[core.1]** A record's status is read from its events, never stored
  on the record.
- **[core.2]** A view is computed from the events and never stored as
  the truth.
";

/// The rules page imported for `shop` with a drafts file naming each
/// rule and giving its scenario, and both accepted by ada.
fn rules_in_shop(place: &Place) {
    let ada = place.made();
    let page = place.folder("rules", &[("core.md", RULES)]);
    let drafts = place.file(
        "drafts.json",
        &json!({
            "format": "boom-import-drafts/2",
            "records": {
                "core.1": {"name": "status-from-events"},
                "core.2": {"name": "views-computed"},
            },
        })
        .to_string(),
    );
    import_and_accept(place, &ada, &["rules", &format!("{page}/core.md")], &drafts);
}

/// `boom import <what> --repository shop` with `drafts`, and every
/// version it proposes accepted together by `key`.
fn import_and_accept(place: &Place, key: &str, what: &[&str], drafts: &str) {
    let mut args = vec!["import"];
    args.extend_from_slice(what);
    args.extend(["--drafts", drafts, "--provenance", "run-1"]);
    args.extend_from_slice(SHOP);
    let answer = place.json(&args);
    let versions = answer["accept"]["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2, "{answer}");
    let mut accept = vec!["accept"];
    accept.extend(versions.iter().map(|version| version.as_str().unwrap()));
    accept.extend(["--key", key]);
    place.json(&accept);
}

#[test]
fn found_by_its_legacy_id() {
    let place = Place::new();
    rules_in_shop(&place);
    let answer = search(&place, &["core.2"]);
    assert_eq!(names(&answer), ["views-computed"]);
    assert_eq!(answer["claims"][0]["legacy"], "core.2");
    assert_eq!(answer["claims"][0]["kind"], "invariant");
    let mut both = names(&search(&place, &["core"]));
    both.sort();
    assert_eq!(both, ["status-from-events", "views-computed"]);
}

#[test]
fn found_by_its_section() {
    let place = Place::new();
    rules_in_shop(&place);
    let mut both = names(&search(&place, &["ledger"]));
    both.sort();
    assert_eq!(both, ["status-from-events", "views-computed"]);
}

const AMOUNTS_AS_FLOATS: &str = "---
status: superseded by [ADR-0003](0003-amounts-in-cents.md)
---

# Amounts are floats

Amounts are kept as floats.
";

const AMOUNTS_IN_CENTS: &str = "---
status: accepted
---

# Amounts are kept in cents

Every amount is an integer of cents.
";

#[test]
fn a_version_that_came_in_as_history_is_not_given() {
    let place = Place::new();
    let ada = place.made();
    let dir = place.folder(
        "adr",
        &[
            ("0002-amounts-as-floats.md", AMOUNTS_AS_FLOATS),
            ("0003-amounts-in-cents.md", AMOUNTS_IN_CENTS),
        ],
    );
    let drafts = place.file(
        "drafts.json",
        &json!({
            "format": "boom-import-drafts/2",
            "records": {"0003-amounts-in-cents.md": {}},
        })
        .to_string(),
    );
    import_and_accept(&place, &ada, &["adr", &dir], &drafts);
    assert_eq!(
        place.show("amounts-as-floats")["versions"][0]["status"],
        "superseded"
    );
    let answer = search(&place, &["floats"]);
    assert_eq!(answer["claims"], json!([]));
    assert_eq!(answer["total"], 0);
    let answer = search(&place, &["amounts"]);
    assert_eq!(names(&answer), ["amounts-in-cents"]);
    assert_eq!(answer["claims"][0]["legacy"], "0003");
}

/// Invariants holding `refund` standing, one attached to `shop`, one to
/// the whole system and one to `ledger`.
fn refunds(place: &Place) {
    let ada = place.made();
    Claim::invariant("refund-in-shop", "A refund in a shop is paid back.").standing(place, &ada);
    Claim::invariant("refund-everywhere", "A refund is never negative.")
        .binds(&["--system"])
        .standing(place, &ada);
    Claim::invariant("refund-in-ledger", "A refund in a ledger is a line.")
        .binds(&["--repository", "ledger"])
        .standing(place, &ada);
}

/// The names an answer gives, sorted.
fn sorted(answer: &Value) -> Vec<String> {
    let mut names = names(answer);
    names.sort();
    names
}

/// Runs `args` in `cwd`, asserting it refuses, writing nothing to
/// standard output and appending nothing, and gives its words.
fn refused_in(place: &Place, cwd: &std::path::Path, args: &[&str]) -> String {
    let before = place.rows();
    let outcome = place.run_in(cwd, args);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert_eq!(outcome.stdout, "", "{outcome:?}");
    assert_eq!(place.rows(), before);
    outcome.stderr
}

#[test]
fn in_the_repository_it_works_in() {
    let place = Place::new();
    refunds(&place);
    let answer = search(&place, &["refund"]);
    assert_eq!(answer["repository"], "shop");
    assert_eq!(sorted(&answer), ["refund-everywhere", "refund-in-shop"]);
}

#[test]
fn declared_by_the_repository() {
    let place = Place::new();
    refunds(&place);
    place.repository("shop-repo", Some(support::SHOP_REMOTE));
    place.repo_added(&place.key("ada", 1), "shop", support::SHOP_REMOTE);
    let inside = place.empty_folder("shop-repo/src/refunds");
    let answer = place.json_in(&inside, &["search", "refund"]);
    assert_eq!(answer["repository"], "shop");
    assert_eq!(sorted(&answer), ["refund-everywhere", "refund-in-shop"]);
    let words = place.run_in(&inside, &["search", "refund"]);
    assert!(
        words
            .stdout
            .starts_with("2 claims match \"refund\" for shop, at entry "),
        "{words:?}"
    );
}

#[test]
fn across_every_repository() {
    let place = Place::new();
    refunds(&place);
    let answer = place.json(&["search", "refund", "--all"]);
    assert_eq!(answer["repository"], Value::Null);
    assert_eq!(
        sorted(&answer),
        ["refund-everywhere", "refund-in-ledger", "refund-in-shop"]
    );
    let binds = |name: &str| {
        answer["claims"]
            .as_array()
            .unwrap()
            .iter()
            .find(|claim| claim["name"] == name)
            .unwrap()["repositories"]
            .clone()
    };
    assert_eq!(binds("refund-in-shop"), json!(["shop"]));
    assert_eq!(binds("refund-everywhere"), json!([]));
    assert_eq!(binds("refund-in-ledger"), json!(["ledger"]));
    let words = place.run(&["search", "refund", "--all"]);
    assert!(
        words
            .stdout
            .starts_with("3 claims match \"refund\" across every repository, at entry "),
        "{words:?}"
    );
    assert!(
        words.stdout.contains(", for the whole system: "),
        "{words:?}"
    );
}

#[test]
fn neither_named_declared_nor_all_is_refused() {
    let place = Place::new();
    refunds(&place);
    let repo = place.repository("undeclared", None);
    let words = refused_in(&place, &repo, &["search", "refund"]);
    assert!(words.contains("--repository"), "{words}");
    assert!(words.contains("boom repo add"), "{words}");
    assert!(words.contains("--all"), "{words}");
}

#[test]
fn a_repository_and_every_repository_are_refused() {
    let place = Place::new();
    refunds(&place);
    let repo = place.repository("shop-repo", Some(support::SHOP_REMOTE));
    let words = refused_in(
        &place,
        &repo,
        &["search", "refund", "--repository", "shop", "--all"],
    );
    assert!(words.contains("--all"), "{words}");
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

#[test]
fn a_proposal_is_not_given() {
    let place = Place::new();
    money_in_shop(&place);
    Claim::invariant(
        "refund-within-payment",
        "A refund never exceeds the payment.",
    )
    .proposed(&place);
    let answer = search(&place, &["refund"]);
    assert_eq!(answer["claims"], json!([]));
    assert_eq!(answer["total"], 0);
}

#[test]
fn an_amendment_waiting_gives_the_standing_words() {
    let place = Place::new();
    let ada = place.made();
    let base = Claim::decision("amounts-in-cents", CENTS).standing(&place, &ada);
    let amendment = place.json(&[
        "propose",
        "decision",
        "amounts-in-cents",
        "Amounts are kept in integer mills.",
        "--amends",
        &base,
        "--body",
        &place.decision_body(),
        "--provenance",
        "run-2",
    ]);
    assert_eq!(amendment["status"], "proposed");
    assert_eq!(amendment["repositories"], json!(["shop"]));
    assert_eq!(search(&place, &["mills"])["claims"], json!([]));
    let answer = search(&place, &["cents"]);
    assert_eq!(names(&answer), ["amounts-in-cents"]);
    assert_eq!(answer["claims"][0]["version"], base.as_str());
    assert_eq!(answer["claims"][0]["statement"], CENTS);
}

#[test]
fn a_statement_edited_in_the_views_is_built_again_by_verify() {
    let place = Place::new();
    let (_, version) = money_in_shop(&place);
    let edited = place
        .sqlite()
        .execute(
            "UPDATE versions SET statement = 'Every amount is forged.' \
             WHERE name = 'money-has-currency' AND version = ?1",
            [&version],
        )
        .unwrap();
    assert_eq!(edited, 1);
    assert_eq!(
        place.show(&format!("money-has-currency@{version}"))["statement"],
        "Every amount is forged.",
        "the views were edited and left as fresh"
    );
    let verified = place.json(&["verify"]);
    assert_eq!(verified["rebuilt"], json!(["versions"]));
    assert_eq!(search(&place, &["forged"])["claims"], json!([]));
    let answer = search(&place, &["currency"]);
    assert_eq!(names(&answer), ["money-has-currency"]);
    assert_eq!(answer["claims"][0]["version"], version.as_str());
    assert_eq!(answer["claims"][0]["statement"], MONEY);
}

#[test]
fn an_altered_body_stops_the_search() {
    let place = Place::new();
    let ada = place.made();
    Claim::decision("amounts-in-cents", CENTS)
        .body(ROUNDING)
        .standing(&place, &ada);
    let fingerprint = boom::sha256_hex(ROUNDING.as_bytes());
    let altered = place
        .sqlite()
        .execute(
            "UPDATE blobs SET bytes = ?1 WHERE fingerprint = ?2",
            rusqlite::params![b"Amounts are floats.".to_vec(), fingerprint],
        )
        .unwrap();
    assert_eq!(altered, 1);
    let before = log_rows(&place);
    let outcome = place.run(&["search", "integer", "--repository", "shop"]);
    assert_eq!(outcome.code, 3, "{outcome:?}");
    assert_eq!(outcome.stdout, "", "{outcome:?}");
    assert!(outcome.stderr.contains(&fingerprint), "{outcome:?}");
    assert_eq!(log_rows(&place), before);
}

#[test]
fn nothing_is_kept() {
    let place = Place::new();
    refunds(&place);
    let tables = || -> Vec<(String, String)> {
        let conn = place.sqlite();
        let mut statement = conn
            .prepare("SELECT type, name FROM sqlite_master ORDER BY type, name")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let (before, rows) = (tables(), log_rows(&place));
    let outcome = place.run(&["search", "refund", "--repository", "shop"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(tables(), before);
    assert_eq!(log_rows(&place), rows);
    assert!(
        !before.iter().any(|(_, name)| name.contains("found")),
        "{before:?}"
    );
}

#[test]
fn a_name_above_a_body() {
    let place = Place::new();
    let ada = place.made();
    Claim::invariant(
        "refund-never-exceeds-payment",
        "A refund never exceeds the payment.",
    )
    .standing(&place, &ada);
    Claim::decision("orders-are-paid-once", "An order is paid once.")
        .body("An order is paid once, and a refund is paid against it.\n")
        .standing(&place, &ada);
    assert_eq!(
        names(&search(&place, &["refund"])),
        ["refund-never-exceeds-payment", "orders-are-paid-once"]
    );
    // Above the order of kinds: a decision's name above an invariant's
    // body.
    Claim::invariant("payments-are-recorded", "A payment is recorded.")
        .body("A payment is recorded, and a rebate is recorded against it.\n")
        .standing(&place, &ada);
    Claim::decision("rebate-is-capped", "A rebate is capped.").standing(&place, &ada);
    assert_eq!(
        names(&search(&place, &["rebate"])),
        ["rebate-is-capped", "payments-are-recorded"]
    );
}

/// The decisions `beta-kept` and `alpha-kept` and the invariant
/// `gamma-kept` standing for `shop`, each with the same scenario and no
/// body.
fn kept(place: &Place) {
    let ada = place.made();
    Claim::decision("beta-kept", "Beta is kept.").standing(place, &ada);
    Claim::decision("alpha-kept", "Alpha is kept.").standing(place, &ada);
    Claim::invariant("gamma-kept", "Gamma is kept.").standing(place, &ada);
}

#[test]
fn equal_matches_by_kind_and_name() {
    let place = Place::new();
    kept(&place);
    assert_eq!(
        names(&search(&place, &["kept"])),
        ["gamma-kept", "alpha-kept", "beta-kept"]
    );
}

#[test]
fn limited() {
    let place = Place::new();
    kept(&place);
    let answer = search(&place, &["kept", "--limit", "2"]);
    assert_eq!(answer["total"], 3);
    assert_eq!(names(&answer), ["gamma-kept", "alpha-kept"]);
    let words = place.run(&["search", "kept", "--repository", "shop", "--limit", "2"]);
    assert_eq!(words.code, 0, "{words:?}");
    let first = words.stdout.lines().next().unwrap();
    assert!(
        first.starts_with("3 claims match \"kept\" for shop, at entry "),
        "{words:?}"
    );
    assert!(
        first.ends_with(" of the store's log; the first 2 follow."),
        "{words:?}"
    );
    // The header, two claims, and the decision's excerpt and body.
    assert_eq!(words.stdout.lines().count(), 5, "{words:?}");
    assert_eq!(
        search(&place, &["kept"])["claims"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn a_limit_of_none_is_refused() {
    let place = Place::new();
    kept(&place);
    let outcome = place.refused(
        &["search", "kept", "--repository", "shop", "--limit", "0"],
        "--limit",
    );
    assert_eq!(outcome.stdout, "", "{outcome:?}");
}

#[test]
fn answered_as_json() {
    let place = Place::new();
    let ada = place.made();
    let version = Claim::invariant("money-has-currency", MONEY).proposed(&place);
    let accepted = place.accepted("money-has-currency", &version, &ada);
    let outcome = place.run(&["search", "amount", "--repository", "shop", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    let last = place.log().pop().unwrap();
    assert_eq!(
        answer,
        json!({
            "format": "boom-search/2",
            "words": ["amount"],
            "repository": "shop",
            "head": {"number": last["number"], "hash": last["hash"]},
            "total": 1,
            "claims": [{
                "name": "money-has-currency",
                "kind": "invariant",
                "version": version,
                "statement": MONEY,
                "scenarios": [],
                "body": null,
                "excerpt": null,
                "legacy": null,
                "repositories": ["shop"],
                "contexts": [],
                "accepted": {"by": "ada", "entry": accepted["entry"]},
            }],
        })
    );
    let dropped = place.json(&["search", "***", "amount", "--repository", "shop"]);
    assert_eq!(dropped["words"], json!(["amount"]));
}

#[test]
fn answered_in_words() {
    let place = Place::new();
    let ada = place.made();
    let version = Claim::decision("amounts-in-cents", CENTS)
        .body(ROUNDING)
        .standing(&place, &ada);
    let excerpt = search(&place, &["rounding"])["claims"][0]["excerpt"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(excerpt.contains("rounding"), "{excerpt}");
    let before = log_rows(&place);
    let last = before.last().unwrap().0;
    let outcome = place.run(&["search", "rounding", "--repository", "shop"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stderr, "", "{outcome:?}");
    assert_eq!(
        outcome.stdout,
        format!(
            "1 claim matches \"rounding\" for shop, at entry {last} of the store's log.\n\
             decision amounts-in-cents@{version}, for shop: {CENTS}\n\
             \x20 excerpt: {excerpt}\n\
             \x20 body: boom show amounts-in-cents@{version}\n"
        )
    );
    assert!(!outcome.stdout.contains(HOLDS), "{outcome:?}");
    assert_eq!(log_rows(&place), before);
}

#[test]
fn nothing_matches() {
    let place = Place::new();
    money_in_shop(&place);
    let outcome = place.run(&["search", "parsnip", "--repository", "shop", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    let answer: Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["total"], 0);
    assert_eq!(answer["claims"], json!([]));
    let last = log_rows(&place).last().unwrap().0;
    let words = place.run(&["search", "parsnip", "--repository", "shop"]);
    assert_eq!(words.code, 0, "{words:?}");
    assert_eq!(
        words.stdout,
        format!("No claim matches \"parsnip\" for shop, at entry {last} of the store's log.\n")
    );
}

#[test]
fn the_same_log_the_same_answer() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    Claim::decision("amounts-in-cents", CENTS)
        .body(ROUNDING)
        .binds(&["--system"])
        .standing(&place, &ada);
    let before = log_rows(&place);
    let args = ["search", "amount", "--repository", "shop", "--json"];
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
