## Context

`boom page` serves the design book and the page of each narrative on
`127.0.0.1`, under a token, computed from the log for each request
(`narrative-page`). The change `the-book-page-has-navigation` adds a
sidebar with the book's contents, a search with the matching of
`boom search`, a page for each claim at `<base>/c/<name>`, and, at the
end of each level of the book, the accepted claims that no narrative
includes. It is not built yet. This change builds on its shell.

The user asked on 2026-11-23 for a claims viewer, "probably separate,
since book, ADRs, context map, constitution/rules are different
concepts". Boom's site names those views (`how-its-used`, the views
table): the rules for a scope, the decision records, the design book
and the context map, each made by `boom render`. `boom render` writes
files for publishing, and stays in the design for that. `boom page` is
where the person reads them now.

No other tool of the family serves a view of the claims. The cloud's
rail shows a proposal as a card (the proposed decision
`a-waiting-proposal-is-a-card-on-the-rail`), and points to `boom page`
where no drive is attached. boom owns this design, and no other tool
must change with it.

This design keeps these claims of the store:

- `views-computed-from-the-ledger`: each view is computed from the
  views at the log's last entry, for each request. No file holds a
  view, and no index of the search is kept.
- `the-design-book-follows-the-three-binding-levels`: each view has the
  book's levels, the whole system, each repository and each context,
  from what each claim's accepted version binds.
- `an-agent-gets-a-repositorys-rules-from-boom-query-each-claim-attached-to-the-repositories-it-binds`:
  the rules view of a repository gives what `boom query` gives an
  agent of that repository.
- `the-status-words`: the views say proposed, accepted, rejected and
  superseded, and no other word for a status.
- `a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key`: the
  page gives the command that accepts or rejects, and never does it.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  `boom mcp` serves no page, so this change adds no tool.
- `no-credential-in-the-repository`: the token stays random for each
  run of `boom page`, and each new path starts with it.

It follows two decisions that are proposed and not accepted, and
changes with them if the user changes them:
`only-an-accepted-version-binds-work` (a proposed version shows,
marked, only where a person judges it, and a closed version only in
history) and `a-waiting-proposal-is-a-card-on-the-rail` (`boom page`
gives what waits, with the command that accepts it).

## Goals / Non-Goals

**Goals:**

- A reader sees every accepted claim of the store, by level, without a
  narrative that includes it.
- A reader sees the rules that an agent of a repository gets, as the
  agent gets them.
- A reader sees what waits on them apart from what is accepted, with
  the command that accepts it.
- A reader reaches each claim's whole story from its page: versions,
  history, sources, the claims that name it, origin, narratives and
  notes.

**Non-Goals:**

- `boom render rules`, `boom render adrs`, `boom render map` or any file
  written. They stay in the design for publishing.
- The context map, drawn. It is a view of its own, in a change of its
  own.
- A person's act from the page. The page gives commands, and a
  terminal with the key runs them.
- A script of the page's own.
- A tool of `boom mcp`.

## Decisions

1. **Where it lives: views of `boom page`, beside the book.** The bar at
   the head of each page gives the links Book, Rules, Decisions, Terms
   and Claims, as `<nav class="views">`, the current one with
   `aria-current="page"`. The routes are `<base>/rules`,
   `<base>/decisions`, `<base>/terms` and `<base>/claims`. On a view,
   the sidebar of `the-book-page-has-navigation` gives that view's
   levels in place of the book's narratives: Part I the whole system,
   Part II a chapter for each repository and a section for each of its
   contexts, each with its count, as a link to that level on the view.
   The search form stays at the head of the sidebar.

   Options:
   1. Views of `boom page`, beside the book, in its shell
      (recommended). One server, one token, one search, and a claim's
      page links to its book and its views alike.
   2. Part of the book: an appendix of the claims after Part II. The
      book stays one thing to read, but the rules and the decisions
      are other concepts, as the user said, and a long appendix buries
      them.
   3. A command of its own, `boom claims`, with a server of its own.
      It parts the views, but doubles the token, the port and the
      shell, and a link from a claim's box to its view crosses servers.
2. **The rules view is the constitution of each level.** `<base>/rules`
   gives each accepted invariant, then each accepted assumption, of
   each level, in the order of their names. Each gives its kind,
   `<name>@<version>` as a link to its page, and its statement, and
   folds its body in `<details>`: what breaks it, what does not, and
   its source. A claim that binds two repositories is given in the
   chapter of each. `?repository=<name>` gives the levels that `boom
   query --repository <name>` gives an agent: the whole system, the
   repository and each of its contexts, with each claim that `boom
   query` gives of those kinds.

   Options:
   1. Invariants and assumptions, by level, the body folded
      (recommended). It is what an agent keeps and what an agent
      assumes, as the session's hook gives them.
   2. Invariants alone. Assumptions then show only in Claims.
   3. Invariants with their bodies open. A reader scans a level by its
      statements, and the body is one click.
3. **The decisions view is the decision records.** `<base>/decisions`
   gives each accepted decision by level, in the order of the entry
   that accepted its first version, the oldest first, as records are
   numbered. Each gives `<name>@<version>` as a link to its page, its
   statement and the date it was accepted. Its page gives its body: the
   context, the options considered and the outcome. A decision with a
   proposed version newer than the accepted one is marked "a newer
   version is proposed".

   Options:
   1. A list in the order accepted, the body on the claim's page
      (recommended). A record's number is its place in that order, and
      the list stays short.
   2. In the order of names. It is found faster by name, and the search
      finds it as well.
   3. Each body open in the list, as one long document. It reads as the
      `docs/adr/` folder, but scrolls through tens of records.
4. **Terms have a view; contexts are levels.** `<base>/terms` gives each
   accepted term by repository, in the order of its words, its word
   and its meaning, as a link to its page. A term binds one repository,
   so the view has no Part I. The term's page gives its old words, as
   `boom show` gives them to a person. The terms view does not, and
   the page's search does not find a term by an old word.
   A context is a level of each view: a section under its repository,
   headed by its name and its accepted globs. A context's page gives
   its globs and each accepted claim that binds it, by kind. The
   context map, drawn, waits for its own change.

   Options:
   1. A view of terms, and contexts as the levels of each view
      (recommended). A glossary reads apart, and a context is where a
      claim binds.
   2. Terms inside the rules view, as the session's hook gives them.
      The hook's "Rules" holds the terms, but a glossary is read by its
      words, not by a level.
   3. A view of contexts too, with their globs and counts. It is the
      context map without the drawing, which its own change gives.
5. **A claim's page in full.** The page of a claim, from
   `the-book-page-has-navigation`, gives its kind, version, status,
   statement, what it binds, body, versions, sources, the claims it
   changes and the narratives that include it. This change adds:
   - **History.** Each version that is neither proposed nor accepted,
     rejected or superseded, under "History", folded, with its status,
     the entry that closed it, the person and, for a rejection, why.
     The versions line gives the accepted and the proposed versions
     alone.
   - **The claims that name it.** Each claim whose accepted or
     proposed version names it with `--source` or `--changes`, with
     the version meant, marked out of date where it meant another
     version, as `boom show` gives "source of".
   - **Its origin.** The origin's name and fingerprint and the command
     `boom show --artifact <fingerprint>`, or `git:<repository>@<commit>:<path>`
     as it is. The page serves no artifact's bytes: a kept file could
     be a page that runs on the page's own origin.
   - **Its notes.** Each note on a paragraph of a narrative of the book
     that includes the claim at that paragraph, open or answered, with
     the person, the time and a link to the paragraph.
   - **A proposed version.** The page of a proposed version gives it in
     a dashed frame marked "proposed, binds nothing", with
     `boom accept <name>@<version>` and
     `boom reject <name>@<version> --why "<why>"`.
6. **Proposed versions apart from accepted ones.** Each view gives
   first, before its levels, under "Proposed, binding nothing", each
   proposed version of its kinds, grouped by the level that it binds,
   each in a dashed frame marked "proposed" in words, with
   `boom accept <name>@<version>` and, at the head of the list, one
   `boom accept` with every version of it, which one touch of the key
   signs. The sidebar gives each level's count of proposed versions.
   Then the view gives its levels with the accepted versions, plain. No
   view gives a rejected or a superseded version: they show only in a
   claim's history.

   Options:
   1. A list of the proposed versions at the head of each view, grouped
      by level (recommended). The person sees what waits first, and
      where it will bind, and the levels below stay what binds.
   2. One view of everything that waits, across kinds. It is one pile,
      but it parts a proposal from the rules or decisions it joins.
   3. Proposed versions at the head of their own level. A proposal
      sits where it binds, but a level far down the view hides it.
   4. Proposed versions inline, among the accepted ones, marked. A
      reader then reads what binds and what does not as one list.
7. **Filters and search with no script.** Each view gives a
   `<form method="get">` at its head, with `kind`, `status`,
   `repository` and `level` as `<select>` fields, `q` as a search
   field, and a button. `status` is `both`, the default, `accepted` or `proposed`.
   `level` is `system`, a repository's name, or `<repository>/<context>`.
   `repository` gives what `boom query` gives that repository. `q`
   keeps the claims whose words hold each word, with the matching of
   `boom search`, over the claims that the other fields keep, so a
   proposed claim is found where `status` keeps it. A value that is not
   a kind, a status, a repository or a level answers 400 with the
   values the field takes. The view says what it keeps, and how many.
8. **What it keeps.** Each response is computed from the views at the
   log's last entry. The token on each path, the `Host` check, the
   referrer policy and the policy header stay as they are. A view
   holds no `<script>` element and no URL of another host. No new path
   answers a `POST`. Each value of a filter is escaped where the page
   gives it back.

## Risks / Trade-offs

- [Two proposed decisions shape the status] → The design follows
  `only-an-accepted-version-binds-work` and
  `a-waiting-proposal-is-a-card-on-the-rail`, both proposed. If the
  user changes them, the tasks change before the build.
- [Each request computes every level] → The store holds a few hundred
  claims, and boom holds its views in memory for a request, as the
  book does.
- [The sidebar of a view differs from the book's] → The view's levels
  are the book's levels, so a reader moves between them in the same
  places.
- [It waits for `the-book-page-has-navigation`] → Its sidebar, search
  and claim's page are the shell. This change lands after it.
