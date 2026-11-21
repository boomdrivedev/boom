## Context

Turn 2 of the roadmap of Boom is context at session start. Today:

- `repository.rs` finds the repository of a folder from
  `.config/boom.toml`, between the working folder and the folder that
  holds `.git`.
- The SessionStart hook of the plugin runs `boom query --if-declared`,
  and prints what it gives.
- The log holds four acts: `registered`, `proposed`, `accepted` and
  `rejected`. Nothing in the log names a repository except the
  attachment of a version.

The user approved two ways that a repository gets the context of Boom
(`context-either-at-session-start-or-synced-into-the-repository@3c81cbd43414`).
This change builds the first. Turn 2b builds the settings of
`boom sync`.

This design keeps to the rulings of the user of 2026-11-01. The
repositories of a store and the remote of each are settings of the
ledger, which only an act of an owner changes. Where a repository is
checked out is a setting of the host. A file in a repository is no
setting that boom trusts.

## Goals / Non-Goals

**Goals:**

- A session in a repository that the store registers gets its claims
  from the hook, with no brief and no file in the repository.
- The repositories, their remotes and their delivery are acts of an
  owner in the log.
- A person sees in the session which repository Boom found, how many
  claims stand for it, and at which entry.

**Non-Goals:**

- The settings of `boom sync`, and the files that it writes (turn 2b).
- The rules for a file before each edit (turn 7).
- The slice of a stage of flume (turn 8).
- The change in flume that sets `BOOM_REPOSITORY`. This change names
  the variable, and flume sets it in a change of its own.

## Decisions

### 1. The repository of a session, in one order

Boom finds the repository in this order, and stops at the first that
gives one:

1. `--repository <name>`.
2. `BOOM_REPOSITORY`, which a launcher sets.
3. The remotes of the git repository that holds the working folder,
   matched against the registered remotes.

Each command uses this one order: `query`, `search`, `propose` and the
tools of `boom mcp`.

Alternative: `.config/boom.toml`, as now. It is a file in the
repository, which a change can edit, and the rulings keep settings out
of the repository.

### 2. The launcher names the repository by one variable

flume knows the repository of each place that it launches. Boom reads
`BOOM_REPOSITORY`, and flume sets it when it starts the agent of a
place. Boom owns the name of the variable, and any launcher can set it.
A name that the store does not register is a refusal, because the
launcher made a mistake that a person must see.

Alternative: boom asks flume. That starts another tool from a client,
which `clients-never-fetch-or-start-a-tool` rules out for the tools
that it binds.

### 3. Boom reads the remotes from the git configuration

Boom finds the folder that holds `.git`. For a worktree, it follows
the `gitdir` of the `.git` file to the common folder. It reads each
`remote.<name>.url` of `config` there. It starts no `git` process.
Each remote goes into one form (decision 4), and boom compares that
form with the registered remotes. One match gives the repository. Two
matches are a refusal. No match gives no repository.

### 4. One form of a remote

A remote goes into one form: the host in lowercase, then the path,
with no scheme, no user, no port and no `.git` at its end.
`git@github.com:boomdrivedev/sluice.git`,
`https://github.com/boomdrivedev/sluice` and
`ssh://git@GitHub.com/boomdrivedev/sluice.git` each give
`github.com/boomdrivedev/sluice`.

### 5. A `repository` act, signed by an owner

A new act, `repository`, names a repository, its remote and its
delivery at session start. Only an owner signs it, as only an owner
registers a person. The newest counted act for a name gives its
settings. A remote belongs to one repository. The views keep one row
for each repository, built from the log as each view is.

`boom repo add` makes the first act for a name, and `boom repo set`
makes each later one. `boom mcp` names both as absent, because a
person signs them. It also names `repo show` as absent in this turn.

### 6. Delivery at session start, on or off

Each repository has one delivery setting in this turn, at session
start, which starts on. With it off, `boom query --if-declared` is
silent for that repository. `boom query --repository <name>` still
answers, because the person or the agent asked for it by name.

### 7. The first line names what Boom found

The first line in words becomes `Boom: <n> claims for <repository>, at
entry <number>`. A person who reads the start of a session sees which
repository Boom found and how many claims stand for it.

### 8. `.config/boom.toml` is not read

No command reads it. A repository that holds it gets nothing from it.
A later change can remove the file from the repositories of the
family.

## Risks / Trade-offs

- [A boom older than this one] → It reads a `repository` entry as an
  entry that fails its checks, and counts it for nothing. It also still
  reads `.config/boom.toml`. Each host needs this boom or a later one.
- [A repository with no registered remote] → Its sessions get nothing
  until an owner runs `boom repo add`. The hook stays silent, as it is
  now where no file declares a repository.
- [A fork with the same remote path on another host] → The form keeps
  the host, so `github.com/a/b` and `gitlab.com/a/b` do not match.
- [flume does not set `BOOM_REPOSITORY` yet] → A place of flume falls
  back to its git remote, which gives the same repository.
