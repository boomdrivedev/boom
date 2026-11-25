## Context

Boom reads each version's status from the log (core.1). Today a version
is proposed, accepted, superseded, rejected, or, for a record that came
in as history, deprecated. Only a person's signed reject closes a
proposal, one version for each entry. An agent's entry only proposes. A
person's entry with no signature is only a note, since a note binds no
later work.

The person accepted the decision `the-author-of-a-proposal-withdraws-it`
on 2026-11-23. This change builds it.

Sluice has the same act: a submitter withdraws its own submission, which
stays readable as withdrawn (`sluice withdraw`). Boom takes the same
word. Sluice asks for no reason. Boom keeps one, since the decision asks
for it and a reader of the history needs it.

## Goals / Non-Goals

**Goals:**

- The author of a proposed version closes it with no key, and it takes
  the status `withdrawn`.
- A person rejects several versions with one touch of the key.
- An agent or a person lists the versions that wait, with their author
  and their age.

**Non-Goals:**

- No withdrawal of a version that is not proposed.
- No tool of `boom mcp` withdraws. The tool `search` takes `proposed`, as
  its command does.
- No transition code: nothing names an entry, a store or a claim.

## Decisions

### 1. The act `withdrawn`

A withdrawal is an entry with the act `withdrawn`, its fields `versions`,
each `{name, version}`, and `reason`. Its actor is the author: an agent
by its provenance, or a person by name. It carries no signature, as a
note carries none, since it binds no later work
(`a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key`).

The judge counts it when these hold:

- it names one version at least, each one time, with a reason;
- each version is proposed. A draft that a newer draft superseded is not
  (`a-new-draft-replaces-the-one-before`);
- for an agent's entry, an agent proposed each version, and the entry
  names a provenance;
- for a person's entry, the store registers the person, and that person
  proposed each version.

Where each version is withdrawn already, by an author of the same kind,
the entry repeats the first withdrawal and adds nothing. Where some are
and some are not, the judge refuses it, as it refuses such an accept.

Boom cannot prove which agent proposed a version, since a provenance is
words. Thus an agent can withdraw the proposal of another agent. The
decision accepts this risk: a false withdrawal costs one proposal again.

### 2. The status `withdrawn`

A counted withdrawal gives each version the status `withdrawn`, its
entry, and its reason. For a person's withdrawal, the version keeps the
person's name as the one who decided it. For an agent's, it keeps no
name: its author is its proposer.

A withdrawn version is closed:

- an accept, a reject or a withdrawal of it is refused;
- the lists of what waits (`boom accept --waiting`, `boom page`) leave
  it out, since they take proposed versions only;
- the accept of a narrative that includes it is refused, with the
  versions that are rejected or superseded;
- an outline of the book counts it as a rejected version counts;
- a withdrawn version answers no note, as a rejected version answers
  none;
- a new version may amend it, as a new version may amend a rejected
  one. The withdrawn version keeps its status.

### 3. A reject of several versions

`boom reject` takes several targets with one `--reason` and one
`--key`. A reject of one version keeps its form: `name`, `version` and
`reason`. A reject of several names them in `versions`, with one
`reason`, as an accept of several does. The judge judges it whole, as
it judges such an accept. Each version is one that a person may reject,
or each is rejected already by this person and the entry repeats.

Two versions of one claim may be rejected together, since neither
stands. An accept of two is refused, since both cannot stand.

The answer is `boom-reject/3`: `rejected`, each `{claim, version,
status}`, then `by`, `reason` and `entry`.

### 4. The versions that wait

`boom search --proposed` searches the proposed versions in place of the
accepted ones, in the same scope: a repository, the repository of the
session, or every repository with `--all`. With words, it ranks the
matches as a search does. With no word, it gives each version that
waits, the oldest first. Each version has its author, the entry that
proposed it, and its age in whole days, from the time of that entry to
the clock of the machine. The age changes with the clock, so two runs on
one log can differ in `age_days` alone.

The answer stays `boom-search/2`, with `proposed` true. Each version
holds `status`, `proposer`, `proposed_in`, `proposed_at` and `age_days`.

### 5. No rule set and no new views

The act and the form are new, so no entry in a log has them. The rule
set stays 1 (record-log, "An entry is judged by the rules it was
written under"). The views keep their tables: a withdrawn version is a
row of `versions` with the status `withdrawn`, its entry and its
reason. Their form stays 10.

## Risks / Trade-offs

- [An agent withdraws another agent's proposal] → The decision accepts
  this. The entry keeps its provenance, so `boom log` shows who did it.
- [A boom before this change reads a log that holds a withdrawal] → It
  refuses the entry by its form, and the version stays proposed for it.
  Each machine takes the new release before it reads such a log.
