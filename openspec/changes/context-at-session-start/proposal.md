## Why

Turn 2 of the roadmap of Boom gives an agent's session the claims of
its repository when the session starts. The session needs no brief, and
the repository holds no file for it. On 2026-11-21 the user approved that a repository gets
the context of Boom at session start, or as files that `boom sync`
writes, or both
(`context-either-at-session-start-or-synced-into-the-repository@3c81cbd43414`).
This change builds the first of the two ways.

Today boom finds the repository of a folder from `.config/boom.toml`,
a file in the repository. On 2026-11-01 the user ruled that the
repositories of a store, and the remote of each, are settings that an
owner signs. A file in a repository is not such a setting.

## What Changes

- **The repository of a session.** Boom finds it in this order:
  - `--repository <name>`, where the caller names it.
  - Else `BOOM_REPOSITORY`, which a launcher such as flume sets when it
    starts the agent of a place.
  - Else the remotes of the git repository that holds the working
    folder, matched against the remotes that the log registers.
- `.config/boom.toml` stops being read, by each command.
- **The repositories are settings in the log, signed by an owner.**
  `boom repo add <name> --remote <url> --key <key>` registers a
  repository and its remote. `boom repo set <name>` changes the remote
  or the delivery of a repository. `boom repo show` gives each
  repository with its remote, its delivery, and the entry that set it.
- **Delivery at session start, on or off.** Each repository has one
  delivery setting in this turn: at session start, on or off. The
  setting starts on. The settings of `boom sync` come in turn 2b.
- **The hook.** The SessionStart hook of the plugin runs
  `boom query --if-declared`, as now. Its first line names the
  repository, the number of claims and the entry, so a person sees it
  in the session. A folder that matches no repository gets nothing, and
  a repository whose delivery is off gets nothing. The hook stays
  silent for both.

## Capabilities

### New Capabilities

- `repository-settings`: the repositories that a store registers, and
  the remote and the delivery of each. Also the act of an owner that
  sets them, and `boom repo`.

### Modified Capabilities

- `repository-rules`: the repository of a session comes from
  `--repository`, `BOOM_REPOSITORY` or a registered remote, and never
  from `.config/boom.toml`. `--if-declared` is silent where the
  delivery is off. The first line gives the number of claims.
- `claim-attachments`: a proposal with no `--repository` binds the
  repository of the session.
- `claim-search`: a search with no `--repository` searches the
  repository of the session.
- `agent-tools`: a tool finds the repository of the session as the
  command does.
- `claude-code-plugin`: the hook names the repository, the number and
  the entry, and is silent where no repository matches or the delivery
  is off.
- `command-line`: the commands `repo add`, `repo set` and `repo show`.

## Impact

- `crates/boom`: `repository.rs` (the repository of a session), a new
  act in `entry.rs` and `judge.rs`, a view of the repositories in
  `store.rs`, `cli.rs` (`repo`, `query`, `search`, `propose`), `mcp.rs`
  (the `repo` commands are absent: an owner signs them), and their tests.
- `plugins/claude-code`: the tests of the hook.
- flume sets `BOOM_REPOSITORY` when it starts the agent of a place. That
  is a change in flume, which this change asks for and does not make.
- A boom older than this one reads a `repository` entry as an entry
  that fails its checks, and counts it for nothing.
- The open changes `boom-keeps-terms` and `the-graphs-first-edges`
  modify some of the same requirements. The change that lands second
  takes the text of the first.
