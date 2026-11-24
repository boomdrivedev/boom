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

The user also ruled on 2026-11-23 that the book, the decision records,
the context map and the rules are different concepts. The claims get a
view of their own, apart from the book, which another change designs. The views of `boom page` then need one shell, so that a reader
moves between them.

## What Changes

- **One shell for each view.** Each page of `boom page` has the same
  shell: a bar at the top that names each view that boom serves, the
  book and the claims, as links, with the current view marked; a search
  form; a sidebar that the view fills with its own contents; and the
  page. A later view, such as the context map, adds itself to the bar.
- **The book's contents in the sidebar.** In the book, the sidebar gives
  Part I, Part II with a chapter for each repository, and a section for
  each context. Each narrative is a link with its title, in the book's
  order. The sidebar marks the narrative of the page, and marks each
  narrative that the outline does not hold.
- **Next and previous, as built.** A narrative's page keeps its links
  to the narrative before it, to the book and to the narrative after
  it, in the book's order.
- **A search from each view.** The form in the bar searches the store
  with `boom search`'s matching. The page of results gives the book's
  narratives, each linked to its first paragraph that holds each word
  searched, then the claims that stand, each linked to its page in the
  claims view.
- **An included claim links to the claims view.** Each claim that a
  narrative includes links to its page in the claims view, at the
  version that the narrative meant.
- **The sidebar folds on a narrow window.** Under 60rem, the sidebar
  folds behind a "Contents" control, which opens it with no script.
- **What the page keeps.** Each response is computed from the log for
  each request. The notes on paragraphs, the status, the diagrams, the
  token, `127.0.0.1` and the policy header stay as they are. The shell
  runs no script, and loads nothing from another host.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `narrative-page`: each page has one shell that names the views and
  holds a search form and a sidebar that folds on a narrow window. The
  book fills the sidebar with its contents. A search gives its results
  on a page. An included claim links to its page in the claims view.

## Impact

- `crates/boom`: `page.rs` (the shell, the views of the bar, the book's
  sidebar, the search, the route `<base>/search`, the link of a claim's
  box, and the style), `search.rs` (a search of the book's narratives by
  paragraph, with the matching of `boom search`), and the tests in
  `crates/boom/tests/page.rs`.
- The change of the claims view serves `<base>/claims/` and fills the
  shell's sidebar for its view. The two changes land in one bolt.
- `README.md`: the page has a bar of views, contents and a search.
- No new crate, no new dependency in `Cargo.lock`, no new moon project,
  and no tool of `boom mcp`.
