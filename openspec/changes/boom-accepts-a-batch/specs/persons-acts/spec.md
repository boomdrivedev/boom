## ADDED Requirements

### Requirement: The accept of a narrative takes the claims that it includes

For each target of `boom accept` that is a version of a narrative, Boom SHALL put in the one entry that version, then each version that it includes whose status is proposed, in the narrative's order. Boom SHALL leave out an included version that is accepted. Boom SHALL refuse the accept while a narrative of it includes a version that is rejected, superseded or deprecated, SHALL name each such version, and SHALL append nothing. A version that two targets take SHALL be in the entry one time.

#### Scenario: A narrative and its proposed claim

- **WHEN** `boom-end-to-end@<v>` includes `status-read-from-events@<v>`, proposed, and `money-has-currency@<v>`, accepted, and ada runs `boom accept boom-end-to-end@<v> --key <ada's key> --json`
- **THEN** it exits 0, the log gets one `accepted` entry that names `boom-end-to-end@<v>` then `status-read-from-events@<v>`, the key of ada signs one time, and `accepted` lists the two

#### Scenario: A narrative that includes a rejected version

- **WHEN** `boom-end-to-end@<v>` includes `status-read-from-events@<v>`, which a person rejected, and ada accepts `boom-end-to-end@<v>`
- **THEN** it exits 2 and names `status-read-from-events@<v>` as rejected, the key signs nothing, and the log is as it was

#### Scenario: A narrative that includes a superseded draft

- **WHEN** `boom-end-to-end@<v>` includes a draft of `status-read-from-events`, a newer draft supersedes it, and ada accepts `boom-end-to-end@<v>`
- **THEN** it exits 2 and names the included draft as superseded, and the log is as it was

#### Scenario: Two narratives that include one claim

- **WHEN** `boom-end-to-end@<v>` and `change-records@<v>` each include `status-read-from-events@<v>`, proposed, and ada accepts the two narratives in one command
- **THEN** it exits 0, and the one `accepted` entry names `status-read-from-events@<v>` one time

#### Scenario: A narrative accepted again

- **WHEN** ada runs that accept of `boom-end-to-end@<v>` again
- **THEN** it exits 0 and lists the two versions under `accepted`, the key signs nothing, and the log does not change

### Requirement: The accept of a cause takes its revision

`boom accept --cause <name>@<version> --key <key>` SHALL put in one entry the cause, where it is proposed, then each version whose edge `cause` names it and whose status is proposed, in the order of the entries that proposed them. A version that names the cause and is accepted SHALL be listed and SHALL not be signed again. Boom SHALL refuse a cause that is rejected, superseded or deprecated, and a cause that no version names, and SHALL append nothing.

#### Scenario: A cause and its two versions

- **WHEN** `names-no-plan@<v>` is proposed, `money-has-currency@<v2>` and `status-read-from-events@<v2>` each amend an accepted version with `--cause names-no-plan@<v>`, and ada runs `boom accept --cause names-no-plan@<v> --key <ada's key> --json`
- **THEN** it exits 0, the one `accepted` entry names `names-no-plan@<v>`, `money-has-currency@<v2>` and `status-read-from-events@<v2>`, and each of the two amended claims has its new version accepted

#### Scenario: A superseded draft is left out

- **WHEN** a newer draft of `money-has-currency` with the same cause supersedes `money-has-currency@<v2>`, and ada accepts `--cause names-no-plan@<v>`
- **THEN** the entry names the newer draft, and not `money-has-currency@<v2>`

#### Scenario: A cause that no version names

- **WHEN** ada runs `boom accept --cause money-has-currency@<v> --key <ada's key>`, and no version names it as its cause
- **THEN** it exits 2 and says that no version names `money-has-currency@<v>` as its cause, and the log is as it was

### Requirement: An accept lists what it takes

Before the key signs, `boom accept` SHALL write on standard error each version that its entry takes, one on each line, with its kind, after a line that gives their number. Its answer SHALL list under `accepted` each version that the accept takes, with its status.

#### Scenario: The list before the key

- **WHEN** ada accepts `boom-end-to-end@<v>`, which includes `status-read-from-events@<v>`, proposed
- **THEN** standard error says that the accept takes 2 versions in one entry, then gives `narrative boom-end-to-end@<v>` and `invariant status-read-from-events@<v>`, each on its line

### Requirement: boom accept --waiting

`boom accept --waiting` SHALL give the batches that wait on a person, as the view "Waiting on you" of `boom page` gives them, each with its shape, its versions, its counts and the command that accepts it, and SHALL take no target and no `--cause`. With no `--key` it SHALL exit 0 and append nothing. With `--key` it SHALL accept, in one entry that the key signs one time, each version of each batch that Boom does not refuse, and SHALL name each batch that it leaves out.

#### Scenario: The batches listed

- **WHEN** a narrative with one proposed claim, a cause with one version that names it, and one other proposed claim wait, and `boom accept --waiting --json` runs with no `--key`
- **THEN** it exits 0 and gives three batches, a narrative, a revision and a ruling, each with its command, and the log does not change

#### Scenario: Nothing waits

- **WHEN** no version is proposed, and `boom accept --waiting` runs
- **THEN** it exits 0 and says that nothing waits on a person

#### Scenario: Each batch in one sign

- **WHEN** ada runs `boom accept --waiting --key <ada's key> --json` on those three batches
- **THEN** it exits 0, the key of ada signs one time, one `accepted` entry names each of the five versions, and `boom accept --waiting` then gives no batch

#### Scenario: A refused batch is left out

- **WHEN** a narrative that includes a rejected version waits beside a ruling, and ada runs `boom accept --waiting --key <ada's key>`
- **THEN** it exits 0, the entry names the ruling's version, the narrative stays proposed, and the answer names the narrative as left out with the rejected version
