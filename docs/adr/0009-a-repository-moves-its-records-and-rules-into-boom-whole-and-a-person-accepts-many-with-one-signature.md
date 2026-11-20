---
number: 9
title: A repository moves its records and rules into Boom whole, and a person accepts many versions with one signature
status: accepted
date: 2026-11-18
decision-makers:
- boomdrive-design
---

# A repository moves its records and rules into Boom whole, and a person accepts many versions with one signature

Status: accepted. This record is the decision of boomdrive-design. It
rests on records 0005, 0006, 0007 and 0008, and on the requirements of
the interview that those records keep. It is cheap to change until a
repository moves.

Deciders: boomdrive-design.

Sources:

- Records 0005 to 0008.
- `interview.md`, the answers of the user of 2026-11-01:
  `persons-acts-signed-with-presence`,
  `import-proposes-a-person-accepts`, `import-once`, `cite-by-name`,
  `caches-derived-only`, `hook-before-each-edit` and `callers-pin-boom`.
- core.2.

## Context and problem statement

Record 0007 imports the decision records and the rules page of a
repository. Record 0008 gives the agents of the repository the claims
that stand for it, from `boom query`. No record says these things yet:

- What the repository keeps after it moves.
- When it can move.
- How a person accepts what one import proposes.

A repository with a hundred records and a hundred rules proposes two
hundred versions. `boom accept` signs each version as an entry of its
own. Thus, with a security key, the person touches the key two hundred
times (record 0006).

## Considered options

For what a moved repository keeps:

1. Its records and rules move whole. Its pages leave it in one change,
   after a person accepts the import (chosen).
2. Its pages stay beside Boom as a copy for readers.
3. Its rules move first, and its records move later.

For a person who accepts many versions at one time:

4. One signature over each version of one accept, with the person
   present (chosen).
5. One signature for each version, as built.

## Decision outcome

1. **A repository moves whole, one time.** Boom imports each decision
   record and each rules page at one time, and a person accepts the
   import. Then one change does these things:
   - It removes the records and the rules pages from the repository.
   - It declares the repository in `.config/boom.toml`.
   - It removes the rules page from what the instructions of its agents
     load.

   That change lands only after the accept. Thus no agent is ever
   without its rules. The specs of the repository stay, as record 0005
   says. Its instructions for building and landing work also stay.

   Not taken: a copy kept for readers (option 2). That copy is a page
   beside Boom that its writers edit by hand, and it drifts from Boom.
   It is a view kept as the truth (core.2, `caches-derived-only`). Also
   not taken: rules first (option 3). A rule names the record that it
   rests on (`cite-by-name`), so the two move together.
2. **A repository moves when its agents get each rule where they got it
   before.** These four conditions must hold:
   - boom is on the machine at a release that its callers pin
     (`callers-pin-boom`).
   - Each agent session in the repository starts with the claims that
     stand for it (record 0008).
   - An agent that is about to edit a file gets the claims bound to that
     part of the repository (`hook-before-each-edit`).
   - An agent finds a record by its words, as it found one when it
     searched the pages.

   Until all four hold, a repository keeps its pages, and Boom imports
   nothing of it.
3. **After the move, a record or a rule is only proposed.** A new
   record, rule or amendment is a proposal. An amendment names the
   version that it amends. The proposal stands after a person accepts
   it. No one writes it as a file.
4. **One signature for many versions.** `boom accept` that names several
   versions is one act of the person. The person signs it one time,
   while present. Boom judges the act whole before the key signs. Boom
   appends it whole or not at all.

   Not taken: a signature for each version (option 5). Two hundred
   touches teach a person to touch the key without reading. Presence was
   asked for to prevent that.

## Consequences

- Good: an agent reads the rules of its repository from one place. Each
  rule is as a person accepted it. No page needs edits by hand.
- Good: a person accepts an import, or the proposals of a day, with one
  touch.
- Bad: a reader of the repository no longer finds its records in Git, as
  record 0005 says, without `boom export`.
- Bad: until a person can accept away from the machine that holds their
  key, a proposal waits for them at that machine.
- The requirement that each version accepted together has its own
  signature changes to one signature over all of them. That change
  occurs in the change that builds point 4.
