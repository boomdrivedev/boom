## 1. The claims

- [x] 1.1 Propose in Boom the claims that this change makes true, each amending what it changes, each passing `ste-lint.py` with 0 hard findings

## 2. The settings and the key

- [x] 2.1 Add `settings.rs`: the file, the table `key`, its paths and its refusals, and how a command names the key. `boom:test` proves the scenarios of "A person's key in Boom's settings"
- [x] 2.2 In `sign.rs`, give a key its passphrase program, and run `ssh-keygen` with `SSH_ASKPASS` and `SSH_ASKPASS_REQUIRE=force` where it has one. `boom:test` proves "The passphrase program reaches ssh-keygen" with a key and a program made for the test
- [x] 2.3 In `cli.rs`, sign each act of a person with the key of the settings where no `--key` is given. `boom:test` proves the scenarios of "A person's act signs with the key of the settings" and "Only a person accepts or rejects"

## 3. Several batches and the page

- [x] 3.1 In `cli.rs` and `batch.rs`, take `--cause` more than one time, and give one command for several batches in `boom accept --waiting`. `boom:test` proves "Two causes in one sign"
- [x] 3.2 In `page.rs`, `page/waiting.rs` and `page/claims.rs`, give the person's own command, the line of the settings, the boxes and the form. `boom:test` proves the scenarios of "The view \"Waiting on you\" gives the person's own command" and "The person picks batches into one command"

## 4. The pages and the gate

- [x] 4.1 Say in `README.md` and the site's `commands.md`, in Plain, how the settings name a person's key
- [x] 4.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `site:format`, `site:lint`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
