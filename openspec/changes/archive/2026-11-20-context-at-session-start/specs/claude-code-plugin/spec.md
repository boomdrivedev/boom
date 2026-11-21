## MODIFIED Requirements

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
