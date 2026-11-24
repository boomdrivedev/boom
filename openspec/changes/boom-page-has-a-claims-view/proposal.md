## Why

A person reads the store on `boom page`, and the page gives the design
book: narratives, each with the claims that it includes. A claim that
no narrative includes shows nowhere on the page. On 2026-11-23 the user
missed the accepted residents claims that way, and said: "I think we
need a claims viewer. Either separate or part of the book. Probably
separate, since book, ADRs, context map, constitution/rules are
different concepts."

Boom's design names four views of the log: the rules for a scope, the
decision records, the design book and the context map
(`views-computed-from-the-ledger`). `boom render` writes them for
publishing, and no part of it is built. `boom page` is where the person
reads, notes and decides. The change `the-book-page-has-navigation`
gives the page a sidebar, a search and a page for each claim. This
change gives the page the rules and the decision records as views of
their own, beside the book, in that shell.

## What Changes

- **Views beside the book.** The bar at the head of each page of
  `boom page` links to its views: The book, The rules, The decisions,
  The terms and The claims, the current one marked.
- **The rules view.** `<base>/rules` gives each accepted invariant and
  assumption by level: the whole system, each repository, and each
  context under its repository. Each rule gives its name, its version
  and its statement, and folds its details. `?repository=<name>` gives
  the levels that `boom query --repository <name>` gives an agent.
- **The decisions view.** `<base>/decisions` gives each accepted
  decision by level, in the order the person accepted them, each with
  its statement and a link to its page, where its context, options and
  outcome are.
- **The terms view.** `<base>/terms` gives each accepted term by
  repository, its word and its meaning.
- **The claims view.** `<base>/claims` gives each claim of every kind
  but narrative and outline, by level, then by kind.
- **Contexts as levels.** Each context is a level of each view, under
  its repository, with its globs. A context's page gives the claims
  that bind it.
- **Proposed versions apart.** Each view gives its proposed versions
  first, each marked "proposed, binds nothing" in words, with the
  command that accepts it and one command that accepts them all. The
  accepted versions follow, plain.
- **A claim's page in full.** The page of a claim adds its history (each
  version that is neither proposed nor accepted, with its status and
  why), the claims that name it, its origin, and the notes on the
  paragraphs that include it. A proposed version's page gives the
  commands that accept and reject it.
- **Filters and search.** Each view takes `kind`, `status`,
  `repository`, `level` and `q` from a form with no script. `q`
  matches as `boom search` does.
- **What the page keeps.** Each view is computed from the log for each
  request, under the page's token, on `127.0.0.1`, with the policy
  header, no script, nothing from another host and no `POST`.

## Capabilities

### New Capabilities

- `claims-view`: the views of the claims on `boom page`, beside the
  book: rules, decisions, terms and every claim, by level, with the
  proposed versions apart, the filters and the search, and a claim's
  page with its history, the claims that name it, its origin and its
  notes.

### Modified Capabilities

None.

## Impact

- `crates/boom`: `view.rs` (the claims of each level, by kind and
  status, and the claims that name a claim), `page.rs` (the bar's
  views, the routes `<base>/rules`, `<base>/decisions`,
  `<base>/terms` and `<base>/claims`, the filter form, and the
  sections that a claim's page adds), and the tests in
  `crates/boom/tests/page.rs`.
- `README.md` and the site's views table: `boom page` gives the rules,
  the decisions and the terms.
- It is built after `the-book-page-has-navigation` lands, on its
  sidebar, its search and its claim's page.
- No new crate, no new dependency in `Cargo.lock`, no new moon project,
  and no tool of `boom mcp`.
