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
    scenario: &'a str,
    body: Option<&'a str>,
    /// What it binds, as `boom propose` takes it.
    binds: &'a [&'a str],
}

impl<'a> Claim<'a> {
    /// A claim of `kind` for `shop`, with the scenario `HOLDS` and no
    /// body.
    fn new(kind: &'a str, name: &'a str, statement: &'a str) -> Self {
        Claim {
            kind,
            name,
            statement,
            scenario: HOLDS,
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

    fn scenario(self, scenario: &'a str) -> Self {
        Claim { scenario, ..self }
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
        let body = self
            .body
            .map(|text| place.file(&format!("{}.md", self.name), text));
        let mut args = vec![
            "propose",
            self.kind,
            self.name,
            self.statement,
            "--scenario",
            self.scenario,
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
fn found_by_its_scenarios() {
    let place = Place::new();
    let (ada, _) = money_in_shop(&place);
    Claim::decision("orders-are-paid-once", "An order is paid once.")
        .scenario("Given a payment, then the ledger records it once.")
        .standing(&place, &ada);
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
