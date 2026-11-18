# Verify

The verify stage's report on `change/boom-imports-a-repositorys-records-and-rules`
at `2ab9592`, cut from `bolt/boom-holds-the-rules-agents-read`.

**Outcome: not archived.** The gate's checks fail on `boom:lint`. Every
spec scenario has a test that passes and every task is ticked, so the
work holds once the lint passes. The change goes back to its code stage.

## The whole checks

Run once, `sluice exec main`, against `main` (`54c73d1`), the checks
`2ab9592` affects, the layer `#check-fast`:

| task | outcome |
|---|---|
| `boom:format` | passed |
| `boom:typecheck` | passed |
| `boom:test` | passed: 122 tests, none failed |
| `boom:lint` | **failed** |
| `root:format` | passed |
| `root:specs` | passed: 5 specs valid |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to valid specs |

The `site` project's tasks were not affected and did not run.

`moon` reported `Tasks: 6 completed, 1 failed`, and sluice exited 1.

### What fails

`cargo clippy -D warnings` refuses the crate's library and its tests:

```
error: large size difference between variants
   --> crates/boom/src/entry.rs:100:1
    |
100 | /   pub enum Act {
...
111 | | /     Proposed {
112 | | |         /// The claim's version.
113 | | |         claim: Proposal,
114 | | |         /// Where it came in from, for a version an import proposes.
115 | | |         imported: Option<Imported>,
116 | | |     },
    | | |_____- the largest variant contains at least 360 bytes
...
125 | | /     Rejected {
...
132 | | |     },
    | | |_____- the second-largest variant contains at least 72 bytes
    = note: `-D clippy::large-enum-variant` implied by `-D warnings`
help: consider boxing the large fields or introducing indirection in some other way
    |
113 +         claim: Box<Proposal>,
error: could not compile `boom` (lib) due to 1 previous error
error: could not compile `boom` (lib test) due to 1 previous error
```

`imported: Option<Imported>`, added to `Act::Proposed` by task 1.2, makes
that variant at least 360 bytes against the next largest variant's 72.
The fix belongs to the code stage. clippy suggests boxing a field,
`Box<Proposal>` or `Option<Box<Imported>>`. The fix runs the gate again.

sluice also counted `boom:lint` as a run it could not record. Its
sandbox refused moon's name lookups (`network-outbound remote:*:53`,
`/private/var/run/mDNSResponder`), cargo's write of
`~/.cargo/.package-cache`, and 10 more. It said
`nothing was recorded for a test that could not run`. clippy finished
its check and reported the error above, so the error is real whatever the
sandbox refused. The gate's next run shows whether the refusals stop
`boom:lint` from recording once clippy passes.

## The specs

Each scenario of the change's delta specs and the test that proves it.
Each test is in `crates/boom/tests/` unless named in `src/`, and passed in
the run above.

### record-import

| requirement | scenario | test |
|---|---|---|
| Decision records read from a folder | A folder of MADR files read | `import.rs` `a_folder_of_madr_files_read` |
| | A record named in its frontmatter | `a_record_named_in_its_frontmatter` |
| | A record with no title | `a_record_with_no_title_is_refused` |
| | Two numbers that differ | `two_numbers_that_differ_are_refused` |
| Rules read from a page | A rules page read | `a_rules_page_read` |
| | Details on the page beside it | `details_on_the_page_beside_it` |
| | A page with no rules | `a_page_with_no_rules_is_refused` |
| | One id twice | `one_id_twice_is_refused` |
| An agent drafts what the files lack | What a draft still needs | `what_a_draft_still_needs` |
| | Imported with drafts | `imported_with_drafts` |
| | A rule named by its draft | `a_rule_named_by_its_draft` |
| | A draft missing | `a_draft_missing_is_refused` |
| | A draft for no record | `a_draft_for_no_record_is_refused` |
| An import proposes, all or nothing | An import proposes and accepts nothing | `an_import_proposes_and_accepts_nothing` |
| | One record refused, none appended | `one_record_refused_appends_none` |
| | No provenance | `an_import_with_no_provenance_is_refused` |
| | No repository | `an_import_with_no_repository_is_refused` |
| Superseded, deprecated and rejected records come in as history | A superseded record | `a_superseded_record_comes_in_as_history` |
| | A successor by its number | `a_successor_by_its_number` |
| | A successor not in the folder | `a_successor_not_in_the_folder_is_refused` |
| | A status boom does not know | `a_status_boom_does_not_know_is_refused` |
| | A record with no status | `a_record_with_no_status_is_read_as_accepted` |
| One name and one number for each record | A name the store holds | `a_name_the_store_holds_is_refused` |
| | Named otherwise by its draft | `named_otherwise_by_its_draft` |
| | A number the store holds | `a_number_the_store_holds_is_refused` |
| | Two records under one name | `two_records_under_one_name_are_refused` |
| Each record read once | The same import twice | `the_same_import_twice_adds_nothing` |
| | A file edited after its import | `a_file_edited_after_its_import_is_refused` |
| | Another repository's page of the same name | `another_repositorys_page_of_the_same_name` |
| Accepted together | An import accepted together | `an_import_accepted_together` |
| | A record left proposed waits | `a_record_left_proposed_waits` |

Each refusal goes through the tests' `Place::refused`. It asserts exit 2,
the words the scenario names and the log unchanged, so "nothing is
appended" is proved for each.

### claim-versions

| requirement | scenario | test |
|---|---|---|
| A claim and its name | History with no scenario | `claims.rs` `history_with_no_scenario`; `src/claim.rs` `a_history_version_needs_no_scenario`, `a_history_version_has_no_base` |
| | A claim proposed, A name not in kebab case, No scenario, A statement of two lines | the tests that proved them before, unchanged and passing |
| A version is the fingerprint of its content | The same words as history | `claims.rs` `the_same_words_as_history_are_another_version`; `src/claim.rs` `history_is_part_of_a_versions_content`, `no_history_keeps_the_fingerprint_it_had` |
| | The same proposal twice, One word changed | unchanged and passing |
| Each version's status is read from the log | History accepted | `claims.rs` `history_accepted_never_stands` |
| | Proposed, standing, replaced; Rejected | unchanged and passing |

### persons-acts

| requirement | scenario | test |
|---|---|---|
| Several versions accepted together | Three accepted together | `persons.rs` `three_accepted_together` |
| | One of them refused | `persons.rs` `one_of_them_refused_signs_and_appends_nothing`, asserting `SoftKeys`' count of signatures unchanged |

### command-line

| requirement | scenario | test |
|---|---|---|
| The commands | An import answered as JSON | `command_line.rs` `an_import_answered_as_json` |
| | The version, The log of a new store | unchanged and passing |

The code meets design decisions 1 to 9 as written, with the departures
record 0007 keeps under "More information". The judge holds an imported
proposal to the import's rules before the rules of any proposal
(`src/judge.rs` `check_imported`, called first in `check_act`), where
design 7 says "after". A dry run reads no store. A draft holding a field
other than `scenarios` and `name` is refused. A rule's body is its
details' bullet as written. Each is the code stage's own, cheap to
change, and recorded.

## The tasks

All ten tasks, 1.1 to 5.1, are ticked done. Each task's named tests are
in the tree and passed in the gate's `boom:test`. Task 3.1's dependency,
`yaml-rust2` 0.13.0, is in `crates/boom/Cargo.toml` and locked in
`Cargo.lock`. Task 5.1's README and `CLAUDE.md` say what the import does.
`devenv shell -- vale README.md` gives `0 errors, 0 warnings and 0
suggestions`.

## What goes back

- `boom:lint` fails: `clippy::large_enum_variant` on `Act` in
  `crates/boom/src/entry.rs:100`, the variant `Proposed` at least 360
  bytes since it gained `imported`. The code stage makes the lint pass,
  boxing a field as clippy suggests or otherwise, and runs the gate again.
