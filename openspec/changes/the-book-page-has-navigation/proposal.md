## Why

A person reads the design book on `boom page`, and writes notes there.
The page of the book is one list of narratives by level. A narrative's
page links to the book and to the narratives before it and after it,
at its foot. Nothing else on the page leads to another part of the
book, to a claim that the narrative includes, or to a word that the
book holds. A reader of a book expects contents, a way to the next
page and a search.

The user ruled on 2026-11-23: "Navigation in boom page now; render
book later, for publishing." `boom render book`, which writes mdBook
source and then runs `mdbook`, stays in Boom's design for publishing.
This change does not build it.

## What Changes

- **A sidebar on each page of the book.** The book page, each
  narrative's page, each claim's page and the page of a search give
  the book's contents at their side: Part I, Part II with a chapter
  for each repository, and a section for each context in its part or
  chapter. Each narrative is a link with its title, in the book's
  order. The sidebar marks the narrative of the page, and marks each
  narrative that the outline does not hold.
- **Next and previous, as built.** A narrative's page keeps its links
  to the narrative before it, to the book and to the narrative after
  it, in the book's order.
- **A search on each page of the book.** A form at the head of the
  sidebar searches the store with `boom search`'s matching. The page
  of results gives each claim that stands, and each narrative of the
  book. A claim links to its page, and a narrative links to its first
  paragraph that holds each word searched.
- **A page for each claim.** Each claim that a narrative includes
  links to the claim's own page. That page gives the claim's kind,
  status, statement, body, what it binds, its sources and each of its
  versions with its status, and the narratives of the book that
  include it.
- **The sidebar folds on a narrow window.** Under 60rem, the sidebar
  folds behind a "Contents" control, which opens it with no script.
- **What the page keeps.** Each response is computed from the log for
  each request. The notes on paragraphs, the status, the diagrams, the
  token, `127.0.0.1` and the policy header stay as they are. The
  navigation runs no script, and loads nothing from another host.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `narrative-page`: each page of the book gives the book's contents in
  a sidebar that folds on a narrow window, and a search form. A search
  gives its results on a page. Each claim has a page, and an included
  claim links to it.

## Impact

- `crates/boom`: `page.rs` (the sidebar, the search, the claim's page,
  the routes `<base>/search` and `<base>/c/<name>`, and the style),
  `search.rs` (a search of the book's narratives by paragraph, with
  the matching of `boom search`), and the tests in
  `crates/boom/tests/page.rs`.
- `README.md`: the page has contents, a search and a page for each
  claim.
- No new crate, no new dependency in `Cargo.lock`, no new moon project,
  and no tool of `boom mcp`.
