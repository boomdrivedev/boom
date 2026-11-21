# Spec Delta

## Purpose

What a context is: a claim that owns path globs, each glob in one
repository. A claim binds a context, and the entry of the claim keeps
the version of the context that it meant. `boom query --path` gives the
claims for a file from the globs of the accepted contexts. Each
repository is one context until someone draws contexts
(`names-use-the-domains-terms`, `a-link-remembers-the-version-it-meant`,
`a-term-binds-one-repository`, `views-computed-from-the-ledger`).

## ADDED Requirements

### Requirement: A context owns path globs

A claim of the kind `context` SHALL have one glob at least. Each glob SHALL be `<repository>:<pattern>`, with a repository name in kebab case. The pattern SHALL be relative to the root of the repository, with `/` between its parts. Boom SHALL refuse a pattern with a `/` at its start, a `..` part, a `\`, or a form that `globset` does not compile. Boom SHALL keep each glob one time, sorted.

#### Scenario: A context proposed

- **WHEN** `boom propose context sluice-overview "The overview that a reader of the queue sees." --path sluice:crates/sluice/src/overview*.rs --provenance run-1` runs
- **THEN** boom exits 0, and `boom show sluice-overview --json` gives a context with one proposed version and the glob `sluice:crates/sluice/src/overview*.rs`

#### Scenario: A context with no glob

- **WHEN** an agent proposes a context with no `--path`
- **THEN** boom exits 2 and says that a context owns one glob at least, and boom appends nothing

#### Scenario: A glob with no repository

- **WHEN** an agent proposes a context with `--path crates/sluice/src/overview.rs`
- **THEN** boom exits 2 and says that a glob is `<repository>:<pattern>`, and boom appends nothing

#### Scenario: A glob out of the tree

- **WHEN** an agent proposes a context with `--path sluice:../flume/src/status.rs`
- **THEN** boom exits 2 and says that a pattern is relative, with no `..` part, and boom appends nothing

#### Scenario: The same globs in another order

- **WHEN** an agent proposes a context with `--path flume:src/status.rs --path sluice:src/a.rs`, and then with the two globs in the other order
- **THEN** both runs give the same version

### Requirement: A context binds the repositories of its globs

The attachment of a context version SHALL hold the repositories that its globs name, sorted, each one time. Boom SHALL refuse `--repository`, `--system` and `--context` on a context. An amendment of a context SHALL bind the repositories of its own globs.

#### Scenario: A context in two repositories

- **WHEN** an agent proposes a context with `--path sluice:crates/sluice/src/overview.rs --path flume:src/status.rs`
- **THEN** the entry of the proposal holds `attach` with `repositories` `["flume", "sluice"]`

#### Scenario: A context with a repository named

- **WHEN** an agent proposes a context with `--path sluice:src/a.rs --repository sluice`
- **THEN** boom exits 2 and says that a context binds the repositories of its globs, and boom appends nothing

### Requirement: A claim binds a context

`--context <name>` SHALL bind a version to the context of that name. The entry SHALL store `<name>@<version>` in the `contexts` of its attachment: the accepted version of the context, or its newest version when none is accepted. `--context <name>@<version>` SHALL store that version. Boom SHALL refuse a name that is not a context of the store, and a version that the context does not have. Boom SHALL refuse `--context` on a term.

#### Scenario: Bound to a context

- **WHEN** `sluice-overview@<v1>` is accepted, and `boom propose invariant the-overview-names-each-branch "<statement>" --context sluice-overview --provenance run-1` runs
- **THEN** boom exits 0, and the entry holds `attach` with `contexts` `["sluice-overview@<v1>"]` and no `repositories`

#### Scenario: Bound to a context and a repository

- **WHEN** an agent proposes a claim with `--context sluice-overview --repository flume`
- **THEN** the entry holds `attach` with `repositories` `["flume"]` and `contexts` `["sluice-overview@<v1>"]`

#### Scenario: A name that is not a context

- **WHEN** an agent proposes a claim with `--context money-has-currency`, and `money-has-currency` is an invariant
- **THEN** boom exits 2 and says that `money-has-currency` is not a context, and boom appends nothing

#### Scenario: A term bound to a context

- **WHEN** an agent proposes a term with `--context sluice-overview`
- **THEN** boom exits 2 and says that a term binds one repository, and boom appends nothing

### Requirement: The same binding by name

Two attachments SHALL be the same when they name the same repositories and the same contexts by name. A proposal of the same content with the same attachment SHALL add nothing, and SHALL answer as the first did. This SHALL hold also after a person accepts another version of a context.

#### Scenario: Proposed again after the context changed

- **WHEN** a version bound to `sluice-overview@<v1>` is proposed, a person accepts `sluice-overview@<v2>`, and the same proposal with `--context sluice-overview` runs again
- **THEN** boom answers as the first proposal did, and the log holds one entry for the version

### Requirement: The claims for a path

`boom query --path <repository>:<path>` SHALL give each claim whose accepted version binds one of these: the whole system, the repository of the path, or a context with a match. A context has a match when a glob of its accepted version matches the path. Boom SHALL match the text of the path and SHALL NOT read a repository. A binding to a context with no accepted version SHALL match no path. Without `--path`, `boom query` SHALL NOT give a claim that binds only contexts.

#### Scenario: A file in a context

- **WHEN** `sluice-overview` is accepted with the glob `sluice:crates/sluice/src/overview*.rs`, the invariant `the-overview-names-each-branch` is accepted bound to `sluice-overview`, and `boom query --path sluice:crates/sluice/src/overview.rs --json` runs
- **THEN** the answer gives `the-overview-names-each-branch`, the claims of `sluice`, the claims of the whole system, and `sluice-overview`

#### Scenario: A file in no context

- **WHEN** `boom query --path sluice:crates/sluice/src/submit.rs --json` runs in that store
- **THEN** the answer gives the claims of `sluice` and of the whole system, and not `the-overview-names-each-branch`

#### Scenario: A glob does not cross a folder

- **WHEN** `boom query --path sluice:crates/sluice/src/overview/watch.rs --json` runs in that store
- **THEN** the answer does not give `the-overview-names-each-branch`

#### Scenario: A file that does not exist

- **WHEN** `boom query --path sluice:crates/sluice/src/overview_new.rs --json` runs, and no such file exists
- **THEN** the answer gives `the-overview-names-each-branch`

#### Scenario: A repository with no context

- **WHEN** no context names `flume`, and `boom query --path flume:src/status.rs --json` runs
- **THEN** the answer gives the same claims as `boom query --repository flume --json`

#### Scenario: Only the repository

- **WHEN** `boom query --repository sluice --json` runs in the first store
- **THEN** the answer gives `sluice-overview`, and does not give `the-overview-names-each-branch`

#### Scenario: A context that waits

- **WHEN** a claim is accepted bound to `sluice-overview@<v1>`, no version of `sluice-overview` is accepted, and `boom query --path sluice:crates/sluice/src/overview.rs --json` runs
- **THEN** the answer does not give the claim

### Requirement: The globs of the accepted context

`boom query --path` SHALL match the globs of the accepted version of each context, and not the globs of the version that a binding meant. A new accepted version of a context SHALL change the paths of each claim that binds it, with no new version of that claim.

#### Scenario: A context with a new glob

- **WHEN** a claim is bound to `sluice-overview@<v1>`, a person accepts `sluice-overview@<v2>` with the new glob `sluice:crates/sluice-core/src/overview.rs`, and `boom query --path sluice:crates/sluice-core/src/overview.rs --json` runs
- **THEN** the answer gives the claim

### Requirement: The paths of one query

`--path` MAY come more than one time. Each path SHALL name the same repository, and boom SHALL answer for that repository. The answer SHALL give each claim one time. Boom SHALL refuse two paths of two repositories. Boom SHALL refuse a path that names no repository, or that has a `/` at its start, a `..` part or a `\`. Boom SHALL refuse `--path` with `--repository` and with `--if-declared`.

#### Scenario: Two paths of one repository

- **WHEN** `boom query --path sluice:crates/sluice/src/overview.rs --path sluice:crates/sluice/src/submit.rs --json` runs
- **THEN** the answer gives each claim one time

#### Scenario: Paths of two repositories

- **WHEN** `boom query --path sluice:src/a.rs --path flume:src/b.rs` runs
- **THEN** boom exits 2 and says that the paths of one query name one repository

#### Scenario: Named by a path

- **WHEN** `boom query --path ledger:src/post.rs --json` runs in a worktree of `sluice`
- **THEN** the answer gives `repository` `ledger`

#### Scenario: A path, when declared

- **WHEN** `boom query --path sluice:src/a.rs --if-declared` runs
- **THEN** boom exits 2 and says that the two flags cannot be given together

#### Scenario: A path and a repository

- **WHEN** `boom query --path sluice:src/a.rs --repository sluice` runs
- **THEN** boom exits 2 and says that a path names its repository

### Requirement: The contexts of each path in the answer

With `--path`, the JSON SHALL give `paths`: each path, in the order given, with the contexts that it falls in, as `<name>@<version>` of their accepted versions, sorted. Each claim in the answer SHALL give `contexts`, the bindings of its accepted version. In words, boom SHALL give one line for each path after the first line: the path and its contexts, or `no context` where it falls in none.

#### Scenario: The contexts of a path

- **WHEN** `boom query --path sluice:crates/sluice/src/overview.rs --json` runs in the first store
- **THEN** `paths` gives that path with the contexts `["sluice-overview@<v1>"]`, and `the-overview-names-each-branch` gives `contexts` `["sluice-overview@<v1>"]`

#### Scenario: The contexts in words

- **WHEN** `boom query --path sluice:crates/sluice/src/submit.rs` runs in the first store
- **THEN** the second line is `path sluice:crates/sluice/src/submit.rs: no context`

#### Scenario: The same answer after the views are built again

- **WHEN** `boom query --path sluice:crates/sluice/src/overview.rs --json` runs two times, and `boom verify` drops and builds the views between the two runs
- **THEN** both runs print the same bytes

### Requirement: A search of a repository finds the claims of its contexts

`boom search --repository <name>` SHALL also search each claim whose accepted version binds a context with a match in that repository. A context has a match there when a glob of its accepted version names that repository.

#### Scenario: A claim bound to a context of the repository

- **WHEN** an invariant about refunds is accepted bound only to the context `shop-billing`, whose glob is `shop:src/billing/**`
- **THEN** `boom search refund --repository shop --json` gives it, and `boom search refund --repository ledger --json` does not

### Requirement: boom show gives a context and a binding

`boom show <name>@<version>` SHALL give the globs of a context version. For a version that binds contexts, it SHALL give each binding with the version that it meant. It SHALL mark a binding out of date where another version of that context is accepted now. `boom show <context>` SHALL give the claims whose accepted version binds the context, under `bound:`. With `--json`, it SHALL give `paths`, `contexts` and `bound`.

#### Scenario: A binding out of date

- **WHEN** a claim binds `sluice-overview@<v1>`, a person accepts `sluice-overview@<v2>`, and `boom show <claim>@<version>` runs
- **THEN** boom gives the binding to `sluice-overview@<v1>`, and marks it out of date, with `<v2>` as the accepted version

#### Scenario: The claims bound to a context

- **WHEN** the accepted version of `the-overview-names-each-branch` binds `sluice-overview`, and `boom show sluice-overview --json` runs
- **THEN** `bound` gives `the-overview-names-each-branch` and its version
