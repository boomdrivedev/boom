## MODIFIED Requirements

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
