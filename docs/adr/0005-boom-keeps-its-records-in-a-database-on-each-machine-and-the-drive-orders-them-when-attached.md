---
number: 5
title: Boom keeps its records in a database on each machine, and the drive orders them when attached
status: proposed
date: 2026-11-18
decision-makers:
- Chuck Swanberg
- boomdrive-design
---

# Boom keeps its records in a database on each machine, and the drive orders them when attached

Status: proposed to the user on 2026-11-18, as `briefs.15` has it; accepted only on their yes.

Boom keeps decisions and rules in a database on each machine, not in a Git repository. Every change is added to a log that is never edited, each record keeps its earlier versions, and long text is stored once under its fingerprint. On its own, Boom numbers the changes itself. Attached to Boomdrive, the drive in the cloud puts every change in order, every machine keeps a full copy, and the runner carries changes up and down. You review on the rail, not in pull requests. Each repository's specs stay in that repository; Boom holds the decisions and rules that cross repositories. This replaces the plan of 1 November to keep Boom's records as text files on a Git branch.

Deciders: the user, in the design session of 2026-11-15 (`design/research/2026-11-15-planning-and-boom-storage/`), for the move off Git, which was their idea, and SQLite, which they agreed to; for review on the rail rather than in pull requests; for Boom in the cloud being Boomdrive itself; for a local copy on every machine with the cloud ordering writes; for no machine being Boom's special host in the cloud; and for Boom keeping its own long text, with no pluggable stores and no dependency on timber. boomdrive-design, for points 6 and 7, adopted from that session's suggestions, each cheap to change.

Sources: `design/research/2026-11-15-planning-and-boom-storage/`, files 04, 05 and 06; `interview.md`, the user's answers of 2026-11-01; core.1 and core.2; boomdrivedev/blueprints `design/principles.md` 5 and 7.

## Context and problem statement

On 2026-11-01 the user settled that Boom's ledger is plain-text recfiles on a shared Git branch, written by the family's shared-branch protocol, with `boom sync` handing each repository its files by one pull request (`interview.md`: `ledger-on-a-shared-branch`, `ledger-written-by-the-family-protocol`, `records-name-their-format`, `sync-by-one-pull-request`, `slice-only-as-the-ledger-gives`). In the session of 2026-11-15 the user turned from that: a separate Git repository only to hold claims was always awkward, agent state that changes fast does not belong in Git, and review happens on the rail at agent speed, so no pull request is needed.

## Considered options

1. A database on each machine, an append-only log with a view per record type and blobs by fingerprint, ordered by the drive when attached (chosen).
2. Recfiles on a shared Git branch, as settled on 2026-11-01.
3. JSONL files per drive.

## Decision outcome

1. **A database on each machine.** Boom keeps one SQLite file per drive on each machine. It holds the log, a table per record type with the current version of each record, and the index of links between records, each append updating all of them in one transaction, so they never disagree. Many readers read while one writer appends. Not taken: recfiles on a Git branch (option 2), the awkward separate repository the user turned from; JSONL files (option 3), whose locking, crash safety and indexing would be written by hand.
2. **A log never edited, and records with versions.** Every change is an entry of the log: a record proposed, revised, superseded, amended or accepted, with its order, its record type, the record's name and version, its author and its time. A record keeps one name for life and a version for each change, each version naming the one it replaces. The current version of a record is read from the log, never stored as the truth (core.1, core.2): the tables are views the log rebuilds.
3. **Long text stored once, by fingerprint.** A narrative, a design or a mockup is a blob stored under the hash of its content, and each version of its record names a hash. Standalone, Boom keeps its blobs itself; attached, they are kept in the drive's storage in the cloud. No pluggable stores, and no dependency on timber.
4. **Attached, the drive orders every change.** Boom in the cloud is Boomdrive: there is no other Boom service. Attached to a drive, a machine's new entries wait as proposals until the drive's own object in the cloud gives them their order and runs their review on the rail, and every machine then takes the entries it has not seen. No machine is special, and any machine may propose. The runner carries the proposals up and the log down, so Boom needs to know little of the cloud. Standalone, Boom orders its own entries. Several machines sharing one store without the cloud is not settled.
5. **Repositories keep their specs; Boom keeps what crosses them.** Each repository's specs stay beside its code, in OpenSpec. Boom holds the decisions, rules, narratives and context maps that cross repositories.
6. **The log is chained.** Each entry carries the hash of the one before it, so a changed or removed entry is found when the log is read, as `validated-on-read` asks (boomdrive-design's).
7. **What changes by the second stays out.** A bolt's live status, a lease and an agent's progress belong to Flume, never Boom's log. `boom export` writes the log as JSONL or recfiles for review, backup or Git (boomdrive-design's).

What the session of 2026-11-01 settled and this keeps: an agent proposes and only a person accepts or rejects (`agents-propose`, `accept-by-role-never-agent-never-author`, `reject-is-a-persons`); a person's act is signed (`persons-acts-signed-with-presence`); an import proposes and a person accepts (`import-proposes-a-person-accepts`); an event that fails Boom's checks counts for nothing (`validated-on-read`); and nothing kept beside the log is read as the truth (`caches-derived-only`). What it replaces: `ledger-on-a-shared-branch`, `ledger-written-by-the-family-protocol`, `records-name-their-format` (recutils at rest), `sync-by-one-pull-request` and `slice-only-as-the-ledger-gives`.

## Consequences

- Good: Boom works on one machine with nothing else, and agents read decisions and rules locally, quickly and offline.
- Good: the database's tables in the cloud and on a machine are the same, so a machine catches up by reading the entries after the last one it holds.
- Bad: the decisions are no longer readable in Git without `boom export`.
- Open, for the change that builds it: how `boom check` at a merge reads the rules a change must keep, now that no slice is committed to the repository; and whether Millpond, Flume and the signal readers follow the same pattern.
