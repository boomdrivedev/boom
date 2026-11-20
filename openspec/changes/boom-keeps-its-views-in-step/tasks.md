# Tasks

## 1. The views hold the whole state

- [ ] 1.1 `versions` holds each version's `repositories`, `imported` and `history`, and its proposer as JSON; `views_head` holds `form` 2 (design 1). `store::load_state` builds from the views the same `State` a full replay builds. A unit test in `store.rs` compares the two over a store with persons, proposals, amendments, accepts, rejects and imported history. Proven by `devenv shell -- moon run boom:test`.
- [ ] 1.2 A store whose views are of form 1 gets views of form 2 on its next command, from a full replay (record-store, "Views of an older form"). Proven by a test in `tests/store.rs`, run by `devenv shell -- moon run boom:test`.

## 2. Catch-up

- [ ] 2.1 `store::catch_up` checks the head entry and the entries after it, applies each entry after the head through `judge::step`, and writes only the rows it touched, in one transaction (design 2). Missing views, or views of another form, are built again from the whole log. Proven by `tests/store.rs` "Views deleted", "Views behind the log" and "One transaction", run by `devenv shell -- moon run boom:test`.
- [ ] 2.2 `Store::read` answers from the views when the head names the log's last entry and that entry hashes to it, and catches up first otherwise. `Store::append_all` catches up, loads the state from the views, and judges and appends as before. `Store::state` loads the state from the views. Proven by the existing tests of `tests/claims.rs`, `tests/persons.rs`, `tests/attachments.rs` and `tests/import.rs`, unchanged, run by `devenv shell -- moon run boom:test`.
- [ ] 2.3 A changed head entry, and an entry appended by another program that does not follow the log's last entry, each make a command exit 3 naming the entry (record-log, "The head entry's words changed", "An entry appended by hand that does not follow"). Proven by tests in `tests/log.rs`, run by `devenv shell -- moon run boom:test`.
- [ ] 2.4 `boom log` checks the whole chain as it reads every entry (design 3). Proven by a test in `tests/log.rs`, run by `devenv shell -- moon run boom:test`.

## 3. Reads answer from the views

- [ ] 3.1 `query` and `search` in `cli.rs` take their state from `store::load_state`, not from a replay of the log; their JSON and words keep their form (design 3). Proven by the existing `tests/query.rs` and `tests/search.rs`, with the scenarios that edit the views changed to run `boom verify` first (repository-rules and claim-search, "A statement edited in the views", "A status edited in the views"), run by `devenv shell -- moon run boom:test`.
- [ ] 3.2 `boom mcp`'s `query` and `search` tools answer as their commands do. Proven by the existing `tests/mcp.rs`, run by `devenv shell -- moon run boom:test`.

## 4. boom verify

- [ ] 4.1 `boom verify` checks the whole chain, replays the whole log, rebuilds every view, and names each table whose rows differed, answering `boom-verify/1`; it exits 3 naming the first altered entry, and writes nothing, when the chain does not hold (design 6; record-store, "boom verify checks the log and rebuilds the views"; record-log, "An entry's words changed", "An entry removed"; command-line, "Verify answered as JSON"). Proven by tests in a new `tests/verify.rs` and in `tests/log.rs`, run by `devenv shell -- moon run boom:test`.

## 5. A query does not grow with the log

- [ ] 5.1 `store::entries_read` counts, per thread, the log rows a command reads (design 4). Proven by a unit test in `store.rs`, run by `devenv shell -- moon run boom:test`.
- [ ] 5.2 `tests/scale.rs` makes a store of 20 000 entries and shows that `boom query` and `boom search` read one entry, that a query three entries behind reads four, that `boom propose` reads one before it appends, and that a query gives the same bytes after the views are dropped (record-store, "A large log", "A large log, three entries behind", "A write on a large log"; claim-search, "A large log"). Proven by `devenv shell -- moon run boom:test`.
- [ ] 5.3 Over a random mix of 200 proposals, amendments, accepts and rejects, a store that keeps its views in step answers every query exactly as one whose views are dropped before each read (repository-rules, "The views and a full replay agree"). Proven by a test in `tests/scale.rs`, run by `devenv shell -- moon run boom:test`.

## 6. Records and pages

- [ ] 6.1 Records 0008 and 0009 (claim search) each end with `## Amendment of 2026-11-20 (proposed)`, saying what design 7 says, for the user to accept. `README.md` and `CLAUDE.md` say that boom answers from views kept in step with the log and that `boom verify` replays it. No task reads these pages (core.5); their writers keep them.
- [ ] 6.2 The change validates and archives cleanly. Proven by `devenv shell -- moon run root:changes` and `devenv shell -- moon run root:specs`.

## 7. The checks of the projects this change touches

- [ ] 7.1 The `boom` crate's checks pass: `devenv shell -- moon run boom:format boom:lint boom:typecheck boom:test`, and the root's: `devenv shell -- moon run root:format root:specs root:changes`.
