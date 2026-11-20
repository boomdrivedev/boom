//! What a version binds: the repositories its proposal names, or the
//! whole system; no part of the version; kept by an amendment; shown
//! before it is accepted; judged at the write and on every read
//! (claim-attachments).

mod support;

use serde_json::json;
use support::Place;

const MONEY: &str = "Every amount names its currency.";

/// An agent's proposal of `money-has-currency` in `MONEY`, with `extra`.
fn money<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "propose",
        "invariant",
        "money-has-currency",
        MONEY,
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
    let inside = place.empty_folder("shop-repo/src/money");
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
                Vec::new(),
                None,
                None,
                None,
            ),
            imported: None,
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

#[test]
fn one_fingerprint_wherever_it_binds() {
    let shop = Place::new();
    shop.made();
    let system = Place::new();
    system.made();
    let bound = shop.json(&money(&["--repository", "shop"]));
    let whole = system.json(&money(&["--system"]));
    assert_eq!(bound["repositories"], json!(["shop"]));
    assert_eq!(whole["repositories"], json!([]));
    assert_eq!(bound["version"], whole["version"]);
    let unbound = boom::claim::Proposal::new(
        boom::claim::Kind::Invariant,
        "money-has-currency",
        MONEY,
        Vec::new(),
        None,
        None,
        None,
    );
    assert_eq!(bound["version"], unbound.version.as_str());
}

#[test]
fn proposed_again_for_the_same_repository_adds_nothing() {
    let place = Place::new();
    place.made();
    let first = place.json(&money(&["--repository", "shop"]));
    let second = place.json(&money(&["--repository", "shop"]));
    assert_eq!(second, first);
    assert_eq!(place.acts(), ["registered", "proposed"]);
}

#[test]
fn the_same_words_for_another_repository_are_refused() {
    let place = Place::new();
    place.made();
    let first = place.json(&money(&["--repository", "shop"]));
    let version = first["version"].as_str().unwrap();
    let outcome = place.refused(
        &money(&["--repository", "ledger"]),
        &format!("money-has-currency@{version} is attached to shop"),
    );
    assert!(
        outcome
            .stderr
            .contains("a version's attachment does not move"),
        "{outcome:?}"
    );
    place.refused(&money(&["--system"]), "is attached to shop");
    assert_eq!(place.acts(), ["registered", "proposed"]);

    // Written by hand under another attachment, it counts for nothing.
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
                Vec::new(),
                None,
                None,
                None,
            ),
            imported: None,
            attach: Some(boom::entry::Attach {
                repositories: vec!["ledger".to_owned()],
            }),
        },
    };
    place.append_by_hand(&entry, None);
    let moved = place.log().pop().unwrap();
    assert_eq!(moved["counted"], false);
    assert_eq!(
        moved["reason"],
        format!(
            "money-has-currency@{version} is attached to shop; a version's attachment does not move"
        )
    );
    let shown = place.show("money-has-currency");
    assert_eq!(shown["versions"].as_array().unwrap().len(), 1);
}

/// A version of `money-has-currency` attached to `shop`, standing,
/// accepted by ada; gives its version.
fn standing_in_shop(place: &Place) -> String {
    let ada = place.made();
    let first = place.json(&money(&["--repository", "shop"]));
    let version = first["version"].as_str().unwrap().to_owned();
    place.accepted("money-has-currency", &version, &ada);
    version
}

/// An agent's amendment of `money-has-currency`'s `base`, with `extra`.
fn amending<'a>(base: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "propose",
        "invariant",
        "money-has-currency",
        "Every amount names exactly one currency.",
        "--amends",
        base,
        "--provenance",
        "run-2",
    ];
    args.extend_from_slice(extra);
    args
}

#[test]
fn amended_with_nothing_named_keeps_its_bases() {
    let place = Place::new();
    let base = standing_in_shop(&place);
    let ledger = place.repository("ledger-repo", Some("repository = \"ledger\"\n"));
    let amendment = place.json_in(&ledger, &amending(&base, &[]));
    assert_eq!(amendment["base"], base.as_str());
    assert_eq!(amendment["repositories"], json!(["shop"]));
    assert_eq!(
        place.last_signed()["attach"],
        json!({"repositories": ["shop"]})
    );
}

#[test]
fn amended_for_another_repository() {
    let place = Place::new();
    let base = standing_in_shop(&place);
    let amendment = place.json(&amending(&base, &["--repository", "ledger"]));
    assert_eq!(amendment["base"], base.as_str());
    assert_eq!(amendment["repositories"], json!(["ledger"]));
    assert_eq!(
        place.last_signed()["attach"],
        json!({"repositories": ["ledger"]})
    );
}

#[test]
fn a_version_shown_gives_what_it_binds() {
    let place = Place::new();
    place.made();
    let answer = place.json(&money(&["--repository", "shop"]));
    let target = format!("money-has-currency@{}", answer["version"].as_str().unwrap());
    let shown = place.show(&target);
    assert_eq!(shown["repositories"], json!(["shop"]));
    assert_eq!(shown["statement"], MONEY);
    assert_eq!(shown["status"], "proposed");
    let words = place.run(&["show", &target]);
    assert_eq!(words.code, 0, "{words:?}");
    assert!(words.stdout.contains("\nbinds: shop\n"), "{words:?}");
    let schema: i64 = place
        .sqlite()
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(schema, 1);
}

#[test]
fn a_claim_shown_gives_each_versions_repositories() {
    let place = Place::new();
    let ada = place.made();
    let first = place.json(&money(&["--system"]));
    let base = first["version"].as_str().unwrap().to_owned();
    place.accepted("money-has-currency", &base, &ada);
    let amendment = place.json(&amending(&base, &["--repository", "shop"]));
    let shown = place.show("money-has-currency");
    let versions = shown["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0]["version"], base.as_str());
    assert_eq!(versions[0]["repositories"], json!([]));
    assert_eq!(versions[1]["version"], amendment["version"]);
    assert_eq!(versions[1]["repositories"], json!(["shop"]));
    let words = place.run(&["show", "money-has-currency"]);
    assert_eq!(words.code, 0, "{words:?}");
    assert!(
        words.stdout.contains("    binds: the whole system\n"),
        "{words:?}"
    );
    assert!(words.stdout.contains("    binds: shop\n"), "{words:?}");
}
