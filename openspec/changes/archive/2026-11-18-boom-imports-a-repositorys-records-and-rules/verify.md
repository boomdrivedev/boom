# Verify

The verify stage's report on `change/boom-imports-a-repositorys-records-and-rules`
at `ad37401`, cut from `bolt/boom-holds-the-rules-agents-read`.

**Outcome: archived.** The gate's checks pass, every spec scenario has a
test that passes, and every task is ticked.

## The whole checks

Run once, `sluice exec main`, against `main` (`54c73d1`), the checks
`ad37401` affects, the layer `#check-fast`:

| task | outcome |
|---|---|
| `boom:format` | passed |
| `boom:typecheck` | passed |
| `boom:lint` | passed: `cargo clippy -D warnings`, no warning |
| `boom:test` | passed: 122 tests, none failed |
| `root:format` | passed: 73 files formatted, 0 changed |
| `root:specs` | passed: 5 specs valid |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to valid specs |

The `site` project's tasks and `root:changes-cases` were not affected and
did not run.

`moon` reported `Tasks: 7 completed`. sluice exited 0 and said
`recorded 7 results under their keys for ad37401`. moon warned on each
task that it could not write the task's output archive under
`.moon/cache/outputs`. This did not fail a task or stop sluice recording
it.

## The specs

Each scenario of the change's delta specs and the test that proves it.
Each test is in `crates/boom/tests/` unless it is named in `src/`. Each
passed in the run above.

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
the words the scenario names and the log unchanged, so it proves for each
refusal that nothing is appended.

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
record 0007 keeps under "More information". The judge checks an imported
proposal against the import's rules first, before the rules every
proposal meets (`src/judge.rs` `check_imported`, called first in
`check_act`). Design 7 says "after". A dry run reads no store. A draft
holding a field other than `scenarios` and `name` is refused. A rule's
body is its details' bullet as written. The code stage made each of these
choices, each is cheap to change, and record 0007 holds each.

## The tasks

All eleven tasks, 1.1 to 5.1, are ticked done. Each test a task names is
in the tree and passed in the gate's `boom:test`, 46 in all.

- Task 1.3 boxes `Act::Proposed`'s `imported` as `Option<Box<Imported>>`
  in `crates/boom/src/entry.rs` and in the signed part. serde writes a box
  as its contents, so the signed part's bytes are as before, and
  `an_imported_proposal_keeps_where_it_came_from` passes. The state a
  replay builds keeps it unboxed: `src/judge.rs` `apply` takes
  `imported.as_deref().cloned()`. `boom:lint` and `boom:test` pass.
- Task 3.1's dependency, `yaml-rust2` 0.13.0, is in
  `crates/boom/Cargo.toml` and locked in `Cargo.lock`.
- Task 5.1's README and `CLAUDE.md` say what the import does.
  `devenv shell -- vale README.md` gives `0 errors, 0 warnings and 0
  suggestions`.
