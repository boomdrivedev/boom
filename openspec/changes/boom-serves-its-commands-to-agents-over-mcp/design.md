# Design

## Context

`crates/boom`'s command line (`src/cli.rs`) is a thin layer over the
engine: `cli::run(args, keys, env)` parses with clap, runs the command
and gives an `Outcome`, its exit code and what it printed, which
`src/main.rs` writes out. With `--json` a command prints one JSON object
naming its format, a refusal `boom-refused/1` (exit 2) and a command
that could not run `boom-not-run/1` (exit 3) (command-line). The
commands are `init`, `register`, `propose`, `accept`, `reject`, `show`,
`log`, `query`, `search` and `import adr` / `import rules`. A person's acts,
`init`, `register`, `accept`, `reject` and a `propose` with `--key`,
are signed through `sign::Keys`, which turns a `--key` path into a
signer; `main.rs` hands `cli::run` the person's own `ssh-keygen`
(record 0006). Every other act is an agent's, unsigned and naming its
provenance.

flume already serves its commands over MCP as `flume mcp`
(`crates/flume/src/commands/mcp.rs`; flume's clause 60 and record
0069): one JSON-RPC message a line on standard input and output; the
client's protocol version echoed, or `2025-06-18`; `ping`; each tool
call turned into the command line's words, `--` before its positional
words, and run through the command line's parser and act; the act's
JSON as one text item and `isError` where it exits non-zero; `-32700`
for a line that is not JSON, `-32601` for a method, `-32602` for a
tool; a loop over `serde_json` with no protocol crate. Its Claude Code
plugin names it in `plugins/claude/.mcp.json`.

The rules this design relies on: core.1 and core.2 (a tool answers
from the log through its command, and the server keeps nothing), core.3
(the server holds no key, and the tests' keys are made when they run),
core.5 (no new file type, and no test reads a Markdown file of the
tree) and core.6 (no new dependency or tool, so nothing new to pin). It
reaches no control plane. Record 0011 records the choices below that
are this change's own.

## Goals / Non-Goals

**Goals:**

- An agent's harness starts `boom mcp` and its agent calls boom's
  commands as tools, each answering what its command answers with
  `--json`.
- No call can do, or start, an act that needs a person's signature.
- Each command boom gains later is served or left out on purpose, never
  by default.

**Non-Goals:**

- The Claude Code plugin, its hook before each edit and its skills
  (`claude-code-plugin-three-moments`, `hook-before-each-edit`), and the
  herdr plugin: each is a moon project of its own under `plugins/`
  (`plugins-in-this-repository`), a change of its own naming this
  server.
- The commands the interview serves that boom lacks yet (`supersede`,
  `annotate`, `challenge`, `check`, `backlog`, `render`, `judge`,
  `reconcile`, `sync`): each change that builds one names it served or
  absent (design 3).
- MCP resources, prompts, sampling, progress or cancellation; serving
  over HTTP; a tool answering in words rather than JSON.
- A body or drafts given as text: `propose` and the imports take a file's
  path, as their commands do.

## Decisions

### 1. flume's server, which flume owns and boom cites

`boom mcp` is `flume mcp`'s design, cited and copied: boom links no
other tool's code (`tools-call-commands`), so the design has one owner,
flume, and boom keeps to it. It answers `initialize` with the protocol
version the client names, or `2025-06-18` where it names none,
`capabilities` `{"tools": {}}` and `serverInfo` `{"name": "boom",
"version": <the crate's>}`; `ping` with `{}`; a message with no `id`
with nothing; a line that is not JSON with `-32700` and a null id;
`tools/list` and `tools/call`; any other method with `-32601` naming it;
and a call to a tool it does not serve with `-32602` naming it. It
answers each line before reading the next and flushes standard output
after each answer, and stops when its input ends or its output closes.

It departs from flume's in three places, each additive and each one
flume's server could take: it refuses an argument a tool does not take
(decision 4), where flume's reads the arguments it knows and ignores the
rest; it reads its tools from the command line's definition (decision
3), where flume's are written beside its commands; and its `initialize`
gives `instructions` (decision 7).

Not taken: the `rmcp` crate, which brings an async runtime to a loop
that runs one synchronous command at a time, and crates the gate must
pin and fetch (core.6), and would make a second design beside flume's;
HTTP, which no harness that starts boom needs.

### 2. Where it runs

`Command::Mcp` joins `cli.rs`'s commands, described as serving boom's
commands to an agent's program as tools, none of them a person's act.
`cli::run` takes it before resolving the store's path: with `--json` it
refuses, exit 2, saying `boom mcp` answers in the protocol's messages;
otherwise it serves the process's standard input and output through
`mcp::serve` and answers an empty `Outcome`, exit 0, once the input
ends, as flume's `run` does. `main.rs` is unchanged.

`mcp::serve(input: impl BufRead, output: impl Write, store:
Option<&Path>, env: &Env)` takes the `--store` `boom mcp` was given, if
any, and the `Env` `cli::run` was given. It takes no `Keys` (decision
5).

### 3. The tools, read from the command line's definition

`cli` gives `pub(crate) fn definition() -> clap::Command`, clap's own
reading of `Cli`. `src/mcp.rs` holds three lists:

- `SERVED`, each command served, by its words: `propose`, `show`,
  `log`, `query`, `search`, `import adr`, `import rules`. `search`
  finds the claims that stand by their words (claim-search, record
  0009), an agent's read that signs nothing, so it is served.
- `ABSENT`, each command left out, with why: `init`, the first owner's
  act, signed with their key; `register`, an owner's act, signed;
  `accept` and `reject`, a person's acts, signed; `mcp`, the server
  itself.
- `LEFT_OUT`, the arguments of a served command no tool takes: `key`,
  with which a person signs a proposal.

A tool's name is its command's words joined by `_`; its description is
its command's `about`; its input schema is an object whose properties
are its command's arguments by their clap ids, but `LEFT_OUT` and the
global `store` and `json`, each typed by its action: a switch
(`SetTrue`) a boolean, a flag taken again and again (`Append`) an array
of strings, any other a string, with an `enum` where clap gives the
possible values (`kind`) and its help as its description. `required`
holds the arguments clap requires, the positional ones, and
`additionalProperties` is false. `show`, `log`, `query` and `search`
carry the annotation `readOnlyHint: true`, as flume marks `status`.
`search`'s `limit` is a string, as any other option is, and clap
refuses one that is not a whole number above zero, exit 2.

The tools follow the commands, so `propose` takes what `boom propose`
takes, and a flag added to a served command reaches its tool with no
second edit. A unit test walks `definition()`'s commands to their
leaves, `import adr` and `import rules` under `import`, and holds each
to exactly one of `SERVED` and `ABSENT`, so a change that adds a command
fails until it says which. Not taken: tools written by hand beside the
commands, flume's way, which drift from their commands unseen; serving
every command and refusing a person's act inside its tool, which is
logic the command lacks; `search` named absent, which would leave an
agent a shell for the command claim-search made for agents.

### 4. A call turned into its command's words

A call to a served tool first holds its arguments to the tool's schema:
`arguments` absent reads as `{}`, and one that is not an object, a name
the schema lacks, or a value not of its property's type, a string, an
array of strings or a boolean, is answered at once as an error whose
text is `boom-refused/1` naming the argument, nothing run. This is the
command line's own refusal of a flag it does not take, said in the
tool's terms, so a misspelled `dry_run` never runs a real import.

Then its words: the command's words; each option given as
`--<long>=<value>`, a flag taken again and again once for each item, a
true switch as `--<long>` and a false one left out; `--json`;
`--store=<path>` where `boom mcp` was given one; `--`; and the
positional values in their order. The `=` form and the `--` keep a value
beginning with a hyphen from being read as a switch. A required
argument left out reaches clap missing, and clap refuses it, exit 2.
The server runs `cli::run(words, &NoKeys, env)` and answers
`{"content": [{"type": "text", "text": <its standard output, trimmed>}],
"isError": <its exit code is not 0>}`: the command's JSON object, its
answer, `boom-refused/1` or `boom-not-run/1`.

### 5. No key reaches the server

`src/mcp.rs` holds `NoKeys`, a `sign::Keys` whose `signer` refuses
every path, saying `boom mcp` signs nothing and that a person accepts,
rejects or registers in their own terminal. `serve` takes no key source
and gives each command it runs `NoKeys`, so no call can reach a
person's `ssh-keygen` and set off their key's prompt, even by an
argument a later change lets through. `main.rs` still gives `cli::run`
the person's `ssh-keygen`; `cli::run` does not pass it to `serve`.

The interview's `mcp-serves-no-persons-act` is held three ways: the
person's commands are absent (decision 3), `key` is left out of every
tool, and the server holds no key that signs. Not taken: the absent
list alone, which a flag added later to a served command, a person's
answer to a check among them, could get past.

### 6. The server's store, folder and environment

Each call runs with the `Env` `boom mcp` was started with: its
`XDG_DATA_HOME`, `HOME`, working folder and the system's clock; and on
the store its `--store` names, else the default each command resolves.
A harness starting `boom mcp` in a repository gets `query` for the
repository that folder declares, and `propose`'s first versions attached
to it, as the command line run there would. No tool takes a store, so an
agent works in the store its harness named. Not taken: a `store`
argument on each tool, which would let an agent pick a store its person
did not.

### 7. What waits on a person is said

`initialize`'s result gives `instructions`:

```text
boom keeps this project's claims. Its tools propose claims and read
what stands; none accepts, rejects or registers. Only a person accepts
or rejects, in their own terminal, signing with their security key:
boom accept <name>@<version> --key <key>, or boom reject
<name>@<version> --reason <why> --key <key>.
```

`boom propose`'s answer, `boom-propose/1`, gains `accept`, `"boom
accept <name>@<version> --key <key>"` while the version is proposed and
null otherwise, a field added within its major; its words gain the line
`a person accepts it with: boom accept <name>@<version> --key <key>`
while it is proposed. The command says it, so the command line and the
tool say it alike, and the server adds nothing to an answer. Not taken:
the server appending it to a tool's answer, which is logic the command
lacks.

### 8. How a harness starts it

`README.md` shows a repository's `.mcp.json` naming `boom mcp`, as
flume's plugin names `flume mcp`:

```json
{ "mcpServers": { "boom": { "command": "boom", "args": ["mcp"] } } }
```

The Claude Code plugin, a later change, names the same server.

### 9. The tests

`crates/boom/tests/mcp.rs` holds the agent-tools scenarios, each test
named for its scenario. The tests' `Place` gains `mcp(requests)` and
`mcp_in(cwd, requests)`, which drive `boom::mcp::serve` in the test's
own process with the place's `Env` and store, one request a line, and
give each answer read back as JSON, and `call(tool, arguments)`, which
sends `initialize` and one `tools/call` and gives the call's result. One
test runs the binary, `CARGO_BIN_EXE_boom`, as `boom mcp --store
<path>`, writing one request, reading its answer on a thread within ten
seconds, and only then writing the next, so a server that held its
answers until its input closed fails rather than hangs. Unit tests in
`src/mcp.rs` hold `SERVED` and `ABSENT` to `definition()`, each tool's
properties to its command's arguments, the words a call is turned into,
and `NoKeys`'s refusal.

## Risks / Trade-offs

- [A client naming a protocol version newer than `2025-06-18` is
  answered with its own] → as flume's server does; `tools/list` and
  `tools/call` are unchanged in what boom uses, and when a version
  changes them, flume's and boom's servers change together.
- [A flag a later change adds to a served command reaches its tool
  unless named in `LEFT_OUT`] → a flag that needs a person's signature
  meets `NoKeys` and is refused, so the worst it does is fail; the change
  adding it names it in `LEFT_OUT`.
- [An agent proposes at every turn] → each version waits on a person,
  the same words again add nothing, and `boom show` and the store's log
  name the provenance.
- [Each call opens the store and replays as its command does] → as the
  command line; calls in a session are few.
- [`body` and `drafts` are paths read in the server's working folder,
  not the agent's] → a harness starts its MCP servers in the agent's
  project, as Claude Code does; flume's `launch` takes its brief as text
  because its command does, and boom's tools take what boom's commands
  take.
- [A harness's environment differs from the person's terminal, so the
  default store differs] → the README says to name `--store` where it
  does.
