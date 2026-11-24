## 1. The claim

- [x] 1.1 Propose the invariant `an-entry-is-judged-by-the-rules-it-was-written-under` in Boom, for `boom`, with the person's ruling of 2026-11-24, and with `--source` to `events-are-appended-never-changed` and `status-read-from-events`. Its body passes `ste-lint.py` with 0 hard findings

## 2. An entry names its rule set

- [x] 2.1 In `entry.rs`, give an entry `rules`, written after `at` where it is not 0 and read as 0 where it is absent. `boom:test` proves that an entry of rule set 0 writes and reads as before
- [x] 2.2 In `cli.rs` and `page.rs`, write each new entry with `judge::RULES`, and give `rules` in `boom log --json`. `boom:test` proves the scenario "A new entry names its rule set"

## 3. Replay judges each entry by its rule set

- [x] 3.1 In `judge.rs`, hold "a narrative binds one level" only for an entry of rule set 1 or later. `boom:test` proves the scenarios "An accept signed before a rule" and "A proposal written under the rule"
- [x] 3.2 In `judge.rs`, hold the refusals of a draft that a newer draft superseded only for an entry of rule set 1 or later, and judge such a draft as a draft that waits under rule set 0. `boom:test` proves the scenarios "A draft accepted before a newer draft superseded it", "A draft rejected before a newer draft superseded it" and "A superseded draft accepted or rejected under the rule"
- [x] 3.3 In `judge.rs`, refuse an entry of a rule set greater than `RULES`. `boom:test` proves the scenario "A rule set that this boom does not hold"

## 4. The views

- [x] 4.1 Raise the views' form in `store.rs` from 9 to 10, so that each store builds its views again from its log

## 5. The pages and the gate

- [x] 5.1 Say in `README.md` and `CLAUDE.md`, in Plain, that each entry is judged by the rules it was written under
- [x] 5.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
