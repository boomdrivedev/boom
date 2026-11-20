---
number: 7
title: An import proposes a repository's records and rules in their own words, with what its agent drafts
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-imports-a-repositorys-records-and-rules's construct stage
---

# An import proposes a repository's records and rules in their own words, with what its agent drafts

Status: accepted. The construct stage made this decision. It settles
what the interview of 2026-11-01 did not decide about an import, on the
requirements that the user gave there. Record 0005 keeps those
requirements. The decision is cheap to change until a store holds an
import. The entries of an import name the format `boom-entry/1`. The
fields that this decision adds are optional.

Deciders: the construct stage of the change
`boom-imports-a-repositorys-records-and-rules`. Its brief asks that
boom read "a repository's existing decision records and rules once and
proposes each one, so the person can accept them together; from then
on Boom keeps them, and no one edits them by hand."

Sources:

- `interview.md`, the answers of the user of 2026-11-01:
  `import-proposes-a-person-accepts`, `import-drafts-scenarios`,
  `import-names-from-files`, `legacy-numbers-kept-count-goes-on`,
  `one-name-space`, `cite-by-name` and `import-once`.
- Record 0005, which keeps them.
- Record 0006.
- core.2, core.3 and core.5.
- boomdrivedev/blueprints principle 16.
- The design of the change.

## Context and problem statement

The user decided these points:

- An import proposes, and a person accepts. One command accepts the
  batch, with one signed event for each version.
- Boom names each decision from its file. Each decision gets a
  scenario that an agent drafted.
- A superseded file comes in as history and never stands.
- Boom keeps the old numbers.

The interview did not discuss rules. It did not decide these points:

- How the drafts of an agent get to boom, when boom holds no model.
- How boom names a rule, which has an id and not a name.
- What history is in the log.
- How a person accepts a batch.
- What prevents a person from editing a record by hand after boom holds
  it.

## Considered options

1. The agent that runs the import drafts into a file that boom reads,
   and boom proposes the words of the files (chosen).
2. boom runs a drafting command that its settings name, as the check
   will run its reviewer.
3. The agent writes each proposal whole, with the statement and the
   body.

## Decision outcome

1. **The agent drafts. boom runs no model.** The agent that runs
   `boom import` reads the files and writes a drafts file: a scenario
   for each record and a name for each rule. boom takes only the
   scenarios and the names from it. The statement and the body of a
   version are always those of the file, so a person accepts what the
   record said. A dry run gives what each draft still needs. Not taken:
   - A drafting command that boom runs (option 2). It needs settings
     that the owner signs and that name the program. These settings do
     not exist. Also it starts a model when an agent is already
     present.
   - The words of the agent for the record (option 3). With them, a
     person can accept a record without the words of the file in front
     of the person.
2. **A rule is an invariant, and its draft names it.** Each bullet
   `**[<id>]**` of a rules page in the form of the family is an
   invariant. Its details are its body. Boom keeps its id as a legacy
   alias. Not taken: a name made from the id, `core-1`. That name is a
   number used as a name, and `cite-by-name` does not allow it.
3. **History is a version that a person accepts and that never
   stands.** A record whose file marks it superseded, deprecated or
   rejected comes in as a version with its history. It needs no
   scenario. After a person accepts it with the other versions, its
   status is `history`. It never stands and it replaces nothing. A
   superseded record names its successor. Not taken:
   - Records of this type left out of the import. This loses their
     numbers and their place.
   - A version that stands, and that a later version then replaces.
4. **A person accepts a batch by naming each version.** `boom accept`
   takes several `<name>@<version>` targets. It judges all of them
   before the key signs one of them. It appends one signed entry for
   each, in one transaction. The answer of an import gives that command
   for the versions that it proposed. Not taken:
   - An accept that names an import by an id. An agent can add to that
     set after the person read it.
   - One entry that the person signs one time for the batch. The user
     asked for one signed event for each version.
5. **Boom reads a record one time.** The source of a record is its
   repository, as the import names it, its file, and its number or id.
   When an import reads the record again with no change, the import
   adds nothing. When the file changed after its import, boom refuses
   it, and the refusal says to amend the version that boom holds. boom
   never takes the file again as the record. boom writes nothing back
   into the repository (core.2).

## Consequences

- Good: boom stays a ledger with no model in it. A person accepts
  words from an import. Each of these words comes from the file, or it
  has a mark as the draft of an agent.
- Good: the history of a repository is in the log with the decisions
  that stand. This history includes the superseded records and their
  numbers.
- Bad: when a person accepts a batch, the key of the person needs a
  touch for each version.
- Bad: nothing prevents a hand edit to an imported file. Boom only
  never takes the edit. The files stay until a view writes them, and no
  check reads them (principle 16).
- Neutral: the kept numbers wait for the change that gives numbers at
  acceptance. Until that change, boom refuses a second repository whose
  numbers are the same as the numbers of the first.

## More information

The code stage of the change decided these points, on the decisions 2
to 7 of the design. They are its own decisions and are cheap to change:

- **An imported proposal meets the rules of the import first.** The
  judge applies these rules to a proposal with `imported` before the
  rules of any proposal:
  - Boom reads its record one time.
  - No claim holds its name.
  - No other claim holds its number.

  Thus when the record of another repository holds the same words under
  the same name, boom refuses the record on the name. The agent then
  gives it another name in its draft. Not taken: the repeat rule of any
  proposal first. That rule counts such a record as the record of the
  other repository, and it drops its source.
- **A dry run reads the files and no store.** It gives what each record
  still needs, and an agent runs it before a store exists. Not taken: a
  dry run judged against the store. That needs a store that somebody
  made.
- **A draft holds only `scenarios` and `name`.** When a draft gives any
  other field, a statement or a body, boom refuses it and names the
  field. Thus an agent learns that boom takes the words of the file.
  Not taken: boom ignores such fields. That tells the agent nothing.
- **The body of a rule is its details as written.** The body is the
  bullet of the details and the lines under it, which include
  `- **core.1** ...`. Not taken: only the words after the id. That
  drops the bullet that the file wrote.

## Amendment of 2026-11-20 (proposed)

Status: proposed, for the user to accept. The change
`claims-hold-what-their-kind-needs` makes it. The user approved the
claim model on 2026-11-20.

This amendment changes what an agent drafts and what an import
proposes.

1. An agent drafts names only. A drafts file has the format
   `boom-import-drafts/2`. A draft gives a record a name. A draft gives
   no scenarios.
2. An imported version has no scenarios.
3. An imported decision needs a body. The body is the text of its file
   after the title. An import refuses a decision file that has a title
   and no other text.
4. A record that comes in as history keeps the status of its file. That
   status is `superseded`, `deprecated` or `rejected`. Boom does not use
   the status `history`.

An entry from an earlier import keeps its scenarios. Boom shows them.
