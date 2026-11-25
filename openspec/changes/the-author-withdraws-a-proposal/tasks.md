## 1. The claim

- [x] 1.1 Propose in Boom a new version of `the-status-words` that names `withdrawn`, amending its newest draft, with `--source` to `the-author-of-a-proposal-withdraws-it`. Its body passes `ste-lint.py` with 0 hard findings

## 2. The act and the status

- [x] 2.1 In `entry.rs`, add the act `withdrawn`, with `versions` and `reason`, and give a reject of several versions the form `versions` and `reason`, a reject of one keeping `name` and `version`. `boom:test` proves that each form writes and reads back as itself
- [x] 2.2 In `judge.rs`, add the status `withdrawn`, and judge a withdrawal: unsigned, by its author, of proposed versions, whole. `boom:test` proves the scenarios of "The author withdraws a proposed version" and "Only a note or a withdrawal counts with no signature"
- [x] 2.3 In `judge.rs`, `batch.rs` and `view.rs`, close a withdrawn version: no accept, reject or withdrawal of it, no accept of a narrative that includes it, and no answer to a note. `boom:test` proves the scenarios of "A withdrawn version is closed"
- [x] 2.4 In `judge.rs`, judge a reject of several versions whole. `boom:test` proves the scenarios of "Several versions rejected together"

## 3. The commands

- [x] 3.1 In `cli.rs`, add `boom withdraw`, answered as `boom-withdraw/1`, and give `withdrawn` in `boom show`. `boom:test` proves the scenarios "A withdrawal answered as JSON", "An agent's withdrawal", "A withdrawal with no author" and "Withdrawn"
- [x] 3.2 In `cli.rs`, let `boom reject` take several targets, answered as `boom-reject/3`. `boom:test` proves "Three rejected together"
- [x] 3.3 In `cli.rs` and `store.rs`, add `boom search --proposed`. `boom:test` proves the scenarios of "The versions that wait"

## 4. The pages and the gate

- [x] 4.1 Say in `README.md`, `CLAUDE.md`, the site's `commands.md` and the skills `using-boom` and `proposing-claims`, in Plain, that an author withdraws its own proposal, that a person rejects several versions together, and that `boom search --proposed` lists what waits
- [x] 4.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `claude-code:format`, `claude-code:lint`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
