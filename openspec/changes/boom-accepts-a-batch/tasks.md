No task adds a part of the repository. The work is in the crate `boom`,
so this change adds no moon project. Each task names the moon task that
proves it, run as `sluice exec main -- moon run <project>:<task>`. Each
scenario of the delta specs is one test, named by its scenario:
`persons-acts` and the batches of `narrative-page` in
`crates/boom/tests/batches.rs`, `claim-edges` in
`crates/boom/tests/edges.rs`, and the pages of `narrative-page` in
`crates/boom/tests/page.rs`.

## 1. The edge cause

- [x] 1.1 In `claim.rs`, add `cause` to a version, written after each other edge and part of its fingerprint; in `judge.rs`, refuse a cause that the store does not hold or that names its own claim; in `store.rs`, keep it in `edges` as the edge `cause`. Tests for "A revision with its cause", "A cause that the store does not hold" and "The cause is part of the content". Proved by `boom:test`
- [x] 1.2 In `cli.rs`, add `boom propose --cause`, resolved as `--source` is, and give the cause in `boom show <name>@<version>`. Proved by `boom:test`

## 2. The batches

- [x] 2.1 Add `batch.rs`: the batches that wait, computed from the views, each with its shape, its versions, its counts, its command and what refuses it, the oldest first. Tests for "The three shapes" and "A version that a narrative includes is no ruling". Proved by `boom:test`

## 3. The accept

- [x] 3.1 In `cli.rs`, take for each target that is a narrative its proposed includes, refuse a closed include, and keep each version one time. Tests for "A narrative and its proposed claim", "A narrative that includes a rejected version", "A narrative that includes a superseded draft", "Two narratives that include one claim" and "A narrative accepted again". Proved by `boom:test`
- [x] 3.2 Add `boom accept --cause`. Tests for "A cause and its two versions", "A superseded draft is left out" and "A cause that no version names". Proved by `boom:test`
- [x] 3.3 Write the list of what an accept takes on standard error before the key signs, and list each version under `accepted`. Test for "The list before the key". Proved by `boom:test`
- [x] 3.4 Add `boom accept --waiting`. Tests for "The batches listed", "Nothing waits", "Each batch in one sign" and "A refused batch is left out". Proved by `boom:test`

## 4. The page

- [ ] 4.1 In `page.rs`, add the view "Waiting on you" at `<base>/waiting`, first in the bar with its count. Tests for "The view of three batches", "A narrative batch refused" and "Nothing waits". Proved by `boom:test`
- [ ] 4.2 On a narrative's page, list under "To accept" what its accept takes, or name the includes that refuse it. Tests for "What the accept takes" and "An include that refuses the accept". Proved by `boom:test`

## 5. The words

- [ ] 5.1 Give `boom accept --cause` and `--waiting`, `boom propose --cause` and the view in the README and the site's page of the commands. Proved by `site:build` and `root:format`
