## Context

A version binds the repositories that its proposal names, or the whole
system (`claim-attachments`). The entry of the proposal holds the
attachment after the claim, as `attach` with `repositories`. The
attachment is no part of the fingerprint. `boom query` finds the
repository of a session and gives each claim whose accepted version
binds that repository or the whole system (`repository-rules`).

The design of Boom says that a context owns paths in any repository of
the drive, that one claim can bind several contexts, and that each
repository is one context until someone draws the map. The user
approved turn 4b on 2026-11-21. Turn 4 (`the-graphs-first-edges`)
stores each link of a version in its entry, with the version that the
link meant (`a-link-remembers-the-version-it-meant`). It lands before
this change. Turn 3 (`boom-keeps-terms`) made the kind `term`, which
binds one repository (`a-term-binds-one-repository`).

This design keeps these claims of the store:

- `status-read-from-events`: a context and a binding have no status of
  their own. Boom reads the accepted version of a context from the log.
- `views-computed-from-the-ledger`: the rows of the globs and the
  bindings are views, which a replay of the log builds again.
- `a-link-remembers-the-version-it-meant`: a binding stores the version
  of the context that it meant, and `boom show` marks it out of date.
- `a-term-binds-one-repository`: a term binds no context.
- `names-use-the-domains-terms`: the words are `context`, `path` and
  `glob`, the words of Claude Code's rules files and of git.
- `an-agent-gets-a-repositorys-rules-from-boom-query-each-claim-attached-to-the-repositories-it-binds`:
  a binding is part of the attachment, and out of the fingerprint.
- `an-agent-finds-claims-by-their-words-searched-in-memory-from-the-log`:
  a search of a repository finds the claims of its contexts, with no
  index.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  the tools take the new arguments, read from the command line.
- `context-either-at-session-start-or-synced-into-the-repository`: the
  globs of a context are the `paths:` line that `boom sync` writes.
- `tools-at-their-pinned-versions`: `globset` is pinned in
  `Cargo.lock`.

## Goals / Non-Goals

**Goals:**

- A context is a claim that a person accepts, with its globs.
- A claim binds the whole system, repositories, contexts, or
  repositories and contexts.
- `boom query --path` gives the claims for a file, from the log, with
  no repository read.
- Each claim that binds a repository today keeps its binding, its
  version and its answer.
- One answer to "which contexts does this file fall in", for the later
  turns to call.

**Non-Goals:**

- The relationships between contexts, the map as a view, and the check
  that code crosses contexts only through a relationship.
- A glob that excludes paths, such as `!crates/sluice/src/overview_test.rs`.
- A check that two contexts own one file.
- `touches` (turn 5), the overlap check (turn 6), the hook before each
  edit (turn 7) and `boom sync` (turn 2b). This change gives them the
  path scope that they call.

## Decisions

### 1. A context is a claim of its own kind

`boom propose context sluice-overview "The overview that a reader of the queue sees." --path sluice:crates/sluice/src/overview*.rs --path sluice:crates/sluice-core/src/overview.rs`.
The name is kebab case, in the one name space of the store. The
statement says what the context holds, in one line. A body is
optional. A context has one glob at least. A person accepts a context
as any other claim, so the map has the same history as the claims.

The globs are part of the content of the version, as the repository
and `replaces` of a term are. A new glob is a new version, which
amends its base, and a person accepts it.

Alternative: a context as a setting of a repository, as `boom repo
set` keeps a remote. Then the map has no version, no proposal and no
accept, and a claim cannot name the version of the context that it
meant.

### 2. A glob names its repository

A glob is `<repository>:<pattern>`. The repository is kebab case, as
an attachment names it. The pattern is relative to the root of the
repository, with `/` between its parts. It has no `/` at its start, no
`..` part and no `\`. `*` matches in one part of the path, `**`
matches any number of parts, `?` matches one character, `[...]`
matches one character of a set, and `{a,b}` matches either. This is
the form that the `paths:` line of a Claude Code rules file reads, so
`boom sync` (turn 2b) writes a glob with no change other than the
repository at its start. `globset` compiles the pattern, with
`literal_separator`, so `*` does not cross a `/`.

Boom keeps each glob one time, sorted. A context
binds the repositories that its globs name. Boom writes them as the
`repositories` of the attachment of the context, and refuses
`--repository`, `--system` and `--context` on a context.

Alternative: one repository for each context. The design of Boom says
that a context owns paths in any repository of the drive, and the
user approved globs that each name a repository.

### 3. A binding is a link in the attachment, with the version it meant

`--context <name>` binds the version to a context. The entry stores
`<name>@<version>` in `attach.contexts`: the accepted version of the
context, or its newest version when none is accepted.
`--context <name>@<version>` stores that version. This is the
resolution that turn 4 gives `--source`. Boom refuses a name that is
not a context of the store, and a version that the context does not
have. `contexts` is sorted by name, and names each context one time.

The binding stays in the attachment, and out of the fingerprint, as a
repository does now (`claim-attachments`). Where a claim binds is not
what it says. So one claim in the same words keeps one version in two
stores with two maps. A version keeps the one attachment of its
proposal, so a person who accepts the version accepts its binding.

Two attachments are the same when they name the same repositories and
the same contexts by name. So the same proposal, given again after a
new version of its context is accepted, adds nothing.

Alternative: the binding as an edge in the content, as `source` is.
Then each claim that binds a context needs a new version whenever its
binding changes, and a claim bound to a repository today has no such
edge. The attachment already holds where a version binds.

### 4. The binding follows the accepted context

`boom query --path` matches the path against the globs of the accepted
version of each context, and not against the version that the binding
meant. So a new version of a context, with a new glob, moves the
claims that bind it with no new version of each claim. A binding to a
context with no accepted version matches no path. `boom show` gives
the version that the binding meant, and marks the binding out of date
where another version of the context is accepted now. A person then
sees which claims were bound to an older map.

Alternative: match the version that the binding meant. Then each
redraw of a context needs a new version of every claim that binds it.

### 5. Each repository is one context until someone draws contexts

Boom makes no context for a repository. A binding to a repository
means the whole repository: every path of it, under each context of it.
So each claim of today keeps its answer for every file. A context
narrows a claim only when the claim binds the context and not the
repository. A claim moves into a context only by an amendment that
binds the context and not the repository, which a person accepts.
Boom never moves a claim when a context is drawn.

### 6. The claims for a path

`boom query --path sluice:crates/sluice/src/overview.rs` gives each
claim whose accepted version binds one of these:

- the whole system
- `sluice`
- a context whose accepted version has a glob that matches the path

`--path` can come more than one time, and each path names one
repository, the same for each. The answer gives each claim one time.
`--path` names its repository, so boom refuses it with `--repository`.
Boom also refuses it with `--if-declared`. Turn 7 decides how its hook
asks.

Boom compiles the accepted globs of the repository when asked, as
`boom search` reads its claims when asked. It keeps no index of the
paths (core.2). Boom matches the text of the path, so a file that does
not exist yet, such as a file that a plan names, matches. Boom refuses
a path with a `/` at its start, a `..` part or a `\`, or a path that
names no repository.

The JSON is `boom-query/2`, with one more field, `paths`: for each
path, the contexts that it falls in, as `<name>@<version>`. Each claim
also gives `contexts`, the contexts that it binds. These are additions,
so the major version stays. The words give one line for each path
after the first line, and the claims as now.

`boom query --repository sluice` gives what it gives today, and the
accepted contexts that bind `sluice`. It does not give a claim that
binds only contexts. An agent at the start of a session gets the
claims of the repository and the map of its contexts, and gets the
claims of a context when it asks for a path. `boom search --repository
sluice` also searches the claims bound to the accepted contexts of
`sluice`, because a search finds a claim on demand.

### 7. One answer for the later turns

The library has one function: the contexts that `<repository>:<path>`
falls in, at the last entry of the log. Each later turn calls it, and
none matches a glob of its own.

- **Turn 5, `touches`.** Reconcile stores a `touches` link as
  `<repository>:<path>`, the form of `--path`. A path link names a
  file, never a context, so a redraw of the map never changes a link.
- **Turn 6, the overlap between plan units.** Two units overlap when
  they touch one file, or when their paths fall in one context. The
  check calls the function for each path of each unit and compares
  the sets. A path of a repository with no context drawn falls in the
  context of its repository.
- **Turn 7, the claims for a file before each edit.** The hook finds
  the repository of the session and the path of the file from the root
  of its worktree, as `boom terms check` finds the repository of a
  file. It then asks `boom query --path`.
- **Turn 2b, `boom sync`.** For each context with a glob in the
  repository, sync writes a rules file. The file gives the claims of
  the context and a `paths:` line with its globs, each with no
  repository at its start.

### 8. A term binds a repository, never a context

`a-term-binds-one-repository` holds. Boom refuses `--context` on a
term. A context can own paths in two repositories, so a term that
binds a context could bind two.

## Risks / Trade-offs

- [Two contexts own one file] → `boom query --path` gives the claims of
  both. Nothing refuses it in this change. The map as a view can show
  it later.
- [A glob that matches nothing] → Boom reads no repository, so it
  cannot tell. The context still holds its other globs.
- [An older boom on one host] → An older boom reads the proposal of a
  context as an entry that fails its checks, and counts it for nothing.
  Install this release on each host before anyone proposes a context
  in the real store.
- [A claim bound to a context only, at the start of a session] →
  `boom query --repository` does not give it. The agent gets it when it
  asks for the path, and turn 7 asks before each edit. Until turn 7, a
  claim that must reach every session of a repository binds the
  repository.
- [Requirements that turns 3 and 4 also modify] → This change takes
  their text. Archive `boom-keeps-terms` and `the-graphs-first-edges`
  first.
