# Boom Storage

## The shift (LEANING — Chuck's idea)

**Before:** Boom state as GNU recutils recfiles in a dedicated Git "Boom repo" per drive.

**Now:** Boom as a self-contained tool: a CLI with a local database, optionally synced to the cloud. Inspired by DeltaDB's argument that fast-evolving agent state doesn't belong in Git.

Reasons:
- A separate Git repo just to hold cross-system claims was always awkward.
- Boom works on its own; the cloud adds many hosts sharing one store.
- Review goes through the rail at agent speed, so Git PR review isn't needed.

**Split (LEANING):**
- **Repo-level specs** stay in each repo with OpenSpec, next to the code.
- **Cross-system claims** live in Boom's database.

## Append-only log + materialized views (LEANING — Chuck agreed with SQLite)

- **Log = source of truth.** Every change is an entry: `create`, `revise`, `supersede`, `amend`, `approve`. Fields: sequence number, entity type, record ID, revision, author, timestamp, payload. Nothing is edited in place.
- **Revisions:** each record has a stable ID + revision number; a new revision points at the one it replaces.
- **Views:** one table per entity type holding the current revision of each record. Agents and the planner query these.
- **Entity types become the schema**: they define both log payloads and view tables. Recfiles disappear as storage (at most an export format).
- **Graph index** is derived from the views in the same database.

## Why SQLite (LEANING)

- One file holds log, views, and graph index; each append updates views in the same transaction, so they can't drift.
- WAL mode: many local agents read while one writer appends; locking handled.
- Parity with the cloud: Durable Objects use SQLite, so local and cloud share a schema. Sync = "rows after sequence N."
- Alternative considered: JSONL per drive (simple, readable) — rejected as primary store because locking, crash safety, and indexing would be hand-rolled.

## Export (PROPOSED)

`boom export` writes JSONL or recfiles for review, backup, or anyone who wants it in Git.

## Tamper evidence (PROPOSED)

Each log entry includes the hash of the previous entry → hash-chained, tamper-evident log. Feeds signed attestation / SOX-style audit goals and links up with Timber.

## Long content: versioned + content-addressed (PROPOSED)

- **Record layer:** stable ID + revision (e.g. narrative N12 @ rev 3) — what the graph and log refer to.
- **Content layer:** immutable blob stored under its hash; each revision points at a hash; identical content stored once.
- Benefits: integrity proof, cheap sync (hosts fetch only missing blobs), stable revision-pinned pointers in packets.

## Blob location (LEANING — Chuck agreed pluggable is overcomplicated)

- Boom owns its blobs: local files or a SQLite table standalone; synced to **R2** behind the drive when attached.
- No pluggable backends, no dependency on Timber.
- Content addressing keeps the door open to move big assets (Timber, customer bucket) later without changing pointers.

## Live state stays out of Boom (PROPOSED)

Fast-changing bolt status, leases, agent progress belong to Flume, not Boom's log.
