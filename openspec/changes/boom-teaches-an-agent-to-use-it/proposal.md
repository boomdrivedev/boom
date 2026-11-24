## Why

Boom's plugin for Claude Code has hooks and no skill. An agent that
proposes to Boom learns how only from its brief, and a brief loses
rules. On 2026-11-23 and 2026-11-24, agents proposed claims that named
waves and the happy path, chains of drafts that no person could accept,
bodies that were not in Plain, and lists of versions with no narrative.

On 2026-11-24 the user said of the rule that a claim says what the
system is, never the plan: "agreed, but shouldn't it cover all the
details on the different types of claims, how to write them, how to use
the cli, etc. etc. it seems like this is a plugin needed for boom to be
useful standalone, but then we need to wire it into flume places as we
customize kinds."

## What Changes

- Boom's plugin holds three skills. Each one works in any repository
  that Boom serves, with no other tool of the family:
  - `using-boom`: how to read and cite the claims, and which acts are a
    person's.
  - `proposing-claims`: each kind and its fields, writing in Plain, a
    claim that says what the system is and never the plan, the level
    that a claim binds, its edges, drafts and batches.
  - `writing-narratives`: a narrative and the claims that it includes at
    their paragraphs, an open question as a proposed decision, the
    answer to a note, and the outline of the design book.
- `boom propose` warns when the statement or the body of a version holds
  a plan word from a fixed list, such as `wave` or `happy path`. It
  names each word and where it is. The warning never refuses, and boom
  appends the version as before. The answer of `--json`, and of the MCP
  tool `propose`, gives each warning in `plan_words`.
- The descriptions of the plugin, in its manifest and in the
  marketplace, name the skills.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `claude-code-plugin`: a new requirement, "The plugin teaches an agent
  to use Boom": the plugin holds the three skills, and the pinned Claude
  Code validates each one strictly.
- `claim-versions`: a new requirement, "A proposal warns of plan
  words".

## Impact

- `plugins/claude-code/skills/`: the three skills, each a folder with
  its `SKILL.md`. `proposing-claims` also holds `kinds.md`.
- `plugins/claude-code/moon.yml`: `skills/**/*` in the inputs of
  `typecheck` and `test`. `tests/skills.sh` in `test`.
- `plugins/claude-code/.claude-plugin/plugin.json` and
  `.claude-plugin/marketplace.json`: the descriptions.
- `crates/boom/src/cli.rs`: the warning in `propose`, and `plan_words`
  in its JSON.
- `crates/boom/src/terms.rs`: the list of plan words, matched by the
  matcher of `boom terms check`.
- `crates/boom/tests/`: a test for each scenario of the warning.
- `README.md`: the skills and the warning, in Plain.
- flume's kinds load these skills by their names. That is a change in
  flume, which cites the design of this change.
