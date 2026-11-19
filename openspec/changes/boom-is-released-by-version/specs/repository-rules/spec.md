# Spec Delta

## ADDED Requirements

### Requirement: Nothing where nothing is declared, when asked

With `--if-declared`, `boom query` SHALL answer as it does without it where the working folder declares a repository. Where none is declared it SHALL open no store, write nothing to standard error and exit 0, printing nothing in words, and with `--json` `boom-query/1` with `repository` and `head` null and `claims` empty. A declaration it cannot read SHALL be refused as without it, and `--if-declared` with `--repository` refused.

#### Scenario: Declared

- **WHEN** `boom query --if-declared` runs in a repository whose `.config/boom.toml` holds `repository = "shop"`, in words and with `--json`
- **THEN** each prints the same bytes as `boom query` there, and exits 0

#### Scenario: Nothing declared

- **WHEN** `boom query --if-declared` runs in a repository holding no `.config/boom.toml`, with no store at the path it would open
- **THEN** it exits 0, standard output and standard error are empty, and no store is made

#### Scenario: Nothing declared, outside a repository

- **WHEN** `boom query --if-declared` runs in a folder holding `.config/boom.toml` with no `.git` in it or above it
- **THEN** it exits 0 and both streams are empty

#### Scenario: Nothing declared, answered as JSON

- **WHEN** `boom query --if-declared --json` runs in a repository holding no `.config/boom.toml`
- **THEN** it prints one object with `format` `boom-query/1`, `repository` null, `head` null and `claims` empty, and exits 0

#### Scenario: A declaration it cannot read

- **WHEN** `boom query --if-declared` runs in a repository whose `.config/boom.toml` holds `repository = "Shop"`
- **THEN** it exits 2 naming the file, and the log is as it was

#### Scenario: Named as well

- **WHEN** `boom query --if-declared --repository shop` runs
- **THEN** it exits 2, saying the two cannot be given together
