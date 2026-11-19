# Spec Delta

## Purpose

boom's Claude Code plugin: what it holds, where Claude Code finds it,
and the hook that starts every agent session in a declared repository
with the claims that stand for it, calling boom only by its command
line (record 0009, point 2; record 0010).

## ADDED Requirements

### Requirement: The plugin and its marketplace

boom SHALL ship a Claude Code plugin named `boom`, at boom's version, listed in a marketplace named `boom` at the tree's root by its folder and with no version. The plugin and the marketplace SHALL each pass the pinned Claude Code's strict validation.

#### Scenario: The plugin validated

- **WHEN** the pinned Claude Code validates the plugin's folder strictly
- **THEN** it passes

#### Scenario: The marketplace validated

- **WHEN** the pinned Claude Code validates the tree's root strictly
- **THEN** it passes, the marketplace listing the plugin `boom`

### Requirement: A session starts with its repository's claims

The plugin's `SessionStart` hook SHALL run `boom query --if-declared` in the session's folder, by boom's command line from `PATH`, and pass its standard output on as it is, so an agent's session in a repository that declares itself starts with the claims that stand for it, in the words `boom query` gives.

#### Scenario: The hook runs boom

- **WHEN** the hook's command runs as Claude Code runs it, in a repository's folder, with a `boom` on `PATH` that records how it was called and prints a line
- **THEN** boom was called as `boom query --if-declared` in that folder, and the hook's standard output is that line

### Requirement: Silent where no repository is declared

Where the session's folder declares no repository, inside a repository or outside any, the hook SHALL exit 0 with nothing on standard output or standard error, and SHALL make no store.

#### Scenario: A repository that declares nothing

- **WHEN** the hook's command runs, with the boom the tree builds, in a repository holding no `.config/boom.toml`
- **THEN** it exits 0, both streams are empty, and no store is made

#### Scenario: Outside a repository

- **WHEN** the hook's command runs, with the boom the tree builds, in a folder with no `.git` in it or above it
- **THEN** it exits 0, both streams are empty, and no store is made

### Requirement: A declaration boom cannot read is shown

Where the session's folder holds a declaration boom cannot read, the hook SHALL exit as `boom query` refuses, 2, naming the file, so the person sees what to fix.

#### Scenario: A name not in kebab case

- **WHEN** the hook's command runs, with the boom the tree builds, in a repository whose `.config/boom.toml` holds `repository = "Shop"`
- **THEN** it exits 2, its standard error naming `.config/boom.toml`
