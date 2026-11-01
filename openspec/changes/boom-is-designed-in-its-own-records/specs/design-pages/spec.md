# Spec Delta

## Purpose

The site shows boom's design, the statement of invariants and each
decision record, as pages computed from their files, so a page that
describes the design links the clauses and records it rests on and the
build proves each link reaches one.

## ADDED Requirements

### Requirement: The statement is a page of the site

The site SHALL show `design/requirements.md` as the page
`/boom/design/statement/`, written from the file before every type check
and build. Each clause, global invariant and scenario SHALL show its own
number, as the statement gives it, and carry an anchor named for that
number: `#12`, `#93a`, `#I1`, `#S3`.

#### Scenario: A page links a clause

- **WHEN** `devenv shell -- moon run site:build` builds a page that links `/boom/design/statement/#12`, and the statement holds clause 12
- **THEN** the build passes, its link checker finding the anchor `12` on the statement's page

#### Scenario: An invariant and a scenario are linked

- **WHEN** `devenv shell -- moon run site:build` builds the roadmap, which links `/boom/design/statement/#I1` and a scenario's anchor such as `#S1`
- **THEN** the build passes, both anchors standing on the statement's page

### Requirement: Each decision record is a page of the site

The site SHALL show each record under `docs/adr` as the page
`/boom/design/decisions/<number>/`, titled from the record's frontmatter,
with its status and date, its edit link the record's own file, and a
link in one record to another reaching the other's page.

#### Scenario: The records are listed

- **WHEN** `devenv shell -- moon run site:build` runs
- **THEN** the sidebar's Design group lists each record of `docs/adr`, 0001 onward, by its number and title, and every link to `/boom/design/decisions/<number>/` reaches a page

### Requirement: The design pages are computed, never committed

The pages of the statement and the records SHALL be written by the
site's `design-pages` task from the files they show, into a directory
git ignores, and SHALL be left out of the site's prose check, which
reads the pages people write.

#### Scenario: A record changes

- **WHEN** a record under `docs/adr` changes and `devenv shell -- moon run site:build` runs
- **THEN** the build runs again rather than replaying, since the records and the statement are its inputs, and its record page shows the change

#### Scenario: The prose check reads written pages only

- **WHEN** `devenv shell -- moon run site:prose` runs after `site:build` has written the design pages
- **THEN** Vale reads the pages under `site/src/content/docs` that people write, its glob leaving out the `design/` directory, and passes

### Requirement: A page cites the design by links the build checks

A page of the site that cites a clause, an invariant, a scenario or a
record SHALL cite it by a link to its anchor or its page. `site:build`
SHALL fail on any such link that reaches neither.

#### Scenario: Every citation reaches what it cites

- **WHEN** `devenv shell -- moon run site:build` runs over the pages
- **THEN** every link to `/boom/design/statement/#…` reaches an anchor of the statement's page and every link to `/boom/design/decisions/…` a record's page, or the build fails naming the link

### Requirement: The integrations page draws the millpond loop

The integrations page SHALL show a diagram of the loop between millpond
and boom: a claim judged not satisfied, its backlog entry taken in as
an item, the item's work started through flume and landed through
sluice, and the claim judged satisfied; and a signal reaching boom as a
challenge. It SHALL be drawn in a wide and a narrow layout and checked
as every diagram is.

#### Scenario: The loop is drawn

- **WHEN** `devenv shell -- moon run site:test` draws every diagram
- **THEN** the `millpond` diagram draws in both layouts with a title and a description, its markup closes, its ids stay within its prefix, and no label runs out of its box

#### Scenario: The page shows it

- **WHEN** `devenv shell -- moon run site:build` builds the integrations page
- **THEN** the build passes with the page including the `millpond` diagram, which the diagram component refuses to build under a name no diagram has
