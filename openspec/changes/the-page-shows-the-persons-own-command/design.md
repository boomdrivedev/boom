## Context

A person's act that binds later work is an SSH signature by a security
key, made by the person's own `ssh-keygen`
(`a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key`). Each
command that signs takes `--key`, the path of the private key file. The
person's key has a passphrase, which a program reads from 1Password.
`ssh-keygen` runs that program when `SSH_ASKPASS` names it and
`SSH_ASKPASS_REQUIRE` is `force`.

The view "Waiting on you" gives the batches that wait on a person, each
with its one command (`boom-accepts-a-batch`). The views run no script
(`boom-page-has-a-claims-view`, "The views run no script").

Flume keeps the settings of a host in `$XDG_CONFIG_HOME/flume/config.toml`,
under `$HOME/.config` where the variable is unset. It reads a path in
it as absolute or under `~/`, with no `$VAR`. The family's statement
says that a host says only its own, through a tool's own settings
(`what-boomdrive-delivers`).

## Goals / Non-Goals

**Goals:** the person runs the command that the page gives, as it is,
with no script of their own. One command takes the batches that the
person picks.

**Non-Goals:** a key that signs with no touch, a passphrase that Boom
reads or keeps, and settings in the log.

## Decisions

1. **The settings are Boom's own file on the machine, as flume's are.**
   `$XDG_CONFIG_HOME/boom/config.toml`, under `$HOME/.config` where the
   variable is unset. Its one table is `key`, with `path` and an
   optional `askpass`. A path is absolute or starts with `~/`. Boom reads
   no `$VAR`. Boom refuses a file that it cannot read, a key that it
   does not read, and a path of another form, and names the setting. No
   command writes the file. Not taken: the key in the log, signed by an
   owner. A path and a program are of one machine, and the log is the
   same on each machine.
2. **The file names a path, never a key or a passphrase.** The settings
   hold the path of the private key file and the path of a program.
   Boom never reads the passphrase: `ssh-keygen` runs the program
   (core.3).
3. **Each act that needs a person's key takes the key of the settings.**
   `boom accept`, `reject`, `approve`, `register` and `repo add` and
   `set` sign with it where no `--key` is given. `--key` names another
   key, and takes the passphrase program only where it names the key of
   the settings. `boom propose` and `boom accept --waiting` take only
   `--key`: with no key, an agent proposes, and `--waiting` lists.
   Thus no agent's command signs with the key of the settings.
4. **The key signs as before.** Boom runs `ssh-keygen -Y sign` with
   the key, and with `SSH_ASKPASS` and `SSH_ASKPASS_REQUIRE=force` where
   the settings name a program. The key asks for its touch. The
   signature is as before, and the judge reads it as before.
5. **A command names the key as the settings say.** Boom gives a
   command with no `--key` where the settings name a key whose public
   half, `<path>.pub`, the store registers to a person. Otherwise it
   gives `--key <key>`. The page reads the settings for each request.
6. **Several batches in one command.** A batch's arguments are its head
   for a narrative and a ruling, and `--cause` with its head for a
   revision. One command for several batches gives the heads in the
   order of the batches, then each `--cause`. `boom accept` takes
   `--cause` more than one time. One entry takes each version, and the
   key signs one time.
7. **The person picks with a form and no script.** Each batch that Boom
   does not refuse has a box, `pick`, whose value is the batch's anchor,
   `<shape>-<name>`. A form sends `GET` to `<base>/waiting`. The view
   gives the command for the batches that the form picks, above the
   batches, and keeps each picked box checked. A value that names no
   batch that waits, such as a batch accepted since, is passed over.

## Risks / Trade-offs

- A command with no `--key` that an agent runs now reaches the
  person's key, where the settings name one. The key still asks for its
  touch, and the person sees the list of what the entry takes before
  the touch. An agent never runs `boom accept`, `reject`, `approve`,
  `register` or `repo`
  (`a-persons-act-in-boom-is-an-ssh-signature-by-a-security-key`).
- A long command for many batches. It is one line, and Boom lists each
  version before the key signs.
