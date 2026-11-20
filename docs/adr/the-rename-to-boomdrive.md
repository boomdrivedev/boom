---
name: the-rename-to-boomdrive
title: Flywheel is renamed Boomdrive, and its unit of work a drive
status: proposed
date: 2026-11-04
decision-makers:
- Chuck Swanberg
---

# Flywheel is renamed Boomdrive, and its unit of work a drive

Status: proposed. The decision is the user's, in their words below.
The record waits for their yes.

Deciders: the user. The brief of the rename, of 2026-11-04, gives their
words.

## Context and problem statement

boom is one of a family of tools under one umbrella name, Flywheel. The
name changes in each repository of the family at one time. What is each
name now, and which names stay?

The user: "the ecosystem, the org, the branding, the cloud and the
opensource runner tool are all boomdrive. There is also a "boom" open
source tool that does the claim reconciliation."

## Considered options

1. Keep Flywheel.
2. Rename Flywheel, its products and its words in each repository at one
   time, as the list below gives.

## Decision outcome

Option 2, the decision of the user. The names:

- **Boomdrive** is one word. It has a capital B in prose. It is
  `boomdrive` in code, commands, packages, paths and repository names,
  and `Boomdrive` where the case of the code needs a capital. It is
  never `Boom Drive`, `BoomDrive`, `boom_drive` or `boom-drive`.
- Boomdrive is the name of Flywheel the product, the brand and the
  ecosystem. It is also the name of the family of open tools on their
  sites: sluice, flume, timber, weir and the others.
- Flywheel Cloud, the control plane, is Boomdrive Cloud, in
  agentplot/boomdrive-cloud. Its old name is flywheel-cloud.
- The runner is Boomdrive, in agentplot/boomdrive. Its old name is
  flywheel-runner. Its command is `boomdrive`, and its crates are
  `boomdrive` and `boomdrive-hosted`.
- flywheel-web is agentplot/boomdrive-web.
- tailrace is timber, in agentplot/timber: its crate, command, site and
  records.
- The unit of work, a Flywheel, is a **drive**, plural drives. It is
  `Drive` in code. It is one word in the interface and in the code.
- The word of crew for the state branch of a partition follows the
  unit: the branch of the drive. The runner marks its launches
  `boomdrive:<partition>/<n>`.
- agentplot/flywheel, the original, is archived. A mention of it as
  history names it as archived.
- boom keeps its name. A third-party thing that says flywheel keeps its
  name too.
- The family spells enroll, enrolls, enrolled, enrolling and
  enrollment. It never spells enrol or enrolment.

The names in a record or a rule change, and its decision does not
change. An archived change keeps its text. The history in the words of
a record keeps its text. A link to a renamed repository names the new
address of the repository.

### Consequences

- The tree of boom named Flywheel one time: in the constitution
  interview, which quotes the user on the storage convention. The
  quotation now says "[Boomdrive]". The brackets are the brackets that
  the interview puts around its other edits to a quotation.
- The site takes the bar and the footer of the family from
  `@agentplot/site-theme`. It pins v0.1.3, the release with the new
  names. So its bar and footer say Boomdrive and timber, and they link
  to the new addresses. The site gives the theme no member id that the
  rename changed.
- This record names Boomdrive Cloud. The words "hosted service" of
  core.4 keep that name out. In the constitution interview of
  2026-11-01, the user ruled that a record may name the umbrella and
  its products (`core.no-commercial-strategy`). That ruling rewords
  core.4, and `core.md` does not hold it yet. The record says nothing
  of pricing, a tier or a customer.
- This record has a name and no number, as `boom-names-everything-now`
  has. `check-adrs.sh` reads only numbered records.
