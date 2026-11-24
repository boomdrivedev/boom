## ADDED Requirements

### Requirement: The batches that wait on a person

Boom SHALL compute the batches that wait on a person from the views. A batch SHALL be a narrative, a proposed narrative with each proposed version it includes; a revision, a cause with each proposed version that names it and the cause where proposed; or a ruling, each other proposed version alone. Each batch SHALL have one command. A batch with a closed version that refuses it SHALL name it and give no command. The batches SHALL be the oldest first.

#### Scenario: The three shapes

- **WHEN** the narrative `boom-end-to-end` includes `status-read-from-events`, proposed, `money-has-currency` names the cause `names-no-plan`, proposed, and `amounts-in-cents` is proposed alone
- **THEN** Boom gives a narrative batch of `boom-end-to-end` and `status-read-from-events`, a revision batch of `names-no-plan` and `money-has-currency`, and a ruling of `amounts-in-cents`

#### Scenario: A version that a narrative includes is no ruling

- **WHEN** `status-read-from-events` is proposed and only the proposed narrative `boom-end-to-end` includes it
- **THEN** Boom gives no ruling of `status-read-from-events`

### Requirement: The view "Waiting on you"

`boom page` SHALL give the view "Waiting on you" at `<base>/waiting`, first in the bar, with the number of batches that wait. The view SHALL give each batch in its order: its shape in words, its title, why it exists, its counts, a link to read it, and its one command, or the versions that refuse it. Above the batches, it SHALL give the number of batches and versions, and the command `boom accept --waiting`. When nothing waits, it SHALL say so.

#### Scenario: The view of three batches

- **WHEN** a narrative batch, a revision batch and a ruling wait, and `<base>/waiting` is asked for
- **THEN** the page gives the three, each with its shape, its counts, a link to its page and its command, and the bar gives "Waiting on you" with `3`, marked `aria-current="page"`

#### Scenario: A narrative batch refused

- **WHEN** a narrative includes a rejected version, and `<base>/waiting` is asked for
- **THEN** its batch names the rejected version, and gives no command

#### Scenario: Nothing waits

- **WHEN** no version is proposed, and `<base>/waiting` is asked for
- **THEN** the page says that nothing waits on you, and the bar gives "Waiting on you" with no number

### Requirement: A narrative's page lists what its accept takes

The page of a proposed version of a narrative SHALL give, under "To accept", the command and each version that the accept takes: the narrative, then each proposed version that it includes. Where it includes a rejected, superseded or deprecated version, it SHALL name each one and give no command.

#### Scenario: What the accept takes

- **WHEN** `boom-end-to-end@<v>` is proposed and includes `status-read-from-events@<v>`, proposed, and its page is asked for
- **THEN** "To accept" gives `boom accept boom-end-to-end@<v> --key <key>` and lists `boom-end-to-end@<v>` and `status-read-from-events@<v>`

#### Scenario: An include that refuses the accept

- **WHEN** `boom-end-to-end@<v>` includes a rejected version, and its page is asked for
- **THEN** "To accept" names the rejected version, and gives no command
