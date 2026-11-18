# Agent Context Packets

## Principle (PROPOSED)

**Code finds, agent decides.** The planning agent never sees the whole graph and doesn't "just know" anything. Code computes the affected slice; the agent gets a small packet.

## Three layers

1. **Graph in SQLite.** Nodes are light: ID, type, title, tags, context, revision. Long narratives and designs are pointers (content hashes).
2. **Code walks it.** New claims arrive tagged with contexts. Boom walks out from those claims and contexts to every bolt that covers or depends on them, proposed or running.
3. **Agent gets one packet per affected bolt.**

## Packet layout

```
[stable, cached]
  constitution
  short context map

[slice]
  new / changed claims (with diff vs. the revision the bolt was planned against)
  the affected bolt + its units
  nearest neighbors (one hop), as one-line nodes: ID | type | title | tags | rev

[pointers]
  narrative / design refs as ID@rev (hash-addressed), NOT their text

[question]
  keep | amend | split | cancel
  answer in a fixed structured format (amendments as patches)
```

## Long content

- Heavy content (narratives, designs, mockups) is fetched on demand via a tool, e.g. `boom show N12@3`.
- Most decisions are answerable from one-line claims; that's a large part of what makes cheap models viable.
- Pointers are revision-pinned, so the agent sees exactly the version the plan was built against.
