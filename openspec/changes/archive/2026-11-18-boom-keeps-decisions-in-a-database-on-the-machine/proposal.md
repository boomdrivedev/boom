# Proposal

## Why

boom is designed and not built: the binary prints its version and
nothing more, so a project's decisions and rules still live scattered in
files no tool keeps (signal
`2026-11-18-boomdrive-operator-5f02a78a/03-boom-replaces-the-scattered-context`).
Record 0005, accepted by the user on 2026-11-18, settles where boom keeps
them: a database on each machine, an append-only log with versions and
long text by fingerprint, in place of recfiles on a Git branch. This
change builds that store on one machine, standalone, so the rest of the
bolt `boom-holds-the-rules-agents-read` has something to read.

## What Changes

- boom keeps its records in one SQLite file on the machine,
  `$XDG_DATA_HOME/boom/boom.sqlite` unless `--store` names another: the
  log, the long text by fingerprint, and the views of the current
  records, each append updating them in one transaction (record 0005,
  points 1 and 3).
- Every change is an entry appended to the log and never edited. Each
  entry carries the hash of the one before it, so a changed or removed
  entry is found on every read; boom numbers the entries itself (record
  0005, points 2, 4 and 6).
- A claim, an invariant, a decision or an assumption, keeps one name for
  life and a version for each change, each version naming the one it
  amends. Whether a version is proposed, standing, replaced or rejected
  is read from the log, never stored (core.1, core.2).
- An agent proposes; only a person accepts or rejects. A person's act is
  signed with their security key, the person present, by a key an owner
  registered; the first owner registers their own key when the store is
  made. An entry that fails these checks counts for nothing, at the
  write and on every read (interview: `agents-propose`,
  `accept-by-role-never-agent-never-author`, `reject-is-a-persons`,
  `persons-acts-signed-with-presence`, `self-registered-first-owner`,
  `amend-names-its-base`, `validated-on-read`; record 0006).
- The command line gains `boom init`, `boom register`, `boom propose`,
  `boom accept`, `boom reject`, `boom show` and `boom log`, each with
  `--json`, exiting 0, 2 or 3 as the interview's `exit-codes` has it.

## Capabilities

### New Capabilities

- `record-store`: the database file on the machine, where it is, its
  views rebuilt from the log, many readers beside one writer, and long
  text kept once under its fingerprint.
- `record-log`: the log that is never edited, its order, its chain, and
  every entry validated when it is written and whenever it is read.
- `claim-versions`: claims by name, their versions and amendments, one
  name space, and the status of each version read from the log.
- `persons-acts`: persons, owners and agents; a person's act signed by a
  registered security key with the person present; who may propose,
  accept and reject.
- `command-line`: the commands' arguments, their one JSON object, their
  exit codes, no prompts, and a repeated write adding nothing.

### Modified Capabilities

None: `openspec/specs/` holds no capability yet.

## Impact

- `crates/boom`: gains a library, the engine, beside its binary, which
  becomes a thin command line over it; new dependencies `rusqlite` (its
  SQLite bundled), `ssh-key`, `sha2`, `serde`, `serde_json` and `clap`,
  and `ed25519-dalek` and `tempfile` for its tests, locked in
  `Cargo.lock`.
- Signing a person's act runs that person's `ssh-keygen -Y sign` on
  their machine; no gate task runs it, so the gate pins no OpenSSH.
- `README.md` and `CLAUDE.md` describe the crate as it is built.
- `docs/adr/0006-a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key.md`
  records the signature's form.
- Out of this change: a drive ordering the entries (record 0005, point
  4), `boom export`, supersession of one claim by another, the
  narratives, contexts, relationships and terms, `boom query`, and the
  site's pages.
