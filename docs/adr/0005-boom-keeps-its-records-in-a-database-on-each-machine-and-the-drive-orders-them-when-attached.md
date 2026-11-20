---
number: 5
title: Boom keeps its records in a database on each machine, and the drive orders them when attached
status: accepted
date: 2026-11-18
decision-makers:
- Chuck Swanberg
- boomdrive-design
---

# Boom keeps its records in a database on each machine, and the drive orders them when attached

Status: accepted by the user on 2026-11-18, on their card ("Approved").

Boom keeps decisions and rules in a database on each machine, not in a Git repository. Boom adds every change to a log, and nobody edits the log. Each record keeps its earlier versions. Boom stores long text one time, under its fingerprint. When Boom runs alone, it gives the changes their numbers. When Boom is attached to Boomdrive, the drive in the cloud puts every change in order. Every machine keeps a full copy, and the runner carries the changes up and down. You review on the rail, not in pull requests. The specs of a repository stay in that repository. Boom holds the decisions and rules that cross repositories. This decision replaces the plan of 1 November, which kept the records of Boom as text files on a Git branch.

Deciders:

- The user, in the design session of 2026-11-15 (`design/research/2026-11-15-planning-and-boom-storage/`). The user decided these points:
  - The move off Git, which was the idea of the user.
  - SQLite, which the user agreed to.
  - Review on the rail, not in pull requests.
  - Boom in the cloud is Boomdrive itself.
  - A local copy on every machine, with the cloud putting the writes in order.
  - No machine is the special host of Boom in the cloud.
  - Boom keeps its own long text, with no pluggable stores and no dependency on timber.
- boomdrive-design, for points 6 and 7. boomdrive-design took them from the suggestions of that session. Each one is cheap to change.

Sources:

- `design/research/2026-11-15-planning-and-boom-storage/`, files 04, 05 and 06.
- `interview.md`, the answers of the user of 2026-11-01.
- core.1 and core.2.
- boomdrivedev/blueprints `design/principles.md` 5 and 7.

## Context and problem statement

On 2026-11-01 the user decided that the ledger of Boom is plain-text recfiles on a shared Git branch. The shared-branch protocol of the family writes them. `boom sync` gives each repository its files in one pull request (`interview.md`: `ledger-on-a-shared-branch`, `ledger-written-by-the-family-protocol`, `records-name-their-format`, `sync-by-one-pull-request`, `slice-only-as-the-ledger-gives`). In the session of 2026-11-15 the user turned away from that plan, for three reasons:

- A separate Git repository that only holds claims was always awkward.
- Agent state that changes quickly does not belong in Git.
- Review occurs on the rail at the speed of agents, so no pull request is necessary.

## Considered options

1. A database on each machine: a log that only gets new entries, a view for each record type, and blobs by fingerprint. The drive puts the entries in order when the machine is attached (chosen).
2. Recfiles on a shared Git branch, as the user decided on 2026-11-01.
3. JSONL files for each drive.

## Decision outcome

1. **A database on each machine.** Boom keeps one SQLite file for each drive on each machine. The file holds these:
   - The log.
   - A table for each record type, with the current version of each record.
   - The index of links between records.

   Each append updates all of them in one transaction, so they always agree. Many readers read while one writer appends. Not taken: recfiles on a Git branch (option 2), because they need the awkward separate repository that the user turned away from. Not taken: JSONL files (option 3), because somebody must write their locks, their crash safety and their indexes by hand.
2. **A log that nobody edits, and records with versions.** Every change is an entry of the log: a record proposed, revised, superseded, amended or accepted. The entry holds its order, its record type, the name and version of the record, its author and its time. A record keeps one name for all its life, and it gets a version for each change. Each version names the version that it replaces. Boom reads the current version of a record from the log. Boom never stores it as the truth (core.1, core.2). The tables are views, and the log builds them again.
3. **Long text stored one time, by fingerprint.** A narrative, a design or a mockup is a blob. Boom stores it under the hash of its content, and each version of its record names a hash. When Boom runs alone, Boom keeps its blobs. When Boom is attached, the storage of the drive in the cloud keeps them. Boom has no pluggable stores and no dependency on timber.
4. **When attached, the drive puts every change in order.** Boom in the cloud is Boomdrive. No other Boom service exists. When a machine is attached to a drive, its new entries wait as proposals. The object of the drive in the cloud gives them their order and runs their review on the rail. Then every machine takes the entries that it did not see. No machine is special, and any machine may propose. The runner carries the proposals up and the log down, so Boom must know only a little about the cloud. When Boom runs alone, Boom puts its own entries in order. The decision does not settle how several machines share one store without the cloud.
5. **Repositories keep their specs. Boom keeps what crosses them.** The specs of each repository stay next to its code, in OpenSpec. Boom holds the decisions, rules, narratives and context maps that cross repositories.
6. **The log is a chain.** Each entry holds the hash of the entry before it. Thus when Boom reads the log, it finds an entry that somebody changed or removed, as `validated-on-read` requires (from boomdrive-design).
7. **What changes each second stays out.** The live status of a bolt, a lease and the progress of an agent belong to Flume, never to the log of Boom. `boom export` writes the log as JSONL or recfiles, for review, for a backup or for Git (from boomdrive-design).

This decision keeps these points that the session of 2026-11-01 settled:

- An agent proposes, and only a person accepts or rejects (`agents-propose`, `accept-by-role-never-agent-never-author`, `reject-is-a-persons`).
- A person signs each act of that person (`persons-acts-signed-with-presence`).
- An import proposes, and a person accepts (`import-proposes-a-person-accepts`).
- An event that fails the checks of Boom counts for nothing (`validated-on-read`).
- Boom reads nothing that it keeps next to the log as the truth (`caches-derived-only`).

This decision replaces these points: `ledger-on-a-shared-branch`, `ledger-written-by-the-family-protocol`, `records-name-their-format` (recutils at rest), `sync-by-one-pull-request` and `slice-only-as-the-ledger-gives`.

## Consequences

- Good: Boom works on one machine with nothing else. Agents read decisions and rules on the machine, quickly and offline.
- Good: the tables of the database are the same in the cloud and on a machine. Thus a machine gets the changes that it lacks: it reads the entries after the last entry that it holds.
- Bad: a person cannot read the decisions in Git without `boom export`.
- Open, for the change that builds it: how `boom check` at a merge reads the rules that a change must keep. The repository holds no committed slice. Also open: whether Millpond, Flume and the signal readers use the same pattern.
