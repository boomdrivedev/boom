## ADDED Requirements

### Requirement: The view "Waiting on you" gives the person's own command

Each command SHALL be the one that the person runs. This SHALL hold on the view "Waiting on you", on the page of a proposed claim and on the page of a narrative. It SHALL have no `--key` where Boom's settings name a key that the store registers to a person, and `--key <key>` otherwise. With no such key, the view SHALL give the line of the settings that names one. Boom SHALL read the settings for each request.

#### Scenario: A key in the settings

- **WHEN** the settings name chuck's key, a narrative batch, a revision and a ruling wait, and `<base>/waiting` is asked for
- **THEN** the view gives `boom accept <narrative>@<v>` and `boom accept --cause <cause>@<v>`, each with no `--key`, and says that the key is chuck's

#### Scenario: No key in the settings

- **WHEN** the settings name no key, or a key that the store does not register, and `<base>/waiting` is asked for
- **THEN** each command ends with `--key <key>`, and the view gives the line `key = { path = "~/.ssh/<your key>", askpass = "<program>" }`

### Requirement: The view "Waiting on you" gives one command for every batch

Where more than one batch waits, the view SHALL give one command for each batch that Boom does not refuse, in one sign. It SHALL give the heads of the narratives and the rulings in the order of the batches, then `--cause` with the head of each revision. It SHALL say that `boom accept --waiting` lists the batches. `boom accept --waiting` SHALL give the same command, as `command`.

#### Scenario: Three batches in one command

- **WHEN** a narrative batch, a revision and a ruling wait, and `<base>/waiting` is asked for
- **THEN** the view gives `boom accept <narrative>@<v> <ruling>@<v> --cause <cause>@<v>`, and `boom accept --waiting --json` gives it as `command`

### Requirement: The person picks batches into one command

Each batch that Boom does not refuse SHALL have a box, `pick`, whose value is `<shape>-<name>` of its head. A form SHALL send `GET` to `<base>/waiting` with each box checked. Above the batches, the view SHALL then give the one command for the picked batches, in their order. Each picked box SHALL stay checked. A value that names no batch that waits SHALL be passed over. The view SHALL hold no `<script>` element.

#### Scenario: Two batches picked

- **WHEN** `<base>/waiting?pick=narrative-<narrative>&pick=revision-<cause>&pick=ruling-gone` is asked for
- **THEN** the view gives "The 2 batches you picked, in one sign" and `boom accept <narrative>@<v> --cause <cause>@<v>`, the two boxes checked and the box of the ruling not checked

#### Scenario: Nothing picked

- **WHEN** `<base>/waiting` is asked for with no `pick`
- **THEN** the view gives a box on each batch that Boom does not refuse, no command of picked batches, and no `<script>` element
