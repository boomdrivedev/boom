//! The views kept in step with the log: a command reads the views' head
//! entry and the entries after it, never the log before it, so it does
//! not grow with the log; and the views always hold what a full replay
//! of the log builds (record-store, "A command answers from the views";
//! repository-rules, "The views and a full replay agree").

mod support;

use boom::claim::{Kind, Proposal};
use boom::entry::{Act, Actor, Attach, Entry};
use boom::store::{self, Append, Store};
use support::Place;

const MONEY: &str = "Every amount names its currency.";

/// Appends `count` agents' proposals of first versions bound to the
/// repository `other`, through the library, in batches.
fn proposals_for_other(place: &Place, from: usize, count: usize) {
    let mut store = Store::open(&place.store).unwrap();
    let mut made = from;
    while made < from + count {
        let batch = (from + count - made).min(1000);
        let signed: Vec<String> = (made..made + batch)
            .map(|at| {
                Entry {
                    at: 1_763_424_600,
                    rules: boom::judge::RULES,
                    actor: Actor::Agent {
                        provenance: "a-large-log".to_owned(),
                    },
                    act: Act::Proposed {
                        claim: Box::new(Proposal::new(
                            Kind::Invariant,
                            &format!("other-claim-{at}"),
                            &format!("Other claim number {at} holds."),
                            Vec::new(),
                            None,
                            None,
                            None,
                        )),
                        imported: None,
                        attach: Some(Attach {
                            repositories: vec!["other".to_owned()],
                            contexts: Vec::new(),
                        }),
                    },
                }
                .signed_part()
            })
            .collect();
        let appends: Vec<Append<'_>> = signed
            .iter()
            .map(|signed| Append {
                signed,
                signature: None,
                blobs: &[],
            })
            .collect();
        store.append_all(&appends).unwrap();
        made += batch;
    }
}

/// A store with ada, two claims standing for `shop`, and `other` more
/// entries proposing claims for `other`.
fn store_of(other: usize) -> Place {
    let place = Place::new();
    let ada = place.made();
    for (name, statement) in [
        ("money-has-currency", MONEY),
        ("refunds-name-their-order", "Every refund names its order."),
    ] {
        let answer = place.json(&[
            "propose",
            "invariant",
            name,
            statement,
            "--repository",
            "shop",
            "--provenance",
            "run-1",
        ]);
        let version = answer["version"].as_str().unwrap().to_owned();
        place.accepted(name, &version, &ada);
    }
    proposals_for_other(&place, 0, other);
    // One command brings the views to the log's last entry.
    place.json(&["query", "--repository", "shop"]);
    place
}

/// The entries of the log one command reads, and its outcome.
fn reads(place: &Place, args: &[&str]) -> (u64, boom::cli::Outcome) {
    let before = store::entries_read();
    let outcome = place.run(args);
    (store::entries_read() - before, outcome)
}

/// Appends three agents' proposals to the log as another program does,
/// leaving the views as they were.
fn three_by_hand(place: &Place, from: usize) {
    for at in from..from + 3 {
        let entry = Entry {
            at: 1_763_424_700,
            rules: boom::judge::RULES,
            actor: Actor::Agent {
                provenance: "another-program".to_owned(),
            },
            act: Act::Proposed {
                claim: Box::new(Proposal::new(
                    Kind::Invariant,
                    &format!("by-hand-{at}"),
                    "A claim another program proposed.",
                    Vec::new(),
                    None,
                    None,
                    None,
                )),
                imported: None,
                attach: None,
            },
        };
        place.append_by_hand(&entry, None);
    }
}

#[test]
fn a_command_does_not_grow_with_the_log() {
    let small = store_of(10);
    let large = store_of(20_000);
    let log_size = |place: &Place| -> i64 {
        place
            .sqlite()
            .query_row("SELECT count(*) FROM log", [], |row| row.get(0))
            .unwrap()
    };
    assert!(log_size(&large) > 20_000);

    let query = ["query", "--repository", "shop", "--json"];
    let (small_reads, small_answer) = reads(&small, &query);
    let (large_reads, large_answer) = reads(&large, &query);
    assert_eq!(large_answer.code, 0, "{large_answer:?}");
    assert_eq!(large_reads, 1, "a query reads the log's last entry alone");
    assert_eq!(large_reads, small_reads);
    let claims = |outcome: &boom::cli::Outcome| {
        serde_json::from_str::<serde_json::Value>(&outcome.stdout).unwrap()["claims"].clone()
    };
    assert_eq!(claims(&large_answer), claims(&small_answer));

    let search = ["search", "refund", "--repository", "shop", "--json"];
    let (large_reads, outcome) = reads(&large, &search);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(large_reads, 1, "a search reads the log's last entry alone");

    // Three entries behind: the views' head entry and the three after it.
    three_by_hand(&small, 0);
    three_by_hand(&large, 0);
    let (small_reads, _) = reads(&small, &query);
    let (large_reads, outcome) = reads(&large, &query);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(large_reads, small_reads);
    assert!(large_reads <= 5, "read {large_reads} entries");

    // A write reads the log's last entry, and no other, before it appends.
    let propose = [
        "propose",
        "invariant",
        "orders-are-paid-once",
        "An order is paid once.",
        "--repository",
        "shop",
        "--provenance",
        "run-2",
    ];
    let (small_reads, _) = reads(&small, &propose);
    let (large_reads, outcome) = reads(&large, &propose);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(large_reads, small_reads);
    assert!(large_reads <= 2, "read {large_reads} entries");

    // The views give what a full replay gives.
    let kept = large.run(&query);
    large
        .sqlite()
        .execute_batch(
            "DROP TABLE views_head; DROP TABLE persons; DROP TABLE claims; \
             DROP TABLE versions; DROP TABLE judgments;",
        )
        .unwrap();
    let replayed = large.run(&query);
    assert_eq!(replayed.code, 0, "{replayed:?}");
    assert_eq!(kept.stdout.as_bytes(), replayed.stdout.as_bytes());
}

/// A small generator of numbers, the same on every run.
struct Numbers(u64);

impl Numbers {
    fn below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % bound as u64) as usize
    }
}

/// The state the views hold beside the state a replay of the whole log
/// builds, each as words, for one store.
fn views_and_replay(place: &Place) -> (String, String) {
    let conn = place.sqlite();
    let rows = {
        let mut statement = conn
            .prepare("SELECT seq, signed, signature FROM log ORDER BY seq")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)? as u64,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };
    let (replayed, _) = boom::judge::replay(
        rows.iter()
            .map(|(seq, signed, signature)| (*seq, signed.as_str(), signature.as_deref())),
    );
    let held = store::load_state(&conn).unwrap();
    let words = |state: &boom::judge::State| {
        format!(
            "{:?}\n{:?}\n{:?}",
            state.first_owner, state.persons, state.claims
        )
    };
    (words(&held), words(&replayed))
}

#[test]
fn the_views_and_a_full_replay_agree() {
    let place = Place::new();
    let ada = place.made();
    let bea = place.registered(&ada, "bea", 2);
    let mut numbers = Numbers(20_261_120);
    // Each claim: its name, its standing version, its waiting ones.
    let mut claims: Vec<(String, Option<String>, Vec<String>)> = Vec::new();
    let repositories = ["shop", "ledger"];
    let query = |repository: &str| {
        let outcome = place.run(&["query", "--repository", repository, "--json"]);
        assert_eq!(outcome.code, 0, "{outcome:?}");
        outcome.stdout
    };
    for act in 0..200 {
        let key = if numbers.below(2) == 0 { &ada } else { &bea };
        match numbers.below(4) {
            // A first version, bound to one repository or the system.
            0 => {
                let name = format!("claim-{act}");
                let mut args = vec![
                    "propose",
                    "invariant",
                    &name,
                    "A claim the mix proposed.",
                    "--provenance",
                    "the-mix",
                ];
                let bound = numbers.below(3);
                if bound < 2 {
                    args.extend(["--repository", repositories[bound]]);
                } else {
                    args.push("--system");
                }
                let version = place.json(&args)["version"].as_str().unwrap().to_owned();
                claims.push((name, None, vec![version]));
            }
            // An amendment of a standing version.
            1 => {
                let standing: Vec<usize> = (0..claims.len())
                    .filter(|at| claims[*at].1.is_some())
                    .collect();
                if standing.is_empty() {
                    continue;
                }
                let at = standing[numbers.below(standing.len())];
                let (name, base, waiting) = &mut claims[at];
                let statement = format!("A claim the mix amended at {act}.");
                let version = place.amended(name, &statement, base.as_deref().unwrap());
                waiting.push(version);
            }
            // An accept, or a reject, of a waiting version.
            choice => {
                let waiting: Vec<usize> = (0..claims.len())
                    .filter(|at| !claims[*at].2.is_empty())
                    .collect();
                if waiting.is_empty() {
                    continue;
                }
                let at = waiting[numbers.below(waiting.len())];
                let (name, standing, versions) = &mut claims[at];
                let version = versions.remove(numbers.below(versions.len()));
                let target = format!("{name}@{version}");
                if choice == 2 {
                    let outcome = place.run(&["accept", &target, "--key", key]);
                    if outcome.code == 0 {
                        *standing = Some(version);
                    } else {
                        // Its base no longer stands: refused, as built.
                        assert_eq!(outcome.code, 2, "{outcome:?}");
                    }
                } else {
                    let outcome =
                        place.run(&["reject", &target, "--reason", "the mix", "--key", key]);
                    assert_eq!(outcome.code, 0, "{outcome:?}");
                }
            }
        }
        if act % 20 == 19 {
            let (held, replayed) = views_and_replay(&place);
            assert_eq!(held, replayed, "after act {act}");
            let kept: Vec<String> = repositories.iter().map(|name| query(name)).collect();
            place
                .sqlite()
                .execute_batch(
                    "DROP TABLE views_head; DROP TABLE persons; DROP TABLE claims; \
                     DROP TABLE versions; DROP TABLE judgments;",
                )
                .unwrap();
            let rebuilt: Vec<String> = repositories.iter().map(|name| query(name)).collect();
            assert_eq!(kept, rebuilt, "after act {act}");
        }
    }
    let verified = place.json(&["verify"]);
    assert_eq!(verified["rebuilt"], serde_json::json!([]));
}

/// A claim with `versions` versions, each amending the one before and
/// accepted by `key`, bound to `repository`.
fn amended_times(place: &Place, key: &str, name: &str, repository: &str, versions: usize) {
    let first = place.json(&[
        "propose",
        "invariant",
        name,
        "Version 0 of the claim.",
        "--repository",
        repository,
        "--provenance",
        "run-1",
    ]);
    let mut accepted = first["version"].as_str().unwrap().to_owned();
    place.accepted(name, &accepted, key);
    for at in 1..versions {
        let statement = format!("Version {at} of the claim.");
        accepted = place.amended(name, &statement, &accepted);
        place.accepted(name, &accepted, key);
    }
}

/// The versions of the views one command reads, and its outcome.
fn versions_read(place: &Place, args: &[&str]) -> (u64, boom::cli::Outcome) {
    let before = store::versions_read();
    let outcome = place.run(args);
    (store::versions_read() - before, outcome)
}

#[test]
fn a_command_reads_only_the_versions_it_needs() {
    let place = Place::new();
    let ada = place.made();
    amended_times(&place, &ada, "money-has-currency", "shop", 20);
    amended_times(&place, &ada, "refunds-name-their-order", "ledger", 20);
    // A query reads the accepted version of each claim, and no other.
    let (read, outcome) = versions_read(&place, &["query", "--repository", "shop", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(read, 2, "one accepted version of each of the two claims");
    let (read, outcome) = versions_read(&place, &["search", "claim", "--all", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(read, 2);
    // A write reads the versions of the claim it names, twice: once to
    // judge before it signs, and once in the write's transaction.
    let accepted = place.show("money-has-currency")["accepted"]
        .as_str()
        .unwrap()
        .to_owned();
    let (read, outcome) = versions_read(
        &place,
        &[
            "propose",
            "invariant",
            "money-has-currency",
            "Version 20 of the claim.",
            "--amends",
            &accepted,
            "--provenance",
            "run-2",
        ],
    );
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(
        read, 40,
        "the 20 versions of the claim it names, read twice"
    );
}
