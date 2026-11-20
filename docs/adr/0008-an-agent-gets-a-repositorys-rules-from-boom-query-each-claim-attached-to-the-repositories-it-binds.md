---
number: 8
title: An agent gets a repository's rules from boom query, each claim attached to the repositories it binds
status: accepted
date: 2026-11-18
decision-makers:
- the change an-agent-gets-a-repositorys-rules-from-boom's construct stage
---

# An agent gets a repository's rules from boom query, each claim attached to the repositories it binds

Status: accepted. The construct stage made this decision. It rests on
the brief "An agent working in a repository gets that repository's
rules and decisions from Boom, as the person accepted them, and no one
edits them by hand". It also rests on the requirements of the
interview that record 0005 keeps. The decision is cheap to change. No
store holds an attachment yet. The attachment is an optional field, so
the entries stay `boom-entry/1`.

Deciders: the construct stage of the change
`an-agent-gets-a-repositorys-rules-from-boom`.

Sources:

- Record 0005, points 5 and 7, and its open question: how a change
  reads the rules that it must keep, when no slice is committed.
- `interview.md`, the answers of the user of 2026-11-01:
  `retire-attach-and-rules-need-a-person`, `scope-crosses-by-homes`,
  `flume-asks-what-applies`, `hook-before-each-edit`,
  `claude-code-plugin-three-moments`, `plugins-in-this-repository`,
  `boom-state-by-layer`, `caches-derived-only` and `cite-by-name`.
- `design/research/2026-11-15-planning-and-boom-storage/03-agent-context-packets.md`.
- core.1, core.2 and core.5.

## Context and problem statement

boom keeps the claims of a store in a log on the machine. A person
accepts each version with the security key of that person (record
0006). Two things do not exist yet:

- Nothing says which repository a claim binds.
- Nothing gives an agent the claims that stand for the repository that
  it works in.

Thus the rules and decisions of a repository are still in pages that
its writers edit by hand (`docs/architecture/core.md`, `docs/adr/`). An
agent reads them through a file of its harness. Record 0005 retired the
slice that `boom sync` committed into each repository. Thus an agent
cannot get the claims from a view written into the repository. Also,
core.2 does not allow a view that somebody commits and reads back as
input.

## Considered options

How an agent gets the claims:

1. `boom query`. It gives every claim that stands for a repository. It
   computes them from the log each time it runs. It gives them in words
   that the harness of an agent can put in its context with no change,
   or as one JSON object (chosen).
2. A rendered rules page and decision records written into the
   repository, as `boom sync` did in the earlier plan.
3. A Claude Code plugin, built in this change, whose hook runs boom.

What binds a claim to a repository:

4. An attachment that the proposal names next to its claim. The
   attachment is not part of the content of the version. It takes
   effect when a person accepts that version (chosen).
5. The repository as part of the content and fingerprint of the
   version.
6. An attachment that an agent proposes and a person accepts alone,
   apart from any version.

## Decision outcome

1. **An agent asks boom.** `boom query` gives every claim that stands
   for a repository. It gives the kind, name, version, statement and
   scenarios of each claim, and the person who accepted it. A body is a
   pointer: `boom show <name>@<version>` gives its text. This keeps long
   content out, as the packets of the research do. `boom query` writes
   its words to standard output only. Thus the session hook of a
   harness can give them to the agent as its context with no change.
   boom writes nothing into the repository, so there is no page to edit
   by hand. Not taken:
   - A page written into the repository (option 2). This is the slice
     that record 0005 retired. When an agent reads it back, it is the
     stored view that core.2 does not allow.
   - A plugin now (option 3). `plugins-in-this-repository` makes a
     plugin a moon project of its own, with its own checks and tools.
     That is a separate change, which calls this command.
2. **boom replays the answer from the log.** `boom query` takes its
   answer from the replay of the whole log by the judge. It never takes
   its answer from the view tables. Somebody can change a statement or a
   status in the views by hand, or an entry can fail the judge. In both
   cases, the answer to the agent does not change. Not taken: an answer from the
   views. A program that edits the views and does not change
   `views_head` changes that answer.
3. **An attachment goes with its version.** A proposal names the
   repositories that its version binds. The attachment is in the signed
   part of the proposal, after its claim. It is not part of the content
   of the version. Thus it takes effect only when a person accepts that
   version. It never changes the fingerprint of a version, as
   `retire-attach-and-rules-need-a-person` requires. A proposal that
   names no repository binds the whole system, which is every
   repository. The claim of the site, attached to the whole system, is
   an example. Not taken:
   - The repository in the content of the version (option 5). That
     requirement does not allow it.
   - An attachment that a person accepts alone (option 6). That is the
     move of an attachment, a later change.
4. **The narrow default.** `boom propose` attaches a version to the
   first of these that applies:
   1. The repositories that `--repository` names.
   2. With `--system`, the whole system.
   3. For an amendment, the attachment of its base.
   4. The repository that the working folder declares.
   5. The whole system.

   An agent that names nothing binds the repository that it works in,
   never every repository. boom refuses the same version proposed again
   with another attachment, because the move of an attachment is a
   later change.
5. **A repository declares itself in `.config/boom.toml`.** The file
   has one key, `repository`, a kebab-case name. boom reads it from the
   nearest folder that holds the file, from the working folder up to
   the nearest folder that holds `.git`. Outside a repository, boom
   reads no file. Thus boom never reads the `~/.config` of a host as
   the file of a repository. `--repository` names a repository
   directly. The name is the name that an import gives to
   `--repository`. Not taken: a name read from the git remote. That
   name is different from host to host, and from a fork to its
   upstream.
6. **An import attaches what it proposes to its repository.** The
   import (record 0007) landed first, so this change makes the two
   agree:
   - The signed part of a proposal holds `claim`, then `imported`, then
     `attach`. Each field is there only when it has a value.
   - boom attaches each record that an import proposes to the
     repository that its `--repository` names. Thus the records and
     rules of a repository stand for that repository only.
   - A record that came in as history never stands, so `boom query`
     never gives it.
   - The judge applies the rules of the import to an imported proposal,
     and then the rules of every proposal. Thus the same record
     imported again is a repeat only under the same attachment.

   Not taken: a judge that refuses an imported proposal with an
   attachment other than its `imported` repository. That adds a rule
   that the other proposals of an agent do not have. Any agent may
   propose for any attachment, and the person reads `repositories`
   before the person accepts.

## Consequences

- Good: an agent gets exactly the versions that a person accepted. boom
  computes them from the log on the machine of the agent. The
  repository holds nothing to edit.
- Good: the launch of flume (`flume-asks-what-applies`) and a Claude
  Code plugin (`hook-before-each-edit`) each call one command, whose
  JSON has a version.
- Bad: somebody must tell the harness of an agent to run `boom query`.
  Until the plugin and the launch of flume do it, the agent settings of
  a repository name the hook.
- Bad: `.config/boom.toml` is in the tree that a change edits. Thus a
  change can name another repository for the context of its agent. The
  check that trusts nothing of the tree belongs to `boom check`, a
  later change.
- Neutral: a query replays the whole log, in a time linear in its
  entries. Later, a verified checkpoint can make this shorter without a
  change to an entry.

## Amendment of 2026-11-20 (proposed)

Status: proposed, for the user to accept. The change
`boom-keeps-its-views-in-step` makes it.

This amendment changes point 2.

`boom query` does not replay the whole log for each answer. It reads
the views. The views are tables that boom keeps in step with the log.

Each write updates the views in the same transaction as its entry.
Before a command answers, boom compares the head of the views with the
last entry of the log. If they agree, boom answers from the views. If
they do not agree, boom first applies the entries after the head.

A command reads the head entry and the entries after it. It reads no
older entry. Thus a query does not get slower as the log grows.

`boom verify` checks the whole chain and replays the whole log. It
builds the views again and names each view that was different.

What this changes:

- A command does not find a change to an older entry. `boom verify`
  finds it. The user accepted this on 2026-11-20.
- When another program edits a view, that edit changes what
  `boom query` gives, until `boom verify` runs. The user accepted this
  on 2026-11-20.
- An entry that fails the checks of boom still counts for nothing. boom
  judges each new entry when it applies it to the views.

## Amendment of 2026-11-20, second (proposed)

Status: proposed, for the user to accept. The change
`claims-hold-what-their-kind-needs` makes it. The user approved the
claim model on 2026-11-20.

This amendment changes point 1 and the status words.

1. `boom query` gives each claim that is accepted for the repository.
   It gives the kind, name, version, statement and body pointer of each
   claim. It gives the person who accepted the claim.
2. A new version has no scenarios. `boom query` gives the scenarios of
   an old version that has them.
3. The status words are `proposed`, `accepted`, `rejected`,
   `superseded` and `deprecated`. `accepted` replaces `standing`.
   `superseded` replaces `replaced`.
4. The JSON answer has the format `boom-query/2`.
5. `boom query` reads only the accepted versions. It does not read the
   other versions of a claim.
