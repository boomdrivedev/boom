## MODIFIED Requirements

### Requirement: A session starts with its repository's claims

The `SessionStart` hook of the plugin SHALL run `boom hook session-start` by the command line of boom from `PATH`. It SHALL pass the event of Claude Code on standard input, and pass on the standard output of boom as it is. So a session in a registered repository, with delivery at session start on, starts with the claims of `session-context` in layers. The person SHALL see the line `Boom: <count> claims for <repository>, at entry <number>`.

#### Scenario: The hook runs boom

- **WHEN** the hook's command runs as Claude Code runs it, in a repository's folder. A `boom` on `PATH` records how it was called and what it read
- **THEN** boom was called as `boom hook session-start` in that folder, and it read the event on standard input

#### Scenario: A session in a registered repository

- **WHEN** the hook's command runs, with the boom the tree builds, in a worktree whose remote the store registers for `sluice`, where 36 claims stand for `sluice`
- **THEN** its standard output is one JSON object. Its `systemMessage` is `Boom: 36 claims for sluice, at entry <number>`, and its `hookSpecificOutput.additionalContext` starts with the same line

### Requirement: Silent where no repository is declared

The session hook SHALL exit 0 with nothing on standard output or standard error, and SHALL make no store, in two cases. Boom finds no repository of the session, or the delivery of the repository at session start is off.

#### Scenario: A repository that declares nothing

- **WHEN** the hook's command runs, with the boom the tree builds, in a repository whose remotes match no registered remote
- **THEN** it exits 0, both streams are empty, and no store is made

#### Scenario: Outside a repository

- **WHEN** the hook's command runs, with the boom the tree builds, in a folder with no `.git` in it or above it
- **THEN** it exits 0, both streams are empty, and no store is made

#### Scenario: Delivery off

- **WHEN** the hook's command runs in a worktree of a repository whose delivery at session start is off
- **THEN** it exits 0, and both streams are empty

### Requirement: A declaration boom cannot read is shown

The session hook SHALL exit 2, as `boom query` does, where the launcher names a repository that the store does not register. It SHALL also exit 2 where the remotes of the folder match two registered repositories. Its standard error SHALL name the cause, so the person sees what to fix.

#### Scenario: A name not in kebab case

- **WHEN** the hook's command runs, with the boom the tree builds, with `BOOM_REPOSITORY=Shop`
- **THEN** it exits 2, and its standard error names `Shop`

### Requirement: An agent's write is checked against the terms

The plugin SHALL have one `PreToolUse` hook for `Write`, `Edit` and `MultiEdit`. The hook SHALL run `boom hook pre-tool-use` by the command line of boom from `PATH`, and pass the event of Claude Code on standard input. So the hook stops a write that holds an old word, and gives the rules of a context before the first edit in it (`session-context`). Where boom finds no repository of the file, or no store, the hook SHALL exit 0 with both streams empty.

#### Scenario: The hook runs boom

- **WHEN** the hook's command runs as Claude Code runs it, with a `boom` on `PATH` that records how it was called and what it read
- **THEN** boom was called as `boom hook pre-tool-use`, and it read the event on standard input

#### Scenario: An old word stops a write

- **WHEN** the hook's command runs, with the boom the tree builds, on a `Write` event whose `content` holds an old word of the declared repository
- **THEN** its standard output is one JSON object whose `permissionDecision` is `deny`, and whose reason names the old word and its term

#### Scenario: The first edit in a context

- **WHEN** the hook's command runs, with the boom the tree builds, on an `Edit` event of a file in an accepted context with an accepted rule. The session did not get the context
- **THEN** its standard output is one JSON object whose `permissionDecision` is `deny`, whose reason gives the rule, and whose `systemMessage` names the context

#### Scenario: A file in no registered repository

- **WHEN** the hook's command runs, with the boom the tree builds, on a `Write` event. The remotes of the repository of the file match no registered remote
- **THEN** it exits 0, both streams are empty, and no store is made
