## Why

The accepted decision
`a-note-on-a-narrative-reopens-the-elaboration-that-made-it@fd6d59cec2a5`
says that a note on a narrative reopens the elaboration that made the
narrative: the note is the input of that elaboration's next round. A
narrative that no elaboration made sends its note to millpond as a
signal. Boom's side is to know which elaboration made a version, to give
each note its elaboration, and to give an elaboration's open notes to
the agent or the tool that runs its next round.

Today an origin is a file that boom keeps or a file in a commit. No
origin names an elaboration, though
`a-claim-links-to-its-origin@39282a168d6e` says that a claim links to
the elaboration it came from. So a note reaches no elaboration.

## What Changes

- `--origin elaboration:<repository>/<name>` names the elaboration that
  a version came from, as sounding names it. A version names one
  elaboration at most. Boom stores the repository and the name, and
  reads nothing of the elaboration.
- A note's elaboration is the elaboration that the version it is on
  names, or none. Boom reads it from the version and stores it in no
  entry. `boom show` gives each note with its elaboration.
- `boom query --elaboration <repository>/<name>` gives the open notes on
  the versions of that elaboration, each with its narrative, version,
  paragraph, person, words and elaboration. The MCP tool `query` takes
  `elaboration`.
- The claims `origin-and-source-name-the-links-of-a-claim` and the term
  `origin` get new versions, proposed for a person to accept, that name
  the elaboration as an origin.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `claim-edges`: "The origin of a claim" names an elaboration as an
  origin, and a new requirement gives its form.
- `narrative-notes`: a note reaches the elaboration of its version, and
  a query gives the open notes of an elaboration.
- `agent-tools`: the tool `query` takes `elaboration`.

## Impact

- `crates/boom/src/claim.rs`: the origin `elaboration`, its form, and
  one elaboration at most in a version.
- `crates/boom/src/judge.rs`: a note's elaboration, read from its
  version.
- `crates/boom/src/cli.rs`: `--origin elaboration:…`, a note's
  elaboration in `boom show`, and `boom query --elaboration`.
- The tests of `boom:test`: `edges.rs`, `notes.rs` and `mcp.rs`.
- `README.md`: the origin of an elaboration, and the query of its
  notes.
