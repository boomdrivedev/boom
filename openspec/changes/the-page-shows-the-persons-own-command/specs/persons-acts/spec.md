## ADDED Requirements

### Requirement: A person's key in Boom's settings

Boom SHALL read its settings from `$XDG_CONFIG_HOME/boom/config.toml`, else `$HOME/.config/boom/config.toml`. The table `key` SHALL name the private key file, `path`, and as an option the program that gives its passphrase, `askpass`. Each path SHALL be absolute or start with `~/`, read as the home. No command SHALL write the file.

#### Scenario: A path under the home

- **WHEN** the settings hold `key = { path = "~/.ssh/id_ecdsa_sk", askpass = "~/bin/askpass" }`
- **THEN** Boom reads the key as `<home>/.ssh/id_ecdsa_sk` and the program as `<home>/bin/askpass`

#### Scenario: A path that is not absolute

- **WHEN** the settings hold `key = { path = "keys/ada" }`, and ada runs `boom accept <name>@<version>` with no `--key`
- **THEN** it exits 2 and says that `key.path` is a path that is absolute or starts with `~/`, and nothing is appended

### Requirement: Boom refuses settings that it does not read

Boom SHALL refuse a settings file that it cannot read. It SHALL refuse a file with a key that it does not read, or with a path of another form. The refusal SHALL name the setting, and nothing SHALL be appended.

#### Scenario: A key that Boom does not read

- **WHEN** the settings hold `key = { path = "/keys/ada", passphrase = "x" }`, and ada runs `boom accept <name>@<version>` with no `--key`
- **THEN** it exits 2 and says that the settings are not read, and nothing is appended

### Requirement: A person's act signs with the key of the settings

`boom accept`, `boom reject`, `boom approve`, `boom register`, `boom repo add` and `boom repo set` SHALL sign with the key of Boom's settings where no `--key` is given. `boom propose` and `boom accept --waiting` SHALL take only `--key`.

#### Scenario: An accept with no --key

- **WHEN** the settings name ada's key, and ada runs `boom accept <narrative>@<version>` with no `--key`
- **THEN** it exits 0, ada's key signs one time, and one `accepted` entry names the narrative and each proposed version that it includes

#### Scenario: The batches listed with a key in the settings

- **WHEN** the settings name ada's key, and `boom accept --waiting` runs with no `--key`
- **THEN** it exits 0, gives each batch with its command and no `--key`, and the log does not change

### Requirement: The passphrase program of the settings

Where Boom's settings name a passphrase program, Boom SHALL run `ssh-keygen` with `SSH_ASKPASS` set to it and `SSH_ASKPASS_REQUIRE` set to `force`, for the key of the settings. A `--key` that names the key of the settings SHALL take that program, and a `--key` that names another key SHALL take none. Boom SHALL never read the passphrase.

#### Scenario: The passphrase program reaches ssh-keygen

- **WHEN** a key with a passphrase program signs
- **THEN** `ssh-keygen` runs with `SSH_ASKPASS` set to the program and `SSH_ASKPASS_REQUIRE` set to `force`, and gets the passphrase from the program

#### Scenario: Another key

- **WHEN** the settings name ada's key and a passphrase program, and bo runs `boom accept <name>@<version> --key <bo's key>`
- **THEN** bo's key signs with no passphrase program

### Requirement: One accept takes several batches

`boom accept` SHALL take `--cause` more than one time, with targets. One entry SHALL take each target, after a narrative each proposed version that it includes, then each cause with each version that names it. The key SHALL sign one time.

#### Scenario: Two causes in one sign

- **WHEN** two causes wait, each with one revision that names it, and ada runs `boom accept --cause <first> --cause <second> --key <ada's key>`
- **THEN** it exits 0, ada's key signs one time, and one `accepted` entry names the first cause, its revision, the second cause and its revision

## MODIFIED Requirements

### Requirement: Only a person accepts or rejects

`boom accept <name>@<version> --key <key>` and `boom reject <name>@<version> --reason <text> --key <key>` SHALL each append a person's signed entry. With no `--key` and no key in Boom's settings, Boom SHALL refuse an accept or a reject, and say that only a person accepts or rejects. An unsigned one in the log SHALL count for nothing. A reject SHALL carry its reason.

#### Scenario: An agent's accept

- **WHEN** `boom accept refund-within-payment@<version>` runs with no `--key`, and Boom's settings name no key
- **THEN** it exits 2 saying only a person accepts, and the version stays proposed

#### Scenario: A person accepts an agent's proposal

- **WHEN** ada accepts, with ada's key, a version an agent proposed
- **THEN** the version stands, and its `accepted` entry names ada

#### Scenario: A reject with no reason

- **WHEN** `boom reject <name>@<version> --key <ada's key>` runs with no `--reason`
- **THEN** it exits 2 saying a reject gives its reason, and nothing is appended
