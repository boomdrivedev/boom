---
number: 9
title: A repository moves its records and rules into Boom whole, and a person accepts many versions with one signature
status: accepted
date: 2026-11-18
decision-makers:
- boomdrive-design
---

# A repository moves its records and rules into Boom whole, and a person accepts many versions with one signature

Status: accepted. boomdrive-design's own, on records 0005, 0006, 0007
and 0008 and the interview's requirements those records keep. Cheap to
change until a repository has moved.

Deciders: boomdrive-design.

Sources: records 0005 to 0008; `interview.md`, the user's answers of
2026-11-01: `persons-acts-signed-with-presence`,
`import-proposes-a-person-accepts`, `import-once`, `cite-by-name`,
`caches-derived-only`, `hook-before-each-edit` and `callers-pin-boom`;
core.2.

## Context and problem statement

Record 0007 imports a repository's decision records and rules page, and
record 0008 gives the repository's agents the claims that stand for it
from `boom query`. Nothing yet says what the repository keeps once it
has moved, when it may move, or how a person accepts what one import
proposes. A repository with a hundred records and a hundred rules
proposes two hundred versions, and `boom accept` signs each as an entry
of its own, so with a security key the person touches it two hundred
times (record 0006).

## Considered options

For what a moved repository keeps:

1. Its records and rules move whole, and its pages leave it in one
   change once a person has accepted the import (chosen).
2. Its pages stay beside Boom as a copy for readers.
3. Its rules move first and its records later.

For a person accepting many versions at once:

4. One signature, the person present, over every version of one accept
   (chosen).
5. One signature for each version, as built.

## Decision outcome

1. **A repository moves whole, once.** Every decision record and every
   rules page is imported at once, and a person accepts the import.
   Then one change removes the records and the rules pages from the
   repository, declares it in `.config/boom.toml`, and drops the rules
   page from what its agents' instructions load. That change lands only
   after the accept, so no agent is ever without its rules. The
   repository's specs stay, as record 0005 has it, and so do its
   instructions for building and landing work. Not taken: a copy kept
   for readers (option 2), a page beside Boom that its writers edit by
   hand and that drifts from it, a view kept as the truth (core.2,
   `caches-derived-only`); rules first (option 3), since a rule names
   the record it rests on (`cite-by-name`), so the two move together.
2. **A repository moves once its agents get every rule where they got
   it before.** boom is on the machine at a release its callers pin
   (`callers-pin-boom`); every agent session in the repository starts
   with the claims that stand for it (record 0008); an agent about to
   edit a file is given the claims bound to that part of the repository
   (`hook-before-each-edit`); and an agent finds a record by its words,
   as it found one by searching the pages. Until all four hold, a
   repository keeps its pages and nothing of it is imported.
3. **After the move, a record or a rule is only proposed.** A new
   record, rule or amendment is a proposal, an amendment naming the
   version it amends, standing once a person accepts it. No one writes
   it as a file.
4. **One signature for many versions.** `boom accept` naming several
   versions is one act of the person's, signed once with them present,
   judged whole before the key signs and appended whole or not at all.
   Not taken: a signature for each version (option 5), where two
   hundred touches teach a person to touch without reading, which is
   what presence was asked for to prevent.

## Consequences

- Good: an agent reads its repository's rules from one place, each as a
  person accepted it, with no page to edit by hand.
- Good: a person accepts an import, or a day's proposals, with one
  touch.
- Bad: a reader of the repository no longer finds its records in Git,
  as record 0005 has it, without `boom export`.
- Bad: until a person can accept away from the machine their key is
  on, a proposal waits for them at that machine.
- The requirement that several versions accepted together are each
  signed changes to one signature over all of them, in the change that
  builds point 4.
