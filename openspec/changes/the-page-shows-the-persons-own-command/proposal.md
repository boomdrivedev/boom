## Why

On 2026-11-27 the person read the view "Waiting on you" and said: "do
i have to do 14 accept commands? and the page doesn't give me the
shortcut that you made for me". Each batch gave its own command, with
`--key <key>`. The command for all batches was `boom accept --waiting
--key <key>`. The person does not run that command. The person
accepts through a script that an agent wrote. The script runs `boom
accept` with their key. It sets `SSH_ASKPASS` to a program that reads
the key's passphrase from 1Password.

On the card, the person chose that Boom keeps the person's key and the
program that gives its passphrase in its settings.

## What Changes

- **Boom's settings name the person's key.** One line in
  `$XDG_CONFIG_HOME/boom/config.toml`, `key = { path = "...", askpass =
  "..." }`, names the private key file and, as an option, the program
  that gives its passphrase. Each path is absolute or starts with `~/`.
  The file is on the machine, and the log holds none of it.
- **A person's act takes that key.** `boom accept`, `boom reject`, `boom
  approve`, `boom register` and `boom repo add` and `set` sign with the
  key of the settings when no `--key` names one. When the settings name
  a passphrase program, Boom runs `ssh-keygen` with `SSH_ASKPASS` set to
  it and `SSH_ASKPASS_REQUIRE=force`. The person's script is then not
  necessary.
- **One accept takes several batches.** `boom accept` takes `--cause`
  more than one time, with targets. One command then takes each batch
  that the person picks, in one sign.
- **The page gives the person's own command.** The view "Waiting on
  you" gives each batch's command, and one command for each batch that
  Boom does not refuse. Each command has no `--key` where the settings
  name a key that the store registers. With no such key, the view gives
  the line of the settings. The person picks batches with a box on each
  batch, and a form that sends `GET` to the view gives one command for
  the batches picked. The view runs no script. The page of a proposed
  claim and the page of a narrative give their commands in the same
  way.
- `boom accept --waiting` gives the one command for every batch, as
  `command`, and in words.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `persons-acts`: a person's key in Boom's settings, an accept or a
  reject with no `--key`, and one accept of several batches.
- `narrative-page`: the person's own command on the view "Waiting on
  you", and the batches that the person picks.

## Impact

- `crates/boom`: a new `settings.rs` reads the settings and says how
  a command names the key. `sign.rs` gives a key its passphrase
  program. `cli.rs` takes the key of the settings, and `--cause` more
  than one time. `batch.rs` gives one command for several batches.
  `page.rs`, `page/waiting.rs` and `page/claims.rs` give the commands
  and the form. `mcp.rs` refuses each key as before. The tests are in
  `crates/boom/tests/`.
- A new dependency, `toml` 1.2.3, the reader of TOML that flume uses for
  its own settings.
- `README.md` and the site's `commands.md` say how a person names the
  key.
- No new entry type, no change to the log, and no new moon project.
