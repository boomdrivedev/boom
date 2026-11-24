## Context

A person accepts with `boom accept`, which takes several versions in
one entry that the key signs one time (`persons-acts`, "Several
versions accepted together"). A narrative includes claims at the
version it meant (`narratives`). A version stores its edges `source`,
`changes`, `origin`, `includes` and `narratives` (`claim-edges`).
`boom page` serves the book and the views of the claims, each computed
from the views at the log's last entry (`narrative-page`,
`claims-view`).

The person accepted these, and this change builds them:

- `the-accept-of-a-narrative-accepts-the-claims-it-includes`: one
  signed accept of a narrative accepts each proposed version that it
  includes, and Boom refuses it while it includes a closed version.
  The page lists each version that the accept takes.
- The mockups of 2026-11-24 for the batches (the elaboration
  `design--accepting-proposals-in-batches`): the batches as drawn; a
  revision's one sign accepts its cause too; before the rail, the
  person's card carries the accept command and the link to `boom page`.

The batches decision,
`proposals-reach-the-person-in-batches-a-narrative-a-ruling-or-a-revision`,
waits on the person. Its new version says that Boom computes a batch
from the log's links, and that a revision's sign accepts its cause.

No other tool of the family accepts a claim or computes what waits on
the person. The rail is the cloud's and is not built. A Pending You
card is an agent's notice, which carries the command that this change
gives. boom owns this design, and no other tool must change with it.

This design keeps these claims of the store:

- `status-read-from-events` and `events-are-appended-never-changed`: an
  accept stays one `accepted` entry that names each version. No new
  entry type comes, and the judge judges each version that the entry
  names, as before.
- `a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key`: the
  key signs the entry with each version written in it, so the
  signature covers exactly what the person took.
- `a-link-remembers-the-version-it-meant`: the edge `cause` stores the
  version of the cause that it meant.
- `views-computed-from-the-ledger`: the batches are computed from the
  views for each command and each request, and stored nowhere.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  `boom mcp` gets the argument `cause` of `propose`, and no tool that
  accepts.

## Goals / Non-Goals

**Goals:**

- The person signs each batch once, and Boom, not an agent, says what a
  batch holds.
- The person sees each version that a sign takes before the key signs.
- One place, the page or one command, says what waits on the person.

**Non-Goals:**

- The rail's card. The cloud builds it from the same batches.
- An accept or a reject from the page. The page gives commands, and a
  terminal with the key runs them.
- A batch's order by the plan that waits on it. Boom holds no plan's
  approval yet (`boom-never-stores-a-plan-a-unit-or-a-bolt`), so the
  log names no plan that waits. The batches are in the order of their
  entries until Boom refuses a plan's approval for a proposed version
  (`only-an-accepted-version-binds-work`, decision 3).
- A group of narratives that one place proposed, as the mockup drew
  sluice's four. Each narrative is a batch of its own; one command
  accepts several.

## Decisions

1. **The command writes the versions in the entry.** `boom accept`
   reads what a narrative includes and what names a cause, and writes
   each version that it takes in the one `accepted` entry. The judge
   judges each version as before.

   Options:
   1. The command expands the batch into the entry (chosen). The
      signature covers each version by name, and the entries of the
      log keep their meaning, so no store is built again.
   2. The entry names the narrative alone, and the judge accepts its
      includes. The signature then covers versions that the entry does
      not name, and a replay of an old accept of a narrative would
      accept its includes on that day's rule.
2. **A closed include refuses the accept.** A narrative that includes a
   rejected, superseded or deprecated version refuses the accept, and
   names each one, as the accepted decision says. Its author proposes a
   new version of the narrative. An include that is accepted already is
   left out of the entry.
3. **The cause is one edge of the version.** `--cause` is stored on the
   version as `source` is, written as a field of the proposal after
   each other edge, so a version with no cause keeps its fingerprint.
   The views keep it in `edges` as the edge `cause`. A version names
   one cause at most: the one change that forced it.
4. **The accept of a cause takes the cause.** As the person answered
   on 2026-11-24, the cause comes with its revision: `boom accept
   --cause` takes the cause where it is proposed, then each proposed
   version that names it. A superseded draft is closed, and only its
   newer draft is taken.
5. **A batch is computed from the views.** `batch.rs` reads the
   proposed versions, then the claims that they include and the claims
   that they name as their cause, and nothing else. A proposed version
   that a waiting narrative takes, or that a cause takes, is no ruling.
   A version that a narrative includes and that names a cause shows in
   both, since each command takes it.
6. **`--waiting` lists, and with the key accepts.** With no key, `boom
   accept --waiting` reads only, so an agent tells the person what
   waits from it. With the key, it takes each batch that Boom does not
   refuse, and names those it leaves out.
7. **The list on standard error.** The key asks for a touch on the
   terminal. The list comes before it on standard error, and the
   answer lists the same versions on standard output.

## Risks / Trade-offs

- [A narrative of hundreds of claims makes one long entry] → It is the
  list that the person signs. The page and standard error give it
  before the key.
- [The page reads the proposed versions for each request] → The page
  reads only the proposed versions and the claims that they name, not
  the store.
