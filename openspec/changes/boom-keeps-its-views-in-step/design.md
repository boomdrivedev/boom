# Design

## Context

See proposal.md for why.

The store is one SQLite file (record 0005). It holds the log, the blobs, and five view tables: `views_head`, `persons`, `claims`, `versions` and `judgments`. The code is in `crates/boom/src/store.rs`.

Today:

- `Store::read` reads every log row, checks the whole chain, and checks that `views_head` names the last row. If it does not, it replays the whole log and writes every view again.
- `Store::append_all` reads every row, checks the whole chain, and replays the whole log to get a `judge::State`. Then it judges each new entry against that state.
- `Store::state` reads every row and replays them. `propose`, `accept`, `register`, `init` and the imports call it.
- `query` and `search` in `cli.rs` read every row inside `Store::read` and replay them again with `judge::replay`. Record 0008, point 2, asks for that. They never read the view tables.
- The `versions` view lacks what a version binds (`repositories`), where it came from (`imported`) and its history. So a `State` cannot be built from the views today.

The rules this design relies on:

- core.1: the log is only appended. Nothing here changes an entry.
- core.2: a view is computed from the ledger and never stored as the truth. The views stay computed: deleting them changes no answer, and `boom verify` rebuilds them from the log.
- core.5: the new test is a test of the `boom` crate, run by `boom:test`.
- core.6: no new tool.

This design departs from the interview's `caches-derived-only` in one way. That requirement says a cache is "never read as the truth". After this change, a view edited by hand changes an answer until `boom verify` runs. The reason: an answer that replays the whole log costs time in proportion to the log, on every command and every agent session. The views are still never the source: any command that finds them missing, of another form, or behind the log builds them from the log.

## Goals / Non-Goals

**Goals:**

- A command reads the log's entries after the views' head, and one entry more. It reads no other entry.
- A write updates only the view rows its entries touch.
- `boom verify` keeps the full check and the full replay.
- The views always give what a full replay gives.

**Non-Goals:**

- An index kept for `boom search`. Search still builds its full-text table in memory, from the standing claims in scope, on each call. That grows with the claims in scope, not with the log.
- A `verify` tool in `boom mcp`.
- Any change to the log's entries, their form or their hashes.
- Moving the store into the drive's log (record 0012's draft, a later change).

## Decisions

### 1. The views hold the whole state

The views must hold everything `judge::State` holds, so a command can build the state from them.

- `versions` gains `repositories` (JSON list), `imported` (JSON or null) and `history` (JSON or null). `proposer` holds the actor as JSON.
- `views_head` gains `form`, an integer. This change's form is 2.
- `store::load_state(conn)` builds a `State` from `persons`, `claims` and `versions`. The first owner is the owner whose registration has the lowest entry number.

Not taken: one JSON column holding the whole `State`. Every write would then rewrite all of it.

### 2. Catch-up: one way, for reads and writes

`store::catch_up(tx)` brings the views to the log's last entry. It runs inside a transaction. Every command calls it first.

```
read views_head (seq, hash, form)
  form is not 2, or a view table is missing
      --> rebuild: read the whole log, check the whole chain,
          replay it, write every view           (today's path)
  else
      read the log row at seq; its hash must equal head.hash
          and its words must hash to it          (else: altered, exit 3)
      read the rows after seq, in order; each must follow the one before
      state = load_state
      for each row: judgment = judge::step(state, row)
                    write the judgment, the rows its entry touched, the head
```

A read first takes a deferred transaction. It reads the log's last row and `views_head`. If the head names that row and its hash, and the row's words hash to it, the read answers from the views. If not, it takes an immediate transaction and catches up, as `Store::read` already does for a rebuild.

A write takes an immediate transaction, catches up, loads the state, then judges and appends its entries as `append_all` does today.

Not taken: a background process that keeps the views in step. Nothing runs between commands (blueprints principle 5).

### 3. What a read answers from

- `query` and `search` call `load_state` and then `State::standing_for`, the same function they call today. Only the source of the state changes. The answer's JSON and words keep their form.
- `show` already reads `claim_view` from the views.
- `log` reads every entry by its nature. It checks the whole chain as it reads, at no extra cost.

### 4. Counting the entries a command reads

`store::entries_read()` gives the number of log rows the current thread has read. Every function that reads log rows adds to it. It is a thread-local counter, so tests that run on other threads do not disturb it. The command line runs in the test's own thread, so a test reads the count of one command exactly.

Not taken: a timing bound. Time on a shared host varies, and a gate must not fail by chance.

### 5. The large-log test

`crates/boom/tests/scale.rs` makes a store with 20 000 entries. It writes them with the library, in batches through `Store::append_all`. Most are proposals bound to the repository `other`. A few are bound to `shop` and accepted by a registered person, signed with the test key the other tests use. Then it checks:

- `boom query --repository shop --json` reads one entry;
- after another program appends three proposals, the next query reads four;
- `boom search` reads one entry;
- `boom propose` reads the log's last entry, and no other, before it appends;
- the query gives the same bytes as a query after the views are dropped, which rebuilds them by a full replay.

A second test runs the same random mix of 200 proposals, amendments, accepts and rejects twice. One store keeps its views in step. The other has its views dropped before every read. Every answer must match.

### 6. `boom verify`

`boom verify` takes an immediate transaction. It reads the whole log, checks the whole chain, and replays it. It compares each view table with what the replay gives, as sets of rows. It writes every view again from the replay. It answers `boom-verify/1`:

```json
{ "format": "boom-verify/1", "head": {"number": 3, "hash": "…"}, "chain": "holds", "rebuilt": ["versions"] }
```

It exits 0 when the chain holds, whether or not a view differed. It exits 3, naming the first altered entry, when the chain does not hold, and writes nothing.

Not taken: exit 1 when a view differed. The command line's exit codes are 0, 2 and 3, and verify did what was asked.

### 7. Records 0008 and 0009

Each gets a section at its end, `## Amendment of 2026-11-20 (proposed)`. It says what changes and why. The record's status stays `accepted`. The amendment's own status is `proposed`, for the user to accept.

- Record 0008, point 2: `boom query` answers from views kept in step with the log, not from a replay of the whole log. `boom verify` replays it.
- Record 0009 (claim search): `boom search` searches the standing claims the views hold. It still builds its full-text table in memory and keeps no index.

## Risks / Trade-offs

- [A change to an older entry goes unseen until `boom verify` runs] → Every command still checks the head entry and the entries after it. `boom log` checks the whole chain. `boom verify` checks all of it, and a host can run it on a schedule.
- [A view edited by hand changes answers] → `boom verify` finds and rebuilds it. A program that can edit the views can also edit the log's latest entries, so the gain from replaying every time was small.
- [Incremental views drift from a full replay] → The second test in decision 5 compares them over a random mix of acts. `boom verify` names any view that differs.
- [`load_state` grows with the number of claims] → It grows with claims, not with entries. A claim has many entries over its life, so this is much smaller. An index by repository is a later change if it is needed.

## Migration Plan

- A store with views of form 1 gets views of form 2 on its first command, from a full replay. This is the path boom already takes when views are missing. No other code carries the move.
- Rollback: an older boom sees `views_head` with an extra column. It rebuilds the views from the log, as it does for any views it does not trust.
