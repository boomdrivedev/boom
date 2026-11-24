# Spec Delta

## Purpose

The views of the claims on `boom page`, beside the design book: the
rules, the decisions, the terms and every claim, by level, with the
proposed versions apart from the accepted ones, filters and a search
with no script, and a claim's page with its history, the claims that
name it, its origin and its notes. Each is computed from the log for
each request (`views-computed-from-the-ledger`,
`the-design-book-follows-the-three-binding-levels`, `the-status-words`).

## ADDED Requirements

### Requirement: The views in the bar

Each page of `boom page` SHALL give in its bar the links The book, The rules, The decisions, The terms and The claims, to `<base>/`, `<base>/rules`, `<base>/decisions`, `<base>/terms` and `<base>/claims/`. The link of the page's view SHALL have `aria-current="page"`. On a view, the sidebar SHALL give the view's levels, each with its count, as a link to that level on the view.

#### Scenario: The bar of the rules view

- **WHEN** `<base>/rules` is asked for
- **THEN** the bar gives the five links, and the link The rules has `aria-current="page"`

#### Scenario: The sidebar of a view

- **WHEN** four invariants and one assumption are accepted for `boom`, and `<base>/rules` is asked for
- **THEN** the sidebar gives the chapter `boom` with the count 5, as a link to `<base>/rules#boom`

### Requirement: The rules view

`GET <base>/rules` SHALL give each accepted invariant, then each accepted assumption, of each level: the whole system, then each repository, then each context under its repository. Inside a level and a kind, it SHALL give them in the order of their names. Each SHALL give its kind, `<name>@<version>` as a link to its page, and its statement, with its body in `<details>`. A claim that binds two repositories SHALL be given under each.

#### Scenario: A rule of a repository

- **WHEN** the invariant `status-read-from-events@<v1>` is accepted for `boom`, and `<base>/rules` is asked for
- **THEN** the page gives it under the chapter `boom`, with a link to `<base>/claims/status-read-from-events@<v1>`, its statement, and its body in `<details>`

#### Scenario: A rule of a context

- **WHEN** an invariant is accepted for the context `sluice-host-process`
- **THEN** the rules view gives it under the section `sluice-host-process`, inside the chapter `sluice`

#### Scenario: A rule of two repositories

- **WHEN** an invariant's accepted version binds `boom` and `sluice`
- **THEN** the rules view gives it under the chapter `boom` and under the chapter `sluice`

#### Scenario: Not a rule

- **WHEN** a decision and a term are accepted for `boom`
- **THEN** the rules view gives neither of them

### Requirement: The rules of a repository, as an agent gets them

`GET <base>/rules?repository=<name>` SHALL give the levels that `boom query --repository <name>` gives: the whole system, the repository and each of its contexts. It SHALL give each invariant and each assumption that `boom query --repository <name> --json` gives, and no other.

#### Scenario: The rules of boom

- **WHEN** the store holds accepted invariants of the whole system, of `boom` and of `sluice`, and `<base>/rules?repository=boom` is asked for
- **THEN** the page gives the invariants and the assumptions that `boom query --repository boom --json` gives, and none of `sluice`

### Requirement: The decisions view

`GET <base>/decisions` SHALL give each accepted decision by level, in the order of the entry that accepted its first version, the oldest first. Each SHALL give `<name>@<version>` as a link to its page, its statement and the date of the entry that accepted it. A decision whose newest version is proposed SHALL be marked "a newer version is proposed".

#### Scenario: Decisions in the order accepted

- **WHEN** `the-status-words` was accepted in entry 201 and `a-term-binds-one-repository` in entry 224, both for `boom`
- **THEN** the chapter `boom` of the decisions view gives `the-status-words` before `a-term-binds-one-repository`

#### Scenario: A newer version proposed

- **WHEN** `the-status-words@<v1>` is accepted and `<v2>` of it is proposed
- **THEN** the decisions view gives `the-status-words@<v1>` marked "a newer version is proposed"

### Requirement: The terms view

`GET <base>/terms` SHALL give each accepted term by repository, in the order of its words, with its word and its meaning, as a link to its page. It SHALL NOT give a term's old words. The page of a term SHALL give its old words, as `boom show` does.

#### Scenario: A term in the terms view

- **WHEN** the term `bolt`, which replaces `batch`, is accepted for `sluice`, and `<base>/terms` is asked for
- **THEN** the page gives `bolt` and its meaning under `sluice`, and the word `batch` is not on the page

#### Scenario: The page of a term

- **WHEN** the page of the term `bolt` is asked for
- **THEN** the page gives that it replaces `batch`

### Requirement: The claims view

`GET <base>/claims/` SHALL give each accepted claim of every kind but narrative and outline, by level, then by kind in the order invariant, decision, assumption, term, context, then by name.

#### Scenario: Every kind

- **WHEN** an invariant, a decision, an assumption, a term and a context are accepted for `boom`, and a narrative binds `boom`
- **THEN** the claims view gives the five under the chapter `boom`, in that order, and not the narrative

### Requirement: A context is a level

Each view SHALL give each context as a section under its repository, headed by its name and its accepted globs. The page of a context SHALL give its globs and each accepted claim that binds it, by kind.

#### Scenario: A context's section

- **WHEN** the context `sluice-host-process` owns `sluice:crates/host/**`, and `<base>/rules` is asked for
- **THEN** the section `sluice-host-process` under `sluice` gives the glob `sluice:crates/host/**`

#### Scenario: A context's page

- **WHEN** two invariants and one decision are accepted for `sluice-host-process`, and its page is asked for
- **THEN** the page gives its globs, the two invariants and the decision, each as a link to its page

### Requirement: The proposed versions apart

Each view SHALL give first, before its levels, under "Proposed, binding nothing", each proposed version of its kinds, grouped by the level that it binds, each in a frame of its own marked "proposed" in words, with `boom accept <name>@<version>`. At the head of that list, it SHALL give one `boom accept` that names each version of the list. It SHALL then give the accepted versions. No view SHALL give a rejected or a superseded version.

#### Scenario: A proposed decision

- **WHEN** `only-an-accepted-version-binds-work@<v1>` is proposed for the whole system, and `<base>/decisions` is asked for
- **THEN** the page gives it under "Proposed, binding nothing", in the group of the whole system, marked "proposed", with `boom accept only-an-accepted-version-binds-work@<v1>`, before Part I

#### Scenario: One command for every proposal

- **WHEN** two decisions are proposed, and `<base>/decisions` is asked for
- **THEN** the head of "Proposed, binding nothing" gives one `boom accept` that names both versions

#### Scenario: A rejected version

- **WHEN** `<v1>` of a decision was rejected, and no other version of it is accepted or proposed
- **THEN** no view gives the decision

#### Scenario: Nothing proposed

- **WHEN** no proposed version is of the view's kinds
- **THEN** the view gives no "Proposed, binding nothing"

### Requirement: A claim's page

`GET <base>/claims/<name>[@<version>]` SHALL give the claim at that version, or at its accepted version, else its newest, where none is named: its kind, `<name>@<version>`, its status, its statement, what it binds, its body, its versions, its sources and the claims it changes, each a link to its page, and each narrative of the book that includes it, as a link to the paragraph that names it. The bar SHALL mark The claims. A name that no claim has SHALL answer 404.

#### Scenario: The page of an invariant

- **WHEN** `status-read-from-events@<v1>` is accepted for `boom`, and `boom-end-to-end` includes it at its paragraph 3, and `<base>/claims/status-read-from-events` is asked for
- **THEN** the page gives `invariant status-read-from-events@<v1>`, accepted, its statement, binds `boom`, and a link to `<base>/n/boom-end-to-end#p3`

#### Scenario: No such claim

- **WHEN** `<base>/claims/no-such-claim` is asked for
- **THEN** boom answers 404

### Requirement: A claim's history

The page of a claim SHALL give, under "History", each version that is neither proposed nor accepted, with its status, the entry that closed it, the person, and the reason of a rejection. Its versions line SHALL give the accepted and the proposed versions alone.

#### Scenario: A superseded and a rejected version

- **WHEN** `<v1>` of a claim was superseded by `<v2>`, `<v2>` is accepted, and `<v3>` was rejected with the reason "says standing"
- **THEN** the versions line gives `<v2>`, and "History" gives `<v1>` superseded and `<v3>` rejected with "says standing"

### Requirement: The claims that name a claim

The page of a claim SHALL give each claim whose accepted or proposed version names it with `--source` or `--changes`, with the version meant, as a link to that claim's page, and SHALL mark it out of date where the version meant is not the accepted one.

#### Scenario: A claim named as a source

- **WHEN** `only-an-accepted-version-binds-work@<v1>` names `the-status-words@<v1>` as a source, and `<v2>` of `the-status-words` is accepted
- **THEN** the page of `the-status-words` gives `only-an-accepted-version-binds-work` under the claims that name it, on `@<v1>`, marked out of date

### Requirement: A claim's origin

The page of a claim SHALL give the origin of the version it gives: a kept file as its name, its fingerprint and `boom show --artifact <fingerprint>`, and a commit as `git:<repository>@<commit>:<path>`. `boom page` SHALL NOT serve the bytes of a kept file.

#### Scenario: A kept file

- **WHEN** the origin of a version is `claim-status-brief.md`, kept by the fingerprint `<fp>`
- **THEN** its page gives `claim-status-brief.md`, `<fp>` and `boom show --artifact <fp>`, and no link to the file's bytes

### Requirement: A claim's notes

The page of a claim SHALL give each note on a paragraph of a narrative of the book that includes the claim at that paragraph, open or answered, with its person, its time and a link to `<base>/n/<narrative>#p<n>`.

#### Scenario: A note on a paragraph that includes the claim

- **WHEN** paragraph 2 of `the-drives-log` includes `events-are-appended-never-changed`, and chuck's open note is on paragraph 2
- **THEN** the page of `events-are-appended-never-changed` gives the note, `open`, chuck, its time, and a link to `<base>/n/the-drives-log#p2`

### Requirement: A proposed version's page

The page of a proposed version SHALL give it in a frame of its own marked "proposed, binds nothing", with `boom accept <name>@<version>` and `boom reject <name>@<version> --reason "<why>"`. The page SHALL NOT accept or reject.

#### Scenario: The page of a proposed decision

- **WHEN** the page of `only-an-accepted-version-binds-work@<v1>` is asked for, and `<v1>` is proposed
- **THEN** the page gives "proposed, binds nothing", `boom accept only-an-accepted-version-binds-work@<v1>` and `boom reject only-an-accepted-version-binds-work@<v1> --reason`, and no form that sends a `POST`

### Requirement: Filters with no script

Each view SHALL give a form that sends `GET` to its own path, with the fields `kind`, `status`, `repository`, `level` and `q`. `status` SHALL take `both`, `accepted` or `proposed`, and `both` when left out: the proposed versions apart, then the accepted ones. `level` SHALL take `system`, a repository's name or `<repository>/<context>`. A value that the field does not take SHALL answer 400 with the values it takes. The view SHALL say how many claims it gives.

#### Scenario: The proposed decisions of boom

- **WHEN** `<base>/decisions?status=proposed&repository=boom` is asked for
- **THEN** the page gives the proposed decisions of the whole system and of `boom`, and no accepted decision

#### Scenario: A kind that is not one

- **WHEN** `<base>/claims/?kind=rule` is asked for
- **THEN** boom answers 400, and the page names `invariant`, `decision`, `assumption`, `term` and `context`

### Requirement: A search in a view

The field `q` of a view SHALL keep each claim whose words hold each word, with the matching of `boom search`, over the claims that the other fields keep.

#### Scenario: A search in the rules view

- **WHEN** `<base>/rules?q=RECORDS` is asked for
- **THEN** the page gives `status-read-from-events`, as `boom search RECORDS --all` gives it, and no rule that does not hold the word

#### Scenario: A proposed claim found

- **WHEN** a decision for `boom` that is proposed holds the word `rail`, and `<base>/decisions?status=both&q=rail` is asked for
- **THEN** the page gives that decision, marked "proposed"

### Requirement: The views are computed for each request

Each view SHALL be computed from the log at its last entry when it is asked for. No file SHALL hold a view.

#### Scenario: A claim accepted while the view is served

- **WHEN** `<base>/decisions` gives a decision under "Proposed, binding nothing", a person then accepts it, and the browser asks for the page again
- **THEN** the page gives the decision among the accepted ones, and not under "Proposed, binding nothing"

### Requirement: The views run no script

Each view SHALL work with HTML and CSS alone, under the page's token, and SHALL hold no `<script>` element and no URL of another host. Each response SHALL give the policy header that a narrative's page gives. A request with no token SHALL answer 404. Each value of a filter SHALL be escaped where the page gives it.

#### Scenario: The rules view

- **WHEN** `<base>/rules` is asked for
- **THEN** its HTML holds no `<script>` element and no URL of another host, and the response has the policy header

#### Scenario: A view with no token

- **WHEN** a request asks for `/rules` with no token
- **THEN** boom answers 404, and gives no part of the store

#### Scenario: A filter's value given back

- **WHEN** `<base>/claims/?q=<b>bolt</b>` is asked for
- **THEN** the page gives the search field's value as `&lt;b&gt;bolt&lt;/b&gt;`, and holds no `<b>` element from it
