# Proposal

## Why

An agent reaches boom today only by running its command line in a
shell, so its harness must allow the shell and parse words meant for a
person, and an agent that runs `boom accept` there sets off a person's
security key prompt for an act the person did not start. The interview settled how an agent's program
calls boom instead: `boom mcp`, an MCP server whose tools are boom's
commands, each run by the command's own code and answering its JSON
(`mcp-is-the-commands`), and none doing what only a person may do
(`mcp-serves-no-persons-act`, `mcp-offers-no-accept`). boom now has the
commands an agent needs, `propose`, `show`, `log`, `query`, `search`
and `import`, and the person's acts they wait on, signed with a security
key (record 0006). This change serves the first set and none of the
second.

## What Changes

- `boom mcp`, a new command: an MCP server on standard input and
  output, one JSON-RPC message a line, that an agent's harness starts.
  It follows `flume mcp`'s design, which flume owns and boom cites.
- Its tools are `propose`, `show`, `log`, `query`, `search`,
  `import_adr` and `import_rules`, each taking its command's arguments under their names
  and answering its command's `--json` object, a refusal or a command
  that could not run marked an error. Each tool's schema is read from
  the command line's own definition, so no tool exists without its
  command.
- No tool accepts, rejects, registers a person or makes a store:
  `init`, `register`, `accept` and `reject` are absent, `propose` takes
  no `key`, and the server gives the commands it runs no key that
  signs, so no call sets off a security key's prompt. Every command
  boom has is either served or named absent.
- A call naming an argument its tool does not take is refused, naming
  it, and runs nothing.
- Each tool runs on the store `boom mcp --store` names, in the server's
  working folder, so `query` answers for the repository that folder
  declares; no tool takes a store.
- What waits on a person is said: the server's instructions say only a
  person accepts or rejects, in their own terminal, and `boom propose`'s
  answer gives `accept`, the command a person runs to accept the version
  it left proposed.

## Capabilities

### New Capabilities

- `agent-tools`: `boom mcp`, the protocol it speaks on standard input
  and output; its tools, one for each command served, each taking its
  command's arguments and answering its command's JSON; the arguments
  it refuses; the person's acts it never serves and the key it never
  holds; and what it says waits on a person.

### Modified Capabilities

None. `command-line` keeps its commands' common rules, and `boom mcp`
keeps them: with `--json` it refuses, answering one JSON object.
`boom-propose/1`'s `accept` is a field added within its major.

## Impact

- `crates/boom`: a new module, `mcp`, the server, its tools and the key
  source that signs nothing; `cli` gains `boom mcp`, the definition the
  tools are read from, and `boom-propose/1`'s `accept`. No new
  dependency; the store's schema stays 1.
- `crates/boom/tests`: `mcp.rs`, and `support` gains a way to drive the
  server.
- `README.md` and `CLAUDE.md` say what `boom mcp` serves and what it
  leaves to a person, and the README shows the `.mcp.json` a repository
  names it in.
- `docs/adr/0011-boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign.md`
  records this change's choices.
- Out of this change: the Claude Code plugin, its hook and skills, and
  the herdr plugin, each a moon project of its own under `plugins/`
  (`plugins-in-this-repository`); the commands the interview serves that
  boom lacks yet, `supersede`, `annotate`, `challenge`, `check`,
  `backlog`, `render`, `judge`, `reconcile` and `sync`, each served by
  the change that builds it; serving over HTTP.
