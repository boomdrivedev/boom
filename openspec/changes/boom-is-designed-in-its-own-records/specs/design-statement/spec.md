# Spec Delta

## Purpose

boom's statement of invariants, `design/requirements.md`, holds
together: every clause, global invariant, question and scenario is known
by a name no other item holds, nothing in it is numbered, and every
citation of one of its items names one that exists.

## ADDED Requirements

### Requirement: What the check reads as an item

`root:statement` SHALL read as an item each paragraph opening
`**[<name>]**` in Parts A, B and C, and each list item opening
`- **[<name>]**` under `## Global invariants`,
`## Questions the model must answer` and
`## Scenarios the model must satisfy`, its name two or more lowercase
words of letters and digits joined by hyphens. It SHALL read no line of
another section as an item.

#### Scenario: The statement's items are counted

- **WHEN** `devenv shell -- moon run root:statement` reads `design/requirements.md`
- **THEN** it passes and prints how many clauses, global invariants, questions and scenarios it read

#### Scenario: A numbered list outside the item sections

- **WHEN** a statement's `## What to deliver` holds a list opening `1.`
- **THEN** the check reads no item there and passes (`tests/statement/holds.md`, run by `root:test`)

### Requirement: Each name is held by one item

Each clause, global invariant, question and scenario SHALL carry a name
no other item carries, whatever their kinds. Where two share one,
`root:statement` SHALL fail, naming the name and the line of each.

#### Scenario: Two clauses share a name

- **WHEN** a statement holds two clauses opening `**[same-name]**`
- **THEN** the check fails and names `same-name` with both lines (`tests/statement/shared-name.md`, run by `root:test`)

#### Scenario: A clause and a scenario share a name

- **WHEN** a statement holds a clause opening `**[other-name]**` and a scenario opening `- **[other-name]**`
- **THEN** the check fails and names `other-name` with both lines (`tests/statement/shared-name.md`, run by `root:test`)

### Requirement: Nothing in the statement is numbered

No line of Parts A to C, the global invariants, the questions or the
scenarios SHALL open with a number and a full stop, and no parenthesis
whose every part is a citation or a bare clause number, one at least a
number, SHALL cite an item by number. Where either holds,
`root:statement` SHALL fail, naming the line and saying an item takes a
name.

#### Scenario: A numbered clause and a citation by number

- **WHEN** a statement's Part A holds a line opening `12.` and another clause cites `(12)`
- **THEN** the check fails, naming both lines (`tests/statement/numbered.md`, run by `root:test`)

### Requirement: Every citation names an item

A citation list, a parenthesis outside a code span every comma-separated
part of which is a name, `record` and a name or four digits, or `core.`
and an id, SHALL name with each of its names an item the statement
holds. Where one names none, `root:statement` SHALL fail, naming the
citation and its line. The check SHALL read a citation list that crosses
a line break as one, and SHALL read no parenthesis of prose as a
citation list.

#### Scenario: A clause cites one that does not exist

- **WHEN** a clause cites `(no-such-clause)` and the statement holds no item of that name
- **THEN** the check fails and names `no-such-clause` and the citing line (`tests/statement/unresolved.md`, run by `root:test`)

#### Scenario: A citation across a line break, and a parenthesis of prose

- **WHEN** a clause's citation list `(first-clause,` ends one line and `second-clause)` opens the next, both items exist, and another clause holds a parenthesis of words, one of them hyphened
- **THEN** the check reads both citations, reads the parenthesis of words as prose, and passes (`tests/statement/holds.md`, run by `root:test`)

#### Scenario: boom's statement resolves

- **WHEN** `devenv shell -- moon run root:statement` reads `design/requirements.md`
- **THEN** every name in its citation lists names an item, and the check passes

### Requirement: The statement's check gates main

`root:statement` SHALL be a moon task of the root project tagged
`check-fast`, declaring the statement and its script as inputs, and
`root:test` SHALL run its cases, so the gate sluice runs on `main`'s
candidate runs both.

#### Scenario: The fast layer runs the statement's check

- **WHEN** `devenv shell -- moon run :#check-fast` runs on a tree whose statement changed
- **THEN** it runs `root:statement` against the changed statement rather than replaying an earlier outcome, and runs `root:test` among the fast layer's tasks
