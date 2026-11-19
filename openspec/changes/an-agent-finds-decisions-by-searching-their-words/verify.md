# Verify: an-agent-finds-decisions-by-searching-their-words

Verified on 2026-11-18 at `63f7611`, on
`change/an-agent-finds-decisions-by-searching-their-words`. The branch
was cut from `bolt/boom-holds-the-rules-agents-read` and holds that
bolt at its head, `0ba737b`.

**Verdict: the whole checks fail, so the change isn't archived.**
`boom:lint` fails on three clippy errors in a unit test helper that this
change added to `crates/boom/src/judge.rs` (`0a89442`, task 1.1). The
other six tasks the gate ran pass. OpenSpec's checks pass: all 10 tasks
are ticked, and `openspec validate --strict` accepts the change. Named
tests prove all 30 scenarios of the spec's 6 requirements. The change
goes back to its code stage to fix the lint.

## The whole checks

Run once, as `CLAUDE.md` names them: `sluice exec main`, the gate's
checks (`#check-fast`, `.config/sluice.toml`), in sluice's sandbox, for
what the head affects compared with main (`429dcbc`).

| Task | Outcome |
|---|---|
| `boom:format` | passed |
| `boom:lint` | **failed**: 3 errors, `clippy::some_filter`, below |
| `boom:typecheck` | passed |
| `boom:test` | passed: 205 tests, 0 failed (34 unit; `attachments` 14, `claims` 20, `command_line` 12, `import` 32, `log` 9, `persons` 19, `query` 18, `search` 30, `signatures` 6, `store` 11) |
| `root:format` | passed: treefmt, 80 files, 0 changed |
| `root:specs` | passed: 8 specs, 0 failed |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to specs that pass it |

`Tasks: 6 completed, 1 failed`, exit 1. sluice recorded the 6 passing
results under their keys for `63f7611` and recorded nothing for
`boom:lint`. The site's tasks, `root:toolchain` and
`root:changes-cases` didn't run, because the head changes nothing they
read.

### boom:lint

Clippy from the pinned toolchain, Rust 1.98.1 (`rust-toolchain.toml`),
run with `-D warnings`, refuses three uses of `Some(x).filter(|_| …)` in
the `#[cfg(test)]` helper that builds a `State` for the
`standing_for_*` unit tests:

```
error: use of `Some(x).filter(|_| predicate)`
   --> crates/boom/src/judge.rs:785:29
785 |                 decided_by: Some("ada".to_owned()).filter(|_| status == Status::Standing),
    = note: `-D clippy::some-filter` implied by `-D warnings`
help: consider using `bool::then_some` instead
785 +                 decided_by: (status == Status::Standing).then_some("ada".to_owned()),

error: use of `Some(x).filter(|_| predicate)`
   --> crates/boom/src/judge.rs:786:29
786 |                 decided_in: Some(3).filter(|_| status == Status::Standing),
help: consider using `bool::then_some` instead
786 +                 decided_in: (status == Status::Standing).then_some(3),

error: use of `Some(x).filter(|_| predicate)`
   --> crates/boom/src/judge.rs:798:31
798 |                       standing: Some(version.proposal.version.clone())
799 | |                         .filter(|_| status == Status::Standing),
help: consider using `bool::then_some` instead
798 +                     standing: (status == Status::Standing).then_some(version.proposal.version.clone()),

error: could not compile `boom` (lib test) due to 3 previous errors
```

`git log -L785,800:crates/boom/src/judge.rs` traces all three lines to
`0a89442` on this branch, the commit for task 1.1. Task 1.1's own
condition, `cargo test -p boom --lib judge::`, passes, but it doesn't
run clippy. The gate does.

sluice reported the failure as `could not run: boom:lint: sluice's
sandbox refused the run network-outbound remote:*:53 (by moon),
network-outbound /private/var/run/mDNSResponder (a name lookup, by
moon), file-write-data /Users/chuck/.cargo/.package-cache (by cargo)
and 10 more`. Those refusals happened, but clippy still ran to the end
and failed on the code itself. The errors above come from the
compiler, not the sandbox, so running again on another host wouldn't
pass.

moon logged warnings without failing. It couldn't write its output
archives to `.moon/cache/outputs` from the sandbox. Four inputs it was
given are folders, not files. It found no network, so it skipped
installing dependencies. None of these comes from this change.

## OpenSpec

- `openspec instructions apply --change
  an-agent-finds-decisions-by-searching-their-words --json`: `state`
  `all_done`, all 10 tasks `done: true`, "All tracked tasks are
  complete."
- `openspec validate an-agent-finds-decisions-by-searching-their-words
  --strict`: "Change 'an-agent-finds-decisions-by-searching-their-words'
  is valid", exit 0.
- `openspec archive` wasn't run, because the whole checks fail.

## The tasks

All 10 tasks are ticked (`- [x]`), none open. Each task's named tests
are in the files the task names, and all pass in `boom:test`:

- **1.1:** `State::standing_for` is in `judge.rs:141`, with
  `standing_for_a_repository_gives_its_own_and_the_whole_systems` and
  `standing_for_every_repository_gives_each_in_kind_and_name_order`.
  `cli::query` takes its claims from it, and `query`'s 18 tests pass
  unchanged. Its test helper is what fails `boom:lint`.
- **2.1, 2.2:** `crates/boom/src/search.rs` has the ten unit tests the
  tasks name, from `a_word_is_written_as_a_string` to
  `an_excerpt_is_the_bodys_passage_on_one_line`.
- **3.1 to 3.6:** `crates/boom/tests/search.rs` has the 30 tests the
  tasks name, one per scenario, below.
- **4.1:** `README.md` and `CLAUDE.md` say what `boom search` does
  (`63f7611`).

## The scenarios

Each scenario of `specs/claim-search/spec.md` has a test of its own
name in `crates/boom/tests/search.rs`, and each asserts what its
scenario says. All 30 pass.

| Requirement | Scenario | Test |
|---|---|---|
| The words searched | Found by its statement | `found_by_its_statement` |
| | Found by its name | `found_by_its_name` |
| | Found by its scenarios | `found_by_its_scenarios` |
| | Found in its body | `found_in_its_body`, excerpt holding `rounding` |
| | Found by its legacy id | `found_by_its_legacy_id` |
| | Found by its section | `found_by_its_section` |
| | Every word is found | `every_word_is_found` |
| | Another form of a word | `another_form_of_a_word` |
| | A word written with accents | `a_word_written_with_accents` |
| A word is always a word | Words, not a query language | `words_not_a_query_language` |
| | No word to search | `no_word_to_search_is_refused`, exit 2, log unchanged |
| The repository it works in, or every repository | In the repository it works in | `in_the_repository_it_works_in` |
| | Declared by the repository | `declared_by_the_repository` |
| | Across every repository | `across_every_repository`, `repository` `null`, each claim's `repositories` |
| | Neither named, declared nor all | `neither_named_declared_nor_all_is_refused`, exit 2, log unchanged |
| | A repository and every repository | `a_repository_and_every_repository_are_refused`, exit 2, log unchanged |
| Only what stands, as the person accepted it | A proposal not given | `a_proposal_is_not_given` |
| | An amendment waiting | `an_amendment_waiting_gives_the_standing_words` |
| | A version that came in as history | `a_version_that_came_in_as_history_is_not_given` |
| | A statement edited in the views | `a_statement_edited_in_the_views_changes_nothing` |
| | An altered body | `an_altered_body_stops_the_search`, exit 3 naming the fingerprint, log unchanged |
| | Nothing kept | `nothing_is_kept`, same `sqlite_master` and log rows |
| Ranked, the best first | A name above a body | `a_name_above_a_body` |
| | Equal matches by kind and name | `equal_matches_by_kind_and_name` |
| | Limited | `limited`, `total` 3 and two claims |
| | A limit of none | `a_limit_of_none_is_refused`, exit 2, log unchanged |
| Answered in one JSON object, or in words an agent reads | Answered as JSON | `answered_as_json`, the whole object compared |
| | Answered in words | `answered_in_words`, the whole output compared, standard error empty |
| | Nothing matches | `nothing_matches` |
| | The same log, the same answer | `the_same_log_the_same_answer`, views dropped, the same bytes |

## What goes back to the code stage

Fix `boom:lint`. In `crates/boom/src/judge.rs` at lines 785, 786 and
798–799, write each `Some(x).filter(|_| status == Status::Standing)` as
`(status == Status::Standing).then_some(x)`, as clippy suggests. Then
run the gate again with `sluice exec main`. Nothing else falls short.
