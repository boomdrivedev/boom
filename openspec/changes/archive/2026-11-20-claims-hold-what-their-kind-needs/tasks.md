# Tasks

## 1. The claim shape

- [x] 1.1 A new version has no scenarios, and a decision needs a body. An old version keeps its scenarios and its rules. `boom propose` takes no `--scenario`. Proven by `crates/boom/src/claim.rs` unit tests and `tests/claims.rs` ("An invariant needs a statement alone", "A decision with no body is refused"), run by `devenv shell -- moon run boom:test`.
- [x] 1.2 An import's drafts give names only, in `boom-import-drafts/2`. An imported decision needs a body. Proven by `tests/import.rs`, run by `devenv shell -- moon run boom:test`.

## 2. Status words and JSON

- [x] 2.1 The status words are `proposed`, `accepted`, `superseded`, `rejected` and `deprecated`. An accepted history version takes its file's status. Proven by `tests/claims.rs` and `tests/import.rs`, run by `devenv shell -- moon run boom:test`.
- [x] 2.2 The answers of `propose`, `accept`, `reject`, `show`, `query`, `search` and `import` are major version 2. Proven by `tests/command_line.rs`, `tests/query.rs`, `tests/search.rs` and `tests/mcp.rs`, run by `devenv shell -- moon run boom:test`.

## 3. Each command reads only the rows it needs

- [x] 3.1 `boom query` and `boom search` read each claim's accepted version only. A write reads each person and the claims it names. The views move to form 3. Proven by `tests/scale.rs` ("A command reads only the versions it needs") and `tests/store.rs`, run by `devenv shell -- moon run boom:test`.
- [x] 3.2 The scale harness writes claims in the new form. Re-run at 100,000 and 500,000 entries; the results are in design.md, decision 5.

## 4. Records and pages

- [x] 4.1 Records 0007 and 0008 each end with an amendment of 2026-11-20, in plain English, proposed. `README.md` says what each kind needs and the new status words. No task reads these pages (core.5).
- [x] 4.2 The change validates and archives cleanly. Proven by `devenv shell -- moon run root:changes` and `devenv shell -- moon run root:specs`.

## 5. The checks of the projects this change touches

- [x] 5.1 The `boom` crate's checks pass, and the root's: `sluice exec main -- moon run boom:format boom:lint boom:typecheck boom:test root:format root:specs root:changes`.
