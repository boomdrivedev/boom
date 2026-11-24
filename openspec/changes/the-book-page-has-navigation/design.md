## Context

`boom page` serves the design book and the page of each narrative on
`127.0.0.1`, under a token, computed from the log for each request
(`narrative-page`). The book page gives each part, chapter and section
of the book as a heading, and each narrative under it as a link. A
narrative's page gives its paragraphs, the notes on them and the claims
that it includes, each claim as a box. At its foot, it links to the
book and to the narratives before it and after it in the book's order.
Each page is one column under a bar that links to the book.

The user reads the book there and writes notes. The user ruled on
2026-11-23: "Navigation in boom page now; render book later, for
publishing." `boom render book` writes mdBook source and then runs
`mdbook` (the site's pages `integrations` and `how-its-used`). It stays
in Boom's design for publishing, and this change does not build it.

The user also ruled on 2026-11-23: "book, adrs, context map,
constitution/rules are different concepts." The claims get a view of
their own, apart from the book, in another change. This change gives
`boom page` the shell that each view shares, and the book's part of it.
The change of the claims view builds on the shell.

No other tool of the family serves a page of the book: flume, sluice,
weir, timber and Boomdrive hold no page of it. boom owns this design,
and no other tool must change with it.

This design keeps these claims of the store:

- `views-computed-from-the-ledger`: the shell, the sidebar and the
  search are computed from the views at the log's last entry, for each
  request. No file holds them, and no index of the search is kept.
- `the-design-book-follows-the-three-binding-levels` and
  `a-views-outline-is-a-claim-and-its-levels-come-from-the-view`: the
  book's sidebar is the book that `view::book` computes, its parts from
  the levels and its order from the outline. A narrative still names no
  book, part or chapter.
- `an-agent-finds-claims-by-their-words-searched-in-memory-from-the-log`:
  the page's search uses the matching of `boom search`, in a table
  filled in memory for one answer.
- `the-book-is-read-on-boom-page-and-render-book-waits-for-publishing`,
  proposed: the user's ruling of this change.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  `boom mcp` serves no page, so this change adds no tool.
- `no-credential-in-the-repository`: the token stays random for each
  run of `boom page`, and each new path starts with it.
- `names-use-the-domains-terms`: `view`, `contents`, `sidebar` and
  `search` are the words of a reader of a book and of a page.

## Goals / Non-Goals

**Goals:**

- One shell that each view of `boom page` shares, so a reader moves
  between the book, the claims and later the context map.
- From each page of the book, a reader reaches each other narrative in
  one click, and sees where the page sits in the book.
- A reader finds a word across the book and the store from each view.
- A claim that a narrative includes leads to its page in the claims
  view.
- The page reads on a narrow window.

**Non-Goals:**

- The claims view: its paths' pages, its sidebar and its lists. Another
  change designs it on this shell.
- A list of claims in the book. The book gives narratives only.
- `boom render book`, mdBook source, or a page written to a file. It
  stays in the design for publishing.
- A script of the page's own. The shell works with HTML and CSS.
- A search of the notes, or of a version that is not the book's.
- A tool of `boom mcp` for the page.

## Decisions

1. **The shell's parts.** One function of `page.rs`, `shell`, writes
   each page that `boom page` serves, in place of `document`. It takes
   the page's title, its view, the HTML of the view's sidebar, and the
   HTML of `<main>`. In order, the page holds:
   - `<header class="bar">`: the word "Boom", then `<nav class="views">`
     with a link for each view that boom serves, in the order of
     `VIEWS`, the current view's link with `aria-current="page"`, then
     the search form, then the log's entry, `entry <n>`.
   - `<input type="checkbox" id="contents">` and `<label
     for="contents">Contents</label>`, the control that folds the
     sidebar (decision 6).
   - `<nav class="side">`, which the view fills with its own contents.
     A view with no contents gives none, and the page then gives no
     control.
   - `<main>`, the page of the view.
   - The foot: Mermaid's license, where the page holds a diagram.
2. **The views of the bar.** `VIEWS` is a constant list in `page.rs`:
   for each view, its words in the bar and the path of its first page
   under the token. This change holds two: "The book", `<base>/`, and
   "The claims", `<base>/claims/`. The book's pages are `<base>/`,
   `<base>/v/<outline>` and `<base>/n/<name>[@<version>]`. The claims
   view's pages are `<base>/claims/` and
   `<base>/claims/<name>[@<version>]`, which the change of the claims
   view serves. A later view adds one entry to `VIEWS`, such as "The
   context map", `<base>/map/`, and its own routes. The page of a search
   belongs to no view, and no link of the bar is current there.

   The two changes land in one bolt, so no link of the bar names a path
   that boom does not serve.

   Not taken:
   - The views in the sidebar, above each view's contents. The sidebar
     folds on a narrow window, and the views must show on each window.
   - A view of its own for the search. A search crosses the views.
3. **The book's sidebar.** On the book page and each narrative's page,
   the sidebar gives Part I with its narratives and the section of each
   context of two repositories or more, then Part II with a chapter for
   each repository and a section for each of its contexts. Each
   narrative is a link to its page with its title, in the book's order.
   The narrative of the page has `aria-current="page"`, at any of its
   versions, and each narrative that the outline does not hold carries
   the mark that the book page gives it. The sidebar of
   `<base>/v/<outline>` is the book of that outline. Each other page of
   the book gives the book that the page with no name gives. The page of
   a search gives the book's sidebar too.

   Not taken:
   - A sidebar of the current part alone. The book holds tens of
     narratives, and a reader moves across parts.
4. **Next and previous stay as built.** `neighbors` gives the links to
   the narrative before, the book and the narrative after, at the foot
   of a narrative's page, as the requirement "A narrative's place in
   the book" says. The sidebar gives the rest of the book, so this
   change adds no other link of that kind.
5. **The search is a form that the server answers.** The form in the
   bar is `<form method="get" action="<base>/search">` with one field
   `q`. `GET <base>/search?q=<words>` gives a page of results, computed
   for that request. It runs two searches with the matching of `boom
   search`: `search::words` takes the words, and `search::search` fills
   its table in memory. The page gives the narratives first, then the
   claims.
   - The narratives: each narrative of the book, at the version that
     the book gives, whose paragraphs hold each word. Each paragraph
     is a row of the table, so a narrative that is proposed is found
     too. A narrative's result links to `<base>/n/<name>#p<n>`, the
     first paragraph that holds each word, or to its page where no one
     paragraph holds them all. It gives that paragraph's words as its
     excerpt.
   - The claims: each other claim that `boom search --all` gives for
     those words, in its order, with its kind, `<name>@<version>`,
     status, statement and excerpt, as a link to
     `<base>/claims/<name>@<version>`.

   A `q` with no word that holds a letter or a digit gives the page
   with `boom search`'s words for that refusal, and no result. The
   query is in the URL under the token, and the referrer policy keeps
   that URL on the page's own origin.

   Not taken:
   - A search in the browser over an index that the page sends. It
     needs a script, and the index is a stored view.
   - A search of the narratives' accepted versions alone, as `boom
     search` gives them. The user reads proposed narratives on the page,
     and their words must be found.
6. **An included claim links to the claims view.** On a narrative's
   page, the head of each claim's box links `<name>@<version>` to
   `<base>/claims/<name>@<version>`, the version that the narrative
   meant. The box keeps its kind, status, statement, details and its
   mark of a version out of date.
7. **The sidebar folds with no script.** The style shows the sidebar
   at 60rem or wider, and hides the control there. Under 60rem, it
   hides the sidebar until the control's box is checked. A reader with
   no CSS reads the sidebar before the page.
8. **The layout.** At 60rem or wider, the sidebar is a column of 17rem
   at the left, which scrolls apart from the page and stays in view.
   `<main>` keeps its width of 46rem for reading. Under 60rem, the bar
   wraps: the views on one line, the search on the next.
9. **What stays.** Each response is computed from the views at the
   log's last entry. The notes on paragraphs and their forms, the
   status, the diagrams, the token on each path, the `Host` check, the
   referrer policy and the policy header stay as they are. A page with
   no diagram holds no `<script>` element. No new path answers a
   `POST`.

## Risks / Trade-offs

- [Each page computes the book] → The book is computed from the views
  that boom already holds in memory for a request, as `neighbors` does
  now. A store of tens of narratives computes it in a few milliseconds.
- [A search of every paragraph fills a table for each request] → The
  table holds one row for each paragraph of the book's narratives, a
  few thousand rows, and is dropped with the answer.
- [Two changes in one bolt] → The bar and the claim's link name the
  claims view's paths. If the change of the claims view lands apart,
  those links answer 404 until it lands, so the plan puts both in one
  bolt.
- [A long book makes a long sidebar] → The sidebar scrolls apart from
  the page, and the current narrative is marked.
