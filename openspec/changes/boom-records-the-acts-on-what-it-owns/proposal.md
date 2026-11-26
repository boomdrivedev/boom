## Why

The lineage is the path of linked entries from a capture to a
promotion. The tool that owns the thing that an act changes records
that act, as an entry of its own type in the drive's log
(`each-tool-records-the-acts-on-what-it-owns`). Boom owns two acts of
the lineage that it does not record: a plan's approval and each version
of a change record. Boom's store is the drive's log
(`the-drive-keeps-one-log-and-each-tool-owns-its-entry-types`), but it
takes no entry of another tool, so millpond, sounding and sluice cannot
write their hops. A link names "the hash of an entry's act", and no
entry gives that hash. Thus no reader can walk a version back to its
intent or forward to its landing.

## What Changes

- **Each entry has an act hash.** The act hash is the SHA-256 of the
  entry's signed part, 64 hexadecimal characters. A link names an
  entry by its act hash. `boom log --json` gives each entry's
  `act_hash` and its `links`.
- **A link names an entry that the log holds.** Boom refuses an entry
  whose link names an act hash that the log does not hold before it.
- **Boom takes another tool's entry.** `boom append <tool>.<type>`
  appends an entry of a type that another tool owns, with its links and
  its content. Boom judges its form, its actor and its links, and
  nothing else of it.
- **A plan's approval.** `boom approve <plan>` appends an `approved`
  entry, signed by the person's key. It names the plan by the SHA-256
  of its document, and keeps nothing of the document. It covers each
  claim at its accepted version and links it, links each elaboration's
  end that the log holds, and names each output commit.
- **A change record.** `boom record change <repository>:<name>`
  appends a `recorded` entry: a version of the change record, with no
  accept. Boom keeps each file of the change's folder by its
  fingerprint. The version links its approval, names its unit and its
  bolt, and covers claims that its approval covers. `boom show
  <repository>:<name>` gives the record, its versions and its status,
  `written` or `landed`.
- **The walk.** `boom lineage <target>` gives each entry that the
  target reaches through its links, back to the first, and each entry
  that reaches the target, forward to the last.
- **`boom mcp`** serves `record_change` and `lineage`, and names
  `approve` and `append` absent.
- **The views take a new form**, 11, with the act hash, the links and
  the change records.

## Capabilities

### New Capabilities

- `plan-approvals`: a person's signed approval of a plan, and what it
  links.
- `change-records`: the versions of a change record, its files, its
  links, its status and how `boom show` gives it.
- `lineage`: another tool's entry in the log, and the walk back and
  forward through the links.

### Modified Capabilities

- `record-log`: the act hash of an entry, and a link that names an
  entry that the log holds.
- `command-line`: the commands `approve`, `record change`, `append`
  and `lineage`.
- `agent-tools`: the tools `record_change` and `lineage`.

## Impact

- `crates/boom/src/entry.rs`: the acts `approved` and `recorded`, an
  entry of another tool's type, and the act hash.
- `crates/boom/src/judge.rs`: the checks of each new act and of each
  link.
- `crates/boom/src/store.rs`: the views `acts`, `links` and
  `changes`; the views' form goes from 10 to 11.
- `crates/boom/src/cli.rs`, `crates/boom/src/mcp.rs`: the new commands
  and tools.
- `crates/boom/src/lineage.rs`: the walk.
- `crates/boom/tests/`: a test for each scenario.
- The claims proposed in Boom: `a-link-is-the-hash-of-the-signed-part-it-names`,
  `boom-refuses-a-link-to-an-entry-its-log-does-not-hold`,
  `boom-takes-an-entry-of-another-tools-type-and-judges-only-its-form-and-links`,
  `a-change-records-version-is-the-fingerprint-of-its-files-and-its-links`,
  `a-change-record-covers-only-what-its-approval-covers`, the narrative
  `boom-records-the-acts-on-what-it-owns`, and a new version of the
  narrative `the-links-of-the-lineage`.
- millpond, sounding, sluice and the cloud's planning write their hops
  with `boom append`, each by a change of its own. The design names each
  type and its links.
