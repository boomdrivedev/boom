## 1. The decision

- [x] 1.1 Propose the decision `a-new-draft-replaces-the-one-before` in Boom, for `boom`, with the user's ruling of 2026-11-23, its options and its outcome, and with `--source` to `the-status-words`, `status-read-from-events` and `events-are-appended-never-changed`. `root:terms` passes on this change's files

## 2. A new draft supersedes the draft it amends

- [x] 2.1 In `judge::apply`, give a proposed base the status `superseded` when its amendment counts. `boom:test` proves the scenarios "A draft amended", "A first draft amended" and "The newer draft rejected"
- [x] 2.2 In `judge::check_accept` and `judge::check_reject`, refuse a draft that a newer draft superseded, and name the newer draft. `boom:test` proves the scenarios "An old draft accepted" and "An old draft rejected"
- [x] 2.3 In `judge::check_proposal`, refuse an amendment of a draft that a newer draft superseded, and name the newer draft. `boom:test` proves the scenario "An old draft amended"
- [x] 2.4 `boom:test` proves the scenario "The notes on an old draft"

## 3. The base that an accept checks

- [x] 3.1 In `judge::check_accept`, check the nearest version in the line of bases that a person accepted, or none, against the accepted version of the claim. `boom:test` proves the scenarios "The newest draft accepted", "An amendment of a rejected version" and "A draft of a draft on an old base"
- [x] 3.2 `boom:test` proves the scenarios "Two amendments of one base", "The same words on the new base", "Proposed, standing, replaced", "Rejected" and "History accepted"

## 4. The views

- [x] 4.1 Raise the views' form in `store.rs` from 8 to 9, so that each store builds its views again from its log. `boom:test` proves the scenario "A log from before this rule"

## 5. The pages and the gate

- [x] 5.1 Say in `README.md` and `CLAUDE.md`, in Plain, that a new draft supersedes the draft it amends
- [x] 5.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
