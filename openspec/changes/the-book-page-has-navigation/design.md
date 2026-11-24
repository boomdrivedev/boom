## Context

`boom page` serves the design book and the page of each narrative on
`127.0.0.1`, under a token, computed from the log for each request
(`narrative-page`). The book page gives each part, chapter and section
of the book as a heading, and each narrative under it as a link. A
narrative's page gives its paragraphs, the notes on them and the claims
that it includes, each claim as a box. At its foot, it links to the
book and to the narratives before it and after it in the book's order.

The user reads the book there and writes notes. The user ruled on
2026-11-23: "Navigation in boom page now; render book later, for
publishing." `boom render book` writes mdBook source and then runs
`mdbook` (the site's pages `integrations` and `how-its-used`). It stays
in Boom's design for publishing, and this change does not build it.

No other tool of the family serves a page of the book: flume, sluice,
weir, timber and Boomdrive hold no page of it. boom owns this design,
and no other tool must change with it.

This design keeps these claims of the store:

- `views-computed-from-the-ledger`: the sidebar, the search and the
  claim's page are computed from the views at the log's last entry, for
  each request. No file holds them, and no index of the search is kept.
- `the-design-book-follows-the-three-binding-levels` and
  `a-views-outline-is-a-claim-and-its-levels-come-from-the-view`: the
  sidebar is the book that `view::book` computes, its parts from the
  levels and its order from the outline. A narrative still names no
  book, part or chapter.
- `an-agent-finds-claims-by-their-words-searched-in-memory-from-the-log`:
  the page's search uses the matching of `boom search`, in a table
  filled in memory for one answer.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  `boom mcp` serves no page, so this change adds no tool.
- `no-credential-in-the-repository`: the token stays random for each
  run of `boom page`, and each new path starts with it.
- `names-use-the-domains-terms`: `contents`, `sidebar`, `search` and
  `next` are the words of a reader of a book.

## Goals / Non-Goals

**Goals:**

- From each page of the book, a reader reaches each other narrative in
  one click, and sees where the page sits in the book.
- A reader finds a word across the book and the store from the page.
- A reader opens a claim that a narrative includes, with its status and
  its versions.
- The page reads on a narrow window.

**Non-Goals:**

- `boom render book`, mdBook source, or a page written to a file. It
  stays in the design for publishing.
- A script of the page's own. The navigation works with HTML and CSS.
- A search of the notes, or of a version that is not the book's.
- A tool of `boom mcp` for the page.

## Decisions

1. **The sidebar is the book, on each page of the book.** The book
   page, each narrative's page, each claim's page and the page of a
   search give one `<nav class="side">` beside `<main>`. It holds the
   search form, then Part I with its narratives and the section of
   each context of two repositories or more, then Part II with a
   chapter for each repository and a section for each of its contexts.
   Each narrative is a link to its page with its title, in the book's
   order. The narrative of the page has `aria-current="page"`, and each
   narrative that the outline does not hold carries the mark that the
   book page gives it. The page of a version of a narrative marks that
   narrative too. The sidebar of `<base>/v/<outline>` is the book of
   that outline. Each other page gives the book that the page with no
   name gives.

   Not taken:
   - A sidebar of the current part alone. The book holds tens of
     narratives, and a reader moves across parts.
   - A sidebar that lists each claim. The book's contents are its
     narratives, and the claim's page and the search reach the claims.
2. **Next and previous stay as built.** `neighbors` gives the links to
   the narrative before, the book and the narrative after, at the foot
   of a narrative's page, as the requirement "A narrative's place in
   the book" says. The sidebar gives the rest of the book, so this
   change adds no other link of that kind.
3. **The search is a form that the server answers.** The form is
   `<form method="get" action="<base>/search">` with one field `q`.
   `GET <base>/search?q=<words>` gives a page of results, computed for
   that request. It runs two searches with the matching of `boom
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
     status, statement and excerpt, as a link to the claim's page.

   A narrative that `boom search --all` gives as a claim is given once,
   among the narratives. A `q` with no word that holds a letter or a
   digit gives the page with `boom search`'s words for that refusal,
   and no result. The query is in the URL under the token, and the
   referrer policy keeps that URL on the page's own origin.

   Not taken:
   - A search in the browser over an index that the page sends. It
     needs a script, and the index is a stored view.
   - A search of the narratives' accepted versions alone, as `boom
     search` gives them. The user reads proposed narratives on the page,
     and their words must be found.
4. **A page for each claim.** `GET <base>/c/<name>` gives the claim at
   the version that stands, or its newest version, and
   `<base>/c/<name>@<version>` gives that version. The page gives the
   claim's kind, `<name>@<version>`, status, statement, what it binds
   and its body. It gives each version with its status, as a link. It
   gives each source and each claim that it changes as a link to that
   claim's page, marked out of date where the version meant is not the
   one accepted now. It gives each narrative of the book that includes
   the claim, as a link. The name of a narrative answers 303 to its
   page, and the name of an outline answers 303 to `<base>/v/<name>`.
   A name that no claim has answers 404. In a narrative's page, the
   head of each claim's box links `<name>@<version>` to that version's
   page.
5. **The sidebar folds with no script.** The page gives `<input
   type="checkbox" id="contents">` and a `<label for="contents">` with
   the word "Contents" before the sidebar. The style shows the sidebar
   at 60rem or wider, and hides the box and the label there. Under
   60rem, it hides the sidebar until the box is checked. A reader with
   no CSS reads the sidebar before the page.
6. **The layout.** At 60rem or wider, the sidebar is a column of 17rem
   at the left, which scrolls apart from the page and stays in view.
   `<main>` keeps its width of 46rem for reading. The bar at the head of
   the page stays.
7. **What stays.** Each response is computed from the views at the
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
- [A long book makes a long sidebar] → The sidebar scrolls apart from
  the page, and the current narrative is marked.
