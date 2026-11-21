# repository-settings Specification

## Purpose
The repositories that a store describes, the remote of each, and how
each gets the context of Boom. These are settings in the log, which an
owner signs, and never a file in a repository
(`context-either-at-session-start-or-synced-into-the-repository`).

## Requirements

### Requirement: An owner registers a repository and its remote

`boom repo add <name> --remote <url> --key <key>` SHALL append one `repository` entry that an owner signs. The entry SHALL name the repository, its remote and its delivery at session start, which starts on. Boom SHALL refuse the act of a person who is not an owner, and the act of an agent. Boom SHALL refuse a name that is not kebab case, and a remote that another repository has.

#### Scenario: A repository registered

- **WHEN** the owner ada runs `boom repo add sluice --remote git@github.com:boomdrivedev/sluice.git --key <ada's key>`
- **THEN** boom exits 0, and `boom repo show sluice` gives `sluice`, the remote `github.com/boomdrivedev/sluice`, delivery at session start on, and the entry that ada signed

#### Scenario: Not an owner

- **WHEN** bo, a person who is not an owner, runs `boom repo add sluice --remote <url> --key <bo's key>`
- **THEN** boom exits 2 and says that only an owner sets a repository, and the log does not change

#### Scenario: A remote taken

- **WHEN** `sluice` has the remote `github.com/boomdrivedev/sluice`, and the owner adds `other` with the same remote
- **THEN** boom exits 2 and names `sluice`, and the log does not change

### Requirement: A remote in one form

Boom SHALL keep a remote in one form: the host in lowercase, then the path. The form SHALL have no scheme, no user, no port and no `.git` at its end. Boom SHALL read `https://`, `ssh://` and `git@<host>:` forms into that form. Two remotes in different forms that give one form SHALL match.

#### Scenario: Three forms, one remote

- **WHEN** the remotes are `git@github.com:boomdrivedev/sluice.git`, `https://github.com/boomdrivedev/sluice` and `ssh://git@GitHub.com/boomdrivedev/sluice.git`
- **THEN** boom reads each as `github.com/boomdrivedev/sluice`

### Requirement: An owner changes a repository

`boom repo set <name> --key <key>` SHALL append one `repository` entry that an owner signs. It SHALL take `--remote <url>`, `--session on` or `--session off`, or more than one of them. The newest counted entry for a repository SHALL give its settings. A set that changes nothing SHALL append nothing.

#### Scenario: Delivery at session start turned off

- **WHEN** the owner runs `boom repo set sluice --session off --key <key>`
- **THEN** `boom repo show sluice` gives delivery at session start off, and the entry of that set

#### Scenario: The same setting again

- **WHEN** the owner runs that set again
- **THEN** boom exits 0, and the log does not change

### Requirement: boom repo show gives the settings

`boom repo show` SHALL give each repository that the store registers, with its remote, its delivery at session start, and the entry and person that set it. `boom repo show <name>` SHALL give one. With `--json`, the answer SHALL be `boom-repo/1`. Boom SHALL refuse a name that the store does not register.

#### Scenario: Two repositories

- **WHEN** the store registers `boom` and `sluice`, and `boom repo show --json` runs
- **THEN** it gives `boom` and `sluice`, each with its remote, its delivery and its entry
