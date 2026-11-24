## MODIFIED Requirements

### Requirement: Each tool is one of boom's commands

`tools/list` SHALL give one tool for each command that boom serves: `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`. Each tool SHALL have the name and the description of its command. Each tool SHALL take the arguments of its command, with their names, except `key`. Each tool SHALL require what its command requires. A tool SHALL NOT exist without its command. Boom SHALL serve each of its commands, or name it as absent. Boom SHALL name `terms check` as absent.

#### Scenario: The tools listed

- **WHEN** a client sends `tools/list`
- **THEN** boom answers with the tools `propose`, `show`, `log`, `query`, `search`, `import_adr` and `import_rules`, and no other tool. Each has a description and an object input schema. Only `show`, `log`, `query` and `search` are marked read-only

#### Scenario: A tool's arguments are its command's

- **WHEN** a client sends `tools/list`
- **THEN** `propose` takes `kind` (one of `invariant`, `decision`, `assumption`, `term`, `context`, `narrative` and `outline`), `name`, `statement`, `amends`, `body`, `source`, `changes`, `origin`, `includes`, `answers`, `view`, `narratives`, `provenance`, `repositories`, `contexts`, `system`, `replaces` and `paths`, and requires `kind`, `name` and `statement`. `source`, `changes`, `origin`, `includes`, `answers`, `narratives`, `contexts`, `replaces` and `paths` are arrays of strings. `propose` takes no `scenarios`. `show` takes `target`, `repository` and `artifact`, and requires none of them. `log` takes nothing. `query` takes `repository`, `if_declared` and `paths`, an array of strings. `search` takes `words`, an array of strings, required, and `repository`, `all` and `limit`. `import_adr` takes `dir`, `repository`, `drafts`, `provenance` and `dry_run`, and requires `dir`. `import_rules` takes the same, with `page` for `dir`

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

#### Scenario: An outline proposed through its tool

- **WHEN** the store holds the narrative `boom-end-to-end`, and an agent calls `propose` with `kind` `outline`, `name` `boomdrive-design-book`, `statement` `The design book of Boomdrive`, `view` `book`, `narratives` `["boom-end-to-end"]` and `provenance` `run-1`
- **THEN** the result is not an error, and the log gets one `proposed` entry for an outline that holds `boom-end-to-end`
