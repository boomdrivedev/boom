# Spec Delta

## Purpose

The site shows boom's design, the statement of invariants and each
decision record, as pages computed from their files, each item and
record reached by its name, the numbers it shows computed from the order
things landed on main, so a page that describes the design links the
clauses and records it rests on and the build proves each link reaches
one.

## ADDED Requirements

### Requirement: The statement is a page of the site

The site SHALL show `design/requirements.md` as the page
`/boom/design/statement/`, written from the file before every type check
and build. Each clause, global invariant, question and scenario SHALL
carry an anchor named for its name and SHALL show its name and the
number the site computes for it, or that main lacks it. Each name in a
citation list SHALL link that item's anchor, and each `record <name>`
that record's page.

#### Scenario: A page links a clause by its name

- **WHEN** `devenv shell -- moon run site:build` builds a page that links `/boom/design/statement/#identity-fixed-at-proposal`, and the statement holds that clause
- **THEN** the build passes, its link checker finding the anchor `identity-fixed-at-proposal` on the statement's page

#### Scenario: The roadmap links the questions

- **WHEN** `devenv shell -- moon run site:build` builds the roadmap, which links `/boom/design/statement/#open-signature-format` and the anchors of the statement's other questions
- **THEN** the build passes, each anchor standing on the statement's page

### Requirement: Each decision record is a page of the site

The site SHALL show each record under `docs/adr` as the page
`/boom/design/decisions/<slug>/`, the slug the record's name, or a
legacy record's number, titled from the record's frontmatter, with its
status and date, its number or that it is numbered when first accepted
on main, how to cite it, its
edit link the record's own file, and a link in one record to another
reaching the other's page.

#### Scenario: The records are listed

- **WHEN** `devenv shell -- moon run site:build` runs
- **THEN** the sidebar's Design group lists each record of `docs/adr` by its number and title, record 0001 among them at `/boom/design/decisions/0001/`, and every link to `/boom/design/decisions/<slug>/` reaches a page

### Requirement: A number shown is computed from main's history

Every number a design page shows SHALL be computed when the pages are
written, from main's first-parent history, and written into no file the
tree tracks. A record or item main lacks SHALL show no number, and a
landing SHALL change no number an earlier landing gave.

#### Scenario: Two branches land one after the other

- **WHEN** `ordinals` reads landings in which one branch adds a record and lands, and a second branch, cut before it, adds another and lands after
- **THEN** the first record keeps the number it had before the second landed, and the second takes the next (`site/scripts/design.test.ts`, run by `site:test`)

#### Scenario: A build on a branch

- **WHEN** `devenv shell -- moon run site:build` runs on a branch that adds a record main lacks
- **THEN** the build passes, the record's page saying it is numbered when first accepted on main and showing no number

### Requirement: Records are numbered in the order they were first accepted

A legacy record SHALL keep the number its file's name gives, and every
other record SHALL take the next number after the highest the records
hold, in the order it was first accepted on main: by landing, within a
landing by the first of its own commits that holds it accepted, and
within a commit by name. A record never accepted on main SHALL have no
number.

#### Scenario: The count goes on from the legacy record

- **WHEN** `ordinals` reads a first landing holding the legacy `0001-…` file and a later one adding a named record, accepted
- **THEN** the legacy record keeps 1 and the named record takes 2 (`site/scripts/design.test.ts`, run by `site:test`)

#### Scenario: One landing brings several records

- **WHEN** `ordinals` reads a landing whose own commits add three records, two of them in one commit
- **THEN** the records are numbered in the order of the commits that first hold them, the two of one commit by name (`site/scripts/design.test.ts`, run by `site:test`)

#### Scenario: A record proposed, then accepted later

- **WHEN** `ordinals` reads a record that landed proposed, and a later landing in which it is accepted after another record was accepted
- **THEN** it has no number until the landing that accepts it, and then takes the number after the other's (`site/scripts/design.test.ts`, run by `site:test`)

### Requirement: The statement's items are numbered in the order they landed

The statement's clauses, invariants, questions and scenarios SHALL each
be numbered from 1, apart from the other kinds, in the order their names
first appear in the statement on main: by landing, and within a landing
in reading order.

#### Scenario: Items of four kinds over two landings

- **WHEN** `ordinals` reads a statement whose first landing holds two clauses, an invariant, a question and a scenario, and whose second adds a clause between the two
- **THEN** the clauses read 1 and 2 in the first landing's reading order and the added one 3, and the invariant, the question and the scenario each 1 (`site/scripts/design.test.ts`, run by `site:test`)

### Requirement: A name is never held again, and the history is whole

Where a name main's history held and later lost is held again, by a
later landing or the working tree, the step that writes the design pages
SHALL fail, naming it and the landing that withdrew it. Where the
history is shallow or `main` does not resolve, it SHALL fail, naming
why, rather than show a number it cannot compute.

#### Scenario: A withdrawn name held again

- **WHEN** `ordinals` reads a name held in one landing, gone in the next, and held again in a later one
- **THEN** it fails, naming the name and the landing that withdrew it (`site/scripts/design.test.ts`, run by `site:test`)

### Requirement: The design pages are computed, never committed

The pages of the statement and the records SHALL be written by the
site's `design-pages` task from the files they show and main's history,
into a directory git ignores, and SHALL be left out of the site's prose
check, which reads the pages people write.

#### Scenario: A record changes

- **WHEN** a record under `docs/adr` changes and `devenv shell -- moon run site:build` runs
- **THEN** the build runs again rather than replaying, since the records and the statement are its inputs, and its record page shows the change

#### Scenario: The prose check reads written pages only

- **WHEN** `devenv shell -- moon run site:prose` runs after `site:build` has written the design pages
- **THEN** Vale reads the pages under `site/src/content/docs` that people write, its glob leaving out the `design/` directory, and passes

### Requirement: A page cites the design by links the build checks

A page of the site that cites a clause, an invariant, a question, a
scenario or a record SHALL cite it by a link to its anchor or its page,
named by its name. `site:build` SHALL fail on any such link that reaches
neither.

#### Scenario: Every citation reaches what it cites

- **WHEN** `devenv shell -- moon run site:build` runs over the pages
- **THEN** every link to `/boom/design/statement/#…` reaches an anchor of the statement's page and every link to `/boom/design/decisions/…` a record's page, or the build fails naming the link

### Requirement: The integrations page draws the millpond loop

The integrations page SHALL show a diagram of the loop between millpond
and boom: a claim judged not satisfied, its backlog entry taken in as an
item, the item's work launched through flume naming the claim and landed
through sluice, the cell made due, and the claim judged satisfied; and a
signal reaching boom as a challenge. It SHALL be drawn in a wide and a
narrow layout and checked as every diagram is.

#### Scenario: The loop is drawn

- **WHEN** `devenv shell -- moon run site:test` draws every diagram
- **THEN** the `millpond` diagram draws in both layouts with a title and a description, its markup closes, its ids stay within its prefix, and no label runs out of its box

#### Scenario: The page shows it

- **WHEN** `devenv shell -- moon run site:build` builds the integrations page
- **THEN** the build passes with the page including the `millpond` diagram, which the diagram component refuses to build under a name no diagram has
