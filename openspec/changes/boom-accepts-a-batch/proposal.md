## Why

On 2026-11-24 the person accepted the drive's design through cards that
agents made. Each card held a list of versions, one of them 183 long.
The person said: "it's cumbersome obviously. i think the key is to know
how proposed log changes should get batched and presented to the user
for acceptance."

The person accepted
`the-accept-of-a-narrative-accepts-the-claims-it-includes`: one signed
accept of a narrative accepts each proposed version that it includes,
and Boom refuses it while it includes a closed version. On 2026-11-24
the person also accepted the mockups of the batches: a narrative with
the claims that it includes, a ruling of one claim, and a revision with
its cause. A revision's one sign accepts its cause too. Before the
rail, the person's card carries the accept command and the link to
`boom page`.

Today `boom accept` takes only the versions that the person names. A
narrative's accept leaves the claims that it includes proposed, no
version names the change that forced it, and no command or page says
what waits on the person.

## What Changes

- **A narrative's accept takes its claims.** `boom accept
  <narrative>@<version>` accepts the narrative and each proposed
  version that it includes, in one entry that the key signs one time.
  Boom refuses it while the narrative includes a rejected, superseded
  or deprecated version, and names each one.
- **A revision names its cause.** `boom propose --cause <claim>` stores
  the version of the claim that forced the new version, an edge of the
  version as `source` is. `boom show` gives it.
- **A cause's accept takes its revision.** `boom accept --cause
  <claim>@<version>` accepts the cause and each proposed version that
  names it, in one signed entry.
- **Boom lists what the accept takes.** Before the key signs, Boom
  writes each version that the entry takes. Its answer lists each one.
- **What waits on the person.** Boom computes the batches from the
  log's links, and no agent makes one: a narrative and the proposed
  versions that it includes, a revision and its cause, and each other
  proposed version alone as a ruling. `boom accept --waiting` lists the
  batches, each with its counts and its command, and with `--key`
  accepts each of them in one signed entry.
- **The page gives what waits.** `boom page` gives the view "Waiting
  on you" at `<base>/waiting`, first in the bar, with the number of
  batches. Each batch gives its shape, its title, why it exists, its
  counts, what to read and its one command. A narrative's page lists
  what its accept takes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `persons-acts`: the accept of a narrative, the accept of a cause, the
  list of what an accept takes, and `boom accept --waiting`.
- `claim-edges`: the edge `cause` of a version.
- `narrative-page`: the view "Waiting on you", and the list of what a
  narrative's accept takes on its page.

## Impact

- `crates/boom`: `claim.rs` (the edge `cause`), `judge.rs` (the check
  of the edge), `store.rs` (the edge in the views, and the versions
  that name a cause), a new `batch.rs` (the batches, computed from the
  views), `cli.rs` (`boom propose --cause`, `boom accept --cause` and
  `--waiting`, and an accept's list), `page.rs` (the view and the
  narrative's list), and the tests in `crates/boom/tests/`.
- `boom mcp` gives `propose` the argument `cause`, since its tools
  follow the commands. It gives no tool that accepts.
- No new entry type, no change to the views' form, no new crate, no new
  dependency and no new moon project.
