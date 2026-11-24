## ADDED Requirements

### Requirement: The book's contents in a sidebar

The book page, each narrative's page, each claim's page and the page of a search SHALL give the design book's contents in `<nav class="side">`: Part I with its narratives and a section for each of its contexts, then Part II with a chapter for each repository and a section for each of its contexts. Each narrative SHALL be a link to its page with its title, in the book's order.

#### Scenario: The sidebar of a narrative's page

- **WHEN** the book gives `the-drives-log` in Part I, then `boom-end-to-end` in the chapter `boom`, and the page of `boom-end-to-end` is asked for
- **THEN** the sidebar gives `Part I. The system` with a link to `the-drives-log`, then `Part II. The repositories` with the chapter `boom` and a link to `boom-end-to-end`, which has `aria-current="page"`

#### Scenario: A section of a context

- **WHEN** `the-merge-queue` binds the context `sluice-host-process`, whose globs name `sluice`, and a narrative's page is asked for
- **THEN** the sidebar gives the chapter `sluice` with the section `sluice-host-process`, and a link to `the-merge-queue` under it

### Requirement: The sidebar marks the page's narrative

The sidebar SHALL mark the narrative of the page, at any of its versions, with `aria-current="page"`. It SHALL mark each narrative that the outline does not hold, as the book page marks it.

#### Scenario: The sidebar of the book page

- **WHEN** the page with no name is asked for
- **THEN** the page gives the sidebar, and no link of the sidebar has `aria-current`

#### Scenario: A narrative that the outline does not hold

- **WHEN** a narrative binds `boom`, and the outline does not hold it
- **THEN** the sidebar gives it last in the chapter `boom`, marked as not in the outline

#### Scenario: The sidebar of a version's page

- **WHEN** the page of `boom-end-to-end@<v1>` is asked for, and the book gives `boom-end-to-end@<v2>`
- **THEN** the sidebar marks `boom-end-to-end` with `aria-current="page"`

### Requirement: The sidebar folds on a narrow window

Each page that gives the sidebar SHALL give a control with the word "Contents" before it. Under 60rem wide, the page SHALL hide the sidebar until the control opens it, with no script. At 60rem or wider, the page SHALL show the sidebar beside the page, and SHALL hide the control.

#### Scenario: A narrow window

- **WHEN** a browser 400 pixels wide shows a narrative's page, and then the reader clicks "Contents"
- **THEN** the sidebar is hidden and "Contents" is shown, and after the click the sidebar is shown

#### Scenario: A wide window

- **WHEN** a browser 1280 pixels wide shows a narrative's page
- **THEN** the sidebar is shown at the left of the narrative's text, and "Contents" is hidden

### Requirement: A search from the page

The sidebar SHALL hold a form that sends `GET <base>/search?q=<words>`. The page of the search SHALL give the narratives of the book whose version in the book holds each word in its paragraphs, then each other claim that `boom search --all <words>` gives, in its order. The search SHALL match words as `boom search` does.

#### Scenario: A claim found

- **WHEN** the invariant `status-read-from-events` stands for `boom`, and the search `status events` is asked for
- **THEN** the page gives `invariant status-read-from-events@<version>`, its status and its statement, with a link to its page

#### Scenario: The forms of a word

- **WHEN** the search `STATUSES` is asked for
- **THEN** the page gives `status-read-from-events`, as `boom search STATUSES --all` gives it

#### Scenario: No word to search

- **WHEN** the search `***` is asked for
- **THEN** the page says that a search needs a word that holds a letter or a digit, with status 200, and gives no result

#### Scenario: A search with no token

- **WHEN** a request asks for `/search?q=bolt` with no token
- **THEN** boom answers 404, and gives no part of the store

### Requirement: A narrative found links to its paragraph

A narrative found SHALL link to its page at `#p<n>`, the first paragraph that holds each word, with that paragraph's words, or to its page where no one paragraph holds them all. A claim found SHALL link to its page, with its kind, `<name>@<version>`, status and statement.

#### Scenario: A word in a narrative's paragraph

- **WHEN** paragraph 4 of `boom-end-to-end` is the first that holds the word `replay`, and the search `replay` is asked for
- **THEN** the page gives `boom-end-to-end` with a link to its page at `#p4`, and the words of paragraph 4

#### Scenario: A narrative that is proposed

- **WHEN** `change-records` has one version, proposed, the book gives it, and its paragraph 2 holds the word `bolt`
- **THEN** the search `bolt` gives `change-records` with a link at `#p2`

#### Scenario: Words in two paragraphs

- **WHEN** a narrative's paragraph 2 holds `bolt` and its paragraph 5 holds `landing`, and no one paragraph holds both
- **THEN** the search `bolt landing` gives the narrative with a link to its page and no `#p`

### Requirement: The page of a claim

`GET <base>/c/<name>` SHALL give the page of the claim at its accepted version, or its newest version where none is accepted, and `<base>/c/<name>@<version>` that version. The page SHALL give the kind, `<name>@<version>`, status, statement, what it binds and body. The name of a narrative SHALL answer 303 to its page, the name of an outline 303 to `<base>/v/<name>`, and a name that no claim has 404.

#### Scenario: The name of a narrative

- **WHEN** `<base>/c/boom-end-to-end` is asked for, and `boom-end-to-end` is a narrative
- **THEN** boom answers 303 to `<base>/n/boom-end-to-end`

#### Scenario: A claim accepted while the page is served

- **WHEN** the page of a proposed claim is served, a person then accepts it, and the browser asks for the page again
- **THEN** the page gives the claim as accepted

### Requirement: The links of a claim's page

The page of a claim SHALL give each version with its status as a link, each source and each claim that it changes as a link to its page, marked out of date where another version of it is accepted, and each narrative of the book that includes the claim as a link. The box of an included claim on a narrative's page SHALL link to the claim's page at the version meant.

#### Scenario: A claim's page

- **WHEN** `the-design-book-follows-the-three-binding-levels@<v2>` is accepted, its source is `views-computed-from-the-ledger`, the narrative `the-drives-log` includes it, and its page is asked for
- **THEN** the page gives `decision`, `@<v2>`, `accepted`, its statement and body, a link to each of its versions, a link to `views-computed-from-the-ledger`, and a link to `the-drives-log`

#### Scenario: A source out of date

- **WHEN** a claim's source is `views-computed-from-the-ledger@<v1>`, and `<v2>` of it is accepted
- **THEN** the claim's page marks the source out of date, and names `<v2>`

#### Scenario: An included claim links to its page

- **WHEN** a narrative includes `status-read-from-events@<v1>`, and the narrative's page is asked for
- **THEN** the box of that claim links `status-read-from-events@<v1>` to `<base>/c/status-read-from-events@<v1>`

### Requirement: The navigation runs no script

The sidebar, the control that folds it, the search and the page of a claim SHALL work with HTML and CSS alone. A page of the book that holds no diagram SHALL hold no `<script>` element, and each response SHALL give the policy header that a narrative's page gives.

#### Scenario: The page of a search

- **WHEN** the search `bolt` is asked for
- **THEN** its HTML holds no `<script>` element and no URL of another host, and the response has the policy header

#### Scenario: The page of a claim

- **WHEN** the page of `status-read-from-events` is asked for
- **THEN** its HTML holds no `<script>` element and no URL of another host, and the response has the policy header
