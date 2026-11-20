---
number: 4
title: Each open change is validated as it would archive, alone against the tree's specs
status: proposed
date: 2026-11-15
decision-makers:
- Chuck Swanberg
- the chore booms-changes-gate-validates-each-open-change-as-archived's code stage
- the chore booms-checks-catch-changes-that-wont-archive's code stage
---

# Each open change is validated as it would archive, alone against the tree's specs

Status: proposed. Point 1 is the user's: boomdrivedev/blueprints
principle 16 as amended on 2026-11-15. The brief of the chore carries
the principle to this repository. The record waits for the user's yes.
Points 2 to 5 are the code stage's own. The code stage made them on the
written rules, and they are cheap to change. They are the choices that
Boomdrive's record 0050 made for its own tree.

Deciders: the user. Their answer of 2026-11-15 on their card
`req_278df7ab2cc34988a3f5`: "OpenSpec's own validation stays; the rule
is about checks of what pages say". Principle 16 as amended keeps
"OpenSpec's `openspec validate` of a kit's specs and its open changes,
each change validated as it would archive". The brief of the chore
`booms-changes-gate-validates-each-open-change-as-archived`: "boom's
root:changes fails where an open change would not archive cleanly: each
open change's deltas applied to the specs, and the result valid under
openspec validate --strict, naming the change and what fails". The code
stage of the chore, for points 2 to 5.

Amended 2026-11-16, point 7, by the code stage of the chore
`booms-checks-catch-changes-that-wont-archive`, on the bolt
`bolt/booms-changes-are-checked-as-archived`. Its brief: "Boom's checks
fail a change OpenSpec would refuse to archive, such as one with a
requirement over 500 characters, so it's caught while the code is
written instead of at the end (as Boomdrive's and Flume's checks do)".
`check-changes.sh` already did this. When it was run by hand over
Flume's cases, it failed each case, and named the change with
OpenSpec's words. A MODIFIED requirement past 500 characters was one of
those cases. Nothing in the gate showed this, because the refusals had
no test. Flume's record 0073, point 7, runs its copy of the check over
cases of its own. sluice's record 0128, amended 2026-11-16 by the same
brief, copies those cases. The point is the code stage's own. The code
stage made it on the written rules (core.5, core.7), and it is cheap to
change.

Sources:

- boomdrivedev/blueprints `design/principles.md` 16 at `3a5c19d` on
  `design/a-tools-own-validation-stays`.
- Boomdrive's record 0050 and core.61, and its `check-changes.sh`, the
  pattern that this repository copies.
- OpenSpec 1.14.3, the release that `devenv.nix` pins, with its
  `openspec archive` and `openspec validate`.
- Rules core.5 and core.6.
- For point 7: Flume's record 0073, point 7, its core.59, and its
  `check-changes-cases.sh` at `cdc3ddf3` on its main. Also sluice's
  record 0128, point 8.

## Context and problem statement

`root:specs` runs `openspec validate --specs --strict`. `root:changes`
ran `openspec validate --changes --strict`. A change can pass that
check and still fail at its archive. Or it can archive into specs that
`root:specs` then fails on `main`:

- `openspec archive` refuses a delta that does not apply. Examples are
  an ADDED requirement whose header its spec already holds with other
  words, and a MODIFIED requirement that its spec does not hold.
  `openspec validate --changes --strict` passes both.
- `openspec archive` checks the specs that it rebuilds without
  `--strict`. So it writes a spec with a warning, and `root:specs` fails
  that spec after the archive lands. A new capability whose delta has no
  `## Purpose` gets the placeholder Purpose that the archive writes.
  `--strict` fails that Purpose.

Each problem shows only at the archive, after the gate passed the
branch. Principle 16 as amended keeps OpenSpec's own validation in the
gate, with each change validated as it would archive.

## Considered options

For the specs that a change is applied to:

1. **Each change alone, applied to a scratch copy of the tree's specs**
   (chosen).
2. Every open change applied in turn to one copy.
3. Each change applied to the tree's own specs, which the check then
   restores.

For which specs are held to `--strict` after the archive:

1. **Each spec the archive writes, a new one or one that differs from
   the tree's** (chosen).
2. Every spec in the copy, as `openspec validate --specs --strict` reads
   them.

For where the check lives:

1. **A script of boom's own, `check-changes.sh`, copied from
   Boomdrive's** (chosen).
2. A script that one tool of the family owns, which boom's gate calls.

For how a test shows that the check does what it says (point 7):

1. **Flume's cases copied whole, a task apart, each a scratch tree
   written as data** (chosen).
2. No test of its own. The run of the check in the gate, over the
   tree's own changes, stands for one.
3. The cases inside `check-changes.sh`.
4. Cases of boom's own, with one more case for an ADDED requirement past
   500 characters.

## Decision outcome

1. **`root:changes` holds each open change to OpenSpec's validation as
   it would archive (the user's).** It still runs `openspec validate
   --changes --strict`. Then, for each open change, it runs `openspec
   archive` and validates the resulting specs with `--strict`. It names
   a change that fails, with OpenSpec's words. `check-changes.sh` does
   this, and core.7 says so.
2. **Each change is applied alone to a scratch copy of the tree's
   specs.** The copy holds the tree's specs, `openspec/config.yaml` and
   that change. Each change archives at its own merge, onto `main` as
   `main` is at that time.
   - Not taken: every open change in turn. That option judges a change
     by the other open changes, and by the order in which they apply.
     Neither belongs to the change. The gate finds a conflict between
     two open changes on the branch that lands second.
   - Not taken: the tree's own specs. A check writes nothing into the
     tree.
3. **The specs that the archive writes pass `openspec validate
   --strict`, though `openspec archive` validates them without it.**
   `root:specs` reads them with `--strict` after the change is archived.
4. **The check validates only the specs that the archive writes.**
   `root:specs` checks a spec that the change leaves as it was.
   - Not taken: every spec in the copy. A spec on `main` that fails
     would then fail each open change. The check would name each change
     for a spec that it did not write.
5. **A task that is not checked fails nothing.** The archive runs with
   `--yes`. A change is open while work on its tasks continues.
6. **The script is boom's own, copied from Boomdrive's.** Boomdrive's
   script is in its own tree, for its own gate. Its record 0050 decides
   for Boomdrive alone. boom's gate copies the pattern of its family
   into its own tree, as it copies flume's (core.5). There is one
   design, Boomdrive's, and this record and the script cite it.
   - Not taken: a script that one tool owns, which boom's gate calls.
     boom's gate would then run a file that its tree does not hold and
     does not pin (core.6).
7. **`root:changes-cases` runs the check over cases of its own (option
   1, amended 2026-11-16).** `check-changes-cases.sh` is Flume's script,
   copied whole except for its header. It runs `check-changes.sh` over
   five scratch trees:
   - a change that does not validate
   - a change that the archive refuses
   - a new capability with no Purpose
   - a MODIFIED requirement past OpenSpec's 500 characters
   - two changes applied alone, with their tasks open.

   It checks the exit and the words of the check, and that the tree is
   as it was. The cases write the changes into the scratch trees as
   data, and only OpenSpec reads them. That reading is a tool's own
   validation of the files that it works on. So no task reads a
   Markdown file of the tree (core.5). The inputs of the task are the
   two scripts and the pinned shell. The task reads nothing else of the
   tree.
   - Not taken: option 2. The tree has no open change, so the run of
     the check over the tree shows no refusal. A change to the script
     that let a long requirement pass would pass the gate. The problem
     would show only at an archive.
   - Not taken: option 3. The cases would run over each tree that the
     check reads.
   - Not taken: option 4. `openspec validate --changes --strict`
     already fails an ADDED requirement past 500 characters. The case
     of a change that does not validate stands for it. The MODIFIED
     requirement is the length that only the specs of the archive show.
     A copy that is kept whole stays one diff from Flume's.
   - No OpenSpec change carries this point. boom's specs hold no
     requirement on its gate, so the work changes no requirement.
   - Boomdrive's record 0050, the design of the family for the check,
     has no cases. Boomdrive decides whether it takes Flume's cases.

### Consequences

- Good: a change fails the gate of its own branch before it lands, with
  its name, when it would not archive. It also fails when it would
  archive into specs that `root:specs` fails.
- Good: a delta that adds a capability must carry its own `##
  Purpose`. The archive reads the Purpose of a delta only when it
  creates the capability. The placeholder fails `--strict`.
- Bad: `root:changes` runs `openspec` once more for each open change,
  and once for each spec that its archive writes.
- Bad: two copies of one script, Boomdrive's and boom's, can drift. A
  change to the design goes into Boomdrive's record first, and then
  into boom's.
- Good: `root:changes-cases` fails a change to `check-changes.sh` that
  lets a change pass when the archive would refuse it. It also fails a
  change to the script that lets a change pass when that change
  archives into a spec that `root:specs` fails. The failure names the
  case (point 7).
- Bad: `root:changes-cases` runs `check-changes.sh` over five trees.
  Each tree runs `openspec` two to four times. The task takes about
  eight seconds on deneb.
- Bad: boom's `check-changes-cases.sh` and Flume's can drift, as the two
  `check-changes.sh` can. A change to the cases goes into Flume's record
  first, and then into boom's.
- boom has no spec and no open change yet, so `root:changes` passes
  with none open until the first change is written.
- core.7 says so, in the commit that writes this record. The details of
  core.7 name the scratch trees of the cases, in the commit that amends
  this record.
