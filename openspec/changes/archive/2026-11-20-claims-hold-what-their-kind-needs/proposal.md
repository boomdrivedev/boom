# Proposal

## Why

The user approved Boom's data model on 2026-11-20. A claim has one shape. Its kind says which fields it needs. A constitution rule is one sentence that can already be proven false, so it needs no scenario. The status words change to proposed, accepted, rejected, superseded and deprecated.

The scale test of 2026-11-20 found one more cost. A command reads one log entry, but it loads every version of every claim from the views. At 500,000 entries a query took 564 ms and a propose took 1.1 s. A command must load only the rows it needs.

## What Changes

- Each kind needs a statement. A decision also needs a body: its context, options, outcome and consequences. An invariant's body is optional and says what it rules out. An assumption's body is optional and says what would challenge it.
- **BREAKING**: A new version has no scenarios. `boom propose` no longer takes `--scenario`. An import's drafts give names only. An old entry that has scenarios stays valid, and boom still shows its scenarios.
- **BREAKING**: The status words are `proposed`, `accepted`, `rejected`, `superseded` and `deprecated`. `standing` becomes `accepted`. `replaced` becomes `superseded`. A record that came in as history shows its file's own status: `superseded`, `deprecated` or `rejected`. `history` goes. `deprecated` is a word only. No act makes it yet.
- **BREAKING**: The answers whose words or fields change move to their next major form: `boom-propose/2`, `boom-accept/2`, `boom-reject/2`, `boom-show/2`, `boom-query/2`, `boom-search/2`, `boom-import/2` and `boom-import-drafts/2`.
- `boom query` and `boom search` load only the accepted versions. A write loads only the claims its entries name. `boom show` already loads one claim.
- Amendments to records 0007 and 0008, in plain English, proposed for the user to accept.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `claim-versions`: what each kind needs, no scenarios on new versions, the status words, `boom show`'s answer.
- `record-import`: drafts give names only. History shows the status of its file.
- `repository-rules`: `boom query` answers `boom-query/2`.
- `claim-search`: `boom search` answers `boom-search/2`.
- `agent-tools`: the `propose` tool takes no scenarios.
- `record-store`: a command loads only the rows it needs.

## Impact

- `crates/boom/src/claim.rs`, `judge.rs`, `store.rs`, `cli.rs`, `import.rs`, `mcp.rs`.
- Most tests, which propose with `--scenario`.
- `crates/boom/examples/scale.rs`, re-run at 100,000 and 500,000 entries.
- `docs/adr/0007-…` and `docs/adr/0008-…`: an amendment each, proposed.
- `README.md`, `CLAUDE.md` and the plugin's words where they name scenarios or `standing`.
- A caller of the JSON reads the new major forms. No caller in the family reads them yet.
