//! boom verify: the whole chain checked, the whole log replayed, and
//! every view built again from it, each view that differed named
//! (record-store, "boom verify checks the log and rebuilds the views";
//! command-line, "Verify answered as JSON").

use super::support;

use serde_json::json;
use support::Place;

#[test]
fn verify_answered_as_json_on_a_new_store() {
    let place = Place::new();
    place.made();
    let hash: String = place
        .sqlite()
        .query_row("SELECT hash FROM log WHERE seq = 1", [], |row| row.get(0))
        .unwrap();
    let outcome = place.run(&["verify", "--json"]);
    assert_eq!(outcome.code, 0, "{outcome:?}");
    assert_eq!(outcome.stdout.lines().count(), 1, "{outcome:?}");
    let answer: serde_json::Value = serde_json::from_str(&outcome.stdout).unwrap();
    assert_eq!(answer["format"], "boom-verify/1");
    assert_eq!(answer["head"], json!({"number": 1, "hash": hash}));
    assert_eq!(answer["chain"], "holds");
    assert_eq!(answer["rebuilt"], json!([]));
}

#[test]
fn nothing_differs_where_the_views_were_kept_in_step() {
    let place = Place::new();
    let ada = place.made();
    let version = place.proposed("money-has-currency", "Every amount names its currency.");
    place.accepted("money-has-currency", &version, &ada);
    place.proposed(
        "refund-within-payment",
        "A refund never exceeds the payment.",
    );
    let rows = place.rows();
    let answer = place.json(&["verify"]);
    assert_eq!(answer["rebuilt"], json!([]));
    assert_eq!(answer["head"]["number"], 4);
    assert_eq!(place.rows(), rows, "verify appends nothing");
    let words = place.run(&["verify"]);
    assert_eq!(words.code, 0, "{words:?}");
    assert!(
        words.stdout.contains("The chain holds, to entry 4"),
        "{words:?}"
    );
    assert!(words.stdout.contains("No view differed"), "{words:?}");
}

#[test]
fn views_dropped_are_built_again_and_named() {
    let place = Place::new();
    place.three_entries();
    place
        .sqlite()
        .execute_batch("DROP TABLE persons; DROP TABLE judgments;")
        .unwrap();
    let answer = place.json(&["verify"]);
    assert_eq!(answer["rebuilt"], json!(["persons", "judgments"]));
    assert_eq!(place.json(&["verify"])["rebuilt"], json!([]));
}
