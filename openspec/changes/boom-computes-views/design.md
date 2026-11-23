## Context

Boom keeps six kinds of claim: invariant, decision, assumption, term,
context and narrative (`claim-versions`). A narrative is a title and a
body of Markdown, and it includes claims of other kinds, each at the
version that it meant (`narratives`). Boom refuses a narrative that
includes a narrative (`crates/boom/src/judge.rs`, the check of the
edges). A claim binds the whole system, repositories or contexts
(`claim-attachments`, `context-claims`). A context binds the
repositories that its globs name.

`boom page` with no name lists each narrative under what it binds,
each group in the order of the names (`narrative-page`, "The list of
narratives"). No record holds an order of the author's. Nothing names
a narrative that no view holds.

On 2026-11-23 the store held five narratives: `boom-end-to-end`,
`change-records` and `drive-architecture-model` bind `boom`,
`the-merge-queues-flow` binds `sluice`, and `the-design-book` binds the
whole system. `the-design-book@44f86caafa73` is an overview of the
book. It names its chapters and the narratives of each chapter in its
text, because Boom refuses it the narratives as edges.

The user ruled on 2026-11-23:

1. "Organize the book by Boom's three binding levels: system,
   repository, context."
2. "I'm not sure how I feel about narratives knowing they are part of a
   design book. That makes the narratives less useful for reuse with
   context maps, etc." A narrative stands alone: it binds one level,
   includes claims only, and names no book, part or chapter.
3. "A Boom change: Boom computes views." The design book is the first
   view. A context map is another view over the same narratives.

The user approved this design on 2026-11-23, with its five choices:
the outline is a kind of claim (decision 1), the levels of a view come
from the view (decision 1), contexts sit in the chapter of their
repository (decision 4), faults are refused at the proposal and marked
in the view (decision 5), and the book's page links to each narrative
(decision 7).

This design keeps these claims and rules:

- `views-computed-from-the-ledger` (core.2): the book is computed from
  the log for each request. No file and no commit holds it. "A rendered
  constitution, design book or context map that a commit holds" breaks
  the rule, so the outline holds an order and Boom computes the rest.
- `status-read-from-events` and `events-are-appended-never-changed`
  (core.1): an outline's status comes from the log. A new order is a
  new version, which amends its base.
- `a-link-remembers-the-version-it-meant`: the edge `narratives` stores
  the version of each narrative that it meant.
- `claims-cite-claims-by-name`: an outline names narratives of the
  store only.
- `names-use-the-domains-terms`: `view`, `outline`, `book`, `part`,
  `chapter` and `section` are the words of a book. `level` is Boom's
  word for what a claim binds. The words "lineage" and "link" keep
  their meaning: a link is one edge, and this change adds no lineage.
- `a-narratives-open-question-is-a-proposed-decision-included-at-its-paragraph`
  (proposed): an outline holds no open question. Each question of this
  design is a decision below, with its options and a recommendation.
- `the-design-book-follows-the-three-binding-levels` (proposed): its
  parts by binding level, one level for each narrative, one owner for
  each topic, and the narratives that no view holds come from it.
  Its overviews that include their sections do not, by ruling 2
  (decision 9).
- `the-drive-keeps-one-log-and-each-tool-owns-its-entry-types`
  (proposed): an outline is an entry type that Boom owns, in the one
  log.
- `no-credential-in-the-repository` (core.3) and `nothing-commercial`
  (core.4): the book is served on `127.0.0.1` as the page of a
  narrative is, and this change names no service of another host.
- `each-part-is-a-moon-project` and `one-task-per-check` (core.5): the
  change is Rust code in the crate `boom`, with no file of a new type,
  so it adds no moon project.

## Goals / Non-Goals

**Goals:**

- A view is computed from the log: the parts from the binding levels,
  the order from one record that a person accepts.
- A narrative knows no view. The same narrative can be in the book and
  in a later context map.
- Boom names each narrative that the book's outline does not hold, and
  refuses an outline that holds a narrative twice or names what does
  not resolve.
- `boom page` gives the design book first.

**Non-Goals:**

- The context map. Decision 6 says what it shows. A later change
  builds it.
- The whole book on one page, with each body. The book gives each
  narrative as an entry that links to its page.
- A rules page, decision records, `boom render` and `boom check`.
- An outline for each repository. The store has one design book, for
  the whole drive.
- The new versions of the narratives and of the decision
  `the-design-book-follows-the-three-binding-levels`. Agents propose
  them apart from this change.

## Decisions

### 1. The outline of a view is a claim of the kind `outline`

`boom propose outline <name> "<title>" --view book --narrative <name>
...` proposes an outline. A version holds these:

- **The name**, in kebab case, in the one name space of the store.
- **The title**, as the statement: one line, such as "The design book
  of Boomdrive".
- **The view**, `--view`, required: the view that the outline computes.
  This change has one view, `book`. Boom refuses another view, and
  names `book`.
- **The narratives**, `--narrative`, one at least, in the order given
  (decision 2).
- **A body**, `--body`, optional: Markdown that the page gives under the
  title of the view, as an introduction. It names no narrative's
  place: the place comes from the binding and the order.
- **Its sources and origins**, `--source` and `--origin`, as for each
  claim. `--changes` and `--includes` stay a decision's and a
  narrative's.

An outline binds the whole system. Boom attaches it to the whole
system wherever it runs, and refuses `--repository` and `--context` on
it. `--amends <version>` proposes the next version, and the fingerprint
is the fingerprint of all of the above.

A person accepts or rejects an outline with `boom accept` or
`boom reject` and a key. One signature takes an outline with the
narratives that it holds, as it takes a narrative with its includes.
No one checks an outline. No hook gives it, and session start gives no
line for it.

The levels that a view covers come from the view, not from the
outline: the book covers all three, and the context map covers the
contexts. A version therefore holds no list of levels.

Decided on 2026-11-23: the outline is a claim of the kind `outline`,
not a record of its own. Versions, `--amends`, the accept with the key,
`boom show`, `boom search`, `boom verify` and the one log work with no
new code, and a new order is a version that a person accepts. The
levels that a view covers come from the view.

### 2. The edge `narratives`: the narratives in order, at the version meant

`--narrative <name>` and `--narrative <name>@<version>` name a
narrative that the outline holds. Boom resolves each as `--source`:
`<name>` stores the accepted version, or the newest version where none
is accepted. The entry stores the list in the order given, and the
order is part of the version's fingerprint.

Boom refuses these, and appends nothing:

- a name that no claim has, or a version that its claim does not have
  (the name does not resolve);
- a claim that is not a narrative;
- a narrative given two times, with one version or with two;
- `--narrative` on a claim that is not an outline.

The book gives each narrative at its accepted version, or at its newest
where none is accepted, and not at the version that the edge meant. A
new version of a narrative thus needs no new outline. `boom show
<outline>` marks an edge out of date when its narrative has another
accepted version, as for each edge (`claim-edges`).

The same narrative can be in the outlines of many views. Ruling 3 asks
for this: the context map reuses the narratives of the book.

### 3. A narrative binds one level

A narrative binds exactly one of these: the whole system, one
repository, or one context. Boom refuses a narrative whose attachment
names two repositories, two contexts, or a repository and a context,
and says that a narrative binds one level. A narrative that explains
two levels is two narratives.

The narratives of the store of 2026-11-23 each bind one level, so the
log needs no new entry.

### 4. The design book

The view `book` gives the design book of the store. The store holds one
book. Boom refuses the first version of an outline of the view `book`
while another outline of that view has a version that is not rejected,
and names that outline.

Boom computes the book from the outline's accepted version, or its
newest version where none is accepted. With no outline of the view
`book`, Boom computes the book with no order of the author's.

A narrative's level is what its shown version binds: its accepted
version, or its newest version where none is accepted. A context's
repositories are the repositories that its accepted version binds, or
its newest version where none is accepted.

The book has two parts:

1. **Part I. The system.** First, the narratives that bind the whole
   system. Then a section for each context whose repositories are two or
   more, with the narratives that bind it.
2. **Part II. The repositories.** A chapter for each repository that a
   narrative binds, directly or through a context. A chapter gives the
   narratives that bind the repository, then a section for each context
   of that repository alone, with the narratives that bind it.

A repository or a context that no narrative binds has no chapter or
section.

The order:

- Inside a level, the narratives that the outline holds come first, in
  the outline's order. Then come the narratives that the outline does
  not hold, by name, each marked "not in the outline".
- The chapters of Part II come in the order of the outline's first
  narrative in each chapter or in one of its sections. A chapter with
  no narrative of the outline comes after, by name.
- The sections of a part or a chapter come in the same way: by the
  outline's first narrative in each, then by name.

A narrative that the outline holds and whose shown version is rejected
shows at its place with its status. A narrative whose every version is
rejected and that the outline does not hold is not in the book.

Decided on 2026-11-23: a context is a section inside the chapter of
its repository, and a context of two repositories or more is a section
of Part I. A reader finds a context where its code is. This follows
points 1 and 7 of `the-design-book-follows-the-three-binding-levels`.

### 5. The checks of a view

Boom names each fault of a view in one of two places:

- **At the proposal, a refusal.** An outline that holds a narrative two
  times, a name that does not resolve, a claim that is not a narrative,
  a view other than `book`, and a second book (decisions 1, 2 and 4).
  Boom appends nothing.
- **In the view, a mark.** A narrative that binds a level of the book
  and that the outline does not hold. The book gives it at the end of
  its level, marked "not in the outline". `boom show <outline>` names
  it under `not held`, and `--json` gives `unheld`.

A claim never goes from the store, so a name that resolved at the
proposal resolves on each later read.

Decided on 2026-11-23: refusals at the proposal and marks in the
view, with no command of its own that checks a view. A fault that the
author can fix at once is refused. A fault that comes from another
claim's later version is shown where a reader sees it.

### 6. The context map, the next view

A context map is a view of the kind that the book is, with an outline
of its own: `--view context-map`. It covers the contexts. It gives each
context with its globs and the repositories that they name, grouped by
repository, with a context of two repositories apart. Under each
context, it gives the claims bound to it, as counts by kind, and the
narratives that its outline holds for that context, in its order. The
narratives are the narratives of the book. Neither view writes to
them.

A later change builds it. Until then, Boom refuses `--view context-map`,
and the version's form already holds the view.

### 7. `boom page` gives the design book

- `boom page` with no name serves the design book, and prints
  `Boom: page of the design book at <URL>`.
- `boom page --view <outline>` serves the view of that outline, and
  prints `Boom: page of <outline>@<version> at <URL>`. Boom refuses a
  name that is not an outline, and refuses `--view` with the name of a
  narrative.
- `boom page <name>[@<version>]` serves a narrative as before.

The page of the book gives the title of the outline, its body, and the
outline's `<name>@<version>` and status. It then gives each part,
chapter and section as a heading. Under each heading, it gives each
narrative with its title, `<name>@<version>`, status and count of open
notes, as a link to its page. A narrative that the outline does not
hold has the mark "not in the outline". With no outline, the page says
how to propose one.

The page of a narrative that the book gives links to the book, and to
the narrative before it and the narrative after it in the book's
order. The order runs through Part I, then each chapter of Part II.

The page stays as `narrative-page` sets it: computed from the log for
each request, on `127.0.0.1` under a token, with its own style.

```
Boom · the design book                                          entry <n>
──────────────────────────────────────────────────────────────────────────
The design book of Boomdrive
outline boomdrive-design-book@<version> · proposed · 4 narratives

Part I. The system
  (no narrative binds the whole system yet)

Part II. The repositories
  boom
    The drive's architecture for continuous delivery: ...
      drive-architecture-model@92ee5a1452ac · proposed · <n> open notes
    Boom, end to end
      boom-end-to-end@db46c78278b9 · proposed · <n> open notes
    Boom's change record, and the work a new version of a claim makes stale
      change-records@dc091e19672c · proposed · <n> open notes
  sluice
    Where the merge queue's lead time went on deneb, ...
      the-merge-queues-flow@ee86d9063097 · proposed · <n> open notes
```

When the next version of `drive-architecture-model` binds the whole
system, it moves to Part I with no new outline.

Decided on 2026-11-23: the book is a page of entries, each a link to
the narrative's page, and a narrative's page links to its neighbors. A
note stays on the narrative's page, where its paragraph is.

### 8. `boom mcp`

The tool `propose` takes the kind `outline`, `view`, a string, and
`narratives`, an array of strings. The tool `show` gives an outline as
`boom show` does. No tool serves a page.

### 9. `the-design-book` moves into the book's outline

The narrative `the-design-book@44f86caafa73` is an overview that
names a book, parts and chapters, which ruling 2 rejects. Its content
goes three ways:

- **The order.** Its Part I and Part II name narratives in an order. An
  agent proposes the outline `boomdrive-design-book`, "The design book
  of Boomdrive", `--view book`, with each narrative of that order that
  the store holds: `drive-architecture-model`, `boom-end-to-end`,
  `change-records` and `the-merge-queues-flow`. A narrative
  that the text names and that no claim has yet, such as
  `what-boomdrive-delivers`, joins the outline by an amendment when it
  is proposed.
- **The parts.** "A part for each level" is now what Boom computes, and
  no record says it.
- **What each chapter will cover.** This is the brief of each narrative
  to come. Each one binds its level and names no chapter.

The outline takes a new name, because the narrative holds the name
`the-design-book` in the one name space (`claim-versions`, "One name
space"). A person rejects `the-design-book@44f86caafa73` with the key,
and accepts the outline. The log keeps the narrative's words.

The decision `the-design-book-follows-the-three-binding-levels` says
that overviews include their sections, and its consequences say that
Boom cannot compute the book. Its next version names the outline. An
agent proposes it apart from this change.

Options:

1. A new name for the outline, and the narrative rejected (recommended).
   No rule of the name space changes, and the log keeps the overview's
   words.
2. Let an outline take a name whose every version is rejected. This is
   a change to "One name space" for one record.
3. Keep `the-design-book` as a narrative that the outline holds first.
   It names a book and its chapters, which ruling 2 rejects.

## Risks / Trade-offs

- An outline holds names, and a narrative can change its level in a new
  version. The narrative then moves in the book with no new outline. →
  The book follows the shown version, and `boom show <outline>` marks
  the edge out of date.
- A store with no outline gives a book with every narrative marked "not
  in the outline". → The page says how to propose the outline.
- One book in each store. → A store is one drive, and the book is the
  drive's. A view of one repository is a later view, with an outline of
  its own.
- An older boom counts an outline's entry for nothing. → A store moves
  to the new boom before an agent proposes an outline, as for each new
  kind.

## Migration Plan

No code moves a record. After this change is released, an agent
proposes the outline (decision 9), and a person rejects the narrative
`the-design-book@44f86caafa73` and accepts the outline. The page with
no name gives the book from then on.

## Open Questions

None for this change. The options of decision 9 wait for the outline,
which an agent proposes after the release.
