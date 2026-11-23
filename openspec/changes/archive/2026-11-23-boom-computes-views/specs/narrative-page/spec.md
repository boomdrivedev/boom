## MODIFIED Requirements

### Requirement: boom page serves the page on the machine

`boom page [<name>[@<version>]] [--view <outline>]` SHALL serve the page on `127.0.0.1`, on `--port <port>` or on a free port. With no name and no `--view`, the page SHALL be the design book. Boom SHALL print one line, `Boom: page of <target> at <URL>`, and SHALL then open the URL in the browser, unless `--no-open`. Boom SHALL refuse a name that is not a narrative, a `--view` that is not an outline, and a name with `--view`.

#### Scenario: The page served

- **WHEN** `boom page boom-end-to-end --no-open --port <port>` runs
- **THEN** boom prints one line with a URL on `127.0.0.1:<port>`, and a `GET` of that URL gives the page of the newest version, with status 200

#### Scenario: Not a narrative

- **WHEN** `boom page status-read-from-events` runs
- **THEN** boom exits 2 and says that `boom page` gives a narrative, and serves nothing

#### Scenario: The design book served

- **WHEN** `boom page --no-open --port <port>` runs
- **THEN** boom prints `Boom: page of the design book at <URL>`, and a `GET` of that URL gives the page of the design book, with status 200

#### Scenario: A view served

- **WHEN** `boom page --view boomdrive-design-book --no-open` runs
- **THEN** boom prints `Boom: page of boomdrive-design-book@<version> at <URL>`, and a `GET` of that URL gives the page of the design book

#### Scenario: A view that is not an outline

- **WHEN** `boom page --view boom-end-to-end` runs, and `boom-end-to-end` is a narrative
- **THEN** boom exits 2 and says that `--view` names an outline, and serves nothing

## REMOVED Requirements

### Requirement: The list of narratives

**Reason**: The design book takes its place. The book gives each narrative under its level, in the outline's order, with its status and its count of open notes.

**Migration**: `boom page` with no name gives the design book. A store with no outline gives the book with each narrative marked as not in the outline.

## ADDED Requirements

### Requirement: The design book on the page

The page of the design book SHALL give the outline's title, body, `<name>@<version>` and status. It SHALL give each part, chapter and section of the book as a heading, in the book's order. Under each heading, it SHALL give each narrative with its title, `<name>@<version>`, status and count of open notes, as a link to its page. It SHALL mark each narrative that the outline does not hold. With no outline of the view `book`, it SHALL say how to propose one.

#### Scenario: The book on the page

- **WHEN** the outline holds `the-drives-log`, which binds the whole system, and `boom-end-to-end`, which binds `boom` and has two open notes, and the page with no name is asked for
- **THEN** the page gives `Part I. The system` with `the-drives-log`, then `Part II. The repositories` with the chapter `boom` and `boom-end-to-end`, the count 2, and a link to the page of `boom-end-to-end`

#### Scenario: A narrative marked on the page

- **WHEN** a narrative binds `boom`, the outline does not hold it, and the page with no name is asked for
- **THEN** the page gives the narrative last in the chapter `boom`, marked as not in the outline

#### Scenario: No outline

- **WHEN** the store holds two narratives that bind `boom` and no outline, and the page with no name is asked for
- **THEN** the page gives both narratives in the chapter `boom` by name, each marked as not in the outline, and names `boom propose outline`

### Requirement: A narrative's place in the book

The page of a narrative that the design book gives SHALL link to the book, to the narrative before it in the book, and to the narrative after it. The book's order SHALL run through Part I, then each chapter of Part II.

#### Scenario: The narratives around one

- **WHEN** the book gives `the-drives-log`, `boom-end-to-end` and `change-records` in that order, and the page of `boom-end-to-end` is asked for
- **THEN** the page links to the book, to `the-drives-log` as the one before, and to `change-records` as the one after

#### Scenario: The first narrative

- **WHEN** the page of `the-drives-log` is asked for
- **THEN** the page links to the book and to `boom-end-to-end` as the one after, and gives no link to a narrative before
