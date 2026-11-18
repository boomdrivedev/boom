# Verify: boom-keeps-decisions-in-a-database-on-the-machine

Verified on 2026-11-18 at `73eeba2`, on
`change/boom-keeps-decisions-in-a-database-on-the-machine`, cut from
`bolt/boom-holds-the-rules-agents-read` (at main's `54c73d1`).

**Verdict: the work holds.** The whole checks pass, every task is ticked
and true, and each of the specs' 29 requirements and 65 scenarios is
met by the code and proved by a named test, so the change is archived
with this report.

## The whole checks

Run once, as `CLAUDE.md` names them: `sluice exec main`, the gate's
checks (`#check-fast`, `.config/sluice.toml`) for what the head affects
against main, in sluice's sandbox.

| Task | Outcome |
|---|---|
| `boom:format` | passed |
| `boom:lint` | passed |
| `boom:typecheck` | passed |
| `boom:test` | passed: 79 tests, 0 failed (8 unit; `claims` 17, `command_line` 11, `log` 9, `persons` 17, `signatures` 6, `store` 11) |
| `root:format` | passed: treefmt, 71 files, 0 changed |
| `root:changes` | passed: 1 open change, valid under `openspec validate --strict` and archiving to specs that pass it |

`passed: 6 tasks`, exit 0; sluice recorded the 6 results under their
keys for `73eeba2`. The site's tasks, `root:toolchain`, `root:specs` and
`root:changes-cases` were not run: the head changes nothing they read.

moon warned, without failing, that it could not write its output
archives to `.moon/cache/outputs` from the sandbox, and that four inputs
it was given are directories rather than files. Neither is this change's.

## The tasks

All 15 tasks are ticked (`- [x]`), none open. Each task's every named
test is in the file the task names and passes in `boom:test`. Task 7.1's
condition, `devenv shell -- vale README.md`, reports 0 errors, 0 warnings
and 0 suggestions. The README and `CLAUDE.md` describe the crate as
built. Record 0006 is in `docs/adr/`.

## The specs against the work

Each scenario is proved by a test of `boom:test`, in the file its task
names, and each requirement is met by the code that test runs through
`boom::cli::run` (or the binary, where the task says so).

### record-store

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| One store file on the machine | The default store | `store.rs` `the_default_store_is_under_xdg_data_home` | `store::resolve`, `Store::create` (folders made, WAL, `user_version` 1) |
| | A store named outright | `store.rs` `a_store_named_outright_is_used_and_nothing_else_made` | `--store` first in `store::resolve` |
| | No store yet | `store.rs` `no_store_yet_exits_3_and_makes_nothing` | `Store::open` not run naming the path and `boom init` |
| Views are computed from the log | Views deleted | `store.rs` `views_deleted_change_no_answer` | `Store::read`: `views_fresh` false, `write_views` from `judge::replay` |
| | Views behind the log | `store.rs` `views_behind_the_log_are_rebuilt_before_an_answer` | `views_head` against the log's last entry |
| | One transaction | `store.rs` `an_append_and_its_views_are_one_transaction` | `Store::append`, one `IMMEDIATE` transaction |
| Many readers beside one writer | Two proposals at once | `store.rs` `two_proposals_at_once_both_land_chained` (two binaries) | `busy_timeout` 10 s, `BEGIN IMMEDIATE`, judge asked again inside |
| | A read during a write | `store.rs` `a_read_during_a_write_sees_none_of_it` | WAL; `Store::read` deferred while the views are fresh |
| Long text stored once by its fingerprint | One body, two versions | `store.rs` `one_body_two_versions_one_blob` | `blobs` keyed by SHA-256, `INSERT OR IGNORE` |
| | A body read back | `store.rs` `a_body_read_back_is_its_bytes` | `store::blob` |
| | A blob altered | `store.rs` `an_altered_blob_is_unreadable` | `store::blob` not run naming the fingerprint |

### record-log

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| Every change is an entry appended | A proposal and its accept | `log.rs` `a_proposal_and_its_accept_are_two_entries` | `Store::append` inserts only; no command updates or deletes `log` |
| | An entry names its format | `log.rs` `every_entry_names_its_format` | `entry::FORMAT`, `boom-entry/1` |
| boom numbers the entries in order | Numbered from one | `log.rs` `entries_are_numbered_from_one_in_order` | `seq` = last + 1; `check_chain` holds the numbers in order |
| | A clock set back | `log.rs` `a_clock_set_back_changes_no_status` | `judge::replay` reads order, never `at` |
| The log is chained | The chain given | `log.rs` `each_entry_gives_its_hash_and_the_one_before`; unit tests `an_entrys_hash_chains_the_one_before`, `the_first_entry_chains_the_hash_of_nothing` | `entry::hash`, `GENESIS` the SHA-256 of no bytes |
| A changed or removed entry is found | An entry's words changed | `log.rs` `an_entrys_words_changed_is_found` | `check_chain` on every open, before any answer or write |
| | An entry removed | `log.rs` `an_entry_removed_is_found` | `check_chain`, previous hash |
| An entry that fails boom's checks counts for nothing | An accept written by hand | `log.rs` `an_accept_written_by_hand_counts_for_nothing` | `judge::check_person`, `Proof::Unsigned` |
| | An accept signed by a key no one registered | `log.rs` `an_accept_by_a_key_no_one_registered_counts_for_nothing` | `judge::check_person`, "its key is not registered" |

### claim-versions

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| A claim and its name | A claim proposed | `claims.rs` `a_claim_proposed_waits_on_a_person` | `cli::propose` |
| | A name not in kebab case | `claims.rs` `a_name_not_in_kebab_case_is_refused`; unit `a_name_is_kebab_case` | `claim::check_name` |
| | No scenario | `claims.rs` `a_claim_with_no_scenario_is_refused`; unit `a_claim_needs_a_scenario` | `claim::check_scenarios` |
| | A statement of two lines | `claims.rs` `a_statement_of_two_lines_is_refused`; unit `a_statement_is_one_line` | `claim::check_statement` |
| A version is the fingerprint of its content | The same proposal twice | `claims.rs` `the_same_proposal_twice_is_one_version` | `Verdict::Repeats` |
| | One word changed | `claims.rs` `one_word_changed_is_another_version`; units `a_version_is_the_fingerprint_of_its_content`, `the_base_is_part_of_a_versions_content` | `Proposal::fingerprint`; `Proposal::check` holds a stored version to it |
| One name space | A name taken by another kind | `claims.rs` `a_name_taken_by_another_kind_is_refused` | `judge::check_proposal` |
| | A name whose claim stands | `claims.rs` `a_name_whose_claim_stands_is_refused` | `judge::check_proposal`, "amend it, with --amends" |
| | Two first versions waiting | `claims.rs` `two_first_versions_wait_side_by_side` | `judge::check_proposal` |
| An amendment names the version it amends | An amendment proposed | `claims.rs` `an_amendment_names_its_base` | `--amends` into the version's content |
| | A base the claim does not have | `claims.rs` `a_base_the_claim_does_not_have_is_refused` | `judge::check_proposal` |
| An accept stands only on its base | Two amendments of one base | `claims.rs` `two_amendments_of_one_base` | `judge::check_accept`, base against standing |
| | The same words on the new base | `claims.rs` `the_same_words_on_the_new_base_stand` | the base in the fingerprint |
| Each version's status is read from the log | Proposed, standing, replaced | `claims.rs` `proposed_standing_replaced` | `judge::apply` |
| | Rejected | `claims.rs` `a_rejected_version_gives_who_and_why` | `judge::apply`, `Act::Rejected` |
| A claim shown by its name | A version shown | `claims.rs` `a_version_shown_gives_its_body` | `cli::show` with `@<version>` |
| | An unknown name | `claims.rs` `an_unknown_name_is_refused` | `cli::show`, exit 2 |

### persons-acts

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| The first owner registers their own key | A store made | `persons.rs` `a_store_made_registers_its_first_owner` | `cli::init`; `judge::check_person` on an empty store |
| | Made twice with the same key | `persons.rs` `made_twice_with_the_same_key_adds_nothing` | `cli::init`, the first owner compared |
| | Made again with another key | `persons.rs` `made_again_with_another_key_is_refused` | `cli::init`, refused naming the first owner |
| Only a security key is registered | A plain key file | `persons.rs` `a_plain_key_file_is_refused` (for `init` and `register`) | `sign::check_key_type` |
| Owners register persons | A second person registered | `persons.rs` `an_owner_registers_a_second_person` | `cli::register`, `judge::check_registration` |
| | Registered by someone not an owner | `persons.rs` `a_register_by_someone_not_an_owner_is_refused` | `judge::check_registration` |
| | A key registered to another | `persons.rs` `a_key_registered_to_another_is_refused` | `judge::check_registration`, `person_by_key` |
| A person's act is signed with the person present | Signed without presence | `persons.rs` `signed_without_presence_is_refused`; `signatures.rs` `a_signature_without_presence_is_refused` | `sign::verify`, the user-presence flag |
| | Signed by a key not registered | `persons.rs` `signed_by_a_key_not_registered_is_refused` | `cli::person_of` |
| A signature covers the act, not its place in the log | Verified with no boom | `persons.rs` `a_signature_verifies_with_no_boom` (`ssh-key`'s own `SshSig` verification); unit `the_signed_part_holds_no_number_chain_or_signature` | `Entry::signed_part`; `boom log --json` gives `signed` and `signature` |
| An agent proposes | An agent's proposal | `persons.rs` `an_agents_proposal_names_its_provenance` | `Actor::Agent { provenance }`, unsigned |
| | No provenance | `persons.rs` `an_agents_proposal_with_no_provenance_is_refused` | `cli::propose`, `judge::check_agent` |
| Only a person accepts or rejects | An agent's accept | `persons.rs` `an_agents_accept_is_refused` | `cli::accept` with no `--key`; `judge::check_agent` on read |
| | A person accepts an agent's proposal | `persons.rs` `a_person_accepts_an_agents_proposal` | `cli::accept` |
| | A reject with no reason | `persons.rs` `a_reject_with_no_reason_is_refused` | `cli::reject` |
| No one accepts their own proposal | Ada accepts what Ada proposed | `persons.rs` `no_one_accepts_their_own_proposal` | `judge::check_accept`, proposer against accepter |
| | Bo accepts Ada's | `persons.rs` `another_person_accepts_it` | `judge::check_accept` |

### command-line

| Requirement | Scenario | Test | Code |
|---|---|---|---|
| The commands | The version | `command_line.rs` `the_version` (the binary) | clap `version`; `main.rs` a thin caller of `cli::run` |
| | The log of a new store | `command_line.rs` `the_log_of_a_new_store` | `cli::log` |
| Exit codes | A refusal writes nothing | `command_line.rs` `a_refusal_writes_nothing` | `Failure::Refused`, exit 2, the transaction dropped |
| | Could not run | `command_line.rs` `a_key_whose_program_cannot_sign_could_not_run`; `signatures.rs` `a_key_whose_program_cannot_be_run_does_not_sign` | `SshKeygenSigner::sign`, `Failure::NotRun`, exit 3 |
| | Arguments it cannot read | `command_line.rs` `an_unknown_kind_is_refused_naming_the_kinds` | clap's error a refusal, exit 2 |
| One JSON object with its format | A proposal answered as JSON | `command_line.rs` `a_proposal_answered_as_json` | `boom-propose/1` |
| | A refusal answered as JSON | `command_line.rs` `a_refusal_answered_as_json` | `boom-refused/1` |
| | Not run, answered as JSON | `command_line.rs` `not_run_answered_as_json` | `boom-not-run/1` |
| No prompts | No terminal | `command_line.rs` `no_terminal` (the binary, standard input closed) | nothing reads the terminal |
| A repeated write adds nothing | An accept repeated | `command_line.rs` `an_accept_repeated_adds_nothing` | `Verdict::Repeats` |
| | A registration repeated | `command_line.rs` `a_registration_repeated_adds_nothing` | `Verdict::Repeats` |

## Observations

These do not fall short of the specs. Each is a note for whoever works
on the store next.

- **A read may write the views before it refuses.** `Store::read`
  rebuilds stale views and commits them even when its answer is a
  refusal, such as `boom show no-such-claim` after the views were
  dropped. `Store::append` rolls a rebuild back with its refusal. The
  log is untouched either way, which is what the exit-code scenarios
  hold, and record-store requires the rebuild before any answer. A
  views-only write on exit 2 is a property of the views (core.2), not
  of the truth.
- **"Standard input closed" is `/dev/null`.** `no_terminal` runs the
  binary with `Stdio::null()`. A read gets end of file, as a closed
  descriptor would give, and nothing prompts.
