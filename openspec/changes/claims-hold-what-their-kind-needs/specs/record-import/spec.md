## MODIFIED Requirements

### Requirement: Decision records read from a folder

`boom import adr <dir>` SHALL read as a decision each `.md` file in `<dir>` whose name starts with digits and a hyphen, or whose frontmatter has a `name`. The statement SHALL be the first `# ` heading, and the body the text after it. The name SHALL be the `name` of the frontmatter, or the file name without `.md` and its number. Boom SHALL keep the number, status, date and decision-makers of the file. Boom SHALL name each other file as skipped.

#### Scenario: A folder of MADR files read

- **WHEN** `boom import adr <dir> --repository shop --dry-run --json` runs on a folder that holds `0001-record-architecture-decisions.md`, `0003-amounts-in-cents.md` and `README.md`
- **THEN** boom exits 0 and gives two decisions: `record-architecture-decisions` with the number `0001` and `amounts-in-cents` with `0003`. Each has the title of its file as its statement, and the status, date and decision-makers of its file. Boom gives `README.md` as skipped, and the log does not change

#### Scenario: A record named in its frontmatter

- **WHEN** the folder holds `the-rename.md`, whose frontmatter has `name: the-rename` and no `number`
- **THEN** boom reads it as the decision `the-rename`, with no number

#### Scenario: A record with no title

- **WHEN** the file of a record holds no `# ` heading
- **THEN** boom exits 2 and names the file, and boom appends nothing

#### Scenario: Two numbers that differ

- **WHEN** `0003-amounts-in-cents.md` has `number: 4` in its frontmatter
- **THEN** boom exits 2 and names the file and both numbers, and boom appends nothing

#### Scenario: A decision with no body

- **WHEN** `0001-use-postgres.md` holds a title and no other text, and its status is accepted
- **THEN** the import exits 2 and says that a decision needs a body, and boom appends nothing

### Requirement: An agent drafts what the files lack

An import SHALL take the name of each rule from `--drafts <file>`, a `boom-import-drafts/2` object whose keys are the sources of the records: the file name of an ADR, or the id of a rule. A draft MAY give a decision another name. A draft SHALL give no scenarios. Boom SHALL propose the statements and bodies of the files. Boom SHALL refuse, and name, a record with no name and a draft for no record. With `--dry-run`, boom SHALL write nothing and give what each record needs.

#### Scenario: What a draft still needs

- **WHEN** `boom import rules <page> --repository shop --dry-run --json` runs with no `--drafts`
- **THEN** boom exits 0 and gives each rule as needing a name, and the log does not change

#### Scenario: Imported with drafts

- **WHEN** `boom import adr <dir> --repository shop --drafts <file> --provenance run-1` runs, and then `boom show amounts-in-cents@<version> --json` runs
- **THEN** the import exits 0. The version has the title of its file as its statement, the text after the title as its body, and no scenarios. Its `imported` names the repository `shop`, the file, and the number, status, date and decision-makers of the file

#### Scenario: A rule named by its draft

- **WHEN** `boom import rules <page> --repository shop --drafts <file> --provenance run-1` runs with the name `status-from-events` drafted for `core.1`
- **THEN** boom proposes the invariant `status-from-events`, with the legacy id `core.1` and the words of its rule as its statement

#### Scenario: A draft missing

- **WHEN** an import of a rules page runs with drafts that give no name for `core.2`
- **THEN** boom exits 2 and says that `core.2` needs a name, and boom appends nothing

#### Scenario: A draft for no record

- **WHEN** an import runs with drafts that name `0009-gone.md`, and the folder has no such file
- **THEN** boom exits 2 and names `0009-gone.md`, and boom appends nothing

#### Scenario: Drafts with scenarios

- **WHEN** an import runs with a draft that gives scenarios
- **THEN** boom exits 2 and names the drafts file and the field `scenarios`, and boom appends nothing

### Requirement: An import proposes, all or nothing

An import SHALL name its repository with `--repository <name>`, and SHALL be the act of an agent. Each record SHALL be one `proposed` entry with no signature. The entry SHALL name `--provenance`, and SHALL hold `imported`, which names the repository and the file, and `attach`, which names that repository. An import SHALL accept nothing. If boom refuses a record, the import SHALL append nothing. The JSON answer SHALL be `boom-import/2`.

#### Scenario: An import proposes and accepts nothing

- **WHEN** an import of two records runs with `--provenance run-1`
- **THEN** the log gets two `proposed` entries that name the provenance `run-1`, and no `accepted` entry, and each version is proposed

#### Scenario: One record refused, none appended

- **WHEN** an import of three records runs with drafts that give one of them the name `Bad_Name`
- **THEN** boom exits 2 and names `Bad_Name`, and the log does not change

#### Scenario: No provenance

- **WHEN** an import runs with drafts and no `--provenance`
- **THEN** boom exits 2 and says that the proposal of an agent names its provenance, and boom appends nothing

#### Scenario: No repository

- **WHEN** `boom import adr <dir>` runs with no `--repository`
- **THEN** boom exits 2 and says that an import names its repository, and boom appends nothing

#### Scenario: An imported record stands for its repository alone

- **WHEN** an import with `--repository shop` proposes `0003-amounts-in-cents.md`, a person accepts it, and then `boom show amounts-in-cents --json`, `boom query --repository shop --json` and `boom query --repository billing --json` run
- **THEN** the `repositories` of the version is `["shop"]`, the query for `shop` gives `amounts-in-cents`, and the query for `billing` does not give it

### Requirement: Superseded, deprecated and rejected records come in as history

Boom SHALL propose a record as history, with no scenarios, when its status is `superseded`, `deprecated` or `rejected`. A superseded record SHALL name its successor: the record of its folder that its status links to or numbers. Boom SHALL refuse a superseded record with no successor in the folder. Boom SHALL read a record with no status as accepted. Boom SHALL refuse a record with any other status than these, `accepted` and `proposed`, and name its file.

#### Scenario: A superseded record

- **WHEN** `0002-amounts-as-floats.md` has `status: superseded by [ADR-0003](0003-amounts-in-cents.md)`, and an import reads it
- **THEN** boom proposes `amounts-as-floats` as history, superseded by `amounts-in-cents`, with no scenarios

#### Scenario: A successor by its number

- **WHEN** `0002-amounts-as-floats.md` has `status: superseded by 0003`
- **THEN** boom reads it as history, superseded by `amounts-in-cents`

#### Scenario: A successor not in the folder

- **WHEN** the status of a record is `superseded by 0009`, and no record of the folder has the number `0009`
- **THEN** boom exits 2 and names the file and its status, and boom appends nothing

#### Scenario: A status boom does not know

- **WHEN** the status of a record is `draft`
- **THEN** boom exits 2 and names the file, its status and the statuses that boom reads, and boom appends nothing

#### Scenario: A record with no status

- **WHEN** `0004-one-database.md` has no frontmatter
- **THEN** boom reads it as a decision with no status, which is accepted when a person accepts it

### Requirement: Accepted together

The answer of an import SHALL give the versions to accept together. These SHALL be all the versions that it proposed, except those whose file has the status `proposed`. The answer SHALL give the command that accepts them: `boom accept <name>@<version>... --key <key>`. When a person accepts them, each version SHALL be `accepted`. A version that came in as history SHALL get the status of its file.

#### Scenario: An import accepted together

- **WHEN** ada runs, with the key of ada, the accept that an import gives for `0001-record-architecture-decisions.md`, for `0002-amounts-as-floats.md` superseded by `0003-amounts-in-cents.md`, and for `0003-amounts-in-cents.md`
- **THEN** `record-architecture-decisions` and `amounts-in-cents` are `accepted`, `amounts-as-floats` is `superseded`, and the log gets three `accepted` entries that ada signed

#### Scenario: A record left proposed waits

- **WHEN** the folder of an import holds `0004-one-database.md` with `status: proposed`, and the accept of the import runs
- **THEN** that version is not in the versions to accept together, and it stays proposed
