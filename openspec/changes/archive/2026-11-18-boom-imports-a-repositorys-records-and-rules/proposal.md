# Proposal

## Why

A repository that comes to boom already has decision records, MADR
files under `docs/adr/`, and rules, the bullets of
`docs/architecture/core.md`, each written by hand and kept by no tool.
boom keeps a store's claims in a log on the machine, and nothing yet
brings those records and rules into it. This change has boom read a
repository's records and rules once and propose each one, so the
person can accept them together; from then on boom keeps them, and no
one edits them by hand.

The user settled in the interview of 2026-11-01 how an import works:
it proposes and a person accepts (`import-proposes-a-person-accepts`,
which record 0005 keeps), each decision named from its file
(`import-names-from-files`) and given a scenario an agent drafts
(`import-drafts-scenarios`), its old number kept
(`legacy-numbers-kept-count-goes-on`). The brief adds the rules.

## What Changes

- `boom import adr <dir>` reads a folder of MADR files, and
  `boom import rules <page>` a rules page in the family's form, each
  rule a bullet `**[<id>]**` with its details beside it. Each record
  becomes a version of a claim: a decision record a decision, a rule an
  invariant, its statement and body the file's own words.
- What the files lack, a scenario for each record and a name for each
  rule, the agent that runs the import drafts into a drafts file.
  `--dry-run` gives what each draft still needs, and boom runs no model
  (record 0007, point 1).
- An import names its repository and is an agent's proposals, all or
  none, each saying where it came from: the repository, the file, its
  number or id, its status, date and decision-makers. It accepts
  nothing.
- A record its file marks superseded, deprecated or rejected comes in
  as history: a version a person accepts with the rest that never
  stands, a superseded one naming its successor (record 0007, point 3).
- Each name is held once, and each decision number once; a record
  imported before adds nothing when imported again, and one whose file
  changed since is refused, to be amended in boom (record 0007, point 5).
- `boom accept` takes several versions at once, one signed entry each,
  all in one transaction or none, and an import's answer gives the
  accept for the versions it proposed.

## Capabilities

### New Capabilities

- `record-import`: reading a folder of decision records and a rules
  page, the drafts an agent gives, proposing all or nothing, history,
  one name and one number for each record, reading each once, and the
  versions to accept together.

### Modified Capabilities

- `claim-versions`: a version that comes in as history may hold no
  scenario, its history is part of its fingerprint, and once accepted
  its status is `history`, never standing.
- `persons-acts`: several versions accepted together, one signed entry
  each, all or none.
- `command-line`: the commands gain `import adr` and `import rules`.

## Impact

- `crates/boom`: a module `import`, reading the files and the drafts;
  the entry's `proposed` act gains `imported`, and a claim's version
  `history`, both optional, so entries stay `boom-entry/1` and every
  version proposed before keeps its fingerprint; the judge gains the
  import's rules; the store appends several entries in one transaction.
  New dependency `yaml-rust2`, for MADR's frontmatter, locked in
  `Cargo.lock`.
- `docs/adr/0007-an-import-proposes-a-repositorys-records-and-rules-in-their-own-words-with-what-its-agent-drafts.md`
  records the choices the interview left to this change.
- `README.md` and `CLAUDE.md` describe the import.
- Out of this change: numbers given at acceptance, and numbering a
  second repository's decisions on from the count
  (`number-given-at-acceptance`); the views that write records and rules
  back into a repository; records in any format but MADR's; an area
  page's paths; and the site's pages.
