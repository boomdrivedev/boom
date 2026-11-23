# agent-tools Specification

## Purpose
What an agent's program gets when it starts `boom mcp`: boom's commands
served as tools over the Model Context Protocol, each run by its
command's own code and answering its command's JSON, and none doing
what only a person may do (interview: `mcp-is-the-commands`,
`mcp-serves-no-persons-act`, `mcp-offers-no-accept`; record 0006;
record 0011).

## Requirements

### Requirement: boom mcp speaks the protocol on standard input and output

`boom mcp` SHALL read one JSON-RPC 2.0 message a line on standard input and write each answer as one line on standard output, writing nothing else there, answering each line before it reads the next, until its input ends, then exit 0. It SHALL answer `initialize` with the protocol version the client names, or `2025-06-18` where it names none, the `tools` capability and `serverInfo` naming `boom` and its version; `ping` with an empty result; and a notification with nothing.

#### Scenario: Initialized

- **WHEN** a client sends `initialize` naming the protocol version `2025-06-18`, then the notification `notifications/initialized`, then `ping`
- **THEN** boom answers the first with the protocol version `2025-06-18`, the `tools` capability and `serverInfo` naming `boom` and the crate's version, answers nothing to the second, and answers the third with an empty result

#### Scenario: One answer before the next request

- **WHEN** `boom mcp --store <path>` runs as a program and is sent `initialize`, then `tools/list` once the first is answered, and then its input is closed
- **THEN** it writes each answer as one line of standard output before it is sent the next request, and exits 0

#### Scenario: A line that is not JSON

- **WHEN** a client sends a line that is not JSON, then `ping`
- **THEN** boom answers the first with the JSON-RPC error `-32700` and a null id, and answers the second

#### Scenario: A method it does not serve

- **WHEN** a client sends `resources/list`
- **THEN** boom answers with the JSON-RPC error `-32601` naming `resources/list`

#### Scenario: Asked for one JSON object

- **WHEN** `boom mcp --json` runs
- **THEN** it exits 2 with a `boom-refused/1` object saying `boom mcp` answers in the protocol's messages, and serves nothing

### Requirement: Each tool is one of boom's commands

`tools/list` SHALL give one tool for each command that boom serves: `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`. Each tool SHALL have the name and the description of its command. Each tool SHALL take the arguments of its command, with their names, except `key`. Each tool SHALL require what its command requires. A tool SHALL NOT exist without its command. Boom SHALL serve each of its commands, or name it as absent. Boom SHALL name `terms check` as absent.

#### Scenario: The tools listed

- **WHEN** a client sends `tools/list`
- **THEN** boom answers with the tools `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`, and no other tool. Each has a description and an object input schema. Only `show`, `log`, `query` and `search` are marked read-only

#### Scenario: A tool's arguments are its command's

- **WHEN** a client sends `tools/list`
- **THEN** `propose` takes `kind` (one of `invariant`, `decision`, `assumption`, `term`, `context` and `narrative`), `name`, `statement`, `amends`, `body`, `source`, `changes`, `origin`, `includes`, `answers`, `provenance`, `repositories`, `contexts`, `system`, `replaces` and `paths`, and requires `kind`, `name` and `statement`. `source`, `changes`, `origin`, `includes`, `answers`, `contexts`, `replaces` and `paths` are arrays of strings. `propose` takes no `scenarios`. `show` takes `target`, `repository` and `artifact`, and requires none of them. `log` takes nothing. `query` takes `repository`, `if_declared` and `paths`, an array of strings. `search` takes `words`, an array of strings, required, and `repository`, `all` and `limit`. `import_adr` takes `dir`, `repository`, `drafts`, `provenance` and `dry_run`, and requires `dir`. `import_rules` takes the same, with `page` for `dir`

#### Scenario: Each command served or absent

- **WHEN** boom reads its commands from the definition of its command line
- **THEN** boom serves each command as a tool or names it as absent, and no command is both. Each argument of a served command is an argument of its tool, except `key`

#### Scenario: A term proposed through its tool

- **WHEN** an agent calls `propose` with `kind` `term`, `name` `owner`, `statement` `A person who grants roles and registers keys in a store.`, `replaces` `["keeper"]`, `repositories` `["boom"]` and `provenance` `run-1` <!-- boom: allow keeper -->
- **THEN** the result is not an error, and the log gets one `proposed` entry for a term that replaces `keeper` <!-- boom: allow keeper -->

#### Scenario: A claim proposed with its edges through its tool

- **WHEN** the store holds `flume-has-a-resident`, and an agent calls `propose` with `kind` `invariant`, `name` `the-runner-serves-a-socket`, a statement, `source` `["flume-has-a-resident"]`, `origin` `["git:sluice@<40 hexadecimal characters>:design/overview.svg"]` and `provenance` `run-1`
- **THEN** the result is not an error, and the entry of the proposal holds the edge `source` and the edge `origin`

#### Scenario: An artifact read through its tool

- **WHEN** an agent calls `show` with `artifact` `<fingerprint>`, and boom keeps an artifact with that fingerprint
- **THEN** the result gives the bytes of the artifact

#### Scenario: Both a target and an artifact

- **WHEN** an agent calls `show` with `target` `money-has-currency` and `artifact` `<fingerprint>`
- **THEN** the result is an error that says `show` takes a target or an artifact, and not both

#### Scenario: A context proposed through its tool

- **WHEN** an agent calls `propose` with `kind` `context`, `name` `sluice-overview`, a statement, `paths` `["sluice:crates/sluice/src/overview*.rs"]` and `provenance` `run-1`
- **THEN** the result is not an error, and the log gets one `proposed` entry for a context that binds `sluice`

#### Scenario: The claims for a path through its tool

- **WHEN** an agent calls `query` with `paths` `["sluice:crates/sluice/src/overview.rs"]`
- **THEN** the result gives what `boom query --path sluice:crates/sluice/src/overview.rs --json` gives

### Requirement: A tool runs its command

A call SHALL run the command of its tool with the arguments given. It SHALL use the parser and code of the command line, with `--json`. It SHALL use the store that the `--store` of `boom mcp` names, and the working folder and environment of the server. The result SHALL hold the JSON object of the command as one text item. The result SHALL be marked an error when the command refuses or cannot run. Each value SHALL reach the command as written. Boom SHALL NOT read a value as a switch.

#### Scenario: A proposal through its tool

- **WHEN** an agent calls `propose` with `kind` `invariant`, `name` `money-has-currency`, `statement` `Every amount names its currency.`, `repositories` `["shop"]` and `provenance` `run-1`
- **THEN** the result is not an error. Its text is a `boom-propose/2` object that names the version and `repositories` `["shop"]`. The log gets one `proposed` entry that names the provenance `run-1` and has no signature

#### Scenario: The command's own answer

- **WHEN** a version that binds `shop` is accepted, and an agent calls `query` with `repository` `shop`
- **THEN** the text of the result is the JSON object that `boom query --repository shop --json` gives on that store

#### Scenario: A search through its tool

- **WHEN** a version that binds `shop`, with the statement `Every amount names its currency.`, is accepted, and an agent calls `search` with `words` `["currency"]` and `repository` `shop`
- **THEN** the result is not an error. Its text is the JSON object that `boom search currency --repository shop --json` gives on that store, and it names that version

#### Scenario: A refusal is an error

- **WHEN** an agent calls `propose` with each argument except `provenance`
- **THEN** the result is marked an error. Its text is a `boom-refused/1` object that says the proposal of an agent names its provenance. The log does not change

#### Scenario: Not run is an error

- **WHEN** an agent calls `show` with `target` `money-has-currency` on a server whose `--store` names a path with no store
- **THEN** the result is marked an error. Its text is a `boom-not-run/1` object that says there is no store there

#### Scenario: Words beginning with a hyphen

- **WHEN** an agent calls `propose` with `statement` `-0 is never an amount.` and `provenance` `--run-1`
- **THEN** the statement and the provenance of the version are those words as written

#### Scenario: The working folder's repository

- **WHEN** an agent calls `query` with no `repository`, on a server whose working folder is in a worktree whose remote the store registers for `shop`
- **THEN** boom answers for `shop`

#### Scenario: The launcher's repository

- **WHEN** an agent calls `query` with no `repository`, on a server whose environment holds `BOOM_REPOSITORY=shop`
- **THEN** boom answers for `shop`

### Requirement: An argument a tool does not take is refused

When a call names an argument that its tool does not take, boom SHALL run nothing. When a call gives an argument a value of a type that the schema does not give, boom SHALL run nothing. In both cases, the result SHALL be marked an error. Its text SHALL be a `boom-refused/1` object that names the argument.

#### Scenario: A store given

- **WHEN** an agent calls `show` with `target` `money-has-currency` and `store`, which names another file
- **THEN** the result is marked an error that names `store`, and boom does not open that file

#### Scenario: A value of another type

- **WHEN** an agent calls `propose` with `repositories` given as a string
- **THEN** the result is marked an error that names `repositories`, and the log does not change

#### Scenario: A misspelled switch

- **WHEN** an agent calls `import_adr` with `dryrun` `true` and each argument that an import needs
- **THEN** the result is marked an error that names `dryrun`, and the log does not change

#### Scenario: Scenarios given

- **WHEN** an agent calls `propose` with `scenarios`
- **THEN** the result is marked an error that names `scenarios`, and the log does not change

### Requirement: No tool does what only a person may do

No tool SHALL accept, reject, register a person, make a store or sign. `boom mcp` SHALL serve no `init`, `register`, `accept` or `reject`, no tool SHALL take a `key`, and a command the server runs SHALL reach no person's key, so no call sets off a security key's prompt. A call naming a tool boom does not serve SHALL be answered with the JSON-RPC error `-32602`, naming it, running nothing.

#### Scenario: No person's act listed

- **WHEN** a client sends `tools/list`
- **THEN** no tool is named `init`, `register`, `accept`, `reject` or `mcp`, and no tool takes `key`

#### Scenario: An accept asked of the server

- **WHEN** an agent calls `accept` with the target of a proposed version
- **THEN** boom answers with the JSON-RPC error `-32602` naming `accept`, and the version stays proposed

#### Scenario: A key given to propose

- **WHEN** an agent calls `propose` with every argument a proposal needs and `key` naming a security key registered to `ada`
- **THEN** the result is marked an error naming `key`, the key signs nothing, and the log is as it was

#### Scenario: The server holds no key

- **WHEN** a command the server runs asks for the signer of any key
- **THEN** it is refused, saying `boom mcp` signs nothing, and no key's program runs

### Requirement: What waits on a person is said

The answer of `boom mcp` to `initialize` SHALL give instructions. They SHALL say that the tools propose and read. They SHALL say that only a person accepts or rejects, with `boom accept` or `boom reject` and a key, in the terminal of that person. While a version is proposed, the answer of `boom propose` SHALL give `accept`, the command `boom accept <name>@<version> --key <key>`, and its words SHALL give that line. Otherwise `accept` SHALL be null.

#### Scenario: The instructions

- **WHEN** a client sends `initialize`
- **THEN** the `instructions` of the result name `boom accept` and `boom reject`, and say that a person runs them in the terminal of that person

#### Scenario: A proposal's accept

- **WHEN** an agent proposes a first version of `money-has-currency`, with `boom propose --json` or with the tool `propose`
- **THEN** the `accept` of the answer is `boom accept money-has-currency@<version> --key <key>`, and its words give that line

#### Scenario: A version that stands

- **WHEN** an agent proposes again the words of a version that a person accepted
- **THEN** the answer gives the version as `accepted`, and `accept` as null

### Requirement: No tool writes a note

No tool SHALL write a note or serve a page. Boom SHALL name `note` and `page` as absent. The instructions of `boom mcp` SHALL say that only a person writes a note, on the page or with `boom note`.

#### Scenario: A note asked of the server

- **WHEN** an agent calls `note` with a version of a narrative, a paragraph and words
- **THEN** boom answers with the JSON-RPC error `-32602` naming `note`, and the log is as it was

#### Scenario: The instructions name the note

- **WHEN** a client sends `initialize`
- **THEN** the `instructions` of the result say that only a person writes a note, and name `boom page` and `boom note`
