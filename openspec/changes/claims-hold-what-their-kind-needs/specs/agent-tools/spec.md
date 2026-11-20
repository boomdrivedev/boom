## MODIFIED Requirements

### Requirement: Each tool is one of boom's commands

`tools/list` SHALL give one tool for each command that boom serves: `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`. Each tool SHALL have the name of its command and the description of its command. Each tool SHALL take the arguments of its command, with their names, except `key`. The positional arguments SHALL be required. A tool SHALL NOT exist without its command. Boom SHALL serve each of its commands, or name it as absent.

#### Scenario: The tools listed

- **WHEN** a client sends `tools/list`
- **THEN** boom answers with the tools `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`, and no other tool. Each has a description and an object input schema. Only `show`, `log`, `query` and `search` are marked read-only

#### Scenario: A tool's arguments are its command's

- **WHEN** a client sends `tools/list`
- **THEN** `propose` takes `kind` (one of `invariant`, `decision` and `assumption`), `name`, `statement`, `amends`, `body`, `provenance`, `repositories` and `system`, and requires `kind`, `name` and `statement`. `propose` takes no `scenarios`. `show` takes `target`, required. `log` takes nothing. `query` takes `repository` and `if_declared`. `search` takes `words`, an array of strings, required, and `repository`, `all` and `limit`. `import_adr` takes `dir`, `repository`, `drafts`, `provenance` and `dry_run`, and requires `dir`. `import_rules` takes the same, with `page` for `dir`

#### Scenario: Each command served or absent

- **WHEN** boom reads its commands from the definition of its command line
- **THEN** boom serves each command as a tool or names it as absent, and no command is both. Each argument of a served command is an argument of its tool, except `key`

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

- **WHEN** an agent calls `query` with no `repository`, on a server whose working folder is in a repository whose `.config/boom.toml` declares `shop`
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
