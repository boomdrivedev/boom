## Context

Boom keeps a log of entries. Each entry names its format,
`boom-entry/1`, its act, its time, `at`, and its actor. A person's entry
carries a signature over its signed part. The judge decides whether an
entry counts. It judges an entry before a write signs it, again inside
the write, and for every entry when the views are built again from the
log (`judge::replay`).

The judge has one set of rules, the newest. When a release adds a rule,
each replay holds every old entry to it. The log does not record which
boom wrote an entry, so the judge cannot tell an entry that broke a rule
from an entry written before the rule.

Two rules came after entries were already in the log, and refuse
entries that counted when they were written:

| rule | release | what it refuses on a replay |
|---|---|---|
| a narrative binds one level | 0.15 | entries 338 and 352, and so 601, 723 and 725 |
| a draft that a newer draft superseded is not accepted, rejected or amended | 0.16 | entries 603 to 611 |

Each other rule that came after the first release either came with the
act, the kind or the field that it judges, so no older entry has it, or
it made a rule less strict. "A term binds one repository or the whole
system" let a term bind the whole system. "An accept stands on the
nearest accepted base" let an accept pass where it was refused before.

No other tool of the family judges a signed log again when it reads
it: sluice, flume and timber keep no log that a later rule can refuse.
Thus this design has one owner, boom's judge.

## Goals / Non-Goals

**Goals:**

- Each entry is judged by the rules in force when it was written, on
  every replay, for every entry.
- A rule added later holds for each entry written after it.
- A person's signature covers the rules that the person signed under.

**Non-Goals:**

- No change to an entry already in a log: its signed part, its hash and
  its signature stay as they are.
- No transition code: nothing names an entry, a store or a claim.

## Decisions

### 1. An entry names its rule set

`judge::RULES` is the number of the rule set that this boom writes and
judges new entries by: 1. Each entry that boom writes holds `rules` in
its signed part, after `at`, so a person's signature covers it. When
`rules` is 0, the signed part leaves it out. Thus an entry written
before this change reads, writes and hashes as it did, as rule set 0.

Not taken: the version of boom. Main names `0.0.0` (core.8), so a build
from the tree does not know its release.

Not taken: the entry's time against the time of each release. A
release reaches a machine later, and `at` is the clock of the machine.
Entry 352 was written 70 minutes after 0.15 was released, by a boom
before 0.15, and is valid.

### 2. Each dated rule names the rule set that brought it

A rule that came into force after entries were in a log names the rule
set from which it holds: `ONE_LEVEL` and `ONE_DRAFT_LINE`, each 1. The
judge holds an entry to such a rule only when the entry's rule set is
that one or later. Every other rule holds for every entry.

### 3. Rule set 0 holds only the rules that each boom before held

An entry of rule set 0 was written by some boom before this change, and
the log does not say which. Boom wrote it only after its own judge
counted it. So rule set 0 holds each rule that every boom before this
change held: a rule that one release added does not hold for it, and a
rule that a release made less strict holds in its less strict form.

Under rule set 0, no draft supersedes another for the judge: a draft
that a newer draft superseded is judged as a draft that waits. Thus an
accept or a reject of it, of rule set 0, counts, as it counted when it
was written. The state keeps the supersession, as it does now.

### 4. A newer rule set is refused

An entry that names a rule set greater than `RULES` was written by a
newer boom. This boom does not hold that rule set's rules, so it
refuses the entry and says that a newer boom judges it. A boom before
this change refuses an entry with `rules` by its form, as it refuses
each field that it does not know.

### 5. The views are built again

The views' form goes from 9 to 10, so each store replays its whole log
under the new judge when a command opens it.

## Risks / Trade-offs

- [An entry of rule set 0 written by 0.15 or later is not held to "a
  narrative binds one level" on a replay] → Its writer held it to that
  rule before it signed or appended it, so it passed the rule. Only an
  entry that a program wrote by hand into the file escapes the rule, and
  such an entry escapes no rule of rule set 0.
- [Each later rule must name its rule set] → A rule that a release adds
  for an act, a kind or a field that entries already have names
  `RULES + 1`, and the release raises `RULES`. A rule for a new act,
  kind or field needs no rule set, as no older entry has it.
