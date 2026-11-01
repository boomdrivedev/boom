# Spec Delta

## Purpose

boom's statement of invariants, `design/requirements.md`, holds together:
every clause, global invariant and scenario is reached by one number,
and every citation in it names a clause or a decision record that exists.

## ADDED Requirements

### Requirement: What the check reads as a numbered item

`root:statement` SHALL read as a clause each line of Parts A, B and C
that opens with a number, an optional lower-case letter and a full stop
(`12.`, `93a.`); as a global invariant each list item opening `**I<n>.**`;
and as a scenario each list item opening `**S<n>.**`. It SHALL read no
other line as numbered.

#### Scenario: The statement's items are counted

- **WHEN** `devenv shell -- moon run root:statement` reads `design/requirements.md`
- **THEN** it passes and prints how many clauses, global invariants and scenarios it read

#### Scenario: A numbered list outside the parts

- **WHEN** a statement's section 12, What to deliver, holds a list opening `1.` and Part A holds a clause numbered 1
- **THEN** the check reads one clause numbered 1 and passes (`tests/statement/holds.md`, run by `root:test`)

### Requirement: Each number is held by one item

Each clause, global invariant and scenario SHALL carry a number no other
item of its kind carries. Where two share one, `root:statement` SHALL
fail, naming the number and the line of each.

#### Scenario: Two clauses numbered 4

- **WHEN** a statement holds two clauses opening `4.`
- **THEN** the check fails and names 4 with both lines (`tests/statement/shared-number.md`, run by `root:test`)

#### Scenario: Two invariants numbered I2

- **WHEN** a statement holds two items opening `**I2.**`
- **THEN** the check fails and names I2 with both lines (`tests/statement/shared-number.md`, run by `root:test`)

### Requirement: Every citation names what exists

A citation in the statement, a parenthesised list of clause numbers and
records such as `(4)`, `(3, 5, 54)` or `(12, record 0004)`, SHALL name
only clauses the statement holds and records `docs/adr` holds. Where one
names neither, `root:statement` SHALL fail, naming the citation, the
missing number and its line.

#### Scenario: A clause cites one that does not exist

- **WHEN** a clause cites `(9)` and the statement holds no clause 9
- **THEN** the check fails and names 9 and the citing line (`tests/statement/unresolved.md`, run by `root:test`)

#### Scenario: A clause cites a record that does not exist

- **WHEN** a clause cites `(record 0099)` and `docs/adr` holds no record numbered 0099
- **THEN** the check fails and names record 0099 and the citing line (`tests/statement/unresolved.md`, run by `root:test`)

#### Scenario: A citation across a line break

- **WHEN** a clause's citation `(3,` ends one line and `5)` opens the next, and clauses 3 and 5 exist
- **THEN** the check reads the citation as naming 3 and 5 and passes (`tests/statement/holds.md`, run by `root:test`)

#### Scenario: boom's statement resolves

- **WHEN** `devenv shell -- moon run root:statement` reads `design/requirements.md` against `docs/adr`
- **THEN** every citation it holds names a clause or a record, and the check passes

### Requirement: The check gates main

`root:statement` and `root:test` SHALL be moon tasks of the root project
tagged `check-fast`, each declaring the files it reads as inputs, the
statement, `docs/adr`, the fixtures and its script among them, so the
gate sluice runs on `main` runs both.

#### Scenario: The fast layer runs both

- **WHEN** `devenv shell -- moon run :#check-fast` runs on a tree whose statement changed
- **THEN** it runs `root:statement` against the changed statement rather than replaying an earlier outcome, and runs `root:test` among the fast layer's tasks
