## ADDED Requirements

### Requirement: One shell for each view

Each page of `boom page` SHALL give a bar at its top with a link to each view that boom serves, the book at `<base>/` and the claims at `<base>/claims/`, the current view's link with `aria-current="page"`, a search form, and the log's entry. It SHALL give the view's sidebar in `<nav class="side">`, then the view's page in `<main>`.

#### Scenario: The bar of a narrative's page

- **WHEN** the page of `boom-end-to-end` is asked for
- **THEN** the bar links "The book" to `<base>/` with `aria-current="page"`, links "The claims" to `<base>/claims/`, holds the search form, and gives `entry <n>` of the log's last entry

#### Scenario: The bar of a search

- **WHEN** the search `bolt` is asked for
- **THEN** the bar links to each view, and no link of the bar has `aria-current`

#### Scenario: The order of the views

- **WHEN** the book page is asked for
- **THEN** the bar gives "The book", then "The claims", and no other view

### Requirement: The book's contents in the sidebar

On the book page, each narrative's page and the page of a search, the sidebar SHALL give the design book's contents: Part I with its narratives and a section for each of its contexts, then Part II with a chapter for each repository and a section for each of its contexts. Each narrative SHALL be a link to its page with its title, in the book's order.

#### Scenario: The sidebar of a narrative's page

- **WHEN** the book gives `the-drives-log` in Part I, then `boom-end-to-end` in the chapter `boom`, and the page of `boom-end-to-end` is asked for
- **THEN** the sidebar gives `Part I. The system` with a link to `the-drives-log`, then `Part II. The repositories` with the chapter `boom` and a link to `boom-end-to-end`

#### Scenario: A section of a context

- **WHEN** `the-merge-queue` binds the context `sluice-host-process`, whose globs name `sluice`, and a narrative's page is asked for
- **THEN** the sidebar gives the chapter `sluice` with the section `sluice-host-process`, and a link to `the-merge-queue` under it

#### Scenario: Narratives only

- **WHEN** the invariant `status-read-from-events` is accepted for `boom`, no narrative includes it, and the book page is asked for
- **THEN** neither the sidebar nor the book page names `status-read-from-events`

### Requirement: The sidebar marks the page's narrative

The book's sidebar SHALL mark the narrative of the page, at any of its versions, with `aria-current="page"`. It SHALL mark each narrative that the outline does not hold, as the book page marks it.

#### Scenario: The current narrative

- **WHEN** the page of `boom-end-to-end` is asked for
- **THEN** the sidebar's link to `boom-end-to-end` has `aria-current="page"`, and no other link of the sidebar has it

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

A page that gives a sidebar SHALL give a control with the word "Contents" before it. Under 60rem wide, the page SHALL hide the sidebar until the control opens it, with no script. At 60rem or wider, the page SHALL show the sidebar beside the page, and SHALL hide the control.

#### Scenario: A narrow window

- **WHEN** a browser 400 pixels wide shows a narrative's page, and then the reader clicks "Contents"
- **THEN** the sidebar is hidden and "Contents" is shown, and after the click the sidebar is shown

#### Scenario: A wide window

- **WHEN** a browser 1280 pixels wide shows a narrative's page
- **THEN** the sidebar is shown at the left of the narrative's text, and "Contents" is hidden

### Requirement: A search from the page

The bar's form SHALL send `GET <base>/search?q=<words>`. The page of the search SHALL give the narratives of the book whose version in the book holds each word in its paragraphs, then each other claim that `boom search --all <words>` gives, in its order, with its kind, `<name>@<version>`, status and statement, as a link to `<base>/claims/<name>@<version>`. The search SHALL match words as `boom search` does.

#### Scenario: A claim found

- **WHEN** the invariant `status-read-from-events` stands for `boom`, and the search `status events` is asked for
- **THEN** the page gives `invariant status-read-from-events@<version>`, its status and its statement, with a link to `<base>/claims/status-read-from-events@<version>`

#### Scenario: The forms of a word

- **WHEN** the search `RECORDS` is asked for
- **THEN** the page gives `status-read-from-events`, as `boom search RECORDS --all` gives it

#### Scenario: No word to search

- **WHEN** the search `***` is asked for
- **THEN** the page says that a search needs a word that holds a letter or a digit, with status 200, and gives no result

#### Scenario: A search with no token

- **WHEN** a request asks for `/search?q=bolt` with no token
- **THEN** boom answers 404, and gives no part of the store

### Requirement: A narrative found links to its paragraph

A narrative found SHALL link to its page at `#p<n>`, the first paragraph that holds each word, with that paragraph's words, or to its page where no one paragraph holds them all.

#### Scenario: A word in a narrative's paragraph

- **WHEN** paragraph 4 of `boom-end-to-end` is the first that holds the word `replay`, and the search `replay` is asked for
- **THEN** the page gives `boom-end-to-end` with a link to its page at `#p4`, and the words of paragraph 4

#### Scenario: A narrative that is proposed

- **WHEN** `change-records` has one version, proposed, the book gives it, and its paragraph 2 holds the word `bolt`
- **THEN** the search `bolt` gives `change-records` with a link at `#p2`

#### Scenario: Words in two paragraphs

- **WHEN** a narrative's paragraph 2 holds `bolt` and its paragraph 5 holds `landing`, and no one paragraph holds both
- **THEN** the search `bolt landing` gives the narrative with a link to its page and no `#p`

### Requirement: An included claim links to the claims view

On a narrative's page, the box of each included claim SHALL link its `<name>@<version>` to `<base>/claims/<name>@<version>`, at the version that the narrative meant.

#### Scenario: An included claim's link

- **WHEN** a narrative includes `status-read-from-events@<v1>`, and the narrative's page is asked for
- **THEN** the box of that claim links `status-read-from-events@<v1>` to `<base>/claims/status-read-from-events@<v1>`

#### Scenario: An included claim out of date

- **WHEN** a narrative includes `status-read-from-events@<v1>`, and `<v2>` of it is accepted
- **THEN** the box links to `<base>/claims/status-read-from-events@<v1>`, and marks it out of date with `<v2>`

### Requirement: The shell runs no script

The bar, the sidebar, the control that folds it and the search SHALL work with HTML and CSS alone. A page with no diagram SHALL hold no `<script>` element, and each response SHALL give the policy header that a narrative's page gives.

#### Scenario: The page of a search

- **WHEN** the search `bolt` is asked for
- **THEN** its HTML holds no `<script>` element and no URL of another host, and the response has the policy header

#### Scenario: The book page

- **WHEN** the book page is asked for, and no narrative holds a diagram
- **THEN** its HTML holds no `<script>` element and no URL of another host, and the response has the policy header
