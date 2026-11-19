---
number: 10
title: boom mcp serves an agent the commands it may run, and none a person must sign
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-serves-its-commands-to-agents-over-mcp's construct stage
- its code stage, for `search`
---

# boom mcp serves an agent the commands it may run, and none a person must sign

Status: accepted. The construct stage's own, on the brief "An agent's
program can call Boom's commands as tools, and none of those tools does
what only a person may do", and on the user's answer of 2026-11-01 in
`interview.md`, "no person's act at all". Cheap to change: no caller
calls `boom mcp` yet, and a tool is its command, so a tool added or
left out changes no entry and no answer.

Deciders: the construct stage of the change
`boom-serves-its-commands-to-agents-over-mcp`, and its code stage for
serving `search`.

Sources: `interview.md`, the user's answers of 2026-11-01:
`mcp-is-the-commands`, `mcp-serves-no-persons-act`,
`mcp-offers-no-accept`, `agents-propose`,
`persons-acts-signed-with-presence`, `claude-code-plugin-three-moments`,
`plugins-in-this-repository` and `tools-call-commands`; record 0006;
flume's `flume mcp`, `crates/flume/src/commands/mcp.rs` on flume's main,
its clause 60 and record 0069, and its plugin's `.mcp.json`; record
0009, for `boom search`; core.3 and core.6.

## Context and problem statement

An agent reaches boom only through a shell running its command line,
reading words written for a person, and an agent that runs `boom
accept` there sets off a person's security key prompt for an act the
person did not start. The interview settled that boom serves an agent's
program its commands as MCP tools, each run by its command's own code
and answering its command's JSON, and that no tool performs an act that
needs a person's signature, nor sets off a person's key prompt for an
agent's call. flume already serves its own commands over MCP, so the
family has a design for the protocol's side.

## Considered options

For the server:

1. `flume mcp`'s design, copied and cited: one JSON-RPC message a line
   on standard input and output, a loop over `serde_json`, each call
   turned into the command line's words and run through its parser
   (chosen).
2. The `rmcp` crate, MCP's Rust SDK.

For which tools exist:

3. A tool for each command named served, read from the command line's
   definition, every command named served or absent (chosen).
4. Tools written by hand beside the commands, flume's way.
5. A tool for every command, each person's act refused inside its tool.

For keeping a person's act out:

6. The person's commands absent, `key` left out of every tool, and a
   server that holds no key that signs (chosen).
7. The person's commands absent alone.

## Decision outcome

1. **flume's server, cited.** `boom mcp` answers `initialize` with the
   client's protocol version or `2025-06-18`, `ping`, `tools/list` and
   `tools/call`, with `-32700`, `-32601` and `-32602` as flume's does,
   each answer on one line before the next request is read. flume owns
   that design; boom links no other tool's code
   (`tools-call-commands`), so it copies and cites it. Not taken:
   `rmcp` (option 2), an async runtime for a loop that runs one
   synchronous command at a time, crates the gate must pin and fetch
   (core.6), and a second design beside flume's.
2. **The tools are the commands an agent may run.** `propose`, `show`,
   `log`, `query`, `search`, `import_adr` and `import_rules`, each
   named for its command's words, described as its command is, taking
   its command's arguments by their names and answering its command's
   `--json` object, a refusal or a command that could not run marked an
   error.
   Each tool's schema is read from the command line's own definition,
   and a unit test holds every command to being served or named absent,
   so no tool exists without its command and a command added later is
   left out or served on purpose, never by default. `search` finds the
   claims that stand by their words (record 0009), an agent's read that
   signs nothing, so it is served, read-only as `show`, `log` and
   `query` are. Not taken: tools written by hand (option 4), which drift
   from their commands unseen; a tool for every command refusing a
   person's act inside it (option 5), which is logic its command lacks;
   `search` named absent, which would leave an agent a shell for the
   command record 0009 made for agents.
3. **No person's act, three ways.** `init`, `register`, `accept` and
   `reject` are absent, each named with why; `propose` takes no `key`;
   and the server gives every command it runs a key source that signs
   nothing, so no call reaches a person's `ssh-keygen` or sets off their
   key's prompt, even by an argument a later change lets through, a
   person's answer to an unclear check among them. Not taken: the absent
   list alone (option 7).
4. **An argument a tool does not take is refused.** A call naming one,
   or giving a value of another type than its schema's, runs nothing and
   is an error naming it, as the command line refuses a flag it does not
   take, so a misspelled `dry_run` never runs a real import. flume's
   server ignores such an argument; this departure is additive, and
   flume's server could take it.
5. **The server's store and folder.** Each tool runs on the store
   `boom mcp --store` names, in the server's working folder and
   environment, so `query` answers for the repository that folder
   declares; no tool takes a store, and an agent works in the store its
   harness named.
6. **What waits on a person is said, never done.** `initialize`'s
   instructions say the tools propose and read, and only a person
   accepts or rejects, in their own terminal; `boom propose`'s answer
   gives `accept`, the command a person runs, while the version waits,
   so the command line and the tool say it alike.
7. **The plugin is a change of its own.** A repository names `boom mcp`
   in its `.mcp.json`, as flume's plugin names `flume mcp`. The Claude
   Code plugin, with its hook before each edit and its skills, is a moon
   project of its own under `plugins/` (`plugins-in-this-repository`),
   built by a later change that names this server.

## Consequences

- Good: an agent's program calls boom without a shell and reads the
  same JSON the command line gives, and no call can sign.
- Good: a command added to boom fails a unit test until its change says
  whether it is served.
- Bad: a tool takes a body's or a drafts file's path, read in the
  server's working folder, as its command does, rather than its text.
- Bad: a client naming a protocol version newer than `2025-06-18` is
  answered with its own, as flume's server does; when a version changes
  what boom uses, both servers change together.
- Neutral: the commands the interview serves that boom lacks yet,
  `supersede`, `annotate`, `challenge`, `check`, `backlog`, `render`,
  `judge`, `reconcile` and `sync`, are served or left out by the change
  that builds each.
