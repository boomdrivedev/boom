# Spec Delta

## Purpose

How a writer and the gate find the old words in a file, or in the text
that an agent writes. The check uses the terms that stand for the
repository of the file. It reads a file for the old words of the terms
and for nothing else (`no-task-reads-markdown`).

## ADDED Requirements

### Requirement: boom terms check names each old word

`boom terms check <file>...` SHALL read each file as text. For each file, it SHALL use the accepted terms of the repository of that file. It SHALL give each old word that a file holds. For each, it SHALL give the file, the line, the old word as written, and the term that replaces it. It SHALL append nothing. It SHALL exit 0 when it finds no old word, and 2 when it finds one.

#### Scenario: An old word found

- **WHEN** the term `owner` that replaces `keeper` is accepted for `boom`, and `boom terms check notes.txt` runs on a file of `boom` whose line 3 holds `The keeper signs.`
- **THEN** boom exits 2, and standard error names `notes.txt`, line 3, `keeper` and `owner`

#### Scenario: No old word

- **WHEN** `boom terms check notes.txt` runs on a file of `boom` that holds no old word
- **THEN** boom exits 0, and the log does not change

#### Scenario: Answered as JSON

- **WHEN** `boom terms check notes.txt --json` runs on that file with `keeper` on line 3
- **THEN** standard output is one `boom-refused/1` object. Its `refused.found` holds `file` `notes.txt`, `line` 3, `word` `keeper`, `term` `owner` and `repository` `boom`

#### Scenario: Nothing found, as JSON

- **WHEN** `boom terms check notes.txt --json` runs on a file of `boom` that holds no old word
- **THEN** standard output is one `boom-terms-check/1` object that names `notes.txt` with its repository `boom`, and `found` is empty

#### Scenario: A term of another repository

- **WHEN** the term `owner` that replaces `keeper` is accepted for `sluice` only, and `boom terms check notes.txt` runs on a file of `boom` that holds `keeper`
- **THEN** boom exits 0

### Requirement: The repository of a file

Boom SHALL find the repository of a file as `boom query` finds the repository of a session, with the folder of the file as the working folder. For a file that does not exist, boom SHALL start at the nearest folder above it that exists. With `--if-declared`, boom SHALL skip a file where it finds no repository, and SHALL check nothing where it finds no store. Without `--if-declared`, boom SHALL refuse that file.

#### Scenario: Two files of two repositories

- **WHEN** `boom terms check a/x.md b/y.md` runs, where the remote of `a` is registered for `boom` and the remote of `b` is registered for `sluice`
- **THEN** boom checks `a/x.md` against the terms of `boom`, and `b/y.md` against the terms of `sluice`

#### Scenario: No repository found, when asked

- **WHEN** `boom terms check --if-declared x.md` runs on a file in a repository whose remotes match no registered remote
- **THEN** boom exits 0, and both streams are empty

#### Scenario: No store, when asked

- **WHEN** `boom terms check --if-declared --repository boom x.md` runs with no store at its path
- **THEN** boom exits 0, makes no store, and checks nothing

#### Scenario: No repository found

- **WHEN** `boom terms check x.md` runs on a file in a repository whose remotes match no registered remote
- **THEN** boom exits 2 and says to name the repository with `--repository` or to register its remote with `boom repo add`

### Requirement: The words that the check matches

An old word SHALL match the same word in another form, case or accent, as `boom search` matches a word. An old word of two or more words SHALL match those words in that order. Boom SHALL split a name in code into its words at `_`, `-`, `.` and a change from lowercase to uppercase. Boom SHALL NOT match an old word inside a longer word.

#### Scenario: Another form

- **WHEN** a file of `boom` holds `Keepers` and `KEEPER`
- **THEN** the check names both, each as `keeper`, with `owner`

#### Scenario: A name in code

- **WHEN** a file of `boom` holds `keeper_id` and `KeeperStore`
- **THEN** the check names both

#### Scenario: Inside a longer word

- **WHEN** a file of `boom` holds `gatekeeper` and no other form of `keeper`
- **THEN** the check finds no old word

#### Scenario: Two words

- **WHEN** a term replaces `drive log`, and a file holds `the Drive Log` on one line
- **THEN** the check names `drive log`

### Requirement: A check run by a writer's hook

With `--hook`, `boom terms check` SHALL read one Claude Code `PreToolUse` event from standard input. It SHALL check the text that the tool writes: `content` of `Write`, `new_string` of `Edit`, and each `new_string` of `MultiEdit`. It SHALL take the repository from `file_path`. It SHALL exit 2 when the text holds an old word, and name each old word and its term on standard error. It SHALL exit 0 for any other tool. It SHALL NOT read the file.

#### Scenario: A write with an old word

- **WHEN** the event is a `Write` of a file of `boom` whose `content` holds `keeper`
- **THEN** boom exits 2, and standard error names `keeper` and `owner`

#### Scenario: An edit of a file that holds an old word

- **WHEN** the event is an `Edit` whose `new_string` holds no old word, of a file that holds `keeper` on another line
- **THEN** boom exits 0

#### Scenario: Another tool

- **WHEN** the event is a `Bash` call
- **THEN** boom exits 0, and both streams are empty

### Requirement: A marker allows an old word on its line

A line that holds `boom: allow <old word>` SHALL NOT give that old word as found. Boom SHALL check each other word of that line. The marker SHALL name the old word as the term names it. The hook SHALL read the marker in the text that an agent writes, as the check reads it in a file.

#### Scenario: A comment that allows an old word

- **WHEN** line 3 of a file of `boom` is `// boom: allow keeper. The old field keeper_id stays for the format of 2026.`
- **THEN** the check finds no old word on line 3

#### Scenario: The marker names another word

- **WHEN** a line holds `boom: allow warden` and the word `keeper`
- **THEN** the check names `keeper` with `owner`

#### Scenario: The marker in a write

- **WHEN** the hook reads a `Write` event whose `content` holds `keeper` on a line with `boom: allow keeper`
- **THEN** boom exits 0

### Requirement: The gate checks the files that a change touches

A gate task SHALL run `boom terms check` on each file that a change adds or modifies. The task SHALL find those files from the difference between the change and the branch that it lands on. The task SHALL skip the files under `openspec/changes/archive/`, and the files that the change deletes. The task SHALL fail when the check exits 2, and name each old word.

#### Scenario: A change that adds an old word

- **WHEN** a change adds `The keeper signs.` to `README.md` of `boom`, and the gate runs
- **THEN** the terms task fails, and names `README.md`, the line, `keeper` and `owner`

#### Scenario: An old word that the change does not touch

- **WHEN** `docs/old.md` of `boom` holds `keeper`, and a change does not touch `docs/old.md`
- **THEN** the terms task does not read `docs/old.md`, and passes

#### Scenario: An archived change

- **WHEN** a change archives an OpenSpec change whose words hold `keeper`
- **THEN** the terms task skips the files under `openspec/changes/archive/`, and passes

