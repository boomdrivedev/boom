## Why

Today a claim binds the whole system or a repository. An agent that
edits one file of sluice gets every claim of sluice. A claim about the
overview of sluice binds the whole repository, and a claim that holds
for the overview of sluice and the status of flume binds two whole
repositories. The design of Boom says that the paths that a change
touches name the contexts that they fall in, and that `boom query`
gives the claims of those contexts. The user approved this turn, turn
4b, on 2026-11-21: a context owns path globs, and each glob names its
repository. A claim binds the whole system, a repository or a context.
`boom query --path` gives the claims for one file. Until someone draws
contexts, each repository is one context, and nothing moves until
someone proposes the move. A tool names things with the terms of its
domain (`names-use-the-domains-terms`).

## What Changes

- **A new claim kind, `context`.** A context owns path globs. Each
  glob names its repository: `sluice:crates/sluice/src/overview*.rs`.
  `boom propose context <name> <statement> --path <repository>:<glob>`
  proposes a context, and a proposal can give `--path` more than one
  time. The globs are part of the content of the version, so a person
  signs them. A context binds the repositories that its globs name.
- **A claim binds a context.** `boom propose ... --context <name>`
  binds the version to a context. The entry of the proposal stores the
  binding with the version of the context that it meant: the accepted
  version, or the newest version when none is accepted. `--context
  <name>@<version>` stores that version. A claim can bind several
  contexts, and repositories too. The binding is part of the
  attachment, as a repository is now. A term cannot bind a context.
- **`boom query --path <repository>:<path>`** gives the claims for a
  file: the claims of the whole system, the claims of its repository,
  and the claims bound to each context whose accepted globs match the
  path. A query can give `--path` more than one time, for one
  repository. The answer names the contexts that each path falls in.
  Boom matches the text of the path, and reads no repository.
- **Each repository is one context until someone draws contexts.** A
  claim that binds a repository binds every file of it. Drawing a
  context moves no claim. A claim moves into a context only when an
  agent proposes an amendment that binds the context, and a person
  accepts it.
- `boom query --repository` gives the claims of the repository and of
  the whole system, as now, and the contexts that bind the repository.
  It does not give a claim that binds only contexts.
- `boom search --repository` also searches the claims bound to the
  contexts of that repository.
- `boom show <name>@<version>` gives the contexts that the version
  binds, each with the version that the binding meant. It marks a
  binding out of date where another version of the context is accepted
  now. `boom show <context>` gives its globs, and the claims that bind
  it.
- The MCP tool `propose` takes `paths` and `contexts`. The tool
  `query` takes `paths`.

## Capabilities

### New Capabilities

- `context-claims`: the kind `context`, its globs, a binding to a
  context, the claims for a path, a search of the contexts of a
  repository, and how `boom show` gives a context and a binding.

### Modified Capabilities

- `claim-versions`: the kind `context`, and the globs of a context in
  the content of the fingerprint.
- `claim-attachments`: an attachment holds contexts as well as
  repositories. An amendment keeps the contexts of its base. A context
  binds the repositories of its globs.
- `repository-rules`: the claims that `boom query` gives, `contexts`
  in each claim, and the contexts after the terms.
- `command-line`: `boom propose` takes the kind `context`.
- `agent-tools`: the tool `propose` takes `paths` and `contexts`, and
  the tool `query` takes `paths`.

## Impact

- `crates/boom`: `claim.rs` (the kind and its globs), `entry.rs` (the
  contexts of an attachment), `judge.rs` (the checks of a glob and of a
  binding), `store.rs` (a view row for each glob and each binding),
  `cli.rs` (`propose`, `query` and `show`), `search.rs`, `mcp.rs`, and
  their tests. A new dependency, `globset`, at a pinned version,
  compiles the globs.
- `boom-propose/2`, `boom-query/2` and `boom-show/2` get the fields
  `paths` and `contexts`. These are additions, so the major versions
  stay.
- Each version that is in a store now keeps its fingerprint and its
  attachment.
- An older boom reads the proposal of a context as an entry that fails
  its checks. Install this release on each host before anyone proposes
  a context in the real store.
- This change lands after `the-graphs-first-edges`. It modifies six
  requirements that `boom-keeps-terms` or `the-graphs-first-edges`
  also modifies: the kinds and the fingerprint of `claim-versions`, the
  answer and its JSON in `repository-rules`, the exit codes of
  `command-line`, and the tools of `agent-tools`. It
  takes their text. Archive both of them first.
- Later turns use the path scope. Turn 5 stores a `touches` link as
  `<repository>:<path>`. Turn 6 finds the overlap between plan units
  from the contexts of their paths. Turn 7 gives the claims for a file
  before each edit. Turn 2b writes the globs of a context as the
  `paths:` line of a rules file. The design says how each one uses this
  change.
