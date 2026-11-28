## Why

Boom has no way to take an accepted claim out of force. `boom withdraw`
takes only proposed versions. A decision's `--changes` records only a
link, and `boom query` and `boom search` still give the claim that it
changes. An accepted version leaves force only when a person accepts a
later version of the same claim.

On 2026-11-27 two accepted invariants said nearly the same thing:
`logic-is-tested-as-units-and-few-tests-run-the-binary@3131d0f94b83`,
the one to keep, and
`logic-is-tested-as-units-and-a-few-tests-run-the-binary@7e61f29c49c8`,
the one to retire. The same day the user chose: "Build boom supersede as
the site drafts it". The user then ruled on the case that the draft
does not cover, a duplicate whose claim to keep is accepted already
(`boom-supersede-retires-an-accepted-claim@6cde86c6d5d5`, proposed).

## What Changes

- **`boom supersede <name> <new-name> [<statement>]`**, with `--kind`,
  `--body`, `--reason`, and `--provenance` for an agent or `--key` for a
  person. It proposes one version that supersedes the accepted version
  of `<name>`. The version only waits: `<name>` stays accepted until a
  person accepts the version. An agent may run the command.
- **A new name.** Where `<new-name>` names no accepted claim, the
  version is a first version of `<new-name>`, with `<statement>`, of the
  kind of `<name>` unless `--kind` says otherwise. It binds what
  `<name>` binds.
- **An accepted claim of the same kind.** Where `<new-name>` is an
  accepted claim of the same kind as `<name>`, the version amends its
  accepted version. It copies the statement, the body, the sources and
  the binding of that version unchanged. `<statement>` is optional
  there, and `<statement>` or `--body` replaces the one that it gives.
- **The edge `supersedes`.** A version stores the accepted version that
  it supersedes, `<name>@<version>`, with the reason, in its content.
  Only an invariant, a decision or an assumption supersedes, or is
  superseded.
- **The accept.** When a person accepts the version, the accepted
  version of `<name>` takes the status `superseded`, and `<name>` has no
  accepted version. Thus `boom query`, `boom search` and the hooks no
  longer give it. Boom refuses the accept when that version of `<name>`
  is no longer the accepted one. The log keeps each entry.
- **What the person reads.** `boom accept` lists, before the key signs,
  the claim that each version supersedes, and its answer names the
  version that it superseded. `boom show` gives the edge of a version,
  and the versions that supersede a claim.
- **The answer** of `boom supersede` is `boom-supersede/1`, which names
  `"supersedes": "<name>"`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `claim-versions`: new requirements for `boom supersede`, for its two
  cases, for the edge `supersedes`, for the accept of a version that
  supersedes, and for `boom show` of a superseded claim.

## Impact

- `crates/boom/src/claim.rs`: the field `supersedes` of a version, its
  form, and its place in the fingerprint.
- `crates/boom/src/judge.rs`: the rules of a version that supersedes,
  and of its accept, which takes the accepted version of the other
  claim out of force.
- `crates/boom/src/store.rs`: the edge in the views, the claim that a
  version supersedes read with it, and the claim that an accept
  supersedes written again.
- `crates/boom/src/cli.rs`: `boom supersede`, the accept's list and
  answer, and `boom show`.
- `crates/boom/src/mcp.rs`: `supersede` named absent from `boom mcp`.
- `crates/boom/tests/supersede.rs`: a test for each scenario.
- `README.md`, `CLAUDE.md`, the site's `commands.md` and the skills
  `using-boom` and `proposing-claims`.
- No entry in a log holds the field `supersedes`, so each entry judges
  as before, and the rule set stays 1. A boom before this change refuses
  a version that supersedes, by its form.
