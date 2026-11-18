---
number: 7
title: An import proposes a repository's records and rules in their own words, with what its agent drafts
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-imports-a-repositorys-records-and-rules's construct stage
---

# An import proposes a repository's records and rules in their own words, with what its agent drafts

Status: accepted. The construct stage's own, settling what the
interview of 2026-11-01 left open about an import on the requirements
the user gave there, which record 0005 keeps. Cheap to change until a
store holds an import: an import's entries name the format
`boom-entry/1`, and the fields this adds are optional.

Deciders: the construct stage of the change
`boom-imports-a-repositorys-records-and-rules`, whose brief asks that
boom read "a repository's existing decision records and rules once and
proposes each one, so the person can accept them together; from then
on Boom keeps them, and no one edits them by hand."

Sources: `interview.md`, the user's answers of 2026-11-01:
`import-proposes-a-person-accepts`, `import-drafts-scenarios`,
`import-names-from-files`, `legacy-numbers-kept-count-goes-on`,
`one-name-space`, `cite-by-name` and `import-once`; record 0005, which
keeps them; record 0006; core.2, core.3 and core.5;
boomdrivedev/blueprints principle 16; the change's design.

## Context and problem statement

The user settled that an import proposes and a person accepts, one
command accepting the batch with one signed event each; that each
decision is named from its file and given a scenario an agent drafted;
that a superseded file comes in as history and never stands; and that
old numbers are kept. The interview did not take up rules, and left
open how an agent's drafts reach boom when boom holds no model, how a
rule, which has an id and not a name, is named, what history is in the
log, how a person accepts a batch, and what keeps a record from being
edited by hand once boom holds it.

## Considered options

1. The agent that runs the import drafts into a file boom reads, and
   boom proposes the files' own words (chosen).
2. boom runs a drafting command it is configured with, as the check
   will run its reviewer.
3. The agent writes each proposal whole, statement and body among it.

## Decision outcome

1. **The agent drafts; boom runs no model.** The agent that runs
   `boom import` reads the files and writes a drafts file, a scenario
   for each record and a name for each rule. boom takes from it the
   scenarios and names alone: a version's statement and body are always
   the file's, so what a person accepts is what the record said. A dry
   run gives what each draft still needs. Not taken: a drafting command
   boom runs (option 2), which needs owner-signed settings naming the
   program, not built, and starts a model for an agent already present;
   the agent's own words for the record (option 3), which a person
   could accept without the file's words before them.
2. **A rule is an invariant, named by its draft.** Each bullet
   `**[<id>]**` of a rules page in the family's form is an invariant,
   its details its body, its id kept as a legacy alias. Not taken: a
   name made from the id, `core-1`, a number standing as a name, which
   `cite-by-name` rules out.
3. **History is a version a person accepts that never stands.** A
   record its file marks superseded, deprecated or rejected comes in as
   a version with its history, needing no scenario, and once a person
   accepts it with the rest its status is `history`: never standing,
   replacing nothing, a superseded one naming its successor. Not taken:
   leaving such records out, which loses their numbers and their place;
   a version that stands and is then replaced.
4. **A batch is accepted by naming each version.** `boom accept` takes
   several `<name>@<version>` targets, judges them all before the key
   signs any, and appends one signed entry each in one transaction; an
   import's answer gives that command for what it proposed. Not taken:
   an accept naming an import by an id, a set an agent could add to
   after the person read it; one entry signed once for the batch, where
   the user asked one signed event each.
5. **A record is read once.** A record's source is its repository, as
   the import names it, its file and its number or id. Imported again
   unchanged, it adds nothing; its file changed after its import, it is
   refused, the refusal saying to amend the version boom holds. boom
   never takes the file again as the record, and writes nothing back
   into the repository (core.2).

## Consequences

- Good: boom stays a ledger with no model in it, and every word a
  person accepts from an import is either the file's or marked as an
  agent's draft.
- Good: a repository's history, superseded records and their numbers
  among it, is in the log with the decisions that stand.
- Bad: accepting a batch asks a touch of the person's key for each
  version.
- Bad: a hand edit to an imported file is not stopped, only never taken
  in: the files stay until a view writes them, and no check reads them
  (principle 16).
- Neutral: the numbers kept wait on the change that gives numbers at
  acceptance; until then a second repository whose numbers clash is
  refused.
