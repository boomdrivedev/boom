## Context

The drive keeps one log. Each tool owns its entry types, and Boom's
store is the log (`the-drive-keeps-one-log-and-each-tool-owns-its-entry-types`).
The tool that owns the thing that an act changes records that act
(`each-tool-records-the-acts-on-what-it-owns`). Boom owns the entries on
claims, narratives, change records and a plan's approval.

Boom writes six acts today: `registered`, `proposed`, `accepted`,
`rejected`, `repository` and `noted`. Each entry has a signed part: one
JSON object with no number, no chain and no signature (`entry.rs`). The
entry's hash chains the hash before it, the signed part and the
signature. Thus the entry's hash changes when the drive orders the
entries again, and the signed part does not.

The narratives `the-lineage` and `the-links-of-the-lineage` give each
hop of the lineage and what it links. These decisions bind Boom's part:

- `a-change-record-is-a-record-of-its-own-that-no-person-accepts`
- `a-change-record-names-its-unit-and-bolt-by-their-names-in-the-approved-plan`
- `boom-never-stores-a-plan-a-unit-or-a-bolt`
- `every-change-comes-from-an-approved-plan`
- `only-an-accepted-version-binds-work`, point 3: Boom refuses the
  approval of a plan that covers a version that is not accepted.
- `a-step-links-to-every-work-it-takes-in`
- `sluice-writes-the-landed-entry-when-it-lands`
- `a-link-remembers-the-version-it-meant`

The rules core.1 (status from events) and core.2 (views from the
ledger) hold for each new record. core.3 holds: no entry holds a key.

## Goals / Non-Goals

**Goals:**

- Boom records a plan's approval and each version of a change record.
- Each entry has one hash that a link names, and Boom checks each link.
- Another tool appends its own entries to the log.
- A reader walks from any entry back to the first entry of its lineage
  and forward to the last.

**Non-Goals:**

- No plan, unit or bolt in Boom: Boom names a plan by the SHA-256 of
  its document, and a unit or a bolt by its name.
- No meaning of another tool's entry: Boom judges its form and its
  links, and the owner computes its views.
- No `boom check` of a change's folder at the gate.
- No transition code: nothing names an entry, a store or a claim.

## Decisions

### 1. The act hash is the SHA-256 of the signed part

A link names an entry by the SHA-256 of its signed part, 64
hexadecimal characters. The signed part holds neither the entry's
number nor the hash of the entry before it. Thus a link stays true when
the drive orders the entries later. The act hash leaves out the
signature, which proves the act and is not the act.

Boom computes the act hash of each entry, old and new, and keeps it in
the view `acts`. No entry stores its own act hash. `boom log --json`
gives `act_hash` for each entry.

Not taken: the entry's hash. It covers the chain, so it changes when the
drive orders the entry again. Not taken: 12 characters, as a claim's
version. A link joins entries of every tool of the drive, so it keeps
the full hash. A person names an entry by 12 characters or more, and
Boom finds the one entry whose act hash starts with them.

The claim `a-link-is-the-hash-of-the-signed-part-it-names` holds it, for
the whole system.

### 2. Boom refuses a link to an entry that the log does not hold

An entry's links name earlier entries of the log. Boom refuses an
entry whose link names an act hash that no counted entry before it
has. Thus a step that takes in several entries waits for each of them
(`a-step-links-to-every-work-it-takes-in`). Each link is the full act
hash, each once.

The claim `boom-refuses-a-link-to-an-entry-its-log-does-not-hold` holds
it.

### 3. Another tool's entry: `boom append <tool>.<type>`

The log holds entries of every tool, and only Boom's crate has the
log's code. Boom gives another tool a command:

```console
$ boom append sluice.landed --link <act hash> --content landing.json \
    --provenance "sluice host deneb, round 42"
```

Its signed part is `format`, `act`, `at`, `rules`, `actor`, `links` and
`content`, in that order. The act is `<tool>.<type>`: two kebab names,
the tool not `boom`. A Boom act has no dot. The content is one JSON
object, which Boom keeps as written and never reads. The actor is an
agent with its provenance, or a person that the store registers, named
with `--person` and with no signature
(`a-bolts-hold-and-release-are-entries-with-no-signature`). Boom refuses
a signature on such an entry, since the key signs only an accept, a
reject and a plan's approval.

Boom judges the form, the actor and the links, and nothing else. The
owner of the type says what it means and computes its views.

Options:

1. A command of Boom (chosen). Each tool already calls Boom by its
   command line.
2. A library crate of the log. Each tool then builds Boom's code into
   its own binary, and two versions of the judge can write one log.
3. Each tool writes the SQLite file. Each tool then judges nothing, and
   a chain that one tool breaks breaks every tool.

The claim
`boom-takes-an-entry-of-another-tools-type-and-judges-only-its-form-and-links`
holds it. Each tool takes the command by a decision of its own, as the
consequences of `the-drive-keeps-one-log-and-each-tool-owns-its-entry-types`
say.

### 4. A plan's approval: `approved`

```console
$ boom approve plan.md --link <elaboration-ended> \
    --output git:boom@<commit>:design/narrative.md \
    --covers status-read-from-events --key ~/.ssh/id_ed25519_sk
```

The signed part holds `plan`, the SHA-256 of the plan document's bytes;
`links`, each entry that the plan took in, such as each
`sounding.elaboration-ended`; `outputs`, each output as
`git:<repository>@<commit>:<path>`; and `covers`, each claim as
`<name>@<version>`. Boom reads the document only to hash it, and keeps
nothing of it (`boom-never-stores-a-plan-a-unit-or-a-bolt`).

The key signs it, the person present
(`a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key`). Boom
refuses:

- an approval with no link;
- a covered version that is not accepted, naming it
  (`only-an-accepted-version-binds-work`);
- a second approval of one plan's hash;
- an agent's approval, or one that no registered key signed.

A name in `--covers` stores the accepted version of that claim. Boom
does not read the type of a linked entry: its owner says what it is,
and `boom lineage` gives it.

### 5. A change record: `recorded`

```console
$ boom record change boom:contexts-own-paths --approval <act hash> \
    --unit contexts --bolt b-12 --covers context-claims \
    --origin git:boom@<commit>:openspec/changes/contexts-own-paths \
    --provenance "boom place change--contexts-own-paths, session 1f2e"
```

The change record's name is `<repository>:<name>`, the name of the
change's folder. Change records have a name space of their own, apart
from claims. Boom reads the folder `openspec/changes/<name>` under the
working folder, or the folder that `--folder` names. Boom keeps each
file of the folder once, by its fingerprint, as it keeps a long body.

A version is the first 12 hexadecimal characters of the SHA-256 of its
content: the repository, the name, its base, its approval, its unit,
its bolt, its covered claims, and each file's path and fingerprint. The
`origin` commit is in the entry and out of the content. Thus a rebase
that keeps the files keeps the version, and the same content appends
nothing. The claim
`a-change-records-version-is-the-fingerprint-of-its-files-and-its-links`
holds it.

Options for the commit:

1. Out of the content (chosen). The files fix the change, and the
   commit says where they were.
2. In the content. Each rebase then gives a new version with the same
   files.
3. The commit alone, with no files. A host that does not have the
   commit then cannot read the change.

Boom refuses:

- a first version with `--amends`, and a later version that does not
  amend the newest version;
- an approval that names no counted `approved` entry;
- a covered version that its approval does not cover
  (`a-change-record-covers-only-what-its-approval-covers`);
- a new version of a change record that landed;
- an empty folder, a file in it of more than 1 MiB, or a link in it.

An agent records it, with its provenance and no signature, and no
person accepts it
(`a-change-record-is-a-record-of-its-own-that-no-person-accepts`). A
person's entry with no signature counts for nothing here.

### 6. Its status, from the entries

A change record is `written` from its first version. It is `landed`
once an entry of the type `sluice.landed` links the `recorded` entry of
one of its versions; that version is the landed version. Boom reads no
other entry of sluice. A status is never stored (core.1).

`boom show <repository>:<name>` gives the record: its status, its
versions by base, and for each its approval, unit, bolt, covered
claims, files and origin. A covered claim whose accepted version moved
on is marked out of date, as an edge of a claim is
(`a-link-remembers-the-version-it-meant`). `boom show --artifact
<fingerprint>` gives a file.

### 7. The walk: `boom lineage <target>`

The target is an act hash of 12 characters or more, `<repository>:<name>`
for the newest version of a change record, or
`<repository>:<name>@<version>`. Boom gives:

- `entry`: the target's number, type, act hash and actor;
- `back`: each entry that the target reaches through links, each once,
  the nearest first;
- `forward`: each entry whose links reach the target, each once, the
  nearest first.

Each entry gives its links, and, for `approved` and `recorded`, its
covered claims, each marked where it is out of date. The views `acts`
and `links` hold each entry's act hash and links, kept in step with the
log in the same transaction, and built again by `boom verify` (core.2).

From a change record's version, `back` reaches its approval, each entry
that the plan took in, the route, the intent's answer and the signals,
as their owners write them. `forward` reaches the landing, the publish
and the promotion.

### 8. What the other tools write

Each hop of the lineage is an entry that its owner appends with `boom
append`, by a change of its own. Each link is an act hash.

| Type | Owner | Links | Content |
|---|---|---|---|
| `millpond.capture` | millpond | none | where its raw material is |
| `millpond.signal` | millpond | its capture | the signal |
| `millpond.intent-answered` | millpond | each signal of the intent | the intent's id, and yes, drop or split |
| `millpond.routed` | millpond | `millpond.intent-answered` | the kind of flume |
| `millpond.intent-closed` | millpond | `millpond.routed` | why it closed |
| `sounding.elaboration-ended` | sounding | `millpond.routed` | each output as `git:<repository>@<commit>:<path>` |
| `planning.held`, `planning.released` | the cloud's planning, a person's name | the bolt's `approved` entries | the bolt's name |
| `sluice.landed` | sluice | the `recorded` entry of each change that the bolt lands, or a chore's `millpond.routed` | the landed revision, and each verdict's digest |
| `timber.published`, `timber.promoted` | timber | `sluice.landed`; `timber.published` | the attestation |

Boom reads only `sluice.landed`, for a change record's status. Boom's
own hops are `approved`, which links each `sounding.elaboration-ended`,
and `recorded`, which links its `approved`.

## Risks / Trade-offs

- A tool that does not append its hops leaves a gap. The walk stops at
  the last entry that the log holds. Boom refuses an approval until the
  log holds each entry that the plan took in.
- Boom's command is the one way in. A tool on a host with no boom
  cannot write the log.
- `boom-records-the-acts-on-what-it-owns` changes the requirement "Each
  tool is one of boom's commands", which
  `a-note-reaches-its-elaboration` changed too. This change carries
  that change's `elaboration` argument of `query`.
- The views' form goes from 10 to 11, so each store builds its views
  again from its log at its first command.
