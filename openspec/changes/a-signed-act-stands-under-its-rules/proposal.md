## Why

Boom judges each entry of its log again when it builds its views, and
today it judges every entry by its newest rules. Boom 0.15 added the
rule "a narrative binds one level". On a replay, that rule refused
entries that boom wrote before the rule existed:

- the proposals of the narrative `decisions-that-check-themselves`,
  entries 338 and 352, which bind two repositories;
- the person's accept of 34 versions, entry 601, because one of the
  versions is that narrative;
- the outline, entry 723, and the person's accept of it, entry 725.

Boom 0.16 added the rule "a draft that a newer draft superseded is not
rejected". On a replay, that rule refused the person's rejects of old
drafts, entries 603 to 611, which boom 0.15 wrote. On the person's
store, 115 of 434 claims had no accepted version, many for these
reasons.

On 2026-11-24 the person ruled: fix Boom so that each entry is judged
by the rules in force when it was written. A new rule never voids a
signature given before it.

## What Changes

- **An entry names its rule set.** Each entry that boom writes names
  `rules`, the number of the rule set that boom judged it by, in its
  signed part. A person's signature covers it. This boom writes rule
  set 1.
- **An entry with no rule set is of rule set 0.** Each entry written
  before this change names no rule set, and reads as rule set 0. Its
  signed part, its hash and its signature stay as they are.
- **Replay judges each entry by its own rule set.** A rule that came
  into force with a later rule set does not hold for the entry. Rule
  set 1 brings two rules: "a narrative binds one level", and "a draft
  that a newer draft superseded is not accepted, rejected or amended".
- **A newer rule set is refused.** Boom refuses an entry that names a
  rule set that it does not hold, and says that a newer boom judges it.
- **`boom log --json` gives each entry's `rules`.**
- **The views take a new form**, 10, so each store builds them again
  from its log under its entries' own rules.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `record-log`: two new requirements, "An entry names its rule set"
  and "An entry is judged by the rules it was written under". "An entry
  that fails boom's checks counts for nothing" holds an entry to the
  checks of its own rule set.

## Impact

- `crates/boom/src/entry.rs`: the field `rules` of an entry, written
  after `at` where it is not 0.
- `crates/boom/src/judge.rs`: `RULES`, the rule set of each dated rule,
  and the rule set that each check reads from its entry.
- `crates/boom/src/cli.rs`: each new entry names `RULES`; `boom log
  --json` gives `rules`.
- `crates/boom/src/store.rs`: the views' form goes from 9 to 10.
- `crates/boom/tests/rules.rs`: a test for each scenario.
- The claim `an-entry-is-judged-by-the-rules-it-was-written-under`,
  proposed in Boom for the person to accept.
- The person's store: each entry of its log counts. Entries 338, 352,
  601, 603 to 611, 723 and 725 count again. 79 claims have no accepted
  version: 75 wait on the person, and the person rejected each version
  of 4.
- A boom before this change refuses the form of an entry that names
  `rules`, as it refuses each field that it does not know.
