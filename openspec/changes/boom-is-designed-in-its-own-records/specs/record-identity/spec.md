# Spec Delta

## Purpose

Each decision record and each rule of this repository is known by a
name no other holds, fixed when it is written; the numbered ones are
only the legacy the check names; every citation of a record or a rule in
the tree resolves; and nothing holds a number unique, so two branches
landing side by side never renumber anything.

## ADDED Requirements

### Requirement: Each new record is a file named by its name

`root:adrs` SHALL read each file under `docs/adr` as one record. A file
that is not a legacy record SHALL be `<name>.md`, its name two or more
lowercase words of letters and digits joined by hyphens, and its
frontmatter SHALL hold `name`, equal to the file's name, `title`,
`status` and `date`, and SHALL NOT hold `number`. Where a file's name is
not such a name, one opening with a number among them, or its
frontmatter breaks this, `root:adrs` SHALL fail, naming the file.

#### Scenario: The tree's records and rules hold

- **WHEN** `devenv shell -- moon run root:adrs` reads the files git tracks
- **THEN** it passes and prints how many records, rules and citations it read

#### Scenario: A record numbered on a branch

- **WHEN** a tree holds `docs/adr/0002-amounts-as-floats.md`
- **THEN** the check fails, naming the file and saying a record takes a name, never a number (`tests/adrs/numbered/`, run by `root:test`)

#### Scenario: A frontmatter holding a number, or another name

- **WHEN** a tree holds a named record whose frontmatter holds `number: 2`, and another whose `name` is not its file's
- **THEN** the check fails, naming each file and what its frontmatter breaks (`tests/adrs/frontmatter/`, run by `root:test`)

### Requirement: The legacy records are the ones the check names

A numbered file under `docs/adr` SHALL be accepted only where the
script names it as a legacy record, with its reason, as it names
`0001-boom-is-an-open-source-repository-of-its-own-and-its-binary-is-boom.md`.
A legacy record's file and frontmatter MAY stay as they were written.

#### Scenario: Record 0001 is read as legacy

- **WHEN** `root:adrs` reads a tree holding record 0001 under its numbered file, with `number: 1` in its frontmatter
- **THEN** it reads it as a legacy record, cited as `record 0001`, and passes (`tests/adrs/holds/`, run by `root:test`)

### Requirement: No two records hold one name

No two records SHALL hold one name. Where two do, `root:adrs` SHALL fail,
naming the name and both files.

#### Scenario: Two records hold one name

- **WHEN** two records of a tree each hold `name: money-has-currency` in their frontmatter
- **THEN** the check fails, naming `money-has-currency` and both files (`tests/adrs/shared/`, run by `root:test`)

### Requirement: Each rule opens with a named id or a legacy one

Each rule on a page under `docs/architecture` SHALL open `- **[<id>]**`,
its id the page's area, a full stop and a name as a record's, or a
legacy id the script names with its reason (core.1, core.2, core.3,
core.5 and core.6), and SHALL have a details entry opening `- **<id>**`.
Where a rule breaks this, `root:adrs` SHALL fail, naming the page and
line.

#### Scenario: A rule numbered on a branch

- **WHEN** a tree's `core.md` holds a rule opening `- **[core.7]**`
- **THEN** the check fails, naming the line and saying a rule takes a name (`tests/adrs/numbered/`, run by `root:test`)

### Requirement: No two rules hold one id, and each legacy id is held once

No two rules SHALL hold one id. Each legacy id the script names SHALL be
held either as a rule's id or by exactly one details entry's
`Legacy id:`. Where either breaks, `root:adrs` SHALL fail, naming the id
and each page and line.

#### Scenario: One rule id on two lines

- **WHEN** a tree's `core.md` holds two rules opening `- **[core.views-are-computed]**`
- **THEN** the check fails, naming the id and both lines (`tests/adrs/shared/`, run by `root:test`)

#### Scenario: A legacy id held as an alias

- **WHEN** a tree's `core.md` holds `core.no-commercial-strategy` with a details entry ending `Legacy id: core.4.`, and the legacy rules core.1, core.2, core.3, core.5 and core.6
- **THEN** the check reads core.4 as that rule's alias and passes (`tests/adrs/holds/`, run by `root:test`)

### Requirement: Every citation of a record or a rule resolves

A citation `record <name>` SHALL name a record, `record` followed by four
digits a legacy record, `core.` followed by a name a rule, and `core.`
followed by digits a legacy id or alias. Where one names nothing,
`root:adrs` SHALL fail, naming the citation with its file and line and,
for a number, saying the record or rule takes a name. No check SHALL
hold a number unique.

#### Scenario: A citation of nothing

- **WHEN** a file of a tree cites `record no-such-record`, `core.no-such-rule`, `record 0002` and `core.7` outside code spans
- **THEN** the check fails, naming each citation with its file and line (`tests/adrs/unresolved/`, run by `root:test`)

### Requirement: Where the check reads citations

`root:adrs` SHALL read citations in every file the tree tracks but those
the script exempts by name, each with its reason, and SHALL read none
inside a code span or a fenced block. It SHALL read a citation that
crosses a line break as one.

#### Scenario: Legacy citations, a quoted number and a split citation

- **WHEN** a file of a tree cites `record 0001`, `core.1` and `core.4`, quotes `core.7` and `record 0002` in code spans, and cites a record with `record` ending one line and its name opening the next
- **THEN** the check resolves the legacy citations and the split one, reads no citation in the code spans, and passes (`tests/adrs/holds/`, run by `root:test`)

### Requirement: The record checks gate main

`root:adrs` and `root:test` SHALL be moon tasks of the root project
tagged `check-fast`, `root:adrs` declaring every tracked file an input,
so the gate sluice runs on `main`'s candidate runs both.

#### Scenario: The fast layer runs the record checks

- **WHEN** `devenv shell -- moon run :#check-fast` runs
- **THEN** it runs `root:adrs` and `root:test`, and `root:adrs` runs again when any tracked file changes rather than replaying an earlier outcome
