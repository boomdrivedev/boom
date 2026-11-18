# Verify: an-agent-gets-a-repositorys-rules-from-boom

Verified on 2026-11-18 at `8fabf4c`, on
`change/an-agent-gets-a-repositorys-rules-from-boom`, cut from
`bolt/boom-holds-the-rules-agents-read` at `3af78c6`.

**Verdict: the work falls short, so the change is not archived.** The
whole checks pass, all 9 tasks are ticked and their conditions hold,
and named tests prove all 31 scenarios across the specs' 9
requirements. But the bolt has moved since this branch was cut. The
change `boom-imports-a-repositorys-records-and-rules` landed into it
first (`b87d525`, sluice #11). The proposal's Impact and design 7 say
that whichever of the two lands second reconciles them. This change is
now second, and nothing on the branch reconciles them. The branch
can't land onto its bolt as it stands (see "What falls short").

## The whole checks

Run once, as `CLAUDE.md` names them: `sluice exec main`, the gate's
checks (`#check-fast`, `.config/sluice.toml`), for what the head
affects compared with main (`54c73d1`), in sluice's sandbox.

| Task | Outcome |
|---|---|
| `boom:format` | passed |
| `boom:lint` | passed |
| `boom:typecheck` | passed |
| `boom:test` | passed: 117 tests, 0 failed (15 unit; `attachments` 14, `claims` 17, `command_line` 11, `log` 9, `persons` 17, `query` 17, `signatures` 6, `store` 11) |
| `root:format` | passed: treefmt, 75 files, 0 changed |
| `root:specs` | passed: 5 specs, 0 failed |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to specs that pass it |

`passed: 7 tasks`, exit 0; sluice recorded the 7 results under their
keys for `8fabf4c`. The site's tasks, `root:toolchain` and
`root:changes-cases` weren't run because the head changes nothing they
read.

moon logged warnings without failing. It couldn't write its output
archives to `.moon/cache/outputs` from the sandbox. Four inputs it was
given are directories, not files. It found no network, so it skipped
installing dependencies. None of these comes from this change.

The checks judge the head against main. They don't judge it merged
with the bolt it lands into.

## The tasks

All 9 tasks are ticked (`- [x]`), none open. Each task's named tests
are in the files the task names, and all pass in `boom:test`. The other
conditions hold too:

- **1.2:** `toml` 1.2.1 is in `crates/boom/Cargo.toml` and locked in
  `Cargo.lock`. `cli::Env` has `cwd`, filled by `Env::from_process`
  (which `src/main.rs` calls) and by `Place::new` from the test's own
  folder.
- **1.4:** `claim.rs` is unchanged.
- **1.5:** the store's `user_version` is still 1 (asserted in
  `a_version_shown_gives_what_it_binds`).
- **3.1:** `devenv shell -- vale README.md` reports 0 errors, 0
  warnings and 0 suggestions. `CLAUDE.md` now says what is built and
  what isn't.

Record 0008 is in `docs/adr/`.

## The specs against the work

The tests run each scenario through `boom::cli::run`, in a temporary
folder of their own, with keys made at run time (core.3). Every
`.config/boom.toml` a test reads, it writes from a string in its own
code (core.5).

### claim-attachments

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| A proposal names what its version binds | Bound to one repository | `attachments.rs` `bound_to_one_repository`; unit `an_attachment_follows_the_claim_in_the_signed_part` | `entry::ProposedPart.attach` after `claim`; `cli::attachment` |
| | Several repositories named | `several_repositories_named_are_sorted_once` | `cli::attachment` sorts and dedups |
| | Bound to the whole system | `bound_to_the_whole_system`; unit `a_proposal_with_no_attachment_is_written_as_before` | `attach` skipped where `None` |
| | The repository the working folder declares | `the_repository_the_working_folder_declares` | `repository::find` from `Env.cwd` |
| | A repository name not in kebab case | `a_repository_name_not_in_kebab_case_is_refused` | `judge::check_attach`, `repository::check_name` |
| | Both a repository and the whole system | `both_a_repository_and_the_whole_system_are_refused` | clap `conflicts_with = "system"` |
| An attachment is no part of a version | One fingerprint wherever it binds | `one_fingerprint_wherever_it_binds` | `claim.rs` untouched |
| | Proposed again for the same repository | `proposed_again_for_the_same_repository_adds_nothing` | `judge::check_proposal`, `Verdict::Repeats` |
| | The same words for another repository | `the_same_words_for_another_repository_are_refused` (by hand too) | `check_proposal`: "is attached to …; a version's attachment does not move" |
| An amendment keeps its base's attachment | Amended with nothing named | `amended_with_nothing_named_keeps_its_bases` (in a folder declaring `ledger`) | `cli::attachment` reads the base from the judge's state |
| | Amended for another repository | `amended_for_another_repository` | named repositories first |
| A version shown gives what it binds | A version shown | `a_version_shown_gives_what_it_binds` | `store::claim_view` reads `attach` from the proposing entry |
| | A claim shown | `a_claim_shown_gives_each_versions_repositories` | `version_json`'s `repositories`; `binds:` lines |
| An attachment is judged at the write and on every read | A repository named twice by hand | `a_repository_named_twice_by_hand_counts_for_nothing` | `judge::check_form` → `check_attach` on replay |

### repository-rules

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| The repository is named or declared | Declared by the repository | `query.rs` `declared_by_the_repository`; unit `the_nearest_declaration_in_the_repository` | `repository::find`, `root` |
| | Named outright | `named_outright` | `--repository` first |
| | Neither named nor declared | `neither_named_nor_declared_is_refused` | `cli::query` refusal naming `--repository` and `.config/boom.toml` |
| | A declaration above the repository | `a_declaration_above_the_repository_is_not_read`; unit `no_declaration_above_the_repository` | ancestors taken while within the root |
| | Outside a repository | `outside_a_repository_no_declaration_is_read`; unit `no_declaration_outside_a_repository` | no `.git` found, nothing read |
| | A declaration it cannot read | `a_declaration_it_cannot_read_is_refused`; units `a_declared_name_not_in_kebab_case_is_refused`, `a_declaration_with_another_key_is_refused` | `Declaration` with `deny_unknown_fields`; refusal names the file |
| Every claim that stands for the repository | What stands for a repository | `what_stands_for_a_repository` | `cli::query` filters on the standing version's `repositories` |
| | An amendment waiting | `an_amendment_waiting_gives_the_standing_words` | standing version only |
| | An amendment accepted | `an_amendment_accepted_replaces_its_base` | |
| | A rejected version | `a_rejected_version_is_not_given` | |
| | Nothing stands | `nothing_stands` | exit 0, `claims` empty |
| As the person accepted it | A statement edited in the views | `a_statement_edited_in_the_views_changes_nothing` | `store::log_rows` + `judge::replay` inside `Store::read`, no view table read |
| | A status edited in the views | `a_status_edited_in_the_views_changes_nothing` | |
| | An accept written by hand | `an_accept_written_by_hand_binds_nothing` | `Proof::Unsigned` counts for nothing |
| Answered in one JSON object, or in words an agent reads | Answered as JSON | `answered_as_json` | `boom-query/1`, `head` from the last row |
| | Answered in words | `answered_in_words` | standard output alone; body as a pointer |
| | The same log, the same answer | `the_same_log_the_same_answer` | views dropped between runs, same bytes, log as it was |

## What falls short

1. **The branch is behind its bolt and can't land onto it.**
   `bolt/boom-holds-the-rules-agents-read` is at `b87d525`, which
   landed `boom-imports-a-repositorys-records-and-rules` (#11) after
   this branch was cut at `3af78c6`. Merging the bolt into `8fabf4c`
   (`git merge-tree`, the tree untouched) conflicts in 10 files:
   `CLAUDE.md`, `Cargo.lock`, `README.md`, `crates/boom/Cargo.toml`,
   `crates/boom/src/cli.rs`, `crates/boom/src/entry.rs`,
   `crates/boom/src/judge.rs`, `crates/boom/src/store.rs`,
   `crates/boom/tests/store.rs` and `crates/boom/tests/support/mod.rs`.
   The gate's pass is for this head compared with main. It says nothing
   about the head merged with the bolt.
2. **Design 7 hasn't been done, and the import landed first, so it's
   this change's job.** The proposal's Impact says "whichever lands
   second has an import attach its proposals to that repository
   (design 7)", and design 7 spells it out. A proposal's part holds
   `claim`, then `imported`, then `attach`, each only where present. An
   import's proposals carry `attach` naming the repository its
   `--repository` names, so a repository's imported records and rules
   stand for that repository alone. A `history` version never stands,
   so `boom query` never gives it. Nothing on the branch does this, and
   the change has no task for it.
3. **That reconciliation changes a requirement this change carries no
   delta for.** `record-import` is now in the bolt's specs. Its
   requirement "An import proposes, all or nothing" says what each
   imported `proposed` entry carries. Attaching those entries to the
   import's repository changes that requirement. By `CLAUDE.md`
   ("Where the work changes a requirement, it carries that change in
   the same branch"), the change needs a MODIFIED `record-import`
   delta. It also needs a scenario proving an imported record, once
   accepted, stands for its repository alone and not for another, and
   a task with its tests.

Before this change can be archived, the code stage has to:

- merge the bolt into the branch;
- add design 7's delta and task to the change;
- build and test that work;
- run the gate on the merged head.
