# session-context Specification

## Purpose
What boom gives the session of an agent, and when. At session start,
the agent gets the rules in full and the other claims by name, with a
menu. Before its first edit of a file in a context, it gets the rules of
that context with their details. Each hook shows the person one line.
Boom keeps the record of a session outside each repository and outside
the store
(`session-start-gives-rules-in-full-decisions-as-titles-and-a-menu`,
`the-first-edit-in-a-context-gives-its-rules`,
`a-rules-details-come-on-demand-and-at-the-edit`,
`each-hook-shows-the-person-one-line`).

## Requirements

### Requirement: boom hook answers an event of Claude Code

`boom hook session-start` and `boom hook pre-tool-use` SHALL each read one event of Claude Code as JSON on standard input. Each SHALL write one JSON object of the hook output of Claude Code on standard output, or nothing. Each SHALL take `--store`. Each SHALL refuse `--json`. Each SHALL read the store as `boom terms check` reads it, and SHALL write nothing into the store.

#### Scenario: The store does not change

- **WHEN** `boom hook session-start` and `boom hook pre-tool-use` run on a store whose views are behind the log
- **THEN** each answers from the log, and the store file has the same bytes after each run

#### Scenario: JSON refused

- **WHEN** `boom hook session-start --json` runs
- **THEN** boom exits 2, and says that a hook always answers in the JSON of Claude Code

### Requirement: Session start answers for the repository of the session

`boom hook session-start` SHALL find the repository of the session as `boom query --if-declared` finds it, with the `cwd` of the event as the working folder. It SHALL use the claims that `boom query` gives for that repository. It SHALL give them as text in `hookSpecificOutput.additionalContext`.

#### Scenario: A claim bound to a context only

- **WHEN** an invariant is accepted bound only to the context `sluice-host`, which is accepted with a glob in `sluice`
- **THEN** the text does not give the invariant, and gives `context sluice-host` under the count of contexts

### Requirement: Session start gives the rules in full and the other claims by name

The text SHALL start with the line `Boom: <count> claims for <repository>, at entry <number>`. It SHALL give each invariant, then each term, as `<kind> <name>@<version>: <statement>`. It SHALL give the decisions, the assumptions and the contexts, each kind as a count and a line `<kind> <name>` for each claim. It SHALL end with a menu that names `boom show`, `boom query --path` and `boom search`. It SHALL NOT give a scenario or a body.

#### Scenario: Rules in full, decisions by name

- **WHEN** a session starts in a worktree of `sluice`, where 32 invariants and 7 decisions are accepted for `sluice`
- **THEN** the text gives each invariant with its statement, a count of 7 decisions, and each decision by name with no statement

#### Scenario: A term is a rule

- **WHEN** the term `refund` is accepted for `shop`, and a session starts in a worktree of `shop`
- **THEN** the line of `refund` gives its meaning, after the lines of the invariants

#### Scenario: No body at session start

- **WHEN** an accepted invariant of `sluice` has a body and two scenarios
- **THEN** the text gives its statement on one line, and no line of its body or its scenarios

#### Scenario: No line for a kind with no claim

- **WHEN** no assumption and no context is accepted for `sluice`
- **THEN** the text gives no line for assumptions and no line for contexts

#### Scenario: The menu

- **WHEN** a session starts in a worktree of `sluice`
- **THEN** the last lines of the text name `boom show <name>`, `boom query --path sluice:<path>` and `boom search <words>`

### Requirement: Each hook shows the person one line

Each hook that gives the agent claims SHALL also give `systemMessage` of one line. At session start, the line SHALL be `Boom: <count> claims for <repository>, at entry <number>`. At the edit, the line SHALL be `Boom: <n> rules for <context>`, with the count of decisions and assumptions where the context has them. A hook that gives no claim SHALL give no `systemMessage`.

#### Scenario: The line at session start

- **WHEN** a session starts in a worktree of `sluice`, where 39 claims stand at entry 221
- **THEN** `systemMessage` is `Boom: 39 claims for sluice, at entry 221`

#### Scenario: The line at the edit

- **WHEN** the first edit of a file in the context `sluice-host` gets 4 rules and 1 decision
- **THEN** `systemMessage` is `Boom: 4 rules and 1 decision for sluice-host`

### Requirement: The edit finds the contexts of its file

For a `Write`, an `Edit` or a `MultiEdit`, `boom hook pre-tool-use` SHALL find the repository of `tool_input.file_path` as `boom terms check` finds the repository of a file. It SHALL find the path of the file from the root of its worktree. It SHALL find the contexts of that path with the function that `boom query --path` uses. For any other tool, it SHALL exit 0 with both streams empty.

#### Scenario: Another tool

- **WHEN** the event is a `Bash` call
- **THEN** the hook exits 0, and both streams are empty

### Requirement: The rules of a context come before the first edit in it

A context of the file is new when the record of the session does not hold it and an accepted claim binds it. For each new context, the hook SHALL stop the edit with `permissionDecision` `deny`. The reason SHALL give the context, each rule of the context with its body as `boom show` gives it, and its decisions and assumptions by name. The reason SHALL tell the agent to make the edit again. The hook SHALL put each of these contexts into the record.

#### Scenario: The first edit in a context

- **WHEN** a session edits `crates/sluice/src/host_process/forge.rs` of `sluice` for the first time, and the accepted context `sluice-host` has the glob `sluice:crates/sluice/src/host_process/**` and 4 rules
- **THEN** the hook stops the edit. The reason gives the 4 rules, each with its statement and its body, and tells the agent to make the edit again

#### Scenario: The edit made again

- **WHEN** the agent then makes the same edit again
- **THEN** the hook exits 0 with nothing on standard output

#### Scenario: Another file of the same context

- **WHEN** the session then edits `crates/sluice/src/host_process/runs.rs`
- **THEN** the hook exits 0 with nothing on standard output

#### Scenario: A file in no context

- **WHEN** a session edits `README.md` of `sluice`, and no accepted context has a glob that matches it
- **THEN** the hook exits 0 with nothing on standard output

#### Scenario: A context with no claim

- **WHEN** a session edits a file in the accepted context `sluice-site`, and no accepted claim binds `sluice-site`
- **THEN** the hook exits 0 with nothing on standard output, and the record of the session does not hold `sluice-site`

#### Scenario: A file in two contexts

- **WHEN** a file falls in `sluice-host` and `sluice-reader`, and the record of the session holds `sluice-host`
- **THEN** the reason gives the rules of `sluice-reader` only

### Requirement: The record of a session

Boom SHALL keep the record of a session in `$XDG_STATE_HOME/boom/sessions/<session_id>/`, or in `~/.local/state/boom/sessions/<session_id>/` where `XDG_STATE_HOME` is not set. The record SHALL hold one file for each context that the session got. Boom SHALL write no file into a repository.

#### Scenario: No file in the repository

- **WHEN** a session gets the rules of a context
- **THEN** `git status` of the worktree shows no new file, and the record of the session holds a file for the context

### Requirement: One hook gives the rules of a context

Boom SHALL make the file of a context in the record only where it does not exist. Only the hook that makes the file SHALL give the rules of the context. Boom SHALL refuse a `session_id` with a character that is not a letter, a digit, `-` or `_`. For that session, it SHALL give no rules and SHALL still check the terms.

#### Scenario: Two edits in parallel

- **WHEN** two `pre-tool-use` hooks of one session run at one time, each for a file in the new context `sluice-host`
- **THEN** one hook gives the rules of `sluice-host`, and the other exits 0 with nothing on standard output

#### Scenario: A session_id that is not a name

- **WHEN** the event gives the `session_id` `../x`
- **THEN** boom writes no record, and gives no rules

### Requirement: A cleared or compacted session gets the rules again

`boom hook session-start` SHALL delete the record of the session where `source` is `clear` or `compact`. It SHALL keep the record where `source` is `resume`. It SHALL delete each record that no hook changed in 30 days.

#### Scenario: Compacted

- **WHEN** a session got the rules of `sluice-host`, then Claude Code compacts the session, and the agent edits a file in `sluice-host`
- **THEN** the hook stops the edit, and gives the rules of `sluice-host` again

#### Scenario: Resumed

- **WHEN** a session got the rules of `sluice-host`, then the session is resumed, and the agent edits a file in `sluice-host`
- **THEN** the hook exits 0 with nothing on standard output

### Requirement: Old words and rules in one answer

`boom hook pre-tool-use` SHALL check the text that the tool writes as `boom terms check --hook` checks it. Where the text holds an old word, the hook SHALL stop the edit, and the reason SHALL name each old word and its term. Where the text holds an old word and the edit is also the first in a context, the hook SHALL stop the edit one time. The reason SHALL give the old words first, then the rules.

#### Scenario: An old word only

- **WHEN** the event is a `Write` of a file of `boom` whose `content` holds `keeper`, and the file falls in no context <!-- boom: allow keeper -->
- **THEN** the hook stops the edit, and the reason names `keeper` and `owner` <!-- boom: allow keeper -->

#### Scenario: An old word in a new context

- **WHEN** the event is an `Edit` whose `new_string` holds `keeper`, of a file in a context that the session did not get <!-- boom: allow keeper -->
- **THEN** the hook stops the edit one time, the reason names `keeper` and then gives the rules of the context, and the record holds the context <!-- boom: allow keeper -->

### Requirement: Silent where nothing applies

Where boom finds no store, or no repository of the session or of the file, each hook SHALL exit 0 with both streams empty. It SHALL make no store. Where delivery at session start of the repository is off, `boom hook session-start` SHALL give nothing. Then `boom hook pre-tool-use` SHALL give no rules, and SHALL still check the terms.

#### Scenario: No registered repository

- **WHEN** a session starts in a repository whose remotes match no registered remote
- **THEN** the hook exits 0, both streams are empty, and no store is made

#### Scenario: Delivery off

- **WHEN** delivery at session start of `sluice` is off, and a session edits a file in the context `sluice-host` with a `content` that holds no old word
- **THEN** `boom hook pre-tool-use` exits 0 with nothing on standard output

### Requirement: A hook names what it cannot read

Where `BOOM_REPOSITORY` names a repository that the store does not register, `boom hook session-start` SHALL exit 2 and name it on standard error. Where boom cannot read the store, each hook SHALL exit 3 and name the cause on standard error.

#### Scenario: A name not registered

- **WHEN** a session starts with `BOOM_REPOSITORY=nowhere`, and the store registers no `nowhere`
- **THEN** `boom hook session-start` exits 2, and its standard error names `nowhere`

#### Scenario: A store that cannot be read

- **WHEN** the chain of the store fails at an entry, and an edit runs
- **THEN** `boom hook pre-tool-use` exits 3 and names the entry, so Claude Code shows the error and lets the edit run
