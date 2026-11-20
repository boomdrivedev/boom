---
number: 11
title: boom mcp serves an agent the commands it may run, and none a person must sign
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-serves-its-commands-to-agents-over-mcp's construct stage
- its code stage, for `search`
---

# boom mcp serves an agent the commands it may run, and none a person must sign

Status: accepted. This record is the decision of the construct stage. It
rests on this brief:

> An agent's program can call Boom's commands as tools, and none of
> those tools does what only a person may do

It also rests on the answer of the user of 2026-11-01 in `interview.md`,
"no person's act at all". It is cheap to change. No caller calls `boom mcp` yet. A tool
is its command, so a tool added or left out changes no entry and no
answer.

Deciders: the construct stage of the change
`boom-serves-its-commands-to-agents-over-mcp`, and its code stage for
serving `search`.

Sources:

- `interview.md`, the answers of the user of 2026-11-01:
  `mcp-is-the-commands`, `mcp-serves-no-persons-act`,
  `mcp-offers-no-accept`, `agents-propose`,
  `persons-acts-signed-with-presence`, `claude-code-plugin-three-moments`,
  `plugins-in-this-repository` and `tools-call-commands`.
- Record 0006.
- The `flume mcp` of flume: `crates/flume/src/commands/mcp.rs` on the
  main of flume, its clause 60 and record 0069, and the `.mcp.json` of
  its plugin.
- Record 0009, for `boom search`.
- core.3 and core.6.

## Context and problem statement

An agent gets to boom only through a shell that runs its command line.
It reads words written for a person. An agent that runs `boom accept`
there starts the security key prompt of a person, for an act that the
person did not start.

The interview decided these things:

- boom serves the program of an agent its commands as MCP tools.
- The code of its command runs each tool, and each tool answers the JSON
  of its command.
- No tool does an act that needs the signature of a person.
- No tool starts the key prompt of a person for the call of an agent.

flume already serves its own commands over MCP. Thus the family has a
design for the protocol side.

## Considered options

For the server:

1. The design of `flume mcp`, copied and cited (chosen). It has three
   parts:
   - One JSON-RPC message for each line on standard input and output.
   - A loop over `serde_json`.
   - Each call changed into the words of the command line, and run
     through its parser.
2. The `rmcp` crate, the Rust SDK of MCP.

For which tools exist:

3. A tool for each command named as served, read from the definition of
   the command line. Each command is named as served or absent (chosen).
4. Tools written by hand beside the commands, as flume does.
5. A tool for each command, with each act of a person refused inside its
   tool.

For how to keep the act of a person out:

6. The commands of a person absent, `key` left out of each tool, and a
   server that holds no key that signs (chosen).
7. Only the commands of a person absent.

## Decision outcome

1. **The server of flume, cited.** `boom mcp` answers `initialize` with
   the protocol version of the client or `2025-06-18`. It answers
   `ping`, `tools/list` and `tools/call`. It gives `-32700`, `-32601` and
   `-32602` as the server of flume does. It writes each answer on one
   line before it reads the next request. flume owns that design. boom
   links no code of another tool (`tools-call-commands`), so it copies
   and cites the design.

   Not taken: `rmcp` (option 2). It brings an async runtime for a loop
   that runs one synchronous command at a time. It brings crates that the
   gate must pin and fetch (core.6). It is a second design beside the
   design of flume.
2. **The tools are the commands that an agent may run.** The tools are
   `propose`, `show`, `log`, `query`, `search`, `import_adr` and
   `import_rules`. Each tool has these properties:
   - Its name is the words of its command.
   - Its description is the description of its command.
   - It takes the arguments of its command, by their names.
   - It answers the `--json` object of its command.
   - A refusal, or a command that could not run, is marked an error.

   The server reads the schema of each tool from the definition of the
   command line. A unit test holds each command to one of two states:
   served, or named as absent. Thus no tool exists without its command. A
   command added later is left out or served on purpose, never by
   default.

   `search` finds the claims that stand, by their words (record 0009).
   It is a read of an agent and signs nothing, so the server serves it.
   It is read-only, as `show`, `log` and `query` are.

   Not taken:
   - Tools written by hand (option 4). They drift from their commands,
     and no one sees it.
   - A tool for each command that refuses the act of a person inside it
     (option 5). That is logic that its command does not have.
   - `search` named as absent. That leaves an agent only a shell for the
     command that record 0009 made for agents.
3. **No act of a person, in three ways.**
   - `init`, `register`, `accept` and `reject` are absent, each named
     with the reason.
   - `propose` takes no `key`.
   - The server gives each command that it runs a key source that signs
     nothing.

   Thus no call gets to the `ssh-keygen` of a person, or starts the
   prompt of their key. This holds even for an argument that a later
   change lets through, such as the answer of a person to an unclear
   check.

   Not taken: only the absent list (option 7).
4. **The server refuses an argument that a tool does not take.** A call
   that names such an argument runs nothing. A call that gives a value of
   another type than the type of the schema runs nothing. Each is an
   error that names the argument, as the command line refuses a flag
   that it does not take. Thus a misspelled `dry_run` never runs a real
   import. The server of flume ignores such an argument. This departure
   is additive, and the server of flume could take it.
5. **The store and folder of the server.** Each tool runs on the store
   that `boom mcp --store` names, in the working folder and environment
   of the server. Thus `query` answers for the repository that the folder
   declares. No tool takes a store. An agent works in the store that its
   harness named.
6. **The server says what waits on a person, and never does it.** The
   instructions of `initialize` say that the tools propose and read.
   They say that only a person accepts or rejects, in the terminal of
   that person. While the version waits, the answer of `boom propose`
   gives `accept`, the command that a person runs. Thus the command line
   and the tool say it the same way.
7. **The plugin is a change of its own.** A repository names `boom mcp`
   in its `.mcp.json`, as the plugin of flume names `flume mcp`. The
   Claude Code plugin, with its hook before each edit and its skills, is
   a moon project of its own under `plugins/`
   (`plugins-in-this-repository`). A later change that names this server
   builds it.

## Consequences

- Good: the program of an agent calls boom without a shell. It reads the
  same JSON that the command line gives. No call can sign.
- Good: a command added to boom fails a unit test until its change says
  whether the server serves it.
- Bad: a tool takes the path of a body or of a drafts file, as its
  command does, and not the text. The server reads the path in its
  working folder.
- Bad: when a client names a protocol version newer than `2025-06-18`,
  the server answers with that version, as the server of flume does.
  When a version changes what boom uses, both servers change together.
- Neutral: the interview serves commands that boom does not have yet:
  `supersede`, `annotate`, `challenge`, `check`, `backlog`, `render`,
  `judge`, `reconcile` and `sync`. The change that builds each serves it
  or leaves it out.
