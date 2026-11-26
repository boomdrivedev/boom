## 1. The claims

- [x] 1.1 Propose in Boom the invariant `a-link-is-the-hash-of-the-signed-part-it-names` for the whole system, and the invariants `boom-refuses-a-link-to-an-entry-its-log-does-not-hold` and `a-change-record-covers-only-what-its-approval-covers` and the decisions `boom-takes-an-entry-of-another-tools-type-and-judges-only-its-form-and-links`, `a-change-records-version-is-the-fingerprint-of-its-files-and-its-links` and `a-plans-approval-links-the-claims-it-covers-and-each-elaboration-the-log-holds` for `boom`. Each body passes `ste-lint.py` with 0 hard findings
- [x] 1.2 Propose the narrative `boom-records-the-acts-on-what-it-owns` for `boom`, which includes the claims of 1.1 that bind `boom`, a new version of `the-links-of-the-lineage` that includes `a-link-is-the-hash-of-the-signed-part-it-names` and names each hop's type, and a new version of the outline `boomdrive-design-book` that holds `boom-records-the-acts-on-what-it-owns`

## 2. The act hash, the links and another tool's entry

- [x] 2.1 In `entry.rs`, read and write an entry of the act `<tool>.<type>` with `links` and `content`, and compute each entry's act hash. `boom:test` proves the scenarios of "Each entry has an act hash" and "Another tool appends its entry"
- [x] 2.2 In `judge.rs`, refuse a link to an act hash that no counted entry before it has, and judge another tool's entry by its form, its actor and its links. `boom:test` proves the scenarios of "A link names an entry that the log holds" and "Boom judges another tool's entry by its form, its actor and its links"
- [x] 2.3 In `cli.rs`, add `boom append`, give `act_hash` and `links` in `boom log --json`, and the line of another tool's entry in `boom log`. `boom:test` proves the scenario "The line of a landing"
- [x] 2.4 In `store.rs`, keep the views `acts` and `links`, and raise the views' form from 10 to 11. `boom:test` proves the scenario "The views built again"

## 3. A plan's approval

- [x] 3.1 In `entry.rs` and `judge.rs`, add the act `approved`, signed, with its refusals. `boom:test` proves the scenarios of "A person approves a plan", "An approval links the claims it covers and each elaboration the log holds" and "An approval rests on what the log holds and the person accepted"
- [x] 3.2 In `cli.rs`, add `boom approve` and its line in `boom log`. `boom:test` proves the scenario "The line of an approval"

## 4. The change record

- [x] 4.1 In `entry.rs` and `judge.rs`, add the act `recorded`, its version, its files and its refusals. `boom:test` proves the scenarios of "An agent records a version of a change record", "Boom keeps the change's files by their fingerprints", "A version is the fingerprint of the change's files and links", "A new version amends the newest version" and "A version links its approval and covers what its approval covers"
- [x] 4.2 In `store.rs` and `cli.rs`, keep the view `changes`, add `boom record change`, and give a change record and its status in `boom show` and its line in `boom log`. `boom:test` proves the scenarios of "The status of a change record", "boom show gives a change record" and "The log names a version of a change record"

## 5. The walk

- [x] 5.1 In `lineage.rs` and `cli.rs`, add `boom lineage`. `boom:test` proves the scenarios of "boom lineage walks the links", "The target of a walk" and "The commands"

## 6. The tools of an agent

- [x] 6.1 In `mcp.rs`, serve `record_change` and `lineage`, and name `approve` and `append` absent. `boom:test` proves the scenarios of "Each tool is one of boom's commands"

## 7. The pages and the checks

- [x] 7.1 Say in `README.md` and `CLAUDE.md`, in Plain, that boom records a plan's approval and each version of a change record, takes another tool's entry, and walks the links
- [x] 7.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
