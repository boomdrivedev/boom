# Boom Drive — Planning & Boom Storage Context Pack

Context from a design session (Nov 15, 2026), kept as the user gave it on 2026-11-18, less its commercial lines, which this repository holds none of (core.4) on making Boom Drive's planning stage token-efficient, and on rethinking how Boom stores its data.

## Status tags

- **DECIDED** — Chuck stated or explicitly agreed.
- **LEANING** — Chuck agreed in direction; details not settled.
- **PROPOSED** — suggested in the session, not yet confirmed by Chuck.
- **PARKED** — deliberately deferred for a separate design pass.
- **OPEN** — unresolved question.

Treat PROPOSED items as suggestions to evaluate, not requirements.

## Files

| File | Topic |
|---|---|
| `01-planning-pipeline.md` | Where LLMs sit, the planning column, cheap vs. expensive planning |
| `02-decision-graph.md` | The typed graph of claims, bolts, contexts; impact detection; hidden coupling |
| `03-agent-context-packets.md` | What a planning agent actually sees |
| `04-boom-storage.md` | Append-only log in SQLite, revisions, materialized views, content-addressed blobs |
| `05-local-first-and-cloud.md` | Standalone vs. attached, runner as sync channel |
| `06-open-questions.md` | Parked and open items |

## Terminology (from earlier sessions)

- **Boom Drive** — umbrella brand and the cloud control plane (rail + board).
- **`boomdrive`** — the open-source local runner CLI; attaches a drive to the cloud on login.
- **drive** — one instance (formerly "flywheel"); every drive has a Boom store.
- **Boom** — open-source claims tool: claims, ADRs, constitutions, narratives, context maps.
- **Bolt** — a unit of planned work (from AWS AI-DLC: intent → units → bolts → construction).
- **Elaboration** — LLM work that turns an intent into design output and claims; elaborations can stack (design → mockup).
- **Rail and board columns** — Intents → Plans → Construction → Operation.
- **Other tools** — Millpond (signal inbox), Headwater/Eddy (signal adapters), Flume (agent dispatch/state), Sluice (merge queue/local CI), Timber (artifacts, channels, environments, attestation; separate product), Weir (shared lib).

## One-paragraph summary

Planning is the key LLM-heavy stage. To let cheaper models with less thinking do most of it, code does the "what is affected" reasoning by walking a typed graph of claims, contexts, and bolts, and the model only answers narrow per-bolt questions. Planning stays central per drive. Boom shifts from recfiles in a dedicated Git repo to a self-contained tool with a local SQLite database: an append-only, versioned log with materialized views per entity type, plus content-addressed blobs for long narratives and designs. Standalone it's fully local; attached to Boom Drive, a Durable Object becomes the single ordering authority and every host keeps a local replica. Cloud Boom *is* Boom Drive. CRDTs are parked.
