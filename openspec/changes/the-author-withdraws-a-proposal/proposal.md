## Why

About 440 proposed versions wait in the person's store. Many are drafts
that agents left: no one will accept them, and only a person's reject
closes them, one touch of the key for each version. An agent cannot
close its own proposal.

On 2026-11-23 the person accepted the decision
`the-author-of-a-proposal-withdraws-it`: the author of a proposed
version withdraws it with its reason and no signature, it takes the
status `withdrawn`, and a person rejects several versions with one
touch of the key. The narrative `claim-status-on-the-rail` gives the
same path.

## What Changes

- **The status `withdrawn`.** A sixth status word. A withdrawn version
  is closed: no one accepts, rejects or withdraws it again, and it
  leaves each list of what waits.
- **`boom withdraw <name>@<version>... --reason <text>`.** The author of
  each version withdraws it in one entry, with no signature: an agent
  with `--provenance`, for versions that an agent proposed, or a person
  with `--person`, for versions that the person proposed. Boom judges
  the entry whole: each version is withdrawn, or none.
- **`boom reject` takes several versions.** One entry names each
  version, with one reason, and the key signs it one time. Boom judges
  the entry whole. Its answer is `boom-reject/3`, which lists each
  version under `rejected`.
- **`boom search --proposed`.** It gives the versions that wait, in a
  repository or every repository, each with its author and its age.
  With no word, it gives each version that waits, the oldest first.
- **A narrative that includes a withdrawn version is not accepted**, as
  one that includes a rejected version is not.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `claim-versions`: the status words name `withdrawn`, and a new
  requirement, "The author withdraws a proposed version".
- `persons-acts`: an unsigned entry is an agent's, or a person's note
  or withdrawal; an agent proposes or withdraws; a new requirement,
  "Several versions rejected together".
- `narrative-notes`: an entry of a person with no signature counts
  when its act is `noted` or `withdrawn`.
- `claim-search`: a new requirement, "The versions that wait".
- `command-line`: the commands name `withdraw`.
- `agent-tools`: the tool `search` takes `proposed`, and requires no
  word, as its command does.

## Impact

- `crates/boom/src/entry.rs`: the act `withdrawn`, and the form of a
  reject of several versions.
- `crates/boom/src/judge.rs`: the status `withdrawn`, and the rules of
  a withdrawal and of a reject of several versions.
- `crates/boom/src/cli.rs`: `boom withdraw`, `boom reject` with several
  targets, and `boom search --proposed`.
- `crates/boom/src/store.rs`, `batch.rs`, `view.rs` and the page: a
  withdrawn version is closed.
- `crates/boom/tests/withdraw.rs`: a test for each scenario.
- `README.md`, `CLAUDE.md`, the site's `commands.md` and the skills
  `using-boom` and `proposing-claims`.
- The claim `the-status-words`: a new version that names `withdrawn`,
  for the person to accept.
- The new act and the new form of a reject came with this change, so no
  entry in a log has them, and the rule set stays 1. A boom before this
  change refuses an entry with the act `withdrawn`, or a reject that
  names `versions`, by its form.
