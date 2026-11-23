## Why

Boom keeps claims, but it does not keep the designs that explain them.
`boom propose` gives five kinds: invariant, decision, assumption, term
and context. No kind holds a narrative. Boom also has no page where a
person reads a record and writes a note on it. So each design agent
writes a Markdown file in its repository, and the person reads the
design in an editor. The held branch `design/a-change-is-a-record`
(e8ee35b) edits `design/narrative.md`, and the user could not read the
edit in their editor. `design/narrative.md` says
"This document is the first narrative Boom keeps", but nothing in Boom
makes that true.

The user asked on 2026-11-23: "Isn't the point of Boom that narratives
live in Boom? What are we missing that's making us still write to these
external documents?" The user then ruled: "Before turn 6, make
narratives live in Boom, so turn 6's design is the first one Boom
holds. Yes: a narrative kind and a page to read and annotate it first."
A note on the page goes back as a signal.

## What Changes

- **The kind `narrative`.** `boom propose narrative <name> "<title>"
  --body <file>` proposes a narrative: a title of one line, a body of
  Markdown text kept by its fingerprint, the repositories and the
  contexts that it binds, and the claims that it includes. A narrative
  is in the one name space of the store, gets a version for each
  change, and amends its base with `--amends`.
- **The edge `includes`.** `--includes <name>[@<version>]` names a
  claim that the narrative includes. The edge stores the version that
  it meant, as `--source` does. Only a narrative includes claims.
- **A person accepts a narrative.** A person accepts or rejects a
  narrative with `boom accept` or `boom reject` and a security key, as
  for each other claim. One signature can accept a narrative with the
  claims that it includes. Boom never checks a narrative, and no hook
  gives it as a rule.
- **The paragraphs of a body.** Boom numbers the paragraphs of a
  narrative's body from 1, and gives each paragraph a fingerprint.
  `boom show <name>@<version> --json` gives them.
- **`boom page`.** `boom page [<name>[@<version>]]` serves a page on
  the person's machine, at `127.0.0.1`, and opens it in the browser. The
  page gives the narrative with its paragraphs numbered, each included
  claim at the paragraph that names it, each note at its paragraph, and
  the versions of the narrative. Boom computes the page from the log for
  each request. No file and no commit holds it. The page loads nothing
  from another host and runs no script.
- **`boom note` and the act `noted`.** A person writes a note on one
  paragraph of a version, on the page or with `boom note`. The note is
  an entry of the log that names the person and has no signature. The
  note stays open until a later version of the narrative answers it.
- **An amendment answers the open notes.** `--answers <note>=<words>`
  answers a note. Boom refuses a new version of a narrative that leaves
  a note of the narrative open.
- **A session sees the narratives.** `boom hook session-start` gives
  the narratives of the repository by name, with the count of the open
  notes of each, and its menu names `boom page <name>`.
- **`boom mcp`** gives the kind `narrative`, `includes` and `answers` to
  the tool `propose`. It serves no `note` and no `page`.
- After this change is released, an agent proposes `design/narrative.md`
  from main as the first narrative, `boom-end-to-end`. When the person
  accepts it, one change removes the file from the repository.

## Capabilities

### New Capabilities

- `narratives`: the kind `narrative`, its title and body, the edge
  `includes`, the paragraphs of a body, and what `boom show` gives of a
  narrative.
- `narrative-notes`: the act `noted`, `boom note`, a note's name, an
  open note, and the answers of an amendment.
- `narrative-page`: `boom page`, the page that it serves, and a note
  written on the page.

### Modified Capabilities

- `claim-versions`: a sixth kind, `narrative`, which needs a body, and
  the content of a version holds `includes` and `answers`.
- `claim-edges`: a version stores the edge `includes`.
- `persons-acts`: a note is the act of a person with no signature.
- `record-log`: a note is an entry of the log.
- `command-line`: the commands `note` and `page`.
- `session-context`: session start gives the narratives by name, with
  their open notes, and the menu names `boom page`.
- `agent-tools`: the tool `propose` takes the new kind and edges, and no
  tool writes a note or serves a page.

## Impact

- `crates/boom`: `claim.rs` (the kind, `includes`, `answers`),
  `entry.rs` and `judge.rs` (the act `noted`), a new module
  `narrative.rs` (the paragraphs and the notes), a new module `page.rs`
  (the server and the HTML), `cli.rs`, `mcp.rs` and `hook.rs`, and their
  tests. Two new dependencies, which `Cargo.lock` pins:
  `pulldown-cmark`, to read the Markdown of a body, and `tiny_http`, to
  serve the page with no async runtime.
- `plugins/claude-code`: no change to its files. Its tests see the new
  menu line.
- The store: the new kind and the new act are entries of the same log.
  An older boom counts each such entry for nothing.
- No new part of the repository, and no new moon project. The page is
  Rust code of the crate `boom`, and `boom:test` checks it.
- `design/narrative.md` leaves the repository in a later change, after
  the person accepts the narrative `boom-end-to-end`.
