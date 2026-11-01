# Spec Delta

## Purpose

boom's statement of invariants, `design/requirements.md`, holds
together: every clause, global invariant and scenario is known by a
name no other item holds, nothing in it is numbered, and every citation
of one of its items names one that exists.

## ADDED Requirements

### Requirement: What the check reads as an item

`root:statement` SHALL read as an item each line opening `**[<name>]**`,
or `- **[<name>]**`: a clause in Parts A, B and C, a global invariant
under `## Global invariants`, and a scenario under
`## Scenarios the model must satisfy`. It SHALL read no line of another
section as an item.

#### Scenario: The statement's items are counted

- **WHEN** `devenv shell -- moon run root:statement` reads `design/requirements.md`
- **THEN** it passes and prints how many clauses, global invariants and scenarios it read

#### Scenario: A numbered list outside the item sections

- **WHEN** a statement's `## What to deliver` holds a list opening `1.`
- **THEN** the check reads no item there and passes (`tests/statement/holds.md`, run by `root:test`)

### Requirement: Each name is held by one item

Each clause, global invariant and scenario SHALL carry a name no other
item carries, whatever their kinds. Where two share one,
`root:statement` SHALL fail, naming the name and the line of each.

#### Scenario: Two clauses share a name

- **WHEN** a statement holds two clauses opening `**[same-name]**`
- **THEN** the check fails and names `same-name` with both lines (`tests/statement/shared-name.md`, run by `root:test`)

#### Scenario: A clause and an invariant share a name

- **WHEN** a statement holds a clause and an invariant both opening `**[other-name]**`
- **THEN** the check fails and names `other-name` with both lines (`tests/statement/shared-name.md`, run by `root:test`)

### Requirement: Nothing in the statement is numbered

No line of Parts A to C, the global invariants or the scenarios SHALL
open with a number and a full stop, and no citation SHALL name a clause
by a number. Where either holds, `root:statement` SHALL fail, naming
the line and saying an item takes a name.

#### Scenario: A numbered clause and a citation by number

- **WHEN** a statement's Part A holds a line opening `12.` and another clause cites `(12)`
- **THEN** the check fails, naming both lines (`tests/statement/numbered.md`, run by `root:test`)

### Requirement: Every citation names an item

A citation of one of the statement's items, `§<name>` outside a code
span, SHALL name an item the statement holds. Where one names none,
`root:statement` SHALL fail, naming the citation and its line. The check
SHALL read a citation that crosses a line break as one.

#### Scenario: A clause cites one that does not exist

- **WHEN** a clause cites `§no-such-clause` and the statement holds no item of that name
- **THEN** the check fails and names `§no-such-clause` and the citing line (`tests/statement/unresolved.md`, run by `root:test`)

#### Scenario: A citation across a line break

- **WHEN** a clause's citation `(§first-clause,` ends one line and `§second-clause)` opens the next, and both items exist
- **THEN** the check reads both citations and passes (`tests/statement/holds.md`, run by `root:test`)

#### Scenario: boom's statement resolves

- **WHEN** `devenv shell -- moon run root:statement` reads `design/requirements.md`
- **THEN** every `§` citation it holds names an item, and the check passes

### Requirement: The statement's check gates main

`root:statement` SHALL be a moon task of the root project tagged
`check-fast`, declaring the statement and its script as inputs, and
`root:test` SHALL run its cases, so the gate sluice runs on `main` runs
both.

#### Scenario: The fast layer runs the statement's check

- **WHEN** `devenv shell -- moon run :#check-fast` runs on a tree whose statement changed
- **THEN** it runs `root:statement` against the changed statement rather than replaying an earlier outcome, and runs `root:test` among the fast layer's tasks
