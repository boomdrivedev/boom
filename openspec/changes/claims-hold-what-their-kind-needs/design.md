# Design

## Context

See proposal.md for why. The user approved the claim model on 2026-11-20. The model is on the page "Boom Data Model".

Before this change, every claim had one statement, at least one scenario, and an optional body. Five words named a version's status: proposed, standing, replaced, rejected and history. Each command built the whole state from the views: every person, claim and version.

The rules this design relies on:

- core.1: the log is only appended. Old entries stay valid as they were written.
- core.2: the views are computed from the log. A new view form is built again from a full replay.
- core.5: the tests are tests of the `boom` crate, run by `boom:test`. The example is built by `boom:lint` and `boom:typecheck`. No task runs it.
- core.6: no new tool.

## Goals / Non-Goals

**Goals:**

- One claim shape. The kind decides which fields a version needs.
- New versions have no scenarios. Old entries stay valid.
- The five status words of the model.
- Each command reads only the view rows that it needs.

**Non-Goals:**

- An act that deprecates a claim. Only an imported record can be deprecated now.
- Checking the sections of a decision's body. Boom checks that a decision has a body. It does not read the body's headings.
- A new entry format. Entries stay `boom-entry/1`.

## Decisions

### 1. The old form and the new form

A version with scenarios is in the old form. A version with no scenarios is in the new form. Boom judges each form by its own rules:

| Form | Scenarios | Body rule |
|---|---|---|
| old | one or more, none empty | none |
| new | none | a decision needs a body |
| history | none or more, none empty | none |

The entry writes no `scenarios` field when the list is empty. The fingerprint content still includes `scenarios`, as `[]` when empty. So every old version keeps its fingerprint, including history versions that had no scenarios.

Not taken: a new entry format, `boom-entry/2`. It would change the record-log spec and every reader of the log for one rule. The cost of the chosen way: a program that writes an entry by hand with scenarios gets the old rules. Every entry that boom writes is in the new form.

### 2. Status words

`Status` has five values: `proposed`, `accepted`, `superseded`, `rejected` and `deprecated`. An accepted history version takes its file's status. A person's reject has a reason. A history version accepted as `rejected` has none. Boom uses the reason to tell the two apart, in the answer of `show` and in the judge's repeat check.

### 3. JSON formats

The answers whose fields or words change move to major version 2: `boom-propose/2`, `boom-accept/2`, `boom-reject/2`, `boom-show/2`, `boom-query/2`, `boom-search/2` and `boom-import/2`. The drafts file moves to `boom-import-drafts/2`. In `boom-show/2` the claim's `standing` field is `accepted`. In `boom-accept/2` the `replaced` field is `superseded`. `boom-log/1` and `boom-verify/1` do not change.

### 4. Each command reads only the rows it needs

`store::Scope` names the rows a command needs: every person, the claims named, and, for an import, every claim with an imported version.

- `Store::state_for(scope)` and `catch_up` load only that scope. `catch_up` adds the claims that the entries after the head name.
- `store::accepted_for(repository)` reads each claim's accepted version, and no other version. `boom query` and `boom search` use it.
- `boom show` already read one claim.
- `Store::state()` still loads everything. No command calls it now except through `load_state`, which tests use.

The views move to form 3, because the status words in them changed. A store with views of form 2 gets new views on its next command.

`store::versions_read()` counts, per thread, the versions read from the views. A test in `tests/scale.rs` uses it.

### 5. Measured

The scale harness, `crates/boom/examples/scale.rs`, built stores of 100,000 and 500,000 entries with 5,000 claims. On deneb, under load:

| Entries | Command | Before (e0b405a) | After |
|---|---|---|---|
| 100,000 | query | 152 ms | 26 ms |
| 500,000 | query | 564 ms | 25 ms |
| 500,000 | search | 600 ms | 28 ms |
| 500,000 | propose, accept (in process) | 1.1 s, 1.1 s | 2 ms, 2 ms |
| 500,000 | catch-up of 100 entries, then query | 1.5 s | 29 ms |

## Risks / Trade-offs

- [An import of a decision file with a title and no other text fails] → The refusal names the file and says that a decision needs a body. The person adds the body to the file, or leaves the file out.
- [A caller of a JSON answer reads major version 1] → No caller in the family reads these answers yet. The `/2` names make a mismatch fail fast.
- [A program writes an old-form entry by hand] → It counts under the old rules. Boom writes only new-form entries.

## Migration Plan

- A store with views of form 2 gets views of form 3 on its next command, from a full replay.
- Old entries need no change.
