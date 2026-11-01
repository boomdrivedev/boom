# Spec Delta

## Purpose

Each decision record and each rule of this repository is known by a
name fixed when it is written, every citation of one in the tree and in
the branch's commits names it, and nothing is known by a number a branch
allocates, so two branches landing side by side never renumber anything.

## ADDED Requirements

### Requirement: Each record is a file named by its name

`root:records` SHALL read each file under `docs/adr` as one record whose
name is the file's name without `.md`: two or more lowercase words of
letters and digits joined by hyphens. Each record's frontmatter SHALL
hold its `title`, `status` and `date`, and SHALL NOT hold a `number`.
Where a file's name is not such a name, one opening with a number among
them, or its frontmatter breaks this, `root:records` SHALL fail, naming
the file.

#### Scenario: The tree's records and rules hold

- **WHEN** `devenv shell -- moon run root:records` reads the files git tracks
- **THEN** it passes and prints how many records, rules and citations it read

#### Scenario: A record numbered after the last

- **WHEN** a tree holds `docs/adr/0002-amounts-as-floats.md`
- **THEN** the check fails, naming the file and saying a record takes a name, never a number (`tests/records/numbered/`, run by `root:test`)

### Requirement: An alias is legacy and held by one record

A record that had a number before rule identity-is-a-name MAY hold that
number as its `alias`, four digits. No two records SHALL hold one alias;
where two do, `root:records` SHALL fail, naming both files.

#### Scenario: Two records hold one alias

- **WHEN** two records of a tree each hold `alias: "0001"`
- **THEN** the check fails, naming `0001` and both files (`tests/records/shared/`, run by `root:test`)

### Requirement: Each rule has a name no other rule has

Each rule on a page under `docs/architecture` SHALL open
`- **[<name>]**`, with a name as a record's, held by no other rule on
any page, and SHALL have a details entry opening `- **<name>**`. A
legacy id SHALL be held by at most one rule. Where any of this breaks,
`root:records` SHALL fail, naming the name and each page and line.

#### Scenario: One rule name on two pages

- **WHEN** two rule pages of a tree each hold a rule named `views-are-computed`
- **THEN** the check fails, naming the rule and both pages (`tests/records/shared/`, run by `root:test`)

### Requirement: Every citation of a record or a rule names it

In every file the tree tracks, but those the script exempts by name
with their reasons, and outside code spans and fenced blocks, a citation
`record <name>` SHALL name a record and `rule <name>` SHALL name a rule.
A citation by number, `record` followed by four digits or `core.`
followed by digits, SHALL fail, naming the name of the record whose
alias or the rule whose legacy id it is, or saying that none holds it.
`root:records` SHALL read a citation that crosses a line break as one.

#### Scenario: A citation of nothing

- **WHEN** a file of a tree cites `record no-such-record` and `rule no-such-rule`
- **THEN** the check fails, naming each citation with its file and line (`tests/records/unresolved/`, run by `root:test`)

#### Scenario: A citation by a legacy number

- **WHEN** a file of a tree cites `record 0001` and `core.1` outside code spans, and the tree's records and rules hold them as an alias and a legacy id
- **THEN** the check fails, naming each citation and the name to cite in its place (`tests/records/legacy/`, run by `root:test`)

#### Scenario: A quoted number and a split citation

- **WHEN** a file of a tree quotes `core.1` and `record 0001` in code spans, and cites a record with `record` ending one line and its name opening the next
- **THEN** the check reads no citation in the code spans, resolves the split one, and passes (`tests/records/holds/`, run by `root:test`)

### Requirement: The branch's commits cite by name

`root:commits` SHALL read the messages of the commits the branch holds
and `main` lacks with the same reading of citations, against the tree's
records and rules, and SHALL fail on a citation of nothing or by
number, naming the commit. Where `main` does not resolve, it SHALL fail,
naming the ref.

#### Scenario: A commit cites a rule by its legacy id

- **WHEN** `check-records.sh --messages` reads two commit messages, one citing `core.1` outside a code span
- **THEN** it fails, naming that commit and the rule's name to cite (`tests/records/messages.txt`, run by `root:test`)

#### Scenario: The branch's commits hold

- **WHEN** `devenv shell -- moon run root:commits` runs on a branch whose commits cite rules and records by name
- **THEN** it passes, having read the messages of the commits `main` lacks rather than replaying an earlier outcome

### Requirement: The identity checks gate main

`root:records`, `root:commits` and `root:test` SHALL be moon tasks of
the root project tagged `check-fast`, `root:records` declaring every
tracked file an input and `root:commits` never cached, so the gate
sluice runs on `main` runs all three. No task SHALL hold records to
numbers.

#### Scenario: The fast layer runs the identity checks

- **WHEN** `devenv shell -- moon run :#check-fast` runs
- **THEN** it runs `root:records`, `root:commits` and `root:test`, and runs no task that holds a record to a number
