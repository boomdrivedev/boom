# Local-First & Cloud

## Cloud Boom = Boom Drive (DECIDED)

There's no separate "Boom cloud" product. Every drive has a Boom store; the cloud version of Boom is Boom Drive. Timber remains a separate product.

## Sync model: local-first, cloud is the authority (DECIDED)

- **Always a local copy.** Agents read claims locally: fast, offline-capable, free context compilation.
- **Attached = cloud orders writes.** New claims/plans go up as proposals; the drive's Durable Object assigns official sequence numbers and handles approval through the rail; results come back down to every host.
- **No CRDTs needed.** Decisions have one writer and go through approval; each host just replays entries it hasn't seen.
- **Outbox:** when attached, local writes sit in an outbox as proposals until the cloud sequences them. Standalone, Boom numbers entries itself.

## The runner is the sync channel (LEANING — the "missing link")

This answers the long-open question of how `boomdrive` (the runner) talks to the control plane for Boom: the runner ships proposals up and pulls the log down. Boom itself barely needs to know the cloud exists.

## No designated Boom host in the cloud (DECIDED)

With Boom Drive cloud and five runners, no runner is special. The Durable Object is the one authority; all runners keep local replicas, any can propose, all pull the same log.

Without the cloud, someone must designate one machine as the Boom host and point others at it. The cloud removes that chore.

## Earlier framing that still applies (PROPOSED)

- Decisions (claims, plans) at decision speed; live state (leases, agent status, progress) at agent speed.
- Live state: local storage standalone; a Durable Object when logged in.
