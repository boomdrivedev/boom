---
number: 4
title: Each open change is validated as it would archive, alone against the tree's specs
status: proposed
date: 2026-11-15
decision-makers:
- Chuck Swanberg
- the chore booms-changes-gate-validates-each-open-change-as-archived's code stage
---

# Each open change is validated as it would archive, alone against the tree's specs

Status: proposed. Point 1 is the user's, boomdrivedev/blueprints
principle 16 as amended on 2026-11-15, which the chore's brief carries
to this repository; the record waits on their yes. Points 2 to 5 are
the code stage's own, made on the rules written and cheap to change,
and are the choices Boomdrive's record 0050 made for its own tree.

Deciders: the user. Their answer of 2026-11-15 on their card
`req_278df7ab2cc34988a3f5`: "OpenSpec's own validation stays; the rule
is about checks of what pages say". Principle 16 as amended keeps
"OpenSpec's `openspec validate` of a kit's specs and its open changes,
each change validated as it would archive". The brief of the chore
`booms-changes-gate-validates-each-open-change-as-archived`: "boom's
root:changes fails where an open change would not archive cleanly: each
open change's deltas applied to the specs, and the result valid under
openspec validate --strict, naming the change and what fails". The
chore's code stage, on points 2 to 5.

Sources: boomdrivedev/blueprints `design/principles.md` 16 at `3a5c19d`
on `design/a-tools-own-validation-stays`; Boomdrive's record 0050 and
core.61, and its `check-changes.sh`, the pattern this repository copies;
OpenSpec 1.14.3, the release `devenv.nix` pins, its `openspec archive`
and `openspec validate`; rules core.5 and core.6.

## Context and problem statement

`root:specs` runs `openspec validate --specs --strict`, and
`root:changes` ran `openspec validate --changes --strict`. A change can
pass that and still fail at its archive, or archive into specs that
`root:specs` then fails on `main`:

- `openspec archive` refuses a delta that does not apply: an ADDED
  requirement whose header its spec already holds with other words, or
  a MODIFIED requirement its spec does not hold. `openspec validate
  --changes --strict` passes both.
- `openspec archive` checks the specs it rebuilds without `--strict`,
  so it writes a spec with a warning, which `root:specs` fails once the
  archive lands. A new capability whose delta has no `## Purpose` gets
  the placeholder Purpose archive writes, and `--strict` fails it.

Each is found only at the archive, after the gate has passed the
branch. Principle 16 as amended keeps OpenSpec's own validation in the
gate, each change validated as it would archive.

## Considered options

For what a change is applied to:

1. **Each change alone, applied to a scratch copy of the tree's specs**
   (chosen).
2. Every open change applied in turn to one copy.
3. Each change applied to the tree's own specs, which the check then
   puts back.

For which specs are held to `--strict` after the archive:

1. **Each spec the archive writes, a new one or one that differs from
   the tree's** (chosen).
2. Every spec in the copy, as `openspec validate --specs --strict` reads
   them.

For where the check lives:

1. **A script of boom's own, `check-changes.sh`, copied from
   Boomdrive's** (chosen).
2. A script one tool of the family owns, which boom's gate calls.

## Decision outcome

1. **`root:changes` holds each open change to OpenSpec's validation as
   it would archive (the user's).** It still runs `openspec validate
   --changes --strict`. Then, for each open change, it runs `openspec
   archive` and validates the specs that result with `--strict`. A
   change that fails is named with OpenSpec's words. `check-changes.sh`
   does it, and core.7 says so.
2. **Each change is applied alone to a scratch copy of the tree's
   specs.** The copy holds the tree's specs, `openspec/config.yaml` and
   that change. Each change archives at its own merge, onto `main` as it
   stands then.
   - Not taken: every open change in turn. It judges a change by the
     other open changes and the order they are applied in, and neither
     is the change's own. Two open changes that conflict are found by
     the gate on the branch that lands second.
   - Not taken: the tree's own specs. A check writes nothing into the
     tree.
3. **The specs the archive writes pass `openspec validate --strict`,
   though `openspec archive` validates them without it.** `--strict` is
   how `root:specs` reads them once the change is archived.
4. **Only the specs the archive writes are validated.** A spec the
   change leaves as it was is `root:specs`' to check.
   - Not taken: every spec in the copy. A spec on `main` that fails
     would fail every open change, each named for what it did not write.
5. **A task left unchecked fails nothing.** The archive runs with
   `--yes`, and a change is open while its tasks are worked.
6. **The script is boom's own, copied from Boomdrive's.** Boomdrive's
   is in its own tree, for its own gate, and its record 0050 decides
   Boomdrive's alone; boom's gate copies its family's pattern into its
   own tree, as it copies flume's (core.5). The design is one,
   Boomdrive's, which this record and the script cite.
   - Not taken: a script one tool owns, which boom's gate calls. boom's
     gate would then run a file its tree neither holds nor pins
     (core.6).

### Consequences

- Good: a change that would not archive, or would archive into specs
  `root:specs` fails, fails its own branch's gate, named, before it
  lands.
- Good: a delta that adds a capability has to carry its own `##
  Purpose`. Archive reads a delta's Purpose only when it creates the
  capability, and the placeholder fails `--strict`.
- Bad: `root:changes` runs `openspec` once more for each open change,
  and once for each spec its archive writes.
- Bad: two copies of one script, Boomdrive's and boom's, can drift. A
  change to the design is made in Boomdrive's record first and carried
  to boom's.
- boom has no spec and no open change yet, so `root:changes` passes with
  none open until the first change is written.
- core.7 says so, in the commit that writes this record.
