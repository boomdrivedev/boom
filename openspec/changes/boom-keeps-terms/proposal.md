## Why

An agent writes the words that it reads. When a person corrects a word
today, the correction goes into a chat or a memory note. The note
cleans no source, and it names the old word again. So each correction
adds one more source of the old word.

The user approved terms as turn 2 of Boom on 2026-11-20. Turn 1 is the
constitution: invariants, decisions and assumptions. Turn 2 makes a
word a claim that a person accepts. Then each correction becomes a term
proposal, not a memory note. Turn 3, `boom reconcile terms`, removes the
old words from each source. Turn 3 is not in this change.

## What Changes

- A new claim kind, `term`. The name of a term is its word. The
  statement of a term is the meaning of the word in one repository, in
  Plain. A term can name `replaces`: the old words that the term
  replaces. A term binds one repository, and only one. One word can
  have a term in each repository, with a meaning for each.
- `boom propose term <word> <meaning> --repository <name>` proposes a
  term. `--replaces <old word>` names an old word, and a proposal can
  name it more than one time.
- `replaces` is in the log only. It is part of the content of the
  version, so a person signs it. `boom show` gives it to the person who
  accepts. `boom query` and `boom search` never give it, and boom writes
  no list of old words into the tree.
- `boom query` gives the accepted terms of the repository after its
  invariants, decisions and assumptions, each with its meaning.
- A new command, `boom terms check <file>...`. It reads the accepted
  terms of the repository of each file from the log. It names each old
  word in the file, with its line and the term that replaces it. It
  exits 2 when it finds an old word. It does not use Vale.
- **The gate runs the check too.** A gate task runs `boom terms check`
  on the files that a change touches. It is the check of the term tool
  of its own words, and not a check of what a page says. A new version
  of `no-task-reads-markdown`, `no-task-reads-markdown@a25dda6425d7`,
  names this check as allowed. The user accepts or rejects it.
- **An old word on purpose.** A line may name an old word on purpose
  when it holds the marker `boom: allow <old word>`, in a comment or in
  the text. The check skips that word on that line, and a reviewer sees
  the marker in the diff.
- **History is out of scope.** The gate task does not check
  `openspec/changes/archive/`, which never changes.
- The Claude Code plugin gets a `PreToolUse` hook. Before an agent
  writes a file, the hook runs `boom terms check --hook` on the text
  that the agent writes. When the text holds an old word, the hook stops
  the write and tells the agent the word to use.
- The MCP tool `propose` takes the kind `term` and `replaces`. `boom
  mcp` names `terms check` as absent: the hook of the plugin runs it.

## Capabilities

### New Capabilities

- `term-claims`: the term kind, what a term replaces, and where boom
  gives and keeps the old words.
- `term-check`: `boom terms check`, the words that it matches, the
  marker that allows an old word, the gate task, and the writer's hook
  of the plugin.

### Modified Capabilities

- `claim-versions`: four kinds of claim, `replaces` in the content of a
  term version, and `boom show` gives `replaces`.
- `repository-rules`: `boom query` gives the terms of the repository
  after its other claims, and never their `replaces`.
- `command-line`: the command `terms check`, and the kinds that
  `propose` takes.
- `agent-tools`: the tool `propose` takes the kind `term` and
  `replaces`, and `terms check` is absent.
- `claude-code-plugin`: the hook that checks what an agent writes.

## Impact

- `crates/boom`: `claim.rs` (the kind and its rules), `entry.rs` (the
  field `replaces`), `cli.rs` (`propose`, `query`, `show` and the
  command `terms check`), a new module for the check, `mcp.rs`, and
  their tests.
- `plugins/claude-code/hooks/hooks.json` and the tests of the plugin.
- The JSON formats `boom-propose/2`, `boom-show/2` and `boom-query/2`
  get the kind `term`. `boom-show/2` gets the field `replaces`. These
  are additions, so the major versions stay. `boom terms check` answers
  `boom-terms-check/1`.
- Each version that is in a store now keeps its fingerprint.
- The claim `a-term-binds-one-repository` holds the ruling of the user
  that this change builds.
