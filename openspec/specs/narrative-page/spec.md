# narrative-page Specification

## Purpose
The page where a person reads a narrative with the claims that it
includes, and writes a note on a paragraph. boom serves the page on
the person's machine, and computes it from the log for each request.
No file and no commit holds it
(`views-computed-from-the-ledger`, `no-credential-in-the-repository`,
`nothing-commercial`).

## Requirements

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

### Requirement: Only the browser of the person reads the page

Boom SHALL make a token of 128 random bits for each run of `boom page`. Each path of the page SHALL start with the token. Boom SHALL refuse a request with a `Host` other than `127.0.0.1:<port>`.

#### Scenario: No token

- **WHEN** a request asks for the page with no token, or with another token
- **THEN** boom answers 404, and gives no part of the store

#### Scenario: Another host name

- **WHEN** a request with the token gives the header `Host: example.com`
- **THEN** boom answers 403

### Requirement: boom page stops

Boom SHALL serve the page until it is stopped, or until `--idle <minutes>` pass with no request, 60 by default. Boom SHALL then exit 0.

#### Scenario: Idle

- **WHEN** `boom page boom-end-to-end --no-open --idle 1` runs, and no request comes for one minute
- **THEN** boom exits 0

### Requirement: The page is computed from the log for each request

Boom SHALL compute each response from the views at the last entry of the log. Boom SHALL write no file of the page.

#### Scenario: A note from the terminal

- **WHEN** the page is served, `boom note` then writes a note on paragraph 2, and the browser asks for the page again
- **THEN** the page gives the note at paragraph 2

#### Scenario: No file written

- **WHEN** `boom page` serves 10 requests
- **THEN** the folder of the store and the working folder hold the same files as before

### Requirement: What the page gives

The page of a version SHALL give these:

- The title, `<name>@<version>`, the status, the proposer, the base and what the version binds.
- Each version of the narrative with its status, as a link.
- Each paragraph with its number and the anchor `#p<n>`.
- The command line `boom accept <name>@<version> --key <key>` while the version is proposed.

#### Scenario: The accept of a proposed version

- **WHEN** the page of a proposed version is asked for
- **THEN** the page gives `boom accept <name>@<version> --key <key>`, and no form that accepts

### Requirement: The notes on the page

The page SHALL give each note of the version after its paragraph, with its name, person, time, words, and `open` or its answer. It SHALL give the notes of the earlier versions after the body, each with its paragraph's words and its answer.

#### Scenario: A note at its paragraph

- **WHEN** chuck wrote a note on paragraph 3, and the page of the version is asked for
- **THEN** the page gives the note after paragraph 3, with `chuck`, its words and `open`

#### Scenario: A note answered by the next version

- **WHEN** a note on `<v1>` is answered by `<v2>`, and the page of `<v2>` is asked for
- **THEN** the page gives the note after the body, with the words of its paragraph in `<v1>` and the answer of `<v2>`

### Requirement: The included claims on the page

The page SHALL give each included claim after the first paragraph whose words name it, with its kind, `<name>@<version>`, status, statement and body. It SHALL give the other included claims after the body. It SHALL mark an included version out of date when its claim has another accepted version.

#### Scenario: A claim at its paragraph

- **WHEN** a narrative includes `status-read-from-events`, and its paragraph 3 holds the words `status-read-from-events`
- **THEN** the page gives the claim with its statement after paragraph 3, and not after the body

#### Scenario: A claim that no paragraph names

- **WHEN** a narrative includes a claim whose name no paragraph holds
- **THEN** the page gives the claim after the body

### Requirement: A note written on the page

Each paragraph SHALL have a form that sends a note on it. A `POST` of the form SHALL append the entry that `boom note` appends for the person of the page. Boom SHALL then send the browser to the page at `#p<n>`. The person of the page SHALL be `--person <name>`, or the one person that the store registers. Where boom knows no person of the page, the page SHALL give no form, and SHALL say how to start it with `--person`. Boom SHALL refuse a `POST` whose `Origin` is not the page's own.

#### Scenario: A note sent

- **WHEN** the store registers chuck only, and the form of paragraph 3 sends the words "Say what a drive is."
- **THEN** the log gets one `noted` entry of chuck for paragraph 3, and boom answers 303 to the page at `#p3`

#### Scenario: A form from another site

- **WHEN** a `POST` with the token gives the header `Origin: https://example.com`
- **THEN** boom answers 403, and appends nothing

#### Scenario: Two persons and no --person

- **WHEN** the store registers ada and bo, and `boom page <name>` runs with no `--person`
- **THEN** the page gives no form, and names `boom page <name> --person <name>`

### Requirement: The page runs nothing from a body or another host

The page SHALL give each raw HTML of a body as text. Each response SHALL have the header `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; form-action 'self'`. The page SHALL hold its own style, SHALL load nothing from another host, and SHALL hold no script.

#### Scenario: A script in a body

- **WHEN** a narrative's body holds `<script>alert(1)</script>`
- **THEN** the page gives the characters `<script>alert(1)</script>` as text, and the HTML of the page holds no `<script>` element

#### Scenario: Nothing from another host

- **WHEN** the page of a narrative is asked for
- **THEN** its HTML holds no URL of another host, and the response has the policy header

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
