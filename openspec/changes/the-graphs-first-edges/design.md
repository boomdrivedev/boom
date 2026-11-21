## Context

Turn 4 of the roadmap of Boom builds the first edges of the graph:
`amends`, `source`, `changes` and `origin`. Turns 5 and 6 add
`verdict`, `covers` and `touches`. The rulings of the user of
2026-11-21 set the shape: `claims-cite-claims-by-name`,
`sources-outside-boom-wait-for-their-import`,
`a-claim-links-to-its-origin`,
`a-link-remembers-the-version-it-meant` and
`names-use-the-domains-terms`. The user named the link to a file or a
commit `origin`, and the link to a claim that a claim rests on
`source`. The user kept `amends` and `changes`.

A version holds a statement, a body and a base (`claim.rs`). Its body
names its sources as text, on a "Source:" line. The views keep the
claims and their versions, and boom builds them from the log
(`views-computed-from-the-ledger`). Turn 3 (`boom-keeps-terms`) added
the kind `term`, with its repository and `replaces` in the content of
the fingerprint, and `boom show <word> --repository <name>`.

## Goals / Non-Goals

**Goals:**

- Each edge is data in the entry of the version that makes it.
- Each edge to a claim names the version that it meant.
- An agent follows an `origin` edge back to its artifact.
- A person sees the versions of a claim by base, what a claim rests on,
  and which claims name it as a source.

**Non-Goals:**

- The edges `verdict`, `covers` and `touches` (turns 5 and 6).
- Edges that an import makes from the "Source:" line of a record.
- An edge to anything but a claim of the store, a file that Boom keeps,
  or a file in a commit.
- A graph that Boom keeps. Boom builds the graph when asked.

## Decisions

### 1. The entry that makes an edge stores it

A version's entry holds its edges, as it holds its base now. No other
record holds them, so the edge and the act that made it are one signed
thing. The views keep a row for each edge, built from the log as each
view is.

Alternative: a record of its own for each edge. Then an edge can change
apart from the version that made it, and the person who accepted the
version did not sign it.

### 2. The names of the edges

The names come from the words that developer tools use
(`names-use-the-domains-terms`). `origin` is where a thing came from,
as a git remote or a file of a build. `source` is what a claim rests
on, and a rule says it on its "Source:" line. The edge and that line
are one thing. `amends` and `changes` stay, as the user ruled.

The flags are `--source`, `--changes` and `--origin`. The fields of the
JSON and of the MCP tool `propose` have the same names. The reverse of
`source` is `source_of`, the claims that name a claim as a source. The
reverse of `changes` is `changed_by`, the decisions that change a
claim.

### 3. An edge to a claim stores a version

`--source <name>` resolves the name when boom judges the proposal. Boom
stores the accepted version of the claim, or its newest version when
none is accepted. So the entry always holds `<name>@<version>`, and the
version that the edge meant never changes. Boom compares that version
with the accepted version of the claim, and marks the edge out of date
where they differ. `--changes` resolves its name in the same way.

Alternative: store the name only, and follow the claim. Then a person
cannot tell what the edge meant when the claim changes.

### 4. Boom checks each edge to a claim when an agent proposes

A name must be a claim of the store, at any status. A version must be a
version of that claim. Boom finds no claim by the number of an old
record or rule.

Alternative: refuse an edge to a claim that is not accepted. Then a
record that names another record of the same import as a source cannot
come in until the person accepts the other first.

### 5. Two forms of an origin

An artifact that Boom keeps is a blob, as a long body is now. Boom
keeps its bytes under their SHA-256 fingerprint, with the name of the
file. A file in a commit is `git:<repository>@<commit>:<path>`. It
holds the name of the repository as Boom names repositories. It also
holds the full hexadecimal name of the commit, and the path of the file
in that commit. An agent that builds the claim opens the file itself,
with `git show <commit>:<path>` in that repository. Boom checks the
form of the reference and reads no repository, so it works offline.

Alternative: a URL or a path. The user ruled them out on 2026-11-21.
They point at things that move or go away.

### 6. An agent reads an artifact through boom show

`boom show --artifact <fingerprint>` writes the bytes of the artifact.
It is a part of `boom show`, so it needs no new command, and the MCP
tool `show` serves it. `boom show` takes a target or an artifact, and
refuses both or neither. So the tool `show` requires neither `target`
nor `artifact`, and boom refuses a call with both or neither.

### 7. Only a decision changes claims

`changes` is an edge of a decision only. An invariant or an assumption
that replaces another is an amendment, with `--amends`.

### 8. Before and Now

For a `changes` edge, `boom show` gives `Before:`, the statement of the
version that the edge meant, and `Now:`, the statement of the decision.
The decision is the new state, so its statement says what holds now.

### 9. The edges are part of the content, only when there are edges

The content of the fingerprint includes `source`, `changes` and
`origin` only when one of them holds an edge. The entry omits empty
lists. So each version in a store keeps its fingerprint. The
requirement of the fingerprint in `claim-versions` says so, on the text
that turn 3 gave it.

### 10. The graph is built when asked

`boom show <name>` reads the edge rows that name the claim, and the
versions of the claim. It gives the other versions as a tree by base.
The JSON keeps `versions`, and each version names its `base`, so it
needs no new field for the tree. `boom show <name>` also gives
`source_of` and `changed_by`. A claim names another as a source when
its accepted version holds a `source` edge to it. With no accepted
version, the newest version counts. Only edges count. A name in the
text of a body does not.

## Risks / Trade-offs

- [An edge meant a version that a person later replaced] → `boom show`
  marks it out of date. A new version of the claim names the new
  version as its source.
- [A commit that is gone] → Boom does not see it, because it reads no
  repository. The reference still names what the claim came from.
- [Large artifacts] → Boom keeps each artifact one time, by its
  fingerprint, as it keeps long bodies.
- [Sources as text in bodies] → The claims of 2026-11-21 keep their
  "Source:" lines as text. A later version of each claim gives them as
  `--source`.
- [Three requirements that turn 3 also modifies] → This change takes the
  text of `boom-keeps-terms`. If `boom-keeps-terms` archives after this
  change, its text replaces the edges. Archive `boom-keeps-terms` first.
