# plan-approvals Specification

## Purpose
A person's signed approval of a plan: boom names the plan by the SHA-256
of its document and keeps nothing of it, covers each claim at its
accepted version, and links the claims that it covers and each entry
that the plan took in that the log holds.

## Requirements

### Requirement: A person approves a plan

`boom approve <plan>` SHALL append an `approved` entry, signed by the person's key, the person present. Its signed part SHALL hold `plan`, the SHA-256 of the bytes of the file `<plan>`; `links`; `outputs`, each `--output git:<repository>@<commit>:<path>`; and `covers`, each `--covers <name>[@<version>]` as `<name>@<version>`, each in the order given and once. Boom SHALL keep nothing of the plan's file.

#### Scenario: A plan approved

- **WHEN** the log holds `sounding.elaboration-ended` `<e>`, `status-read-from-events@<v1>` is accepted, and ada runs `boom approve plan.md --link <e> --covers status-read-from-events --key <ada's key>`
- **THEN** boom exits 0, and the `approved` entry holds the SHA-256 of `plan.md`, the link `<e>`, and `status-read-from-events@<v1>`, and is signed by ada

#### Scenario: Nothing of the plan is kept

- **WHEN** ada approves `plan.md`
- **THEN** the store holds no blob of the bytes of `plan.md`

#### Scenario: An output commit

- **WHEN** ada approves a plan with `--output git:boom@<40 hexadecimal characters>:design/narrative.md`
- **THEN** the entry's `outputs` hold that reference as given

### Requirement: An approval links the claims it covers and each elaboration the log holds

The links of an approval SHALL be the act hash of the `proposed` entry of each covered version, in the order of `covers`, then each `--link <act hash>` in the order given, each once. A `--link` SHALL name an entry that the log holds. A plan that came from no elaboration SHALL link only its claims.

#### Scenario: A plan from an elaboration

- **WHEN** the log holds `sounding.elaboration-ended` `<e>`, and `status-read-from-events@<v1>`, proposed by the entry `<p>`, is accepted, and ada runs `boom approve plan.md --link <e> --covers status-read-from-events --key <ada's key>`
- **THEN** the entry's `links` are `[<p>, <e>]`

#### Scenario: A plan from no elaboration

- **WHEN** `status-read-from-events@<v1>`, proposed by the entry `<p>`, is accepted, and ada runs `boom approve plan.md --covers status-read-from-events --key <ada's key>`
- **THEN** boom exits 0, and the entry's `links` are `[<p>]`

### Requirement: An approval rests on what the log holds and the person accepted

Boom SHALL refuse an approval that covers no claim and links no entry, a covered version that is not accepted, and a second approval of a plan's SHA-256 that the log approves. Boom SHALL name each covered version that is not accepted. An agent SHALL NOT approve.

#### Scenario: A covered version that is not accepted

- **WHEN** `money-has-currency@<v1>` is proposed and not accepted, and ada runs `boom approve plan.md --link <e> --covers money-has-currency@<v1> --key <ada's key>`
- **THEN** boom exits 2 and names `money-has-currency@<v1>` as not accepted, and boom appends nothing

#### Scenario: An approval with no link

- **WHEN** ada runs `boom approve plan.md --key <ada's key>`
- **THEN** boom exits 2 and says that an approval covers a claim or links an entry, and boom appends nothing

#### Scenario: One plan approved two times

- **WHEN** the log approves the SHA-256 of `plan.md`, and ada runs `boom approve plan.md --link <e> --key <ada's key>` again
- **THEN** boom exits 2 and names the entry that approves it, and boom appends nothing

#### Scenario: An approval appended by hand with no signature

- **WHEN** an agent appends, chained correctly, an `approved` entry with no signature
- **THEN** boom counts it for nothing, and `boom log --json` says that a person's act needs their signature

### Requirement: The log names an approval

`boom log` SHALL give the line of an approval as its number, `approved plan`, the first 12 characters of the plan's SHA-256, and the person.

#### Scenario: The line of an approval

- **WHEN** entry 7 approves a plan whose SHA-256 starts `9c76d608d2d7`, by ada, and `boom log` runs
- **THEN** line 7 is `7 approved plan 9c76d608d2d7 by ada`
