//! What a version binds: the repositories its proposal names, or the
//! whole system; no part of the version; kept by an amendment; shown
//! before it is accepted; judged at the write and on every read
//! (claim-attachments).

mod support;

use serde_json::json;
use support::{Place, SCENARIO};

const MONEY: &str = "Every amount names its currency.";

/// An agent's proposal of `money-has-currency` in `MONEY`, with `extra`.
fn money<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "propose",
        "invariant",
        "money-has-currency",
        MONEY,
        "--scenario",
        SCENARIO,
        "--provenance",
        "run-1",
    ];
    args.extend_from_slice(extra);
    args
}

#[test]
fn bound_to_one_repository() {
    let place = Place::new();
    place.made();
    let answer = place.json(&money(&["--repository", "shop"]));
    assert_eq!(answer["repositories"], json!(["shop"]));
    let signed = place.log().pop().unwrap()["signed"]
        .as_str()
        .unwrap()
        .to_owned();
    let claim = signed.find(r#""claim":"#).unwrap();
    let attach = signed.find(r#""attach":"#).unwrap();
    assert!(claim < attach, "{signed}");
    assert!(
        signed.ends_with(r#","attach":{"repositories":["shop"]}}"#),
        "{signed}"
    );
    let words = place.run(&money(&["--repository", "shop"]));
    assert_eq!(words.code, 0, "{words:?}");
    assert!(words.stdout.contains("\nit binds shop\n"), "{words:?}");
}

#[test]
fn several_repositories_named_are_sorted_once() {
    let place = Place::new();
    place.made();
    let answer = place.json(&money(&[
        "--repository",
        "shop",
        "--repository",
        "ledger",
        "--repository",
        "shop",
    ]));
    assert_eq!(answer["repositories"], json!(["ledger", "shop"]));
    assert_eq!(
        place.last_signed()["attach"],
        json!({"repositories": ["ledger", "shop"]})
    );
}

#[test]
fn bound_to_the_whole_system() {
    let place = Place::new();
    place.made();
    let answer = place.json(&money(&["--system"]));
    assert_eq!(answer["repositories"], json!([]));
    assert!(place.last_signed().get("attach").is_none());
    let words = place.run(&money(&["--system"]));
    assert!(
        words.stdout.contains("\nit binds the whole system\n"),
        "{words:?}"
    );

    // No --repository, in a folder that declares none: the place's own,
    // and a repository with no declaration.
    let refund = |provenance| {
        vec![
            "propose",
            "invariant",
            "refund-within-payment",
            "A refund never exceeds the payment.",
            "--scenario",
            SCENARIO,
            "--provenance",
            provenance,
        ]
    };
    let answer = place.json(&refund("run-2"));
    assert_eq!(answer["repositories"], json!([]));
    assert!(place.last_signed().get("attach").is_none());
    let repo = place.repository("undeclared", None);
    let again = place.json_in(&repo, &refund("run-2"));
    assert_eq!(again, answer);
    assert_eq!(place.acts(), ["registered", "proposed", "proposed"]);
}

#[test]
fn the_repository_the_working_folder_declares() {
    let place = Place::new();
    place.made();
    let repo = place.repository("shop-repo", Some("repository = \"shop\"\n"));
    let inside = place.folder("shop-repo/src/money");
    let answer = place.json_in(&inside, &money(&[]));
    assert_eq!(answer["repositories"], json!(["shop"]));
    assert_eq!(
        place.last_signed()["attach"],
        json!({"repositories": ["shop"]})
    );
    assert_eq!(place.json_in(&repo, &money(&[])), answer);
}

#[test]
fn a_repository_name_not_in_kebab_case_is_refused() {
    let place = Place::new();
    place.made();
    place.refused(
        &money(&["--repository", "Shop_Front"]),
        "a repository's name is kebab case",
    );
}

#[test]
fn both_a_repository_and_the_whole_system_are_refused() {
    let place = Place::new();
    place.made();
    place.refused(&money(&["--repository", "shop", "--system"]), "--system");
}

#[test]
fn a_repository_named_twice_by_hand_counts_for_nothing() {
    let place = Place::new();
    place.made();
    let entry = boom::entry::Entry {
        at: 1_763_424_600,
        actor: boom::entry::Actor::Agent {
            provenance: "another-program".to_owned(),
        },
        act: boom::entry::Act::Proposed {
            claim: boom::claim::Proposal::new(
                boom::claim::Kind::Invariant,
                "money-has-currency",
                MONEY,
                vec![SCENARIO.to_owned()],
                None,
                None,
            ),
            attach: Some(boom::entry::Attach {
                repositories: vec!["shop".to_owned(), "shop".to_owned()],
            }),
        },
    };
    place.append_by_hand(&entry, None);
    let proposed = place.log().pop().unwrap();
    assert_eq!(proposed["act"], "proposed");
    assert_eq!(proposed["counted"], false);
    assert_eq!(
        proposed["reason"],
        "an attach names each repository once: shop is named twice"
    );
    let shown = place.run(&["show", "money-has-currency"]);
    assert_eq!(shown.code, 2, "{shown:?}");
    assert!(
        shown
            .stderr
            .contains("no claim has the name money-has-currency"),
        "{shown:?}"
    );
}
