---
number: 8
title: An agent gets a repository's rules from boom query, each claim attached to the repositories it binds
status: accepted
date: 2026-11-18
decision-makers:
- the change an-agent-gets-a-repositorys-rules-from-boom's construct stage
---

# An agent gets a repository's rules from boom query, each claim attached to the repositories it binds

Status: accepted. The construct stage's own, on the brief "An agent
working in a repository gets that repository's rules and decisions from
Boom, as the person accepted them, and no one edits them by hand", and
on the interview's requirements record 0005 keeps. Cheap to change: no
store holds an attachment yet, and the attachment is an optional field,
so entries stay `boom-entry/1`.

Deciders: the construct stage of the change
`an-agent-gets-a-repositorys-rules-from-boom`.

Sources: record 0005, points 5 and 7, and its open question of how a
change reads the rules it must keep now that no slice is committed;
`interview.md`, the user's answers of 2026-11-01:
`retire-attach-and-rules-need-a-person`, `scope-crosses-by-homes`,
`flume-asks-what-applies`, `hook-before-each-edit`,
`claude-code-plugin-three-moments`, `plugins-in-this-repository`,
`boom-state-by-layer`, `caches-derived-only` and `cite-by-name`;
`design/research/2026-11-15-planning-and-boom-storage/03-agent-context-packets.md`;
core.1, core.2 and core.5.

## Context and problem statement

boom keeps a store's claims in a log on the machine, and a person
accepts each version with their security key (record 0006). Nothing
yet says which repository a claim binds, and nothing gives an agent
the claims that stand for the repository it works in, so a repository's
rules and decisions still live in pages its writers edit by hand
(`docs/architecture/core.md`, `docs/adr/`), read by an agent through
its harness's own file. Record 0005 retired the slice `boom sync`
committed into each repository, so a view written into the repository
is no longer the way an agent gets them, and core.2 rules out a view
committed and read back as input.

## Considered options

For how an agent gets them:

1. `boom query`, answering every claim that stands for a repository,
   computed from the log each time it runs, in words an agent's harness
   can put in its context as they are, or as one JSON object (chosen).
2. A rendered rules page and decision records written into the
   repository, as `boom sync` would have.
3. A Claude Code plugin, built in this change, whose hook runs boom.

For what binds a claim to a repository:

4. An attachment the proposal names beside its claim, outside the
   version's content, taking effect with that version's accept (chosen).
5. The repository as part of the version's content and fingerprint.
6. An attachment proposed and accepted on its own, apart from any
   version.

## Decision outcome

1. **An agent asks boom.** `boom query` gives every claim standing for
   a repository, each with its kind, name, version, statement,
   scenarios and the person who accepted it; a body is a pointer,
   `boom show <name>@<version>` giving its text, as the research's
   packets keep long content out. Its words go to standard output alone,
   so a harness's session hook can take them as the agent's context as
   they are. Nothing is written into the repository, so there is no
   page to edit by hand. Not taken: a page written into the repository
   (option 2), which is the slice record 0005 retired, and which, read
   back by an agent, is the stored view core.2 rules out; a plugin now
   (option 3), which `plugins-in-this-repository` makes a moon project
   of its own with its own checks and tools, a change of its own that
   calls this command.
2. **The answer is replayed from the log.** `boom query` takes its
   answer from the judge's replay of the whole log, never from the view
   tables, so a statement or a status changed in the views by hand, or
   an entry that fails the judge, changes nothing an agent gets. Not
   taken: answering from the views, which a program that edits them and
   leaves `views_head` alone would change.
3. **An attachment rides with its version.** A proposal names the
   repositories its version binds; the attachment is in the proposal's
   signed part, after its claim, and no part of the version's content,
   so it takes effect only when a person accepts that version and never
   changes a version's fingerprint, as
   `retire-attach-and-rules-need-a-person` asks. A proposal naming none
   binds the whole system, every repository, as the site's claim
   attached to the whole system does. Not taken: the repository in the
   version's content (option 5), which that requirement rules out;
   an attachment accepted on its own (option 6), the move of an
   attachment, a later change.
4. **The narrow default.** `boom propose` attaches a version to the
   repositories `--repository` names; else, with `--system`, to the
   whole system; else, for an amendment, to what its base is attached
   to; else to the repository the working folder declares; else to the
   whole system. An agent that names nothing binds the repository it
   works in, never every repository. The same version proposed again
   with another attachment is refused, since moving an attachment is a
   later change.
5. **A repository declares itself in `.config/boom.toml`.** Its one key,
   `repository`, a kebab-case name, is read from the nearest folder
   holding the file, from the working folder up to the nearest one
   holding `.git`, and outside a repository from none, so a host's own
   `~/.config` is never read as a repository's; `--repository` names
   one outright. The name is the one an import gives `--repository`.
   Not taken: a name read from the git remote, which differs from host
   to host and from fork to upstream.

## Consequences

- Good: an agent gets exactly the versions a person accepted, computed
  from the log on its machine, with nothing in the repository to edit.
- Good: flume's launch (`flume-asks-what-applies`) and a Claude Code
  plugin (`hook-before-each-edit`) each call one command, its JSON
  versioned.
- Bad: an agent's harness must be told to run `boom query`; until the
  plugin and flume's launch do it, a repository's own agent settings
  name the hook.
- Bad: `.config/boom.toml` is in the tree a change edits, so a change
  could name another repository for its agent's context; the check that
  trusts nothing of the tree is `boom check`'s, a later change.
- Neutral: a query replays the whole log, linear in its entries, which
  a verified checkpoint can later shorten without changing an entry.
