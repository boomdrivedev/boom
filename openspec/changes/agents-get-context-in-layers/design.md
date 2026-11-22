## Context

The plugin's `SessionStart` hook runs `boom query --if-declared`, and
Claude Code adds its standard output to the agent's context. The answer
gives every claim that stands for the repository of the session, each
with its statement, its scenarios and a line that names its body
(`repository-rules`). The `PreToolUse` hook runs
`boom terms check --hook --if-declared`, which stops a write that holds
an old word (`term-check`). Turn 4b (`contexts-own-paths`) made
contexts. `boom query --path` gives the claims for a file, and
`context::falls_in` is the one answer for the contexts of a path.

The user ruled on 2026-11-21 how an agent gets its claims. Each ruling
is a decision of the store, proposed the same day:

- `session-start-gives-rules-in-full-decisions-as-titles-and-a-menu`
- `the-first-edit-in-a-context-gives-its-rules`
- `a-rules-details-come-on-demand-and-at-the-edit`
- `each-hook-shows-the-person-one-line`

Claude Code 2.1.296, the version that the tree pins, gives a hook these
means (its hooks reference, read on 2026-11-21):

- Each event gives the hook `session_id`, `transcript_path` and `cwd`
  on standard input. `SessionStart` also gives `source`: `startup`,
  `resume`, `clear`, `compact` or `fork`. `PreToolUse` also gives
  `tool_name` and `tool_input`.
- `hookSpecificOutput.additionalContext` of `SessionStart` reaches the
  agent at the start of the conversation, before the first prompt.
- `hookSpecificOutput.additionalContext` of `PreToolUse` reaches the
  agent "next to the tool result", that is, after the tool runs.
- `permissionDecision: "deny"` stops the tool, and the agent reads
  `permissionDecisionReason`. Exit 2 does the same with standard error.
- `systemMessage` is a "warning message shown to the user". Claude Code
  discards it for some events, but not for `SessionStart` or
  `PreToolUse`.
- The hooks that match one event run in parallel. Claude Code gives
  each `additionalContext`, and the strongest decision wins, `deny`
  first.
- Claude Code caps `additionalContext`, `systemMessage` and plain
  standard output at 10,000 characters. Above the cap, the agent gets a
  file path and the first 2,000 characters.

flume answers the events of Claude Code with `flume hook <event>`
(`flume hook session-start`, `flume hook pre-tool-use`), and keeps the
state of a session under `$XDG_STATE_HOME/flume/`. This change takes the
same form, so the family has one way to answer a hook.

This design keeps these claims of the store:

- `views-computed-from-the-ledger`: each layer is computed from the log
  when a hook asks. The session's record holds no claim.
- `status-read-from-events`: the hooks read the accepted version of
  each claim, as `boom query` reads it.
- `context-either-at-session-start-or-synced-into-the-repository`: the
  hooks are the first way. `boom sync` (turn 2b) is the second way, and
  it writes the same text.
- `names-use-the-domains-terms`: `hook`, `session-start`,
  `pre-tool-use`, `session_id`, `systemMessage` and
  `additionalContext` are the words of Claude Code. `rule`, `decision`,
  `context` and `details` are the words of Boom.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  `boom hook` is no MCP tool. An agent does not call a hook.
- `a-term-binds-one-repository`: a term is a rule of its repository, so
  it comes at session start, never at the edit.

## Goals / Non-Goals

**Goals:**

- A session starts with the rules in full and the other claims by name,
  well under the cap of Claude Code.
- An agent gets the rules of a context, with their details, before its
  first edit of a file in that context, and one time in each session.
- The person sees one line for each hook that gives claims.
- One `PreToolUse` hook gives the old words and the rules.
- Boom writes nothing into a repository and nothing into the store.

**Non-Goals:**

- The claims of another repository at the first edit of its file. An
  agent of `sluice` that edits a file of `flume` gets the rules of the
  contexts of that file, and not the claims of `flume`. A later change
  can treat a repository as the context that it is until contexts are
  drawn.
- A hook for the `Read` tool. The rulings name the edit.
- A hook for harnesses other than Claude Code. The layers are a
  function of the library that another harness can call later.
- A refresh at the edit of the claims that a person accepts during a
  session.
- `boom sync` (turn 2b) and the herdr plugin
  (`herdr-shows-what-waits-and-applies`).

## Decisions

### 1. `boom hook <event>` answers Claude Code

`boom hook session-start` and `boom hook pre-tool-use` each read one
event of Claude Code as JSON on standard input. Each writes one JSON
object of Claude Code's hook output on standard output, or nothing.
They take `--store`. They do not take `--json`, because their answer is
always the JSON of Claude Code.

The plugin's hooks call them:

```json
"SessionStart": [{"hooks": [{"type": "command", "command": "boom hook session-start"}]}],
"PreToolUse": [{"matcher": "Write|Edit|MultiEdit",
                "hooks": [{"type": "command", "command": "boom hook pre-tool-use"}]}]
```

Alternative: `boom query --hook` and a second flag on
`boom terms check`. Then each command reads the event of Claude Code on
its own, and the protocol of Claude Code is in two places. flume
already answers hooks as `flume hook <event>`.

### 2. Session start: the claims in layers

`boom hook session-start` finds the repository of the session as
`boom query --if-declared` finds it, from the `cwd` of the event. It
reads the claims that `boom query` gives for that repository, from the
same function. It writes them in this order:

1. The first line: `Boom: <count> claims for <repository>, at entry
   <number>`. The count is the number of claims that the answer covers.
2. The rules: each invariant, then each term, one line each, as
   `<kind> <name>@<version>: <statement>`. No scenario and no body.
3. The decisions, the assumptions and the contexts: for each kind with
   one claim at least, a line with its count, then one line for each
   claim, as `<kind> <name>`.
4. The menu: `boom show <name>`, `boom query --path
   <repository>:<path>` and `boom search <words>`.

A kind with no claim gives no line. A claim that binds only a context
does not come (`contexts-own-paths`, decision 6). The answer is:

```json
{"systemMessage": "Boom: 39 claims for sluice, at entry 221",
 "hookSpecificOutput": {"hookEventName": "SessionStart",
                        "additionalContext": "Boom: 39 claims for sluice, at entry 221\nRules, 32. ..."}}
```

The contexts come by name, as the decisions do. The rulings do not
name them. Their names tell the agent that more rules wait at the edit,
and where.

For sluice, at entry 221 of the store, the agent gets this text (27
rule lines left out here):

```text
Boom: 39 claims for sluice, at entry 221
Rules, 32. Each one holds for each change in sluice:
invariant a-build-takes-its-version-from-its-tree@b97cf1919925: A build takes its version from its tree alone.
invariant a-decision-that-changes-a-rule-says-before-and-now@87e920a42421: A decision that changes a rule says Before and Now in plain words.
invariant a-leaked-child-fails-its-test@9a82a5147ba6: A test fails when it leaves a child process that continues to run.
invariant a-release-tags-a-release-commit-off-main@c2c371868638: A release's tag names a release commit off main, which writes the version of the tool into each file that carries it.
...
invariant workers-within-their-tools-limit@b48467b4ed47: Each tool runs no more workers at one time than the limit that the settings of the host give for that tool.
Decisions, 7, by name:
decision flume-has-a-resident
decision one-worker-limit-per-tool
decision sluices-runs-end-with-its-host-process
decision the-runner-serves-a-socket
decision the-runner-syncs-the-drives-log
decision the-service-manager-starts-each-resident
decision tokio-streams-before-rxrust
More:
- boom show <name>: a claim in full. A rule gives what breaks it, what does not, and its source.
- boom query --path sluice:<path>: the claims for a file, with those of its contexts.
- boom search <words>: the claims whose words hold <words>.
```

The full text is 5.2 KB. `boom query --repository sluice` gives 7.9 KB
for the same claims, and boom's 42 claims gave 9.9 KB, at the cap.

### 3. The edit: the rules of a context, before the first edit in it

`boom hook pre-tool-use` acts on `Write`, `Edit` and `MultiEdit`. It
finds the repository of `tool_input.file_path` as
`boom terms check` finds the repository of a file, and the path of the
file from the root of its worktree. `context::falls_in` gives the
contexts of the path. For each context that the session's record does
not hold, boom reads the claims whose accepted version binds that
context. A context with no such claim gives nothing.

Where one context or more gives claims, boom stops the edit with
`permissionDecision: "deny"`. The reason gives:

1. The first line: `Boom: <n> rules for <context>, at entry <number>`.
   Where two contexts give claims, the line names each one.
2. Why the edit stopped, and to make it again.
3. For each context: its line, `context <name>@<version>:
   <statement>`, and its globs.
4. Each rule of the context, as at session start, and below it its body
   as `boom show` gives it. The body holds the details: what breaks the
   rule, what does not, and its source.
5. The decisions and the assumptions of the context, by name, as at
   session start.

Boom then writes each of these contexts into the session's record. The
agent makes the edit again. The record now holds the context, so the
hook is silent.

The ruling is that the hook gives the rules before the agent edits.
Claude Code gives `additionalContext` of `PreToolUse` after the tool
runs. So `additionalContext` brings the rules after the first edit is
already in the file. Only a stop brings them before. The requirement
`hook-before-each-edit` also rules out an edit to a file in a claim's
scope with no claim shown to the agent. A stop costs one more tool call
for each context in a session.

Alternative: allow the edit and give the rules as `additionalContext`.
The edit runs without a stop, and the agent reads the rules next to its
result. The agent then fixes the edit if it breaks a rule. This is one
call less, but the first edit in each context runs before the agent
knows the rules.

Boom gives the body of a rule as `boom show` gives it. It reads no
heading in the body, so a body in another form still comes whole.

The message for the person is the first line without its entry:
`Boom: 4 rules for sluice-host`. Where the context also gives
decisions or assumptions, the line names their counts too:
`Boom: 4 rules and 1 decision for sluice-host`.

Suppose a person draws the context `sluice-host`, with the glob
`sluice:crates/sluice/src/host_process/**`, and accepts amendments that
bind four rules and one decision to it. Then the first edit of
`crates/sluice/src/host_process/forge.rs` in a session gets this
answer:

```json
{"systemMessage": "Boom: 4 rules and 1 decision for sluice-host",
 "hookSpecificOutput": {"hookEventName": "PreToolUse",
                        "permissionDecision": "deny",
                        "permissionDecisionReason": "Boom: 4 rules and 1 decision for sluice-host, at entry 230\n..."}}
```

The agent reads this reason:

```text
Boom: 4 rules and 1 decision for sluice-host, at entry 230
This is the first edit in this session of a file in the context sluice-host. Boom stopped it so that you read the rules of the context first. Make the edit again, so that it holds these rules.
context sluice-host@<version>: The host process of sluice: its forge reads, its runs and its workers.
  paths: sluice:crates/sluice/src/host_process/**
Rules, 4:
invariant one-forge-read-per-repository-per-interval@<version>: A resident reads the forge one time at most in each interval for each repository.
  A resident reads the forge one time at most in each interval for each repository.

  Breaks the rule:
  - a forge read for each client request
  - a forge read for each bolt and each pass, as `sluice status <bolt> --all` does today

  Does not break it:
  - a test with a paused clock that counts the reads of an idle repository
invariant one-github-client-per-host@<version>: `sluice host` is the only GitHub client on a host.
  `sluice host` is the only GitHub client on a host.

  Breaks the rule:
  - `boomdrive`, `flume` or a client of `sluice` that reads or writes GitHub itself

  Does not break it:
  - the runner that gives `sluice host` its tokens through its socket

  Source: the claim `the-runner-serves-a-socket`.
invariant one-read-in-flight@<version>: Many triggers of one read make one read in flight, and the newest result replaces the older results.
  Many triggers of one read make one read in flight.

  Breaks the rule:
  - two reads of one thing in flight at one time

  Does not break it:
  - a later trigger that the resident joins to the read in flight
invariant workers-end-with-their-owner@<version>: Each worker is a child of its resident, in a process group of its own, and the worker ends when its owner ends.
  Each worker is a child of its resident, in a process group of its own.

  Breaks the rule:
  - a worker that its resident does not own
  - a worker outside a process group

  Does not break it:
  - a hand-over that passes the runs of `sluice host` to the next host process

  Source: the claim `sluices-runs-end-with-its-host-process`.
Decisions, 1, by name:
decision sluices-runs-end-with-its-host-process
More: boom show <name> gives a claim in full.
```

The rules and their bodies are those of the store today. Their
versions would be the amendments that bind the context. The session
start of sluice then gives 35 claims: these five bind only the context,
and do not come, and the context `sluice-host` comes by name.

### 4. The session's record

Boom keeps the record of a session in
`$XDG_STATE_HOME/boom/sessions/<session_id>/`, or in
`~/.local/state/boom/sessions/<session_id>/` where `XDG_STATE_HOME` is
not set. The record holds one empty file for each context that the
session got, named by the context. Boom makes the file with
`create_new`. The first hook that makes it gives the rules. A hook that
finds the file already made gives nothing. So two edits that run in
parallel give the rules of a context one time.

The record is outside each repository, so no repository holds a file of
Boom. It is outside the store, so the log and the views stay as they
are, and the hook writes nothing that a person must sign. It holds no
claim, so no view reads it. It is the harness's state, not the
ledger's, as flume keeps the state of a session under
`$XDG_STATE_HOME/flume/`.

`boom hook session-start` deletes the record of the session where
`source` is `clear` or `compact`. After either, the agent no longer
holds the rules that it got, so the next edit in each context gives
them again. Where `source` is `resume`, the transcript still holds
them, and the record stays. A `fork` has its own `session_id`, so its
record starts empty.

`boom hook session-start` also deletes each record that no hook touched
in 30 days. Boom refuses a `session_id` with a character other than a
letter, a digit, `-` or `_`, and then gives the terms check only.

Alternatives:

- Read the transcript at `transcript_path` for the rules that it holds.
  This writes nothing, but it reads a private form of Claude Code, and
  after a compaction the transcript still holds rules that the agent
  no longer holds.
- `${CLAUDE_PLUGIN_DATA}`, the folder that Claude Code gives a plugin.
  Claude Code deletes it when the plugin is removed. But then the state
  of boom has a place that only Claude Code gives. The family keeps
  state under `$XDG_STATE_HOME/<tool>/`.
- A table in the store. Then a hook writes the store, and the store
  holds the state of a harness beside the log.

### 5. One `PreToolUse` hook

`boom hook pre-tool-use` reads the event one time and the store one
time. It first runs the terms check of `term-check` on the text that
the tool writes, as `boom terms check --hook` does. It then finds the
rules of the contexts. It gives one answer:

- Old words and no new context: a stop, whose reason names each old
  word and its term, as `boom terms check --hook` names them.
- New contexts and no old word: a stop with the rules, as decision 3
  says.
- Both: one stop. Its reason gives the old words first, then the
  rules. The contexts go into the record, because the agent read their
  rules.
- Neither: nothing on standard output, exit 0.

The message for the person names what stopped the edit:
`Boom: 1 old word in README.md` and the line of decision 3, joined by
`, and ` where both apply.

Alternative: two hooks in the `PreToolUse` group, the terms check and
the rules. Claude Code runs them in parallel, in no order. A write with
an old word would stop, but the hook of the rules would still record
its context. Each hook would also open the store.

`boom terms check --hook` stays as it is, for a harness that only stops
a write on exit 2. It and `boom hook pre-tool-use` call one function.

### 6. Read only, and silent where nothing applies

The hooks read the store as `boom terms check` reads it. They write
nothing into the store, not even the views. Where the views are behind
the log, they replay the log in memory. Each hook exits 0 with no
output where boom finds no store, no repository of the session or the
file, or delivery at session start off. `boom hook pre-tool-use` still
runs the terms check where delivery is off, as the write hook does now.
A name from `BOOM_REPOSITORY` that the store does not register stops
the session hook with exit 2, as now. Where the store cannot be read,
the hook exits 3. Claude Code then shows the error and lets the edit
run, so a broken store never stops an agent's work.

### 7. `boom query` with no session

`boom query` does not change. A person in a terminal, or a script with
`--json`, gets every claim that stands for the repository, each with
its statement, its scenarios and its body line, as
`repository-rules` says. `boom query --path` gives every claim for the
file. The layers are the answer for an agent's session, and they live
in `boom hook`. `boom query` is the view of the rules for a scope
(`views-computed-from-the-ledger`), and stays whole. The MCP tool
`query` gives the same as `boom query`.

`boom query --if-declared` stays, for a harness that adds standard
output to the context of a session. The plugin no longer calls it.

### 8. Turn 2b, `boom sync`

The decision
`context-either-at-session-start-or-synced-into-the-repository` gives a
repository two ways to get its context. This change is the first way,
in the session. Turn 2b is the second way, in files. For each context
with a glob in the repository, `boom sync` writes a rules file of Claude
Code with a `paths:` line. Claude Code loads that file when the agent
works with a file whose path matches. That is the layer of the edit,
given by Claude Code from a file in place of a hook.

So the text has one owner. The library gives three functions: the
session layer of a repository, the edit layer of a context, and the
menu. The hooks call them, and `boom sync` calls them to write its
files. A file of `boom sync` holds the edit layer of its context
without the line about the stop.

A repository that takes the files turns off delivery at session start
with `boom repo set <name> --session off`. The hooks then give that
repository nothing, and the agent does not get the rules two times.
The terms check still runs.

## Risks / Trade-offs

- [A stop at each new context] → Each first edit in a context costs
  one more call. The record makes it one time for each context in a
  session. Decision 3 gives the other choice.
- [Two edits in parallel in one new context] → The first makes the
  record and stops. The second finds the record and runs. It can run
  before the agent reads the rules. The rules still reach the agent
  in the same turn.
- [A long reason] → Claude Code caps `additionalContext`,
  `systemMessage` and standard output at 10,000 characters. It does not
  cap `permissionDecisionReason`. Claude Code 2.1.296 gives the agent
  the whole reason of a stop, after the prefix
  `PreToolUse:Write hook error: `, with no file path and no cut. The
  plugin's `tests/cap.sh` shows this with a reason of 15,000
  characters, through the plugin's own hook. So the rules of a context
  come with their bodies at each length. A context with many long
  bodies costs that many tokens at the first edit in it.
- [A rule accepted during a session] → A context in the record does not
  give a new rule until the session is cleared or compacted. A rule of
  the repository does not come until the next session start, as now.
- [Records that stay] → A session that ends with no later session
  start leaves its folder. The next session start deletes it after 30
  days.
- [An older plugin with a newer boom] → The plugin and boom come at one
  version (`a-boom-release-is-a-tag-on-a-release-commit-and-its-flake-gives-boom-and-its-plugin-at-one-version`).
  `boom query --if-declared` and `boom terms check --hook` stay, so an
  older plugin still works.
