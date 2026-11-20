# Proposal

## Why

Every boom command reads the whole log. It checks every hash in the chain, and it replays every entry. Then it answers. `boom query` and `boom search` do this on every call, and so does every write. The view tables hold the same answers, but no answer reads them. So each command gets slower as the log grows. An agent's session starts with `boom query`, so every session pays this cost.

This is Boom's first step, by the user's decision of 2026-11-20. Next, boom's own constitution goes into Boom. The log must stay fast as it grows.

## What Changes

- A write applies only the entries after the views' head. It checks the chain from the last hash it checked. It updates the rows those entries touch, in the transaction that appends them.
- A command that changes the log builds its state from the view tables. It does not replay the log.
- `boom query` and `boom search` read the view tables. First they make one check: the views' head must match the log's last entry. If it does not match, they catch up first, the same way a write does.
- `boom show` catches up the same way.
- A new command, `boom verify`, checks the whole chain and replays the whole log. It rebuilds any view that differs from the replay, and it says which.
- **BREAKING**: A command no longer checks the whole chain each time it opens the store. It checks the views' head and the entries after it. Only `boom verify` finds a change to an older entry.
- **BREAKING**: A view changed by hand now changes what `boom query` and `boom search` give, until `boom verify` runs.
- The view tables hold all of each version: what it binds and where it came from. A store whose views have the old form gets new views once, from a full replay.
- An amendment to records 0008 and 0009. They say boom replays the log for each answer. The amendment says boom answers from views it keeps in step with the log. It is proposed for the user to accept.
- A test with a large generated log. It shows that a query reads the same number of log entries whatever the log's length.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `record-store`: the views are kept in step, entry by entry, and a read answers from them; `boom verify` checks the log and rebuilds the views.
- `record-log`: each command checks the chain from the views' head; `boom verify` checks all of it.
- `repository-rules`: `boom query` answers from the views, caught up to the log's last entry.
- `claim-search`: `boom search` searches the views, caught up to the log's last entry.
- `command-line`: boom offers `verify`.

## Impact

- `crates/boom/src/store.rs`: catch-up, the views' new form, state read from the views, `verify`.
- `crates/boom/src/judge.rs`: `apply` and `step` used one entry at a time on a state read from the views.
- `crates/boom/src/cli.rs`: `query`, `search`, `propose`, `accept`, `reject`, `register` and the imports read state from the views; a new `verify` command.
- `crates/boom/tests/`: the scenarios that edit the views or an older entry change; a new test with a large log.
- `docs/adr/0008-…` and `docs/adr/0009-an-agent-finds-claims-…`: an amendment each, proposed.
- `README.md` and `CLAUDE.md`: they say boom replays the log for each answer; they change to say what boom does now.
- `boom mcp` serves no `verify` tool in this change. Its `query` and `search` tools answer from the views, as their commands do.
- No caller changes. The JSON of every answer keeps its form.
