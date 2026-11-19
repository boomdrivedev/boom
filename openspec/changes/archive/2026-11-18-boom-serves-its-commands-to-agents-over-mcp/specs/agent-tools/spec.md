# Spec Delta

## Purpose

What an agent's program gets when it starts `boom mcp`: boom's commands
served as tools over the Model Context Protocol, each run by its
command's own code and answering its command's JSON, and none doing
what only a person may do (interview: `mcp-is-the-commands`,
`mcp-serves-no-persons-act`, `mcp-offers-no-accept`; record 0006;
record 0011).

## ADDED Requirements

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

`tools/list` SHALL give one tool for each command served, `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`, each named for its command's words, described as its command is, and taking its command's arguments under their names, its positional ones required, all but `key`. No tool SHALL exist without its command, and each command boom has SHALL be served or named absent.

#### Scenario: The tools listed

- **WHEN** a client sends `tools/list`
- **THEN** boom answers with exactly the tools `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`, each with a description and an object input schema, and `show`, `log`, `query` and `search` alone marked read-only

#### Scenario: A tool's arguments are its command's

- **WHEN** a client sends `tools/list`
- **THEN** `propose` takes `kind`, one of `invariant`, `decision` and `assumption`, `name`, `statement`, `scenarios`, `amends`, `body`, `provenance`, `repositories` and `system`, requiring `kind`, `name` and `statement`; `show` takes `target`, required; `log` takes nothing; `query` takes `repository` and `if_declared`; `search` takes `words`, an array of strings, required, `repository`, `all` and `limit`; and `import_adr` takes `dir`, `repository`, `drafts`, `provenance` and `dry_run`, requiring `dir`, as `import_rules` does with `page` for `dir`

#### Scenario: Each command served or absent

- **WHEN** boom's commands are read from its command line's own definition
- **THEN** each is served as a tool or named absent, none is both, and each argument of a served command is its tool's but `key`

### Requirement: A tool runs its command

A call SHALL run its tool's command with the arguments given, through the command line's own parser and code, with `--json`, on the store `boom mcp`'s `--store` names, in the server's working folder and environment. Its result SHALL hold the command's JSON object as one text item, marked an error where the command refuses or cannot run. Each value SHALL reach the command as written, never read as a switch.

#### Scenario: A proposal through its tool

- **WHEN** an agent calls `propose` with `kind` `invariant`, `name` `money-has-currency`, `statement` `Every amount names its currency.`, `scenarios` `["Given an amount, then it names its currency."]`, `repositories` `["shop"]` and `provenance` `run-1`
- **THEN** the result is not an error, its text is a `boom-propose/1` object naming the version and `repositories` `["shop"]`, and the log gains one `proposed` entry naming the agent's provenance `run-1`, carrying no signature

#### Scenario: The command's own answer

- **WHEN** a version attached to `shop` stands, and an agent calls `query` with `repository` `shop`
- **THEN** the result's text is the JSON object `boom query --repository shop --json` gives on that store

#### Scenario: A search through its tool

- **WHEN** a version attached to `shop` whose statement is `Every amount names its currency.` stands, and an agent calls `search` with `words` `["currency"]` and `repository` `shop`
- **THEN** the result is not an error, and its text is the JSON object `boom search currency --repository shop --json` gives on that store, naming that version

#### Scenario: A refusal is an error

- **WHEN** an agent calls `propose` with every argument but `provenance`
- **THEN** the result is marked an error, its text a `boom-refused/1` object saying an agent's proposal names its provenance, and the log is as it was

#### Scenario: Not run is an error

- **WHEN** an agent calls `show` with `target` `money-has-currency` on a server whose `--store` names a path where no store was made
- **THEN** the result is marked an error, its text a `boom-not-run/1` object saying there is no store there

#### Scenario: Words beginning with a hyphen

- **WHEN** an agent calls `propose` with `statement` `-0 is never an amount.` and a scenario `--given an amount of -0, then it is refused.`
- **THEN** the version's statement and scenario are those words as written

#### Scenario: The working folder's repository

- **WHEN** an agent calls `query` with no `repository` on a server whose working folder lies in a repository whose `.config/boom.toml` declares `shop`
- **THEN** it answers for `shop`

### Requirement: An argument a tool does not take is refused

A call naming an argument its tool does not take, or giving an argument a value of another type than its schema gives, SHALL run nothing and SHALL be marked an error, its text a `boom-refused/1` object naming the argument.

#### Scenario: A store given

- **WHEN** an agent calls `show` with `target` `money-has-currency` and `store` naming another file
- **THEN** the result is marked an error naming `store`, and that file is not opened

#### Scenario: A value of another type

- **WHEN** an agent calls `propose` with `scenarios` given as a string
- **THEN** the result is marked an error naming `scenarios`, and the log is as it was

#### Scenario: A misspelled switch

- **WHEN** an agent calls `import_adr` with `dryrun` `true` and every argument an import needs
- **THEN** the result is marked an error naming `dryrun`, and the log is as it was

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

`boom mcp`'s answer to `initialize` SHALL give instructions saying its tools propose and read, and that only a person accepts or rejects, in their own terminal, with `boom accept` or `boom reject` and their key. `boom propose`'s answer SHALL give `accept`, the command a person runs to accept the version, `boom accept <name>@<version> --key <key>`, while the version is proposed, and null otherwise, and its words that line while proposed.

#### Scenario: The instructions

- **WHEN** a client sends `initialize`
- **THEN** the result's `instructions` name `boom accept` and `boom reject`, and say a person runs them in their own terminal

#### Scenario: A proposal's accept

- **WHEN** an agent proposes a first version of `money-has-currency`, by `boom propose --json` or by the tool `propose`
- **THEN** the answer's `accept` is `boom accept money-has-currency@<version> --key <key>`, and its words give that line

#### Scenario: A version that stands

- **WHEN** an agent proposes again the words of a version a person accepted
- **THEN** the answer gives the version as standing and `accept` as null
