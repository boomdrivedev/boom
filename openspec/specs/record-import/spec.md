# record-import Specification

## Purpose
How boom takes in a repository's decision records and rules once: each
read from its file, proposed in the file's own words with the names and
scenarios its agent drafts, held to one name and one number, and
accepted together by a person, after which boom keeps it and never
takes its file again as the record (interview:
`import-proposes-a-person-accepts`, `import-drafts-scenarios`,
`import-names-from-files`, `legacy-numbers-kept-count-goes-on`,
`import-once`; record 0007).

## Requirements

### Requirement: Decision records read from a folder

`boom import adr <dir>` SHALL read each `.md` file in `<dir>` whose name begins with digits and a hyphen, or whose frontmatter has a `name`, as a decision: its statement its first `# ` heading, its body the text after that heading, and its name the frontmatter's `name` or else the file's name without `.md` and its number. It SHALL keep the file's number, status, date and decision-makers, and SHALL name every other file as skipped.

#### Scenario: A folder of MADR files read

- **WHEN** `boom import adr <dir> --repository shop --dry-run --json` runs on a folder holding `0001-record-architecture-decisions.md`, `0003-amounts-in-cents.md` and `README.md`
- **THEN** it exits 0 giving two decisions, `record-architecture-decisions` with the number `0001` and `amounts-in-cents` with `0003`, each its file's title as its statement and its status, date and decision-makers as the file gives them, `README.md` as skipped, and the log is as it was

#### Scenario: A record named in its frontmatter

- **WHEN** the folder holds `the-rename.md`, whose frontmatter has `name: the-rename` and no `number`
- **THEN** it is read as the decision `the-rename`, with no number

#### Scenario: A record with no title

- **WHEN** a record's file holds no `# ` heading
- **THEN** it exits 2 naming the file, and nothing is appended

#### Scenario: Two numbers that differ

- **WHEN** `0003-amounts-in-cents.md` has `number: 4` in its frontmatter
- **THEN** it exits 2 naming the file and both numbers, and nothing is appended

### Requirement: Rules read from a page

`boom import rules <page>` SHALL read each bullet of the page's `## Rules` section that opens `**[<id>]**` as an invariant: its statement the rule's words on one line, its legacy id `<id>`, its section the `###` heading above it, and its body the bullet opening `**<id>**` under the page's `## Details`, or on `<stem>-details.md` beside the page. A page with no rule, or with one id given twice, SHALL be refused naming it.

#### Scenario: A rules page read

- **WHEN** `boom import rules <dir>/core.md --repository shop --dry-run --json` runs on a page whose `## Rules` holds `core.1` and `core.2` under `### The ledger`, and whose `## Details` holds the details of `core.1`
- **THEN** it exits 0 giving two invariants with the legacy ids `core.1` and `core.2`, each its rule's words as its statement and `The ledger` as its section, `core.1` with its details as its body, and the log is as it was

#### Scenario: Details on the page beside it

- **WHEN** the details of `core.1` are on `core-details.md` beside `core.md`, and `core.md` has no `## Details`
- **THEN** `core.1` is read with those details as its body

#### Scenario: A page with no rules

- **WHEN** `boom import rules <page>` runs on a page with no `## Rules` section
- **THEN** it exits 2 naming the page, and nothing is appended

#### Scenario: One id twice

- **WHEN** a page's `## Rules` holds two rules opening `**[core.1]**`
- **THEN** it exits 2 naming `core.1`, and nothing is appended

### Requirement: An agent drafts what the files lack

An import SHALL take each record's scenarios, and each rule's name, from `--drafts <file>`, a `boom-import-drafts/1` object keyed by each record's source, an ADR's file name or a rule's id; a draft MAY rename a decision. boom SHALL propose the files' own statements and bodies and mark the scenarios drafted. A record lacking what it needs, or a draft naming no record, SHALL be refused naming it; `--dry-run` SHALL write nothing and give what each draft still needs.

#### Scenario: What a draft still needs

- **WHEN** `boom import rules <page> --repository shop --dry-run --json` runs with no `--drafts`
- **THEN** it exits 0 giving each rule as needing a name and a scenario, and the log is as it was

#### Scenario: Imported with drafts

- **WHEN** `boom import adr <dir> --repository shop --drafts <file> --provenance run-1` runs with a scenario drafted for each record, and then `boom show amounts-in-cents@<version> --json`
- **THEN** the import exits 0, and the version gives its file's title as its statement, the text after the title as its body, the drafted scenario, and `imported` naming the repository `shop`, the file, its number, status, date and decision-makers, its scenarios drafted

#### Scenario: A rule named by its draft

- **WHEN** `boom import rules <page> --repository shop --drafts <file> --provenance run-1` runs with `core.1` drafted as `status-from-events`
- **THEN** the invariant `status-from-events` is proposed with the legacy id `core.1` and its rule's words as its statement

#### Scenario: A draft missing

- **WHEN** an import runs with drafts that give no scenario for `0003-amounts-in-cents.md`
- **THEN** it exits 2 naming `0003-amounts-in-cents.md` as needing a scenario, and nothing is appended

#### Scenario: A draft for no record

- **WHEN** an import runs with drafts naming `0009-gone.md`, which is no file of the folder
- **THEN** it exits 2 naming `0009-gone.md`, and nothing is appended

### Requirement: An import proposes, all or nothing

An import SHALL name its repository with `--repository <name>`, and SHALL be an agent's: each record one `proposed` entry with no signature, naming `--provenance`, carrying `imported`, where the record came from, the repository and the file among it, and `attach` naming that repository, so a record accepted stands for that repository alone. It SHALL accept nothing, and where any record is refused it SHALL append none.

#### Scenario: An import proposes and accepts nothing

- **WHEN** an import of two records runs with `--provenance run-1`
- **THEN** the log gains two `proposed` entries naming the agent's provenance `run-1` and no `accepted` entry, and each version is proposed

#### Scenario: One record refused, none appended

- **WHEN** an import of three records runs with drafts naming one of them `Bad_Name`
- **THEN** it exits 2 naming `Bad_Name`, and the log is as it was

#### Scenario: No provenance

- **WHEN** an import runs with drafts and no `--provenance`
- **THEN** it exits 2 saying an agent's proposal names its provenance, and nothing is appended

#### Scenario: No repository

- **WHEN** `boom import adr <dir>` runs with no `--repository`
- **THEN** it exits 2 saying an import names its repository, and nothing is appended

#### Scenario: An imported record stands for its repository alone

- **WHEN** `0003-amounts-in-cents.md` is imported with `--repository shop` and accepted, and then `boom show amounts-in-cents --json`, `boom query --repository shop --json` and `boom query --repository billing --json` run
- **THEN** the version's `repositories` is `["shop"]`, the query for `shop` gives `amounts-in-cents`, and the query for `billing` does not

### Requirement: Superseded, deprecated and rejected records come in as history

A record whose status is `superseded`, `deprecated` or `rejected` SHALL be proposed as history and needs no scenario. A superseded one SHALL name as its successor the record of its folder that its status links to or numbers, and SHALL be refused where there is none. A record with no status SHALL be read as accepted, and one whose status is none of these, `accepted` and `proposed` SHALL be refused naming the file.

#### Scenario: A superseded record

- **WHEN** `0002-amounts-as-floats.md` has `status: superseded by [ADR-0003](0003-amounts-in-cents.md)` and is imported with no scenario drafted for it
- **THEN** `amounts-as-floats` is proposed as history, superseded by `amounts-in-cents`, with no scenario

#### Scenario: A successor by its number

- **WHEN** `0002-amounts-as-floats.md` has `status: superseded by 0003`
- **THEN** it is read as history, superseded by `amounts-in-cents`

#### Scenario: A successor not in the folder

- **WHEN** a record's status is `superseded by 0009` and no record of the folder has the number `0009`
- **THEN** it exits 2 naming the file and its status, and nothing is appended

#### Scenario: A status boom does not know

- **WHEN** a record's status is `draft`
- **THEN** it exits 2 naming the file, its status and the statuses boom reads, and nothing is appended

#### Scenario: A record with no status

- **WHEN** `0004-one-database.md` has no frontmatter
- **THEN** it is read as a decision whose status is none, to stand once accepted

### Requirement: One name and one number for each record

An imported record SHALL take a name no claim of the store holds, and a decision a number no other claim of the store came in with. A name or a number held, or one name given to two records of one import, SHALL be refused naming what holds it, so the record's draft names it otherwise. A record imported before is read as reading once has it.

#### Scenario: A name the store holds

- **WHEN** a folder holding `0002-money-has-currency.md` is imported into a store where the invariant `money-has-currency` was proposed
- **THEN** it exits 2 naming the invariant `money-has-currency`, and nothing is appended

#### Scenario: Named otherwise by its draft

- **WHEN** the same folder is imported with the draft of `0002-money-has-currency.md` naming it `money-carries-currency`
- **THEN** the decision `money-carries-currency` is proposed with the number `0002`

#### Scenario: A number the store holds

- **WHEN** a folder holding `0001-use-postgres.md` is imported for the repository `billing` into a store where `0001-record-architecture-decisions.md` was imported for `shop`
- **THEN** it exits 2 naming the number `0001` and `record-architecture-decisions`, and nothing is appended

#### Scenario: Two records under one name

- **WHEN** an import's drafts give two files one name
- **THEN** it exits 2 naming both files, and nothing is appended

### Requirement: Each record read once

A record's source SHALL be its repository, its file and its number or id. An import run again SHALL add nothing for a record whose version it proposed before from that source, answering as the first did. A record whose file changed after its import SHALL be refused naming the file and the version boom holds, saying to amend that version with `boom propose --amends`: once imported, boom keeps the record and never takes its file again as the record.

#### Scenario: The same import twice

- **WHEN** an import runs twice with the same folder and the same drafts
- **THEN** the second exits 0 answering as the first did, and nothing is appended

#### Scenario: A file edited after its import

- **WHEN** the title of `0003-amounts-in-cents.md` is changed after it was imported, and the import runs again with the same drafts
- **THEN** it exits 2 naming the file and the version imported, saying to amend it with `boom propose --amends`, and nothing is appended

#### Scenario: Another repository's page of the same name

- **WHEN** `core.md` of the repository `billing` is imported after `core.md` of `shop`, its `core.1` drafted under a name of its own
- **THEN** it exits 0, and `billing`'s `core.1` is proposed as a record of its own

### Requirement: Accepted together

An import's answer SHALL give the versions to accept together, every version it proposed but those whose file's status was `proposed`, and the command that accepts them, `boom accept <name>@<version>... --key <key>`. Accepting them SHALL leave each one standing, or history where it came in as history.

#### Scenario: An import accepted together

- **WHEN** ada runs, with ada's key, the accept an import gives for `0001-record-architecture-decisions.md`, `0002-amounts-as-floats.md` superseded by `0003-amounts-in-cents.md`, and `0003-amounts-in-cents.md`
- **THEN** `record-architecture-decisions` and `amounts-in-cents` stand, `amounts-as-floats` is history, and the log gains three `accepted` entries, each signed by ada

#### Scenario: A record left proposed waits

- **WHEN** an import's folder holds `0004-one-database.md` with `status: proposed`, and its accept is run
- **THEN** that version is not among the versions to accept together, and it stays proposed
