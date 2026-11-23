## Why

Boom keeps narratives, but it computes no view of them. `boom page`
with no name lists the narratives in groups by what they bind, each
group in the order of the names. No record gives an order of the
author's, and nothing names a narrative that no view holds. On
2026-11-23 the store held four narratives about the drive. Three of
them bind `boom` and explain the whole system. The user found them "so
random, not thought through in terms of design book organization".

The design agent then proposed the narrative `the-design-book`, an
overview that names its chapters in its text. Boom refuses a narrative
that includes a narrative, so the overview cannot hold its chapters as
edges.

The user ruled on 2026-11-23:

1. "Organize the book by Boom's three binding levels: system,
   repository, context."
2. "I'm not sure how I feel about narratives knowing they are part of a
   design book. That makes the narratives less useful for reuse with
   context maps, etc." So a narrative stands alone. It binds one level,
   it includes claims only, and it names no book, part or chapter.
3. "A Boom change: Boom computes views." The design book is the first
   view. A context map is another view over the same narratives.

## What Changes

- **The kind `outline`.** `boom propose outline <name> "<title>"
  --view book --narrative <name> ...` proposes the outline of a view:
  its title, the view that it computes, and the narratives that it
  holds, in order, each at the version that it meant. An outline binds
  the whole system. A person accepts it with the key, as each claim,
  and no one checks it. No hook gives it.
- **The edge `narratives`.** `--narrative <name>[@<version>]` names a
  narrative that the outline holds. Boom resolves it as `--source`.
  Boom refuses a name that no claim has, a claim that is not a
  narrative, and a narrative that the outline already holds.
- **One design book in a store.** The view `book` is the design book.
  Boom refuses an outline of the view `book` while another outline of
  that view waits or stands.
- **A narrative binds one level.** Boom refuses a narrative that binds
  more than one of these: the whole system, a repository, a context. A
  narrative names no book, part or chapter, and the outline holds it
  by name. The same narrative can be in many views.
- **Boom computes the design book.** The book has two parts. Part I is
  the system: the narratives that bind the whole system, then a
  section for each context whose globs name two repositories or more.
  Part II has a chapter for each repository: the narratives that bind
  the repository, then a section for each context in that repository.
  Inside a level, the outline sets the order. A narrative that binds a
  level and that the outline does not hold comes at the end of its
  level, marked "not in the outline".
- **`boom show <outline>`** gives the narratives of the outline in
  order, each marked out of date as an edge is, the book as Boom
  computes it, and the narratives that the outline does not hold.
- **`boom page` gives the design book.** With no name, the page gives
  the book: its parts, chapters and sections, each narrative with its
  title, `<name>@<version>`, status and open notes, as a link to its
  page. `boom page --view <outline>` gives the view of that outline.
  The page of a narrative in the book links to the narrative before it
  and the narrative after it. The page that lists the narratives by
  binding goes.
- **`boom mcp`** gives the kind `outline`, `view` and `narratives` to
  the tool `propose`.
- **The context map is the next view.** Its outline holds narratives
  that bind contexts, and it reuses them with an order of its own. A
  later change builds it. This change refuses each view but `book`.
- After this change is released, an agent proposes the outline
  `boomdrive-design-book` from the order that the narrative
  `the-design-book@44f86caafa73` gives. A person rejects that
  narrative and accepts the outline.

## Capabilities

### New Capabilities

- `views`: the kind `outline`, the edge `narratives`, the one design
  book of a store, the parts and order of the book, the narratives
  that the outline does not hold, and what `boom show` gives of an
  outline.

### Modified Capabilities

- `narratives`: a narrative binds one level.
- `narrative-page`: `boom page` with no name gives the design book,
  `--view` gives the view of an outline, and the page of a narrative
  links to its neighbors in the book. The list of the narratives by
  binding goes.
- `claim-versions`: a seventh kind, `outline`.
- `claim-edges`: a version stores the edge `narratives`.
- `agent-tools`: the tool `propose` takes the kind `outline`, `view`
  and `narratives`.

## Impact

- `crates/boom`: `claim.rs` (the kind, `view`, `narratives`),
  `judge.rs` (the refusals of an outline, the one book, and one level
  for a narrative), a new module `view.rs` (the book, computed from
  the views of the store), `page.rs` (the book's page, `--view` and the
  links of a narrative), `cli.rs`, `hook.rs` (no hook gives an
  outline), `mcp.rs`, and their tests.
- The store: the new kind is an entry of the same log. An older boom
  counts an outline's entry for nothing. The store of 2026-11-23 holds
  no narrative that binds two levels.
- No new dependency, no new part of the repository, and no new moon
  project. `boom:test` checks the new code.
- The proposed decision `the-design-book-follows-the-three-binding-levels`
  says that overviews include their sections. Under ruling 2, it needs
  a new version that names the outline. That claim is proposed apart
  from this change.
