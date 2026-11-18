# Planning Pipeline & Token Efficiency

## Goal (DECIDED)

Most of the system is deterministic, script-like code. LLMs appear only in a few wedges. Design the planning stage so cheaper models with less thinking get results close to an expensive model.

## Where LLMs sit (DECIDED)

1. **Elaboration** — fully under the user's control (model choice, thinking level). Can be human-driven ("I'm doing a design, then a mockup on top of it") or automatic (a standing design agent researches, consults the ops agent, writes claims). Output: claims in Boom, plus narratives/designs.
2. **Planning** — after Boom reconciliation. This is the column that needs "brains baked in," via predefined Flume kinds that do cheap planning with escalation to expensive planning.

## Why planning is hard (DECIDED)

The system is iterative and constantly changing:

- Several plans may be proposed; the user accepts one; coding starts.
- New input arrives: from Chuck, or from an agent (e.g. a pre-deploy gate test finds a regression → new signal → emergency chore bolt).
- New input can invalidate existing plans: proposed, accepted, or already running.
- Plans are light at this stage: bolts and their units, with pointers back to design, narratives, and signals. (Organizing and delivering the work inside bolts is a separate thread.)

## Five design moves (PROPOSED)

1. **Deterministic impact** — code finds which bolts a changed claim touches by walking the graph (see `02-decision-graph.md`). No LLM.
2. **Small packets** — one narrow, typed question per affected bolt: *keep, amend, split, or cancel*. Cheap models handle this well and can run in parallel.
3. **Patches, not rewrites** — amendments are small structured edits to a bolt record.
4. **Escalate on conflict** — only clashing packets or cross-context changes go to a bigger model with more thinking.
5. **Stable prefix** — constitution and context map always first, in fixed order, so prompt caching works.

## Plan revision semantics (LEANING — Chuck said "yeah")

- A changed plan becomes a **new bolt revision that supersedes** the old one; history stays.
- A **running** bolt gets an **amendment record** pointing at it, not an in-flight edit.
- A regression becomes a **new chore bolt** pointing at its signal; the impact walk flags plans it touches and queues them for cheap rework.

## Centralization (LEANING)

Planning is central per drive (one planner, one consistent view), so cheap-model packets don't step on each other. A Durable Object is single-threaded, which serializes planning for free in the cloud.

## Review (DECIDED)

Review happens through the rail, at agent speed, not through Git PRs.
