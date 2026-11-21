# claude-code-plugin Specification

## Purpose
boom's Claude Code plugin: what it holds, where Claude Code finds it,
and the hook that starts every agent session in a declared repository
with the claims that stand for it, calling boom only by its command
line (record 0009, point 2; record 0010).

## Requirements

### Requirement: The plugin and its marketplace

boom SHALL ship a Claude Code plugin named `boom`, at boom's version, listed in a marketplace named `boom` at the tree's root by its folder and with no version. The plugin and the marketplace SHALL each pass the pinned Claude Code's strict validation.

#### Scenario: The plugin validated

- **WHEN** the pinned Claude Code validates the plugin's folder strictly
- **THEN** it passes

#### Scenario: The marketplace validated

- **WHEN** the pinned Claude Code validates the tree's root strictly
- **THEN** it passes, the marketplace listing the plugin `boom`

### Requirement: A session starts with its repository's claims

The `SessionStart` hook of the plugin SHALL run `boom query --if-declared` in the folder of the session, by the command line of boom from `PATH`. It SHALL pass on its standard output as it is. So a session in a registered repository, with delivery at session start on, starts with the claims that stand for it. The first line SHALL name the repository, the number of claims and the entry.

#### Scenario: The hook runs boom

- **WHEN** the hook's command runs as Claude Code runs it, in a repository's folder, with a `boom` on `PATH` that records how it was called and prints a line
- **THEN** boom was called as `boom query --if-declared` in that folder, and the hook's standard output is that line

#### Scenario: A session in a registered repository

- **WHEN** the hook's command runs, with the boom the tree builds, in a worktree whose remote the store registers for `sluice`, where 36 claims stand for `sluice`
- **THEN** its first line is `Boom: 36 claims for sluice, at entry <number>`, and the claims follow

### Requirement: Silent where no repository is declared

The hook SHALL exit 0 with nothing on standard output or standard error, and SHALL make no store, in two cases. Boom finds no repository of the session, or the delivery of the repository at session start is off.

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

The hook SHALL exit 2, as `boom query` does, where the launcher names a repository that the store does not register. It SHALL also exit 2 where the remotes of the folder match two registered repositories. Its standard error SHALL name the cause, so the person sees what to fix.

#### Scenario: A name not in kebab case

- **WHEN** the hook's command runs, with the boom the tree builds, with `BOOM_REPOSITORY=Shop`
- **THEN** it exits 2, and its standard error names `Shop`

### Requirement: An agent's write is checked against the terms

The plugin SHALL have a `PreToolUse` hook for `Write`, `Edit` and `MultiEdit`. The hook SHALL run `boom terms check --hook --if-declared` by the command line of boom from `PATH`. It SHALL pass the event of Claude Code on standard input. When boom exits 2, Claude Code SHALL stop the write and give standard error to the agent. Where boom finds no repository of the file, or no store, the hook SHALL exit 0 with both streams empty.

#### Scenario: The hook runs boom

- **WHEN** the hook's command runs as Claude Code runs it, with a `boom` on `PATH` that records how it was called and what it read
- **THEN** boom was called as `boom terms check --hook --if-declared`, and it read the event on standard input

#### Scenario: An old word stops a write

- **WHEN** the hook's command runs, with the boom the tree builds, on a `Write` event whose `content` holds an old word of the declared repository
- **THEN** it exits 2, and its standard error names the old word and its term

#### Scenario: A file in no registered repository

- **WHEN** the hook's command runs, with the boom the tree builds, on a `Write` event of a file in a repository whose remotes match no registered remote
- **THEN** it exits 0, both streams are empty, and no store is made
