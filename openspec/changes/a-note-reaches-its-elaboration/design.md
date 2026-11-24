## Context

A note is a person's words on one paragraph of one version of a
narrative, an entry with the person's name and no signature
(narrative-notes). It stays open until a later version answers it.

The accepted decision
`a-note-on-a-narrative-reopens-the-elaboration-that-made-it@fd6d59cec2a5`
sends the note back to the elaboration that made the narrative, whose
next round takes the note as its input and proposes the version that
answers it. `an-elaboration-ends-with-its-findings-in-boom@20e236138a74`
says that each finding of an elaboration is a proposed claim linked to
its origin, and `a-claim-links-to-its-origin@39282a168d6e` says that the
origin is the elaboration and its artifacts.

sounding runs an elaboration as a sounding. It names a sounding by its
repository and its name: its record is at `refs/sounding/<name>` of the
repository, and millpond's offers name it `sounding/<repository>/<name>`
(blueprints, `design/sounding/requirements.md`, C.6 and C.7).

This change keeps these claims:

- `views-computed-from-the-ledger@36a056c1145b` and
  `status-read-from-events@97d0559deafa`: a note's elaboration and
  whether it is open are read from the log, and no entry stores them.
- `events-are-appended-never-changed@886b3f05e89f`: the note's entry is
  as it was. A version's origin is part of its content.
- `origin-and-source-name-the-links-of-a-claim@180510b40995` and the
  term `origin@aa42305b5d98`: the elaboration is an origin, named with
  `--origin`. This change departs from the decision's "an `--origin` in
  a form other than a file or `git:<repository>@<commit>:<path>`
  breaks the rule". It proposes a new version of the decision and of
  the term that names the elaboration as the third form.
- `a-link-remembers-the-version-it-meant@3aa12ed5d5d7`: the note names
  the version it is on, and that version names its elaboration.

## Goals / Non-Goals

**Goals:**

- A version names the elaboration that it came from.
- Each note gives its elaboration.
- An agent or sounding asks Boom for the open notes of an elaboration.

**Non-Goals:**

- sounding's next round, which a change in sounding starts.
- The signal to millpond for a note on a version of no elaboration.
- A notice that tells sounding a note was written.

## Decisions

### The elaboration is an origin, `elaboration:<repository>/<name>`

An origin is where a version came from, and the elaboration is where a
narrative came from. A third form of `--origin` keeps one word for the
link, which the term `origin` and
`origin-and-source-name-the-links-of-a-claim` hold. The form
`elaboration:<repository>/<name>` gives sounding's own two names. Boom
uses the word of its domain, `elaboration`, and not the name of the
tool that runs it.

The repository is in kebab case, as boom names repositories. The name
is lowercase letters, digits and hyphens, beginning and ending with a
letter or a digit: a sounding's name is part of the branch
`sounding/<name>` and of a flume place's name, where a double hyphen
occurs. Boom stores the two names and reads nothing of the elaboration,
as it reads no repository for a `git:` origin.

The origin is part of the version's content, as every edge is, so a
person signs it when they accept the version. The new form leaves the
fingerprint of every earlier version as it was.

Options:

- **A third form of origin.** Chosen.
- **A field `elaboration` of its own.** A second word for where a
  version came from, against the term `origin`.
- **A `git:` origin of the sounding's record.** The record's commit
  changes with each round, and the record is not on a branch that a
  person reads.

### A version names one elaboration at most

A note reaches one elaboration. Boom refuses a version whose origins
name two, at the write and when it reads the entry, so a note never
has two.

### A note's elaboration is read from its version

The note names its version, and the version names its elaboration in
its content, which no event changes. Boom reads the elaboration when it
gives the note, and stores it in no entry. A note on `<v1>` keeps the
elaboration of `<v1>`, whatever a later version names.

### `boom query --elaboration` gives the open notes

`boom query` is how an agent asks Boom for what binds its work, and the
MCP tool `query` serves it. The open notes of an elaboration are the
input of its next round, so the same command gives them. The answer
keeps the format `boom-query/2` with `claims` empty, `repository` null,
and two fields: `elaboration` and `notes`. Each note has the fields
that `boom show` gives it, with `narrative` and `elaboration`. The query
reads the narratives and the notes from the views, as `boom show` of an
outline does, and appends nothing.

## What sounding needs from Boom

sounding's side is a change in sounding. It needs:

1. Each round that proposes a narrative, or a claim that the narrative
   includes, names `--origin elaboration:<repository>/<name>` with the
   sounding's repository and name.
2. When a round starts, `boom query --elaboration <repository>/<name>
   --json` (or the MCP tool `query` with `elaboration`) gives the open
   notes. Each is an input of the round, with its narrative, version,
   paragraph and words.
3. The version that the round proposes names `--answers <note>=<words>`
   for each open note; Boom refuses a version that leaves one open.
4. sounding opens the next round on its own read of the query: Boom
   tells no tool that a note was written.

A note whose `elaboration` is null came from a version that no
elaboration made. The decision sends it to millpond as a signal. That
needs a query of the open notes of no elaboration, which this change
does not give.

## Risks / Trade-offs

- [The name that sounding gives an elaboration changes after Boom
  stores it] → Boom stores the name of the sounding, which its record
  and branch keep for the sounding's life.
- [A narrative proposed before this change names no elaboration] → Its
  notes give `elaboration` null, and go to millpond by the decision.
  The author can propose a new version that names the elaboration.
