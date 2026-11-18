# Verify: an-agent-gets-a-repositorys-rules-from-boom

Verified on 2026-11-18 at `26ac1f2`, on
`change/an-agent-gets-a-repositorys-rules-from-boom`. The branch was cut
from `bolt/boom-holds-the-rules-agents-read` and holds that bolt at its
head, `b87d525`, merged in by `73ef5c9`.

**Verdict: the work holds, so the change is archived.** The whole
checks pass. All 12 tasks are ticked and their conditions hold. Named
tests prove all 37 scenarios across the specs' 10 requirements.

## The whole checks

Run once, as `CLAUDE.md` names them: `sluice exec main`, the gate's
checks (`#check-fast`, `.config/sluice.toml`), in sluice's sandbox, for
what the head affects compared with main (`54c73d1`).

| Task | Outcome |
|---|---|
| `boom:format` | passed |
| `boom:lint` | passed |
| `boom:typecheck` | passed |
| `boom:test` | passed: 163 tests, 0 failed (22 unit; `attachments` 14, `claims` 20, `command_line` 12, `import` 32, `log` 9, `persons` 19, `query` 18, `signatures` 6, `store` 11) |
| `root:format` | passed: treefmt, 77 files, 0 changed |
| `root:specs` | passed: 6 specs, 0 failed |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to specs that pass it |

`passed: 7 tasks`, exit 0. sluice recorded the 7 results under their
keys for `26ac1f2`. The site's tasks, `root:toolchain` and
`root:changes-cases` didn't run, because the head changes nothing they
read.

moon logged warnings without failing. It couldn't write its output
archives to `.moon/cache/outputs` from the sandbox. Four inputs it was
given are folders, not files. It found no network, so it skipped
installing dependencies. None of these comes from this change.

The bolt is an ancestor of the head. So the gate's pass covers this
change merged with what it lands into.

## The tasks

All 12 tasks are ticked (`- [x]`), none open. Each task's named tests
are in the files the task names, and all pass in `boom:test`. The other
conditions hold too:

- **1.2:** `toml` 1.2.1 is in `crates/boom/Cargo.toml` and locked in
  `Cargo.lock`. `cli::Env` has `cwd`, filled from the process's working
  folder by `Env::from_process` and from the test's own folder by
  `Place::new`.
- **1.4:** nothing on this branch changes `claim.rs`: `git diff b87d525
  HEAD -- crates/boom/src/claim.rs` is empty.
- **1.5:** the store's `user_version` is still 1, which
  `a_version_shown_gives_what_it_binds` asserts.
- **3.1:** `CLAUDE.md` says what is built and what isn't.
- **4.1:** `73ef5c9` is a merge with parents `af7c3de` and `b87d525`,
  not a rebase. `Cargo.toml` and `Cargo.lock` hold `toml` 1.2.1 and
  `yaml-rust2` 0.13.0. The tests' `Place` keeps `folder(name, files)`
  and gives a bare folder as `empty_folder(name)`. The gate passes on
  the merged head.
- **4.2:** `cli::import` gives each proposal `attach` naming the
  repository `--repository` names, after `imported`.
- **4.3:** record 0008 holds point 6: design 7's choices and the path
  not taken, the judge refusing an imported proposal attached to
  anything other than its `imported` repository.

For 3.1 and 4.3, `devenv shell -- vale README.md` reports 0 errors and
0 warnings. It makes 1 suggestion, `Boom.SentenceLength` at line 57.
That line is the README's sentence on what an import reads, which 4.3
lengthened.

## The specs against the work

The tests run each scenario through `boom::cli::run`, in a temporary
folder of their own, with keys made at run time (core.3). A test reads
only the `.config/boom.toml` files and MADR records it writes from
strings in its own code (core.5).

### claim-attachments

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| A proposal names what its version binds | Bound to one repository | `attachments.rs` `bound_to_one_repository`; unit `an_attachment_follows_the_claim_in_the_signed_part` | `entry`'s proposed part writes `attach` after `claim`; `cli::attachment` |
| | Several repositories named | `several_repositories_named_are_sorted_once` | `cli::attachment` sorts and dedups |
| | Bound to the whole system | `bound_to_the_whole_system`; unit `a_proposal_with_no_attachment_is_written_as_before` | `attach` skipped where `None` |
| | The repository the working folder declares | `the_repository_the_working_folder_declares` | `repository::find` from `Env.cwd` |
| | A repository name not in kebab case | `a_repository_name_not_in_kebab_case_is_refused` | `judge::check_attach` |
| | Both a repository and the whole system | `both_a_repository_and_the_whole_system_are_refused` | clap `conflicts_with = "system"` |
| An attachment is no part of a version | One fingerprint wherever it binds | `one_fingerprint_wherever_it_binds` | `claim.rs` unchanged |
| | Proposed again for the same repository | `proposed_again_for_the_same_repository_adds_nothing` | `judge::check_proposal`, a repeat |
| | The same words for another repository | `the_same_words_for_another_repository_are_refused` (by hand too) | `check_proposal`: "is attached to …; a version's attachment does not move" |
| An amendment keeps its base's attachment | Amended with nothing named | `amended_with_nothing_named_keeps_its_bases` (in a folder declaring `ledger`) | `cli::attachment` reads the base from the judge's state |
| | Amended for another repository | `amended_for_another_repository` | named repositories first |
| A version shown gives what it binds | A version shown | `a_version_shown_gives_what_it_binds` | `boom show` reads `attach` from the proposing entry |
| | A claim shown | `a_claim_shown_gives_each_versions_repositories` | each version's `repositories`; a `binds` line in words |
| An attachment is judged at the write and on every read | A repository named twice by hand | `a_repository_named_twice_by_hand_counts_for_nothing` | `judge::check_form` calls `check_attach` on replay |

### repository-rules

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| The repository is named or declared | Declared by the repository | `query.rs` `declared_by_the_repository`; unit `the_nearest_declaration_in_the_repository` | `repository::find` |
| | Named outright | `named_outright` | `--repository` first |
| | Neither named nor declared | `neither_named_nor_declared_is_refused` | `cli::query`'s refusal names `--repository` and `.config/boom.toml` |
| | A declaration above the repository | `a_declaration_above_the_repository_is_not_read`; unit `no_declaration_above_the_repository` | read no higher than the folder holding `.git` |
| | Outside a repository | `outside_a_repository_no_declaration_is_read`; unit `no_declaration_outside_a_repository` | no `.git` found, nothing read |
| | A declaration it cannot read | `a_declaration_it_cannot_read_is_refused`; units `a_declared_name_not_in_kebab_case_is_refused`, `a_declaration_with_another_key_is_refused` | unknown keys refused; the refusal names the file |
| Every claim that stands for the repository | What stands for a repository | `what_stands_for_a_repository` | `cli::query` filters on the standing version's `repositories` |
| | An amendment waiting | `an_amendment_waiting_gives_the_standing_words` | standing version only |
| | An amendment accepted | `an_amendment_accepted_replaces_its_base` | |
| | A rejected version | `a_rejected_version_is_not_given` | |
| | A version that came in as history | `a_version_that_came_in_as_history_is_not_given` | a history version never stands |
| | Nothing stands | `nothing_stands` | exit 0, `claims` empty |
| As the person accepted it | A statement edited in the views | `a_statement_edited_in_the_views_changes_nothing` | `judge::replay` of the log inside `Store::read`, no view table read |
| | A status edited in the views | `a_status_edited_in_the_views_changes_nothing` | |
| | An accept written by hand | `an_accept_written_by_hand_binds_nothing` | an unsigned accept counts for nothing |
| Answered in one JSON object, or in words an agent reads | Answered as JSON | `answered_as_json` | `boom-query/1`, `head` from the last row |
| | Answered in words | `answered_in_words` | standard output alone; a body given as a pointer |
| | The same log, the same answer | `the_same_log_the_same_answer` | views dropped between runs, same bytes, log as it was |

### record-import

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| An import proposes, all or nothing | An import proposes and accepts nothing | `import.rs` `an_import_proposes_and_accepts_nothing` | `cli::import` appends unsigned proposals only |
| | One record refused, none appended | `one_record_refused_appends_none` | every record judged before `append_all` |
| | No provenance | `an_import_with_no_provenance_is_refused` | |
| | No repository | `an_import_with_no_repository_is_refused` | |
| | An imported record stands for its repository alone | `an_imported_record_stands_for_its_repository_alone`; unit `an_imported_proposal_is_attached_after_imported` | `cli::import` gives each proposal `attach` naming `--repository`, after `imported` |

`an_imported_record_stands_for_its_repository_alone` reads the
attachment with `boom show amounts-in-cents@<version>`, where the
scenario names `boom show amounts-in-cents --json`. Both give the same
version's `repositories` from its proposing entry. The scenario's THEN
is about that version's `repositories`, so the test proves it.

The import's answer gives the attachment once, as its top-level
`repository`, not as each record's `repositories`. Design 7 asks for
the attachment "before a person accepts them", and no scenario asks for
more than that.
