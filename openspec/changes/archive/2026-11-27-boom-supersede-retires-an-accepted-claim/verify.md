# Verify: boom-supersede-retires-an-accepted-claim

Verified on 2026-11-27 on `change/boom-supersede-retires-an-accepted-claim` at
`8d18b42`, cut from main. Verdict: the whole checks pass, every task is done,
the change validates strictly, and the work meets every requirement and
scenario of its spec. The change is ready to archive.

## The whole checks

`sluice exec main -- moon run :#check-fast`, one run, exit 0:
`Tasks: 27 completed (4 cached)`, in 1m 28s.

| project | tasks |
|---|---|
| `boom` | `format`, `lint`, `typecheck`, `test` |
| `claude-code` | `format`, `lint`, `test`, `typecheck` (cached) |
| `nix` | `lint`, `typecheck`, `test` |
| `site` | `install`, `format`, `lint`, `typecheck`, `test`, `build` |
| `models` | `typecheck` (cached) |
| `root` | `format`, `toolchain`, `terms`, `specs` (cached), `changes`, `changes-cases`, `versions` (cached), `release`, `marketplace` |

`boom:test` ran every test binary with no failure. Among them:
`tests/supersede.rs`, 21 passed, 0 failed; the unit tests
`claim::tests::a_supersession_has_its_form`,
`claim::tests::a_supersession_is_part_of_the_content` and
`mcp::tests::each_command_served_or_absent` pass. `root:specs` gives
`Totals: 25 passed, 0 failed`.

moon warned that it could not write task outputs to `.moon/cache/outputs`
in this worktree. The warning concerns the replay cache only and changes
no task's verdict.

## OpenSpec

- `openspec instructions apply --change boom-supersede-retires-an-accepted-claim --json`:
  `"state": "all_done"`, all 11 tasks `"done": true`.
- `openspec validate boom-supersede-retires-an-accepted-claim --strict`:
  `Change 'boom-supersede-retires-an-accepted-claim' is valid`, exit 0.

## Requirements and scenarios

Each scenario of `specs/claim-versions/spec.md` has a test of its own name
in `crates/boom/tests/supersede.rs`, and each test passes. A refusal is
checked through the helper `Place::refused`, which asserts exit 2, the words
on standard error, and a log that is unchanged.

| requirement | scenario | test | holds |
|---|---|---|---|
| boom supersede proposes a version that supersedes an accepted claim | A proposal that waits | `a_proposal_that_waits` | one `proposed` entry; the new claim `proposed`, the old one `accepted`; `boom query --repository shop` gives only the old one |
| | A supersession answered as JSON | `a_supersession_answered_as_json` | `boom-supersede/1` with `claim`, `version`, `status`, `supersedes`, `supersedes_version`, `reason`, `accept` |
| | A claim with no accepted version | `a_claim_with_no_accepted_version` | exit 2, "cash-only has no accepted version to supersede", nothing appended |
| | A claim that supersedes itself | `a_claim_that_supersedes_itself` | exit 2, "a claim does not supersede itself" and `boom propose --amends` |
| A supersession by a new claim | A new claim replaces an invariant | `a_new_claim_replaces_an_invariant` | an invariant, no `base`, binds `shop`, supersedes `<b1>` |
| | A new claim of another kind | `a_new_claim_of_another_kind` | a decision with the body of the file |
| | A new claim with no statement | `a_new_claim_with_no_statement` | exit 2, "a new claim gives its statement" |
| A supersession by an accepted claim of the same kind | A duplicate retired by the claim to keep | `a_duplicate_retired_by_the_claim_to_keep` | amends `<k1>` with its statement, body fingerprint, two sources and system binding |
| | New words for the claim to keep | `new_words_for_the_claim_to_keep` | the new statement, with the body and sources of `<k1>` |
| | An accepted claim of another kind | `an_accepted_claim_of_another_kind` | exit 2 for a claim of another kind and for a `--kind` of another kind |
| A version names the accepted version that it supersedes | The reason is part of the content | `the_reason_is_part_of_the_content` | one reason gives one version and one entry; another reason gives a second version |
| | A term superseded | `a_term_superseded` | exit 2, "only an invariant, a decision or an assumption is superseded" |
| | A reason of two lines | `a_reason_of_two_lines` | exit 2, "a reason is one line" |
| | A version no longer accepted, from another program | `a_version_no_longer_accepted_from_another_program` | the entry is not counted, and its judgment names `<b2>` as accepted |
| The accept of a supersession takes the claim it names out of force | A new claim accepted | `a_new_claim_accepted` | `<w1>` accepted, `<b1>` superseded, no accepted version; the query gives only `invoice-writers`; the log keeps each entry; `boom verify` rebuilds nothing |
| | The duplicate leaves force | `the_duplicate_leaves_force` | `<k2>` accepted, `<k1>` and `<d1>` superseded; the query and `boom search` no longer give the duplicate |
| | The claim named moved on | `the_claim_named_moved_on` | exit 2 naming `<b2>`; the key signs nothing; `<w1>` stays `proposed` |
| | One accept names both claims | `one_accept_names_both_claims` | exit 2 naming both versions; the key signs nothing; both stay `proposed` |
| The person reads what a supersession takes out of force | The accept lists the claim it supersedes | `the_accept_lists_the_claim_it_supersedes` | the list before the key gives ", which supersedes `<b1>`"; `boom-accept/2` names it under `supersedes` |
| | A superseded claim shown | `a_superseded_claim_shown` | no accepted version, `<b1>` superseded, `<w1>` accepted under `superseded_by`, in JSON and in words |
| | A version that supersedes shown | `a_version_that_supersedes_shown` | `supersedes` with the claim, version, statement and reason, in JSON and in words |

The pages the tasks name are written. `README.md` and `CLAUDE.md` describe
`boom supersede`, and `README.md` lists `supersede` among the commands of
`crates/boom`. The site's `commands.md` has its row and the section
`boom supersede` with `boom-supersede/1`. The skill `proposing-claims` says
when an agent runs it, and `using-boom` says to load that skill first and
to run it only for a duplicate or a replaced claim, never to get its own
change through. `root:terms` passes over the files the change touches.
