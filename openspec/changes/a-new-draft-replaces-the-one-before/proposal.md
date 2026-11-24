## Why

Boom accepts a version only when its base is the accepted version of
its claim. Thus a version that amends a draft, a version that no person
accepted, can never be accepted. On 2026-11-23 Boom refused the user's
accept of `new-design-goes-into-boom-not-blueprints@bd68243ee18f` and 12
claims of another session for this reason. Agents now propose each new
draft with no base, and the old drafts stay open beside it.

On 2026-11-23 the user ruled: "Replace: the new draft closes the old
one." A version that amends a draft replaces that draft. The person sees
only the newest draft of a claim, and an accept of it works. It is like
an edit of an email that is not sent, not a reply to it.

## What Changes

- A version that amends a proposed version supersedes it. The entry that
  proposes the new draft is the event that supersedes the old one. No
  new status word comes: the old draft is `superseded`.
- A person cannot accept or reject a superseded draft. Boom refuses the
  act and names the draft that superseded it.
- Boom refuses an amendment of a draft that a newer draft superseded,
  and names the newer draft. A draft has one line of versions.
- An accept checks the nearest version in the line of bases that a
  person accepted, or none. A draft of a draft is accepted when that
  version is the accepted version of the claim. Two amendments of one
  accepted version stay a conflict: when the person accepts one, Boom
  refuses the other.
- A reject of the newer draft leaves the old draft superseded.
- The new draft of a narrative answers each open note, the notes on the
  draft that it supersedes too, as it does now.
- The views take a new form, so each store builds them again from its
  log under the new rule.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `claim-versions`: a new requirement, "A new draft supersedes the
  draft it amends". "An accept stands only on its base" checks the
  nearest accepted version in the line of bases. "Each version's status
  is read from the log" gives `superseded` to a draft that a newer draft
  amends.

## Impact

- `crates/boom/src/judge.rs`: the supersession of a draft when its
  amendment counts, the refusal of an accept, a reject or an amendment
  of a superseded draft, and the base that an accept checks.
- `crates/boom/src/store.rs`: the views' form goes from 8 to 9.
- `crates/boom/tests/claims.rs`: a test for each new scenario.
- The decision `a-new-draft-replaces-the-one-before`, proposed in Boom
  for a person to accept.
- No change to the log's format or to the fingerprint of a version.
  Today's store reads under the new rule: 39 drafts that a newer draft
  amends become `superseded`, and each of its 599 entries still counts.
