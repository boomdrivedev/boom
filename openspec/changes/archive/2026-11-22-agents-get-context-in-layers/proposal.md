## Why

Today the session hook of the plugin gives an agent every claim that
stands for its repository, each with its statement, its scenarios and a
line that names its body. For boom, 42 claims make 9.9 KB. Claude Code
caps the text of a hook at 10,000 characters. Above the cap, it puts the
text in a file and gives the agent the first 2,000 characters. So an
agent can start a session with a fifth of its claims. The decisions take
most of the text, and an agent needs a decision only when its work
touches it. Also, a claim that binds only a context never reaches a
session: the hook does not ask for a path, and nothing gives the claims
of a context at the edit (`contexts-own-paths`, turn 7).

The user ruled on 2026-11-21:

1. At session start, an agent gets the rules in full, the decisions as
   titles, and a menu
   (`session-start-gives-rules-in-full-decisions-as-titles-and-a-menu`).
2. Before an agent edits a file, the hook gives the rules of the file's
   contexts, the first time in each context, and is silent after that
   (`the-first-edit-in-a-context-gives-its-rules`).
3. A rule's details come on demand with `boom show`, and at the edit
   for the rules of that file
   (`a-rules-details-come-on-demand-and-at-the-edit`).
4. Each hook prints a one-line message that the person sees in the
   agent's pane (`each-hook-shows-the-person-one-line`).

## What Changes

- **`boom hook session-start`** reads the `SessionStart` event of
  Claude Code and gives the session its claims in layers. The rules,
  which are the invariants and the terms, each come as one line with
  the statement. The decisions, the assumptions and the contexts each
  come as a count and their names. A menu names `boom show`,
  `boom query --path` and `boom search`. Bodies and scenarios do not
  come.
- **`boom hook pre-tool-use`** reads the `PreToolUse` event of a
  `Write`, an `Edit` or a `MultiEdit`. At the first edit of a file in a
  context in a session, it stops the edit and gives the agent the rules
  of that context, each with its details. The agent then makes the edit
  again. After that, it is silent for that context in that session. It
  is silent where the file falls in no context with a claim.
- **One hook for each event.** `boom hook pre-tool-use` also runs the
  terms check of the written text. Old words and the rules of a context
  come in one answer. The plugin has one `PreToolUse` hook.
- **The session's record.** Boom keeps the contexts that a session got
  in `$XDG_STATE_HOME/boom/sessions/<session_id>/`, outside each
  repository and outside the store. A cleared or compacted session
  starts a new record, because its agent no longer holds those rules.
- **A line for the person.** Each hook that gives the agent claims
  also gives Claude Code a `systemMessage` of one line:
  `Boom: 39 claims for sluice, at entry 217` at session start, and
  `Boom: 4 rules for sluice-host` at the edit.
- **The plugin** calls `boom hook session-start` and
  `boom hook pre-tool-use`.
- `boom query`, `boom query --if-declared` and
  `boom terms check --hook` do not change. A person or a script that
  runs `boom query` with no session gets every claim in full.

## Capabilities

### New Capabilities

- `session-context`: what boom gives a session of an agent, and when:
  the claims at session start in layers, the rules of a context at the
  first edit in it, the old words of an edit, the session's record, and
  the line for the person.

### Modified Capabilities

- `claude-code-plugin`: the session hook runs `boom hook
  session-start`, and the one `PreToolUse` hook runs `boom hook
  pre-tool-use`.
- `command-line`: the commands `hook session-start` and
  `hook pre-tool-use`.

## Impact

- `crates/boom`: a new module `hook.rs` (the events, the layers and the
  session's record), `cli.rs` (the command `hook`), and their tests.
  `context::falls_in` gives the contexts of the file. The terms check of
  `terms.rs` gives the old words. No new dependency.
- `plugins/claude-code`: `hooks/hooks.json`, the description of the
  plugin, and `tests/session.sh` and `tests/write.sh`.
- The store, its log, its views and the JSON of each other command do
  not change. An older boom reads the same store.
- Turn 2b (`boom sync`) writes the same layers into a repository as
  files. The design says how it calls this change's text.
