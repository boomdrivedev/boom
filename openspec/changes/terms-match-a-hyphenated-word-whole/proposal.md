## Why

The term `origin` is to replace the old word `came-from` for boom. <!-- boom: allow came-from -->
The check splits a name in code into its words at `_`, `-` and `.`.
Thus the old word `came-from` also matches the plain words "came from". <!-- boom: allow came-from -->
Lines of main, such as the README, use "came from" as plain words,
and these words are correct English.

On 2026-11-21 the user chose "Stop only the name came-from". The <!-- boom: allow came-from -->
option said: "Teach the check to match a hyphenated old word only in
its hyphenated form, then propose origin replacing came-from. A small <!-- boom: allow came-from -->
boom code change first. The plain phrase stays free."

## What Changes

- An old word that holds a hyphen matches only its words joined as one
  name. Each two words are joined by `-`, by `_` or by a change from
  lowercase to uppercase. For the old word `came-from`, the check <!-- boom: allow came-from -->
  names `came-from`, `came_from`, `cameFrom`, `CameFrom` and <!-- boom: allow came-from -->
  `CAME_FROM`. <!-- boom: allow came-from -->
- The check never names the plain words "came from", `came.from`, or
  the words with two marks or more between them, such as `came - from`.
- An old word with no hyphen matches as before. The old word
  `drive log` matches "the Drive Log", `drive_log` and `DriveLog`.
- `written` gives a name in code as the line writes it: `DriveLog`,
  not `Drive Log`.
- This change proposes no term. The term `origin` that replaces
  `came-from` is the next step, after this change lands. <!-- boom: allow came-from -->

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `term-check`: the requirement "The words that the check matches"
  gets the rule for an old word that holds a hyphen.

## Impact

- `crates/boom/src/terms.rs`: the check of the marks between the words
  of a match, and its tests.
- `crates/boom/tests/terms.rs`: a test for each new scenario.
- The hook of the plugin and the gate task `root:terms` run the same
  check, so they change with it.
- No change to the log, the views or the fingerprint.
