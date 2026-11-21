## Why

Turn 4 of the roadmap of Boom is the first edges of the graph. A claim
names the claims that it rests on in the text of its body. Boom does
not check such a name, and it cannot tell which version the name
meant. On 2026-11-21 the user ruled that a claim cites another claim
only by its name in the store (`claims-cite-claims-by-name`). The user
also ruled that a claim links to its origin
(`a-claim-links-to-its-origin`), and that a link remembers the version
that it meant (`a-link-remembers-the-version-it-meant`). The same day,
the user named the two new links `origin` and `source`. The user kept
`amends` and `changes`. A tool names things with the terms of its
domain (`names-use-the-domains-terms`).

## What Changes

- A version stores its edges in its own entry. The entry that makes an
  edge holds it, and no other record does. The edges of this change
  are these:
  - `amends`: the version that it amends, its base. Boom keeps this
    edge now.
  - `source`: the claims that it rests on. The "Source:" line of a rule
    and this edge are one thing.
  - `changes`: for a decision, the claims that it changes.
  - `origin`: the file or the commit that it came from, such as a
    mockup, an exploration or the output of a session.
- Each edge to a claim names the version that it meant. `--source
  <name>` stores the accepted version of that claim, or its newest
  version when none is accepted. `--source <name>@<version>` stores
  that version. `--changes` takes the same two forms.
- Boom refuses an edge to a claim or a version that the store does not
  hold. Boom refuses an edge in any other form, such as a URL, a path of
  a file or the number of an old record.
- An `origin` edge names an artifact in one of two forms:
  - `--origin <file>`: Boom keeps the bytes of the file under their
    fingerprint, with the name of the file.
  - `--origin git:<repository>@<commit>:<path>`: a file in a commit of a
    repository. The commit has its full hexadecimal name. The path
    names the file in that commit, so an agent opens it with
    `git show <commit>:<path>`.
- The edges are part of the content of the version, so a person signs
  them. A version with no edge other than its base keeps the
  fingerprint that it has now.
- `boom show <name>@<version>` gives each edge of the version. For an
  edge to a claim, it gives the statement of the version that the edge
  meant. It marks the edge out of date where another version of that
  claim is accepted now. For each `changes` edge, it gives `Before:`
  and `Now:`.
- `boom show <name>` gives its other versions as a tree by base, each
  amendment under the version that it amends. It also gives the claims
  that name the claim as a source, and the decisions that change it.
  Boom builds that answer from the edges when asked, and keeps no graph
  as the truth.
- `boom show --artifact <fingerprint>` writes the bytes of an artifact
  that Boom keeps, so an agent follows the edge back to it.
- The MCP tool `propose` takes `source`, `changes` and `origin`. The
  tool `show` takes `artifact`.
- Turns 5 and 6 add the edges `verdict`, `covers` and `touches`. This
  change does not.

## Capabilities

### New Capabilities

- `claim-edges`: the edges of a version, the version that each edge
  meant, the artifacts that Boom keeps, and how `boom show` gives the
  edges of a version.

### Modified Capabilities

- `claim-versions`: the content of the fingerprint includes the edges.
  `boom show <name>` gives the other versions as a tree by base, the
  claims that name the claim as a source, and the decisions that change
  it.
- `agent-tools`: the tool `propose` takes `source`, `changes` and
  `origin`, and the tool `show` takes `artifact`.

## Impact

- `crates/boom`: `claim.rs` (the edges and their checks), `entry.rs`,
  `store.rs` (a view row for each edge, and the artifacts as blobs),
  `cli.rs` (`propose` and `show`), `mcp.rs`, and their tests.
- `boom-propose/2` and `boom-show/2` get the fields `source`, `changes`,
  `origin`, `source_of` and `changed_by`. These are additions, so the
  major versions stay.
- Each version that is in a store now keeps its fingerprint.
- The open change `boom-keeps-terms` modifies three requirements that
  this change also modifies: the fingerprint and `boom show` of
  `claim-versions`, and the tools of `agent-tools`. This change takes
  the text of `boom-keeps-terms`. Archive `boom-keeps-terms` first.
