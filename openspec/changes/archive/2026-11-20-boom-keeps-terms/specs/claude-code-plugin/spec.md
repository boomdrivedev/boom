## ADDED Requirements

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
