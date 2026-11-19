# Proposal

## Why

An agent working in a repository reads that repository's rules and
decisions from pages its writers keep by hand, `docs/architecture/` and
`docs/adr/`, pulled in by its harness's own file, so what it reads is
whatever someone last typed, accepted or not (signal
`2026-11-18-boomdrive-operator-5f02a78a/03-boom-replaces-the-scattered-context`).
boom now keeps a store's claims in a log on the machine, each version
accepted by a person's signature, but nothing says which repository a
claim binds, and no command gives an agent the claims that stand for
the repository it works in. Record 0005 retired the slice that would
have been committed into each repository, and left open how work reads
the rules it must keep. This change has an agent get them from boom,
as the person accepted them, with nothing in the repository to edit.

## What Changes

- A proposal names the repositories its version binds, or binds the
  whole system. The attachment rides with its version: it is in the
  proposal's signed part and no part of the version's fingerprint, so
  it takes effect only when a person accepts that version
  (`retire-attach-and-rules-need-a-person`).
- `boom propose` gains `--repository <name>`, any number of times, and
  `--system`. Naming neither, an amendment keeps its base's attachment
  and a first version binds the repository its working folder declares,
  else the whole system. The same version proposed again under another
  attachment is refused.
- A repository declares itself in `.config/boom.toml`, one key,
  `repository`, read from the working folder up to the folder holding
  `.git`.
- `boom query` gives every claim that stands for a repository, named by
  `--repository` or declared: each with its kind, name, version,
  statement, scenarios, a pointer to its body, what it binds, and who
  accepted it. It replays the log for each answer and reads no view
  table, so a view edited by hand changes nothing an agent gets. In
  words it writes to standard output alone, for a harness to put in an
  agent's context as it is; with `--json` it answers `boom-query/1`.
- `boom show` gives each version's attachment, so the person reads what
  a version binds before accepting it.
- `boom import` attaches each record it proposes to the repository its
  `--repository` names, so a repository's imported records and rules
  stand for that repository alone; a record that came in as history
  never stands, so `boom query` never gives it (design 7).

## Capabilities

### New Capabilities

- `claim-attachments`: what a version binds, the repositories its
  proposal names or the whole system; how `boom propose` attaches;
  amendments keeping their base's; the attachment no part of the
  version; shown before it is accepted; judged at the write and on
  every read.
- `repository-rules`: a repository named or declared in
  `.config/boom.toml`; `boom query` giving every claim standing for it,
  as the person accepted it, replayed from the log; its JSON and its
  words.

### Modified Capabilities

- `record-import`: an import's proposals carry `attach`, naming the
  repository the import names, after `imported`, so a record accepted
  stands for that repository alone.

`claim-versions` keeps its fingerprint unchanged and `command-line` its
commands' common rules; the new flags, fields and command are the two
new capabilities', each added within its format's major.

## Impact

- `crates/boom`: `entry` gains the proposal's `attach`; `judge` holds
  it; `cli` gains `boom propose --repository` and `--system`, `boom
  show`'s `repositories` and `boom query`, and `Env` the working folder;
  a new module `repository` reads `.config/boom.toml`. New dependency
  `toml`, locked in `Cargo.lock`. The store's schema stays 1.
- `README.md` and `CLAUDE.md` describe `boom query` and attachments,
  and the README the hook a repository's agent settings name.
- `docs/adr/0008-an-agent-gets-a-repositorys-rules-from-boom-query-each-claim-attached-to-the-repositories-it-binds.md`
  records this change's choices.
- `boom-imports-a-repositorys-records-and-rules` landed in the bolt
  first, adding a proposal's `imported` and `boom import --repository`;
  this change merges the bolt and reconciles the two, a proposal's part
  holding `claim`, `imported` and `attach` in that order and an import's
  proposals attached to its repository (design 7).
- Out of this change: paths and contexts, moving an attachment, the
  Claude Code and herdr plugins, `boom mcp`, flume's launch, `boom
  render`, `boom check`, and moving this repository's own rules and
  records into a store.
