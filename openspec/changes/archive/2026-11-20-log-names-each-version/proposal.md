## Why

On 2026-11-20 the user ran `boom log`, and then could not use its
numbers with `boom show`. Each line of the log gave a number, an act and
an actor, and no claim. `boom show` took a name or `<name>@<version>`,
and no entry number. A bare name gave a list of versions with no words.

## What Changes

- `boom log` gives `<name>@<version>` on each line that proposes,
  accepts or rejects a version. An example is `30 proposed
  one-resident-per-tool@777b452f0377 by an agent (...)`. With `--json`,
  each entry gets `target`: the same `<name>@<version>`, or `null` for a
  registration.
- `boom show <number>` shows the version that the entry with that number
  names. It gives the same answer as `boom show <name>@<version>`. Boom
  refuses a number that the log does not hold, and the number of an
  entry that names no version.
- `boom show <name>` gives the newest version first, with its statement,
  what it binds and its body. Then it gives each other version, newest
  first, with its statement and what it binds. With `--json`, the
  answer gets `newest`: the newest version with its statement and body.
  The first line now says `accepted: <version>`, or `no version is
  accepted`, in the status words of record 0007 as amended.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `record-log`: a new requirement, each line of the log names the version
  of its act.
- `claim-versions`: `boom show` takes an entry number, and a bare name
  gives the newest version in full.

## Impact

- `crates/boom/src/cli.rs` (`log`, `show`), `entry.rs` (`Act::target`),
  `store.rs` (`row_at` is public), and the tests `log.rs`, `claims.rs`
  and `attachments.rs`.
- `boom-log/1` and `boom-show/2` get one field each. These are
  additions, so the major versions stay.
- The open change `boom-keeps-terms` also modifies "A claim shown by its
  name". The change that lands second takes the text of the first.
