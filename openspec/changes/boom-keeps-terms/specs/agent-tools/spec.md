## MODIFIED Requirements

### Requirement: Each tool is one of boom's commands

`tools/list` SHALL give one tool for each command that boom serves: `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`. Each tool SHALL have the name of its command and the description of its command. Each tool SHALL take the arguments of its command, with their names, except `key`. The positional arguments SHALL be required. A tool SHALL NOT exist without its command. Boom SHALL serve each of its commands, or name it as absent. Boom SHALL name `terms check` as absent.

#### Scenario: The tools listed

- **WHEN** a client sends `tools/list`
- **THEN** boom answers with the tools `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`, and no other tool. Each has a description and an object input schema. Only `show`, `log`, `query` and `search` are marked read-only

#### Scenario: A tool's arguments are its command's

- **WHEN** a client sends `tools/list`
- **THEN** `propose` takes `kind` (one of `invariant`, `decision`, `assumption` and `term`), `name`, `statement`, `amends`, `body`, `replaces`, `provenance`, `repositories` and `system`, and requires `kind`, `name` and `statement`. `replaces` is an array of strings. `propose` takes no `scenarios`. `show` takes `target`, required. `log` takes nothing. `query` takes `repository` and `if_declared`. `search` takes `words`, an array of strings, required, and `repository`, `all` and `limit`. `import_adr` takes `dir`, `repository`, `drafts`, `provenance` and `dry_run`, and requires `dir`. `import_rules` takes the same, with `page` for `dir`

#### Scenario: Each command served or absent

- **WHEN** boom reads its commands from the definition of its command line
- **THEN** boom serves each command as a tool or names it as absent, and no command is both. Each argument of a served command is an argument of its tool, except `key`

#### Scenario: A term proposed through its tool

- **WHEN** an agent calls `propose` with `kind` `term`, `name` `owner`, `statement` `A person who grants roles and registers keys in a store.`, `replaces` `["keeper"]`, `repositories` `["boom"]` and `provenance` `run-1`
- **THEN** the result is not an error, and the log gets one `proposed` entry for a term that replaces `keeper`
