# The Decision Graph

## Shape (PROPOSED)

A typed graph built from Boom's records.

- **Nodes:** claims, contexts, ADRs, narratives, bolts (and their units), signals, optionally code areas.
- **Edges:** `supersedes`, `depends_on`, `constrains`, `implemented_by`, `covers`, `touches_code`, `raised_by` (signal).
- **Each edge records the revision** of the target it was made against (e.g. bolt B7 covers claim C12 @ rev 3).

## Edges are stored; the index is derived (DECIDED — clarified in session)

- Edges are **data**, stored inside records. When the planner creates a bolt, it writes a bolt record with its links built in: claim IDs + revisions it covers, what it depends on, which signals raised it.
- The **queryable graph index** is rebuilt from records. Like foreign keys in rows plus an index built from them.
- So planning output *does* go back into the graph: as records, and the graph follows.

## What the graph gives you (PROPOSED)

- **Impact:** a claim changes → walk edges → find affected bolts. Plain traversal, no LLM.
- **Staleness:** a bolt is stale when the revision on its edge is behind the claim's current revision. You know which bolts need a look *and* exactly what changed.
- **Caching:** sort context with stable things first (constitution, rarely-changing upstream claims), changing leaves last. Topological order ≈ cache-friendly prompt.

## Hidden coupling (PROPOSED)

The hard case: two things with unrelated claims that still affect each other. Layered nets, cheapest first:

1. **Shared code areas** — bolts record modules/paths they expect to touch; overlap flags them. Deterministic.
2. **Context map** — if billing depends on accounts, an accounts change flags billing bolts. Graph walk.
3. **Similarity net** — embeddings over light node summaries surface "looks related" candidates. Cheap, fuzzy.
4. **Cheap triage** — a small model gets new claims + the short candidate list (not the whole graph) and answers affected / not affected per candidate.
5. **Late catch** — what slips through usually shows up in Sluice as a merge conflict or failing check, which returns as a Millpond signal.

**Learning:** whenever a hidden link is found, write it back as a real edge so next time layer 1 or 2 catches it.

## New-feature gap (PROPOSED)

A brand-new feature has no edges yet. Contexts carry the load at first (elaboration tags claims with contexts), with the similarity check as backup.
