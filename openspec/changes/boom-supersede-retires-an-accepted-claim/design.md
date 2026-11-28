## Context

See proposal.md for why. The user's ruling is the decision
`boom-supersede-retires-an-accepted-claim@6cde86c6d5d5`, proposed for the
person to accept.

Boom reads each status from the log (core.1, `status-read-from-events`).
The judge decides whether an entry counts, and `apply` builds the state
that the views keep (core.2). A claim has one accepted version,
`standing`, and `apply_accept` supersedes the version that stood before.
Only an accepted version binds work
(`only-an-accepted-version-binds-work`): `boom query`, `boom search` and
the two hooks read `standing` from the views, so a claim with no
`standing` leaves each of them.

A version is the fingerprint of its content, `claim::Content`, whose
later fields are each written only where present, so a version with
none keeps the fingerprint that it had before them. Each entry names its
rule set, and boom judges it by those rules alone
(`an-entry-is-judged-by-the-rules-it-was-written-under`). The rule set
is 1.

A command loads only the claims that its scope names. The scope of an
accept names the claims that it accepts, and nothing else.

## Goals / Non-Goals

**Goals:**

- One `boom supersede` and one accept by a person take an accepted claim
  out of force, its history kept in the log.
- The duplicate case: the claim to keep gets a new version with its
  words unchanged.
- Each entry that a log holds today judges as it did.

**Non-Goals:**

- A tool `supersede` in `boom mcp`.
- A mark of a supersession on `boom page`, in `boom log` or in
  `boom accept --waiting`. The page drops the superseded claim from its
  accepted claims, as it reads `standing`.
- A supersession of a term, a context, a narrative or an outline.
- `boom check`, which is not in boom.

## Decisions

1. **A version stores the edge `supersedes`.** `Proposal` gets
   `supersedes`, one object, `{"claim": "<name>@<version>", "reason":
   "<text>"}`, its `reason` written only where given. The entry stores
   the version that the supersession meant
   (`a-link-remembers-the-version-it-meant`). It is the last field of
   `Content`, written only where present, so each version that a log
   holds keeps its fingerprint, and the reason is signed with the rest.
   Not taken: the reason outside the fingerprint, beside `attach`. The
   same content with another reason would then repeat, and lose the
   reason that the person reads.
2. **The judge of a proposal.** `Proposal::check` refuses a
   `supersedes` that is not an edge of another claim, on a version of a
   kind other than invariant, decision or assumption, on a version with
   history, or with a reason that is empty or of two lines.
   `check_proposal` refuses where the claim that it names is of another
   of those kinds, or where the version that it names is not the
   `standing` version of its claim. It names the accepted version, or
   says that the claim has none. Not taken: every kind. A term already
   replaces old words, and a context, a narrative and an outline bind
   other claims, whose supersession needs its own design.
3. **The judge of an accept.** `check_accept` refuses a version that
   supersedes where the version that it names is no longer `standing`,
   and names the accepted one. The check of an `accepted` entry refuses
   an entry that names a version of a claim that another of its
   versions supersedes, or two versions that supersede one claim. Each
   version of an entry is checked against the state before the entry,
   so an accept of both would otherwise take out of force a version
   that the person did not name.
4. **The apply of an accept.** After `apply_accept` makes a version
   that supersedes `standing`, it gives the version that it names the
   status `superseded` and its claim no `standing`, where that version
   is still `standing`. It changes no entry, and the version keeps the
   entry that accepted it (core.1). A later draft of that claim builds
   on a version that is not accepted, so boom refuses its accept as it
   does today.
5. **Each existing entry judges the same, and the rule set stays 1.**
   Each new rule reads the field `supersedes`, and no entry before this
   change holds it: before this change, `Proposal` refuses a field that
   it does not know. Thus no rule changes the verdict of an entry that a
   log holds, `apply` does what it did for each of them, and each
   version keeps its fingerprint. The rule of rule sets gives a new
   rule set to a rule for a field that entries already have, and
   `supersedes` is none (`an-entry-is-judged-by-the-rules-it-was-written-under`).
   This follows `cause` and `withdrawn`, which came at rule set 1. Not
   taken: rule set 2, with which a boom before this change refuses each
   new entry, not only a supersession. That boom refuses a supersession
   by its form either way.
6. **The views.** `write_edges` writes the edge as a row of `edges`,
   `supersedes`, with its target's name and version. `load_state_for`
   loads, with the claims that a scope names, the claims that their
   versions supersede, from those rows, so an accept, its catch-up and
   `boom verify` hold the claim that it takes out of force.
   `Scope::of` names the claim that a proposal supersedes.
   `write_touched` writes again the claim that an accept took out of
   force. No row of a view that a store holds changes, so the views keep
   their form, 11, and no store builds them again.
7. **`boom supersede` decides its case from the state, and proposes
   through `propose`.** It reads the claims `<name>` and `<new-name>`,
   then gives `propose` the version to write:
   - with a new name: a first version, the kind `--kind` or that of
     `<name>`, `<statement>` and `--body`, and the repositories and
     contexts of the accepted version of `<name>`, or `--system` where
     it binds the whole system;
   - with an accepted claim of the same kind: `--amends` its accepted
     version, its statement unless `<statement>` is given, its body
     fingerprint unless `--body` is given, and its sources as stored.
     The amendment binds what its base binds, as each amendment that
     names no binding does.

   `Proposed` gets `supersedes` and a body that the store keeps by its
   fingerprint, so a copied body needs no file. The judge checks the
   version in the write's transaction, so a claim that changed after
   the read is refused there. The answer is `boom-supersede/1`: the
   fields of `boom-propose/2`, and `supersedes` as the name of
   `<name>`, as the draft writes it, with `supersedes_version` and
   `reason`. Not taken: `--scenario`, which the draft writes. A new
   version holds no scenarios (claim-versions, "A claim and its name"),
   and `boom propose` takes none. Not taken: `--source`, `--origin` and
   a binding of its own, which the draft does not name.
8. **What the person reads.** The list that `boom accept` tells the
   person before the key signs gives `, which supersedes <name>@<version>`
   after each version that supersedes. Each version of the answer of
   `boom-accept/2` gives `supersedes`, and its words give the version
   that it superseded. `boom show <name>@<version>` gives the edge and
   its reason. `boom show <name>` gives `superseded_by`, each version
   whose edge names the claim, with its status, from the rows of
   `edges`. Each answer keeps its format, as `plan_words` did.
9. **`boom mcp` names `supersede` absent.** `mcp::ABSENT` gives why: an
   agent proposes a supersession from the command line in this release.
   Not taken: a tool `supersede`. The requirement "Each tool is one of
   boom's commands" lists the tools in a scenario that two open changes
   modify, `a-note-reaches-its-elaboration` and
   `the-author-withdraws-a-proposal`. A third would conflict with both.
10. **The spec adds requirements, and modifies none.** The status
    requirement of claim-versions and the list of commands of
    command-line are each modified by an open change. Its status
    requirement is near OpenSpec's 500 characters. So this change states
    the new path to `superseded`, and the command, in requirements of
    its own (core.7).
11. **The tests.** `crates/boom/tests/supersede.rs` runs `boom::cli::run`
    in the process, with the test keys made when the test runs (core.3),
    one test for each scenario. The judge's own refusals are unit tests
    in `judge.rs`. No test starts the binary
    (`logic-is-tested-as-units-and-few-tests-run-the-binary`), and no
    test reads a Markdown file (core.5).

## Risks / Trade-offs

- [A boom before this change reads a store with a supersession] → It
  refuses the entry by its form and the accept of the version, and gives
  the claim that the supersession took out of force. Each host runs the
  boom of its flake (core.6), and the person supersedes only after the
  release.
- [A first version may take the name of a claim with no accepted
  version] → The name space allows first versions while no version is
  accepted, and a superseded claim has none. A person accepts such a
  version, as any first version, and the log keeps the supersession.
- [A copied body names a blob that the store keeps] → `propose` writes
  no blob for it. The store keeps each body of a version that it holds,
  so the blob is there.
- [Two supersessions of one claim wait together] → The person accepts
  one. The accept of the other is refused, since the claim that it names
  has no accepted version.

## Migration Plan

None. The views keep their form, and no entry changes. The release that
follows the landing carries the command (core.8). After it, the person
runs, for the duplicate:

```text
boom supersede logic-is-tested-as-units-and-a-few-tests-run-the-binary \
  logic-is-tested-as-units-and-few-tests-run-the-binary \
  --reason "<why>" --provenance "<session>"
boom accept logic-is-tested-as-units-and-few-tests-run-the-binary@<new version>
```
