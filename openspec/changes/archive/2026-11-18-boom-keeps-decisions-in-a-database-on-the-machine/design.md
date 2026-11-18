# Design

## Context

`crates/boom` is one binary that answers `--version` (`src/main.rs`).
Its moon project runs format, lint, type check and test, each one task
(core.5), against the toolchain `.prototools` pins (core.6). The gate
runs in sluice's sandbox with no network past localhost
(`.config/sluice.toml`), reading crates from cargo's registry as
flume's crates are read. `openspec/specs/` is empty: this change writes
boom's first capabilities.

What it builds is record 0005, points 1, 2, 3 and 6, for a machine on
its own, with the person's acts as the interview settled them and
record 0006 signs them. The rules it relies on: core.1 (a status read
from events, an event appended and never changed), core.2 (every view
computed from the log), core.3 (no credential in the tree), core.5 and
core.6 (one crate, one project, its tools pinned). It reaches no control
plane: a drive ordering the entries is out of this change.

The family, read for what boom does alike: flume keeps its state under
the data home (`~/.local/share/flume/`), answers JSON with a refusal as
`{"refused": {"reason": ...}}`, and names times in Unix seconds; timber,
sluice and weir sign as their own identity in in-toto statements in
DSSE envelopes. boom's store follows flume's place on disk, its refusal
shape and its times. Its person's signature is a different act from the
tools' own (a person, present, not a host), which record 0006 weighs;
it can later be carried whole inside the family's attestation, as
timber 12 carries another party's claim.

## Goals / Non-Goals

**Goals:**

- One engine, a library in `crates/boom/src/lib.rs`, with the binary a
  thin command line over it (`one-engine-every-command-json`).
- A store whose every answer is computed from its log, and whose log
  shows any entry changed or removed.
- Every scenario of the five specs proven by a named cargo test of
  `boom:test`, with no network, no hardware key and no OpenSSH.

**Non-Goals:**

- A drive ordering the entries, an outbox, or sync (record 0005, point
  4): Boomdrive's side, and a later change of boom's.
- `boom export` (record 0005, point 7), `boom query`, supersession of
  one claim by another, narratives, contexts, relationships, terms, and
  numbers given at acceptance (`number-given-at-acceptance`).
- Roles finer than owner and person, revoking a key, and settings an
  owner signs (`ledger-settings-owner-signed`).
- The site's pages, which keep their sketch until a change of their own.

## Decisions

### 1. One crate: a library and its binary

The crate gains `src/lib.rs`, the engine, beside `src/main.rs`, which
calls `boom::cli::run(args, &keys, &env)` and exits with its code. The
modules: `store` (the file, its schema, transactions, blobs, views),
`entry` (an entry's form, signed part and chain), `claim` (names,
statements, versions), `judge` (the rules that decide whether an entry
counts, and the state they build), `sign` (the signer seam and
verification) and `cli` (arguments, answers, exits). `run` takes the
arguments, the keys seam (decision 8) and an `Env` holding
`XDG_DATA_HOME`, `HOME` and the clock an entry's `at` is read from, so a
test runs any command, at any time it names, with no process
environment changed. Not taken: a second crate for the engine, a moon
project of its own (core.5) for no caller outside this crate yet.

### 2. SQLite through rusqlite, its SQLite bundled

The store is one SQLite file, opened through `rusqlite` with the
feature `bundled`, so every host runs the SQLite `Cargo.lock` pins
(core.6) and none reads the system's. `boom init` sets `journal_mode`
to WAL, which the file keeps, and `user_version` to 1, the schema's
version. Every connection sets a `busy_timeout` of ten seconds, and
every write begins `IMMEDIATE`, so a second writer waits for the first
and readers read beside it (record-store, "Many readers beside one
writer"). The path is `--store`, else `$XDG_DATA_HOME/boom/boom.sqlite`,
else `$HOME/.local/share/boom/boom.sqlite`, as flume's data home is
read. Not taken: JSONL files (record 0005, option 3); the system's
SQLite, whose version differs by host.

### 3. The schema

| table | holds | kind |
|---|---|---|
| `log` | `seq INTEGER PRIMARY KEY`, `prev`, `hash`, `signed` (the entry's signed part, exact text), `signature` (armored, or null) | the truth |
| `blobs` | `fingerprint TEXT PRIMARY KEY`, `bytes BLOB` | content the log names |
| `views_head` | one row: the `seq` and `hash` the views were built at | view |
| `persons` | name, public key, owner, the entry that registered them | view |
| `claims` | name, kind, standing version or null | view |
| `versions` | name, version, base, statement, scenarios, body fingerprint, proposer, the entry that proposed it, status, the person who decided it and the entry, reason | view |
| `judgments` | for each entry: counted or not, and why not | view |

The views are dropped and built again from the log by one function,
`judge::replay`, the same that judges a write (decision 6). A blob is
not a view: the log names it by fingerprint, and its bytes are checked
against that name whenever it is read.

### 4. An entry: its signed part and its chain

An entry's signed part is one JSON object, written by `serde_json` from
a struct whose fields are in a fixed order: `format`
(`boom-entry/1`), `act` (`registered`, `proposed`, `accepted`,
`rejected`), `at` (Unix seconds), `actor` (`{"person": <name>}` or
`{"agent": {"provenance": <text>}}`) and the act's fields: a
registration's `person`, `key` (the OpenSSH public key line) and
`owner`; a proposal's `claim` with `name`, `kind`, `version`, `base`,
`statement`, `scenarios` and `body` (a fingerprint or null); an accept's
`name` and `version`; a reject's `name`, `version` and `reason`. The
text is stored exactly as signed and never written again, so nothing
depends on writing JSON the same way twice.

An entry's hash is the SHA-256 of its `prev` (64 hexadecimal
characters), a newline, its signed part, a newline and its armored
signature or nothing. The first entry's `prev` is the SHA-256 of no
bytes, `e3b0c442…b855`. The number and the chain stay out of the signed
part, so an entry keeps its signature wherever a drive later orders it
(record 0006, point 4). On every open boom reads the log in order and
checks each `prev` and `hash`; the first that fails makes every command
exit 3 naming its `seq` (record-log, "A changed or removed entry is
found"). Not taken: one JSON text holding the signature, which would
make a verifier write the JSON again to find what was signed.

### 5. A version is the fingerprint of its content

A version's name is the first 12 hexadecimal characters of the SHA-256
of `{"kind", "name", "statement", "scenarios", "body", "base"}`, written
in that order, so the same words proposed again are the same version
and add nothing (`no-prompts-repeats-add-nothing`), and two machines
proposing to one base never take one number for two texts. Twelve
characters are 48 bits among one claim's versions. Not taken: a counter
per claim (`@3`), which two proposals waiting side by side would both
take.

### 6. One judge, at the write and on every read

`judge` holds the rules of the specs as one function of the state built
so far and the next entry, answering `Counts(effect)` or `Refused(why)`.
A write asks it before it signs, so a person is never asked to touch
their key for an act that will be refused, and again inside the
`IMMEDIATE` transaction, where it appends the entry and applies the
effect to the views together. A rebuild replays every entry through it,
writing each `judgments` row, so an entry no write would have made
counts for nothing and `boom log` says why (`validated-on-read`). The
views are rebuilt before any answer whenever `views_head` is missing or
is not the log's last entry. A repeat, the same version proposed, the
same accept by the same person, the same registration, is found by the
judge before anything is signed, and answered from the state with
nothing appended; one written by hand counts for nothing, "repeats
entry N".

The rules, in the judge's order for each entry: its form; for a
person's entry, a signature that verifies by the key registered to that
person before it, with presence (for the first entry, the key it
registers); for an agent's, an act that is `proposed` and a provenance;
then the act's own rules as the specs give them.

### 7. A person's signature

As record 0006 has it: OpenSSH's signature format, namespace `boom`,
hash SHA-512, by a key of type `sk-ssh-ed25519@openssh.com` or
`sk-ecdsa-sha2-nistp256@openssh.com`. boom verifies with the `ssh-key`
crate (features `ed25519` and `p256`), which checks a security key's
signature over its application, flags and counter, and then reads the
flags byte itself, refusing a signature whose user-presence bit (`0x01`)
is unset. `boom log --json` gives each entry's `signed` text and
`signature`, which `ssh-keygen -Y verify -n boom` checks with no boom.

### 8. The keys seam

`sign::Keys` turns a `--key <path>` into a `Signer`, which gives its
public key and signs bytes into an armored SSH signature. The binary's
`SshKeygen` reads the public key from `<path>.pub` and runs
`ssh-keygen -Y sign -n boom -f <path>` with the signed part on standard
input and the signature on standard output; a program that cannot be
run, or exits non-zero, is "the key's program did not sign", exit 3.
Its program is a field, `ssh-keygen` by default, so a test points it at
a path that does not exist. The tests' `SoftKeys`, in
`crates/boom/tests/support/mod.rs`, makes security keys in software: an
Ed25519 key from a seed the test names, its application `ssh:`, signing
as an authenticator does, `sha256(application) ‖ flags ‖ counter ‖
sha256(message)`, with the flags the test chooses, wrapped as an SSH
signature; and plain `ssh-ed25519` keys for the refusals. No key is
committed and none signs anything outside a test's store (core.3). Not
taken: an environment variable naming the signing program, which would
let anything that sets it choose what signs.

### 9. The command line

`clap` with derive parses `init`, `register`, `propose`, `accept`,
`reject`, `show` and `log`, each with `--store` and `--json`. An answer
is a value the engine returns; `cli` prints it as words on standard
output, or with `--json` as one object on standard output with every
word on standard error. Its exit is 0 for an answer, 2 for a refusal and
3 for not run; 1 is the check's, which no command here gives. An
argument `clap` cannot read is a refusal, printed as
`boom-refused/1` where `--json` was among the arguments. The formats:
`boom-init/1`, `boom-register/1`, `boom-propose/1`, `boom-accept/1`,
`boom-reject/1`, `boom-show/1`, `boom-log/1`, `boom-refused/1` and
`boom-not-run/1`.

### 10. The tests

Five files under `crates/boom/tests/`, one per capability, `store.rs`,
`log.rs`, `claims.rs`, `persons.rs` and `command_line.rs`, plus
`signatures.rs` for the seam, each test named for its scenario and run
through `boom::cli::run` in a temporary folder, with `SoftKeys`. A test
that alters the file opens it with `rusqlite` itself. The binary is run
only where a scenario needs the process: `--version`, and a command with
its standard input closed.

## Risks / Trade-offs

- [The whole chain is hashed on every open, and the log replayed when
  the views are behind] → linear in the log, small while a store holds
  decisions; a verified checkpoint can come later without changing an
  entry.
- [A program that edits the view tables and leaves `views_head` alone
  makes boom answer from them] → the views are never the truth (core.2):
  deleting them rebuilds them, and the log's chain and signatures stay
  whole; an agent can no more forge a person's accept this way than by
  appending one.
- [A program that rewrites the whole log and its chain from some entry
  on is not found on one machine] → the person's acts in it still need
  their signatures; a drive holding every entry (record 0005, point 4)
  is what finds it.
- [The gate reads crates from cargo's registry with no network] → the
  task that adds the dependencies fetches them outside the sandbox, as
  flume's are; a host whose registry lacks them fails the gate until it
  fetches.
- [A person's machine lacks an OpenSSH with security key support] →
  `boom accept` exits 3 with `ssh-keygen`'s words; record 0006 names it.
