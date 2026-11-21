## Why

On 2026-11-21 the user ran `boom accept` on 56 claims. The user typed
the passphrase and touched the key 56 times, one time for each claim.
`boom accept` signed each version as an entry of its own.

The claim `a-repository-moves-its-records-and-rules-into-boom-whole-and-a-person-accepts-many-with-one-signature`
decided on 2026-11-18, in its point 4: "`boom accept` that names several
versions is one act of the person. The person signs it one time." Boom
did not build that point.

## What Changes

- `boom accept` that names several versions writes one `accepted`
  entry. The entry names each version, in the order given. The key of
  the person signs that entry one time.
- Boom judges the entry whole before the key signs it. When boom refuses
  one version, it refuses the act, the key signs nothing, and the log
  does not change.
- An accept names each claim one time. Boom refuses an accept that names
  two versions of one claim.
- A version that the person accepted already is left out of the entry.
  When the person accepted each version already, boom appends nothing
  and the key signs nothing.
- The views, a full replay and `boom verify` read the entry as an accept
  of each version that it names. An entry that boom refuses counts for
  none of its versions.
- An accept of one version keeps the form of its entry: `name` and
  `version`. An accept of several names them in `versions`. Each entry
  in a store now reads as it did.
- `boom log` gives each version of the entry on its line. With `--json`,
  each entry gives `targets`, the list of its versions. `target` stays,
  for an entry of one version.
- `boom show <number>` of an entry that accepts several versions refuses
  and lists them.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `persons-acts`: several versions accepted together are one entry,
  signed one time.
- `record-log`: a line of the log names each version of its act, and
  `targets` gives them.
- `record-import`: the accept of an import appends one entry.

## Impact

- `crates/boom/src/entry.rs` (`Act::Accepted` names its versions,
  `Named`, the two forms of the entry), `judge.rs` (the check and the
  apply of each version), `store.rs` (the scope and the view rows),
  `cli.rs` (`accept`, `log`, `show`), `examples/scale.rs`, and the
  tests `persons.rs`, `import.rs` and `support/mod.rs`.
- `boom-log/1` gets the field `targets`. This is an addition, so the
  major version stays.
- No entry in a store changes. A store that holds an entry of several
  versions needs this boom or a later one to read it.
