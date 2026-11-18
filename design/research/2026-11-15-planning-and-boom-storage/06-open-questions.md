# Open & Parked Questions

## PARKED: CRDTs for live state

DeltaDB (from Zed) is a CRDT-based, operation-based version control layer that interoperates with Git and records every edit with the conversation behind it. The idea of a CRDT graph synced through Durable Objects was explored.

**Parked because:** with a central planner and a derived graph index, CRDTs add complexity without a current need. None of the independent tools have multi-writer live state today; adding it would cut across how each tool owns its own state. Revisit as its own design pass. If revisited, likely fit is live state only (bolt status, leases, progress, annotations), never claims or plans — CRDTs converge data but don't make decisions coherent.

## OPEN

- **Entity types review:** walk the existing 5–6 Boom entity types and confirm each fits the log + view + revision model.
- **Bolt record schema:** exact fields for claim links (ID + revision), code-area hints, dependencies, units, signal pointers.
- **Packet format:** precise structured output for keep / amend / split / cancel and for amendment patches.
- **Escalation rules:** what counts as a "conflict" or "cross-context" change that goes to the bigger model.
- **Similarity net:** whether to embed at all initially; if so, derived local index (e.g. sqlite-vec) rebuilt incrementally.
- **Other tools' storage:** if Boom moves off Git/recfiles, do Millpond, Flume, Headwater, Eddy follow the same local-SQLite + optional-cloud pattern? Sluice likely stays Git-native.
- **Standalone multi-host:** how a designated Boom host is configured and reached without the cloud.
- **Boom Drive ↔ Timber coupling:** kept separate for blobs; broader integration still open.
- **Inside-bolt delivery:** organizing and delivering work within a bolt is a separate thread.
