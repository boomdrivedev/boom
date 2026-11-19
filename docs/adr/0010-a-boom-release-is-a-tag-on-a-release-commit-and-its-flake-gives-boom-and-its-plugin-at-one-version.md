---
number: 10
title: A boom release is a tag on a release commit that writes its version, and boom's flake gives boom and its Claude Code plugin at one version
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-is-released-by-version's construct stage
---

# A boom release is a tag on a release commit that writes its version, and boom's flake gives boom and its Claude Code plugin at one version

Status: accepted. The construct stage's own choices, cheap to change: no
host holds boom yet, nothing pins a release, and the first release is
the landing of the bolt that carries this change.

Deciders: the construct stage of the change
`boom-is-released-by-version`.

Sources: record 0009, which lets a repository move into boom only once
boom is on the machine at a release its callers pin and every agent
session there starts with the claims that stand for it;
boomdrivedev/blueprints principles 14 (a build of a release says its
version, given nothing) and 17 (a tool's plugins come from its flake at
one revision with its command line); boomdrivedev/boomdrive record 0023
(Boomdrive's flake takes each tool's package from the tool's flake at a
release tag); sluice's record 0111 as amended on 2026-11-18, flume's
release and crew's `code.10`, the form this copies; timber's records
0031 and 0035 and its record 0011, its statement 24 and C.11;
`interview.md`, the user's answers of 2026-11-01:
`plugins-in-this-repository`, `callers-pin-boom` and
`claude-code-plugin-three-moments`; record 0008; core.3 to core.7.

## Context and problem statement

boom's tree names `0.0.0` in `Cargo.toml`'s workspace, nothing tags a
commit, and no flake or archive gives boom, so a machine builds whatever
commit it checks out, and an agent's session gets its claims only where
a repository's own settings name `boom query` as a hook, written by
hand. Record 0009 waits on both: boom installed at a release its
callers pin, and every session starting with its repository's claims.
The family already releases its tools one way, and boom copies that
form rather than inventing one.

## Considered options

For what a release is:

1. A tag on a release commit off main that writes the release's version
   into every file that carries it (chosen).
2. A tag on the landing commit, the version handed to each build.
3. A release commit pushed to main.
4. A binary that says only its revision.

For how a session gets its claims:

5. A Claude Code plugin whose session hook runs
   `boom query --if-declared` (chosen).
6. A script beside the hook that finds the declaration itself.
7. The hook written in each repository's own settings, as the README
   had it.

## Decision outcome

1. **A release commit off main, tagged.** A release's tag `v<x.y.z>`
   names a commit whose one parent is the landing it releases and whose
   only change writes the release's version into the files point 2
   lists. main's tree names `0.0.0`, and a build takes its version from
   its tree alone: no environment variable, no `--impure`, no argument.
   `boom --version` stays clap's. Not taken: the tag on the landing
   commit with the version handed to the build (option 2), which every
   builder but boom's workflow would have to know (principle 14); the
   release commit pushed to main (option 3), which only sluice lands and
   whose gate never saw it; a binary that says only its revision (option
   4), timber's design for a binary timber publishes (timber record
   0011). When timber publishes boom, timber's MVP retires the release
   commit, and this point goes with it.
2. **Three files carry the version, and one writer writes them.**
   `Cargo.toml`'s `[workspace.package]` `version`, `Cargo.lock`'s entry
   for `boom`, the package with no `source`, and the plugin's manifest.
   The marketplace names no version, so the plugin's is said once.
   `set-version.sh`, sluice's, writes each file beside itself and moves
   them over their originals only once all are written, so a refusal
   leaves the tree as it was; `--files` lists the three. `root:versions`
   holds each to `0.0.0` and the writer to its word on a scratch copy.
   Not taken: a `VERSION` file, one fact in two places; a version in the
   marketplace, a second home for the plugin's.
3. **A landing with a new version is released.** On each push to main,
   the workflow reads the head's `Sluice-Tested` trailer; only where it
   is present does git-cliff reckon the next version from the
   Conventional Commits since the newest `v*` tag, `v0.1.0` the first.
   The release job writes git-cliff's notes, runs the writer, commits
   the files it lists by path as `chore(release): <tag>`, tags that
   commit, pushes the tag alone and publishes the GitHub release
   verified against it, under `concurrency: release` and with the run's
   own token (core.3). sluice's gate is the one place boom's checks run,
   so a push no landing made is released with the next landing, whose
   range holds it. Not taken: sluice's CI tiers run again on GitHub
   before a release, which boom's gate already ran.
4. **The binaries are cargo's and run with no Nix.** For
   `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and
   `aarch64-apple-darwin`, `cargo build --release --locked -p boom`
   under the toolchain `rust-toolchain.toml` names, after the writer
   writes the release's version; Linux's static against musl. The job
   installs no Nix and fails unless the built `boom --version` says the
   release's version, so a binary that loads anything from a Nix store
   fails where it was built. Not taken: `nix build`, whose binaries load
   from `/nix/store` and fail on a host without it, as sluice v0.97.1's
   did; x86_64-darwin, which the family releases for no tool.
5. **`root:release` holds the workflow to its steps.** It reads the
   YAML job by job and fails, naming each step missing, on a version
   job that runs off a landing or reckons otherwise, a binaries job that
   builds Linux against anything but musl, runs Nix or skips the writer
   or the version test, a release job that commits other than by path,
   pushes anything but the tag or publishes unverified, and any
   `--impure` or `BOOM_VERSION`. It reads no Markdown file (core.5).
   Not taken: actionlint, which reads a workflow's keys and shell and
   cannot say what a release's steps must be.
6. **boom's flake gives boom and its plugin.** For aarch64-darwin,
   aarch64-linux and x86_64-linux, `packages.<system>.boom`, also
   `default`, built by the Rust `.prototools` pins from the flake's own
   tree at `Cargo.toml`'s workspace version, and
   `packages.<system>.boom-claude-plugin`, the tree's
   `plugins/claude-code` copied whole at the `boom` package's version.
   A machine installs both from one tag. The flake gives no module:
   boom writes no file on a host, so a host's configuration installs
   the package, as Boomdrive's set will (boomdrive record 0023). Not
   taken: a `services.boom` module that installs the package alone; the
   plugin built by Boomdrive from boom's source, which principle 17
   gives to boom's own flake.
7. **The `nix` project checks the flake offline.** statix lints it, and
   the nix `devenv.nix` pins evaluates every system's two packages and
   the cases of `nix/tests` read-only against a dummy store, reading the
   flake's inputs where the devenv shell fetched them, as timber's
   (record 0035). Not taken: `nix flake check`, which needs the network
   and the daemon the gate's sandbox denies.
8. **The plugin's session hook runs `boom query --if-declared`.**
   `plugins/claude-code` holds the manifest, at the tree's version, and
   one `SessionStart` hook, which Claude Code runs in the session's
   folder, putting its standard output in the agent's context. It calls
   `boom` from `PATH`, the boom of the same release beside it
   (`plugins-in-this-repository`). A marketplace at the tree's root
   lists the plugin by its folder, so a person without Nix adds it from
   the release's tag. Not taken: a script that finds the declaration
   itself (option 6), which would repeat boom's discovery in shell;
   `|| true` on the command, which would hide a declaration boom cannot
   read; the hook in each repository's own settings as the only way
   (option 7), written by hand in every repository.
9. **The plugin is a moon project of its own.** `claude-code` formats
   and lints its JSON with Biome, validates the manifest and its hooks
   with the pinned Claude Code through `plugins/claude.sh`, timber's,
   which gives it a scratch home and none of the user's configuration
   or credentials (core.3), and tests the hook as Claude Code runs it,
   with a `boom` of its own and with the boom the tree builds. Claude
   Code, jq, nix and statix are pinned in `devenv.nix` and asserted by
   `root:toolchain` (core.6). Claude Code's validation of the
   marketplace reads the README for advice that changes no verdict: the
   tool's own reading, as Astro's of the site's pages is, which core.5
   does not rule out. Not taken: the claims a declared
   repository gets proven in shell, which cannot make the tests' keys;
   `crates/boom/tests/query.rs` proves them (record 0006).
10. **`boom query --if-declared` answers nothing where nothing is
    declared.** Where the working folder declares a repository it
    answers as `boom query` does, byte for byte; where none is declared
    it opens no store, writes nothing to standard error and exits 0,
    with `--json` giving `boom-query/1` with `repository` and `head`
    null and `claims` empty. A declaration it cannot read is refused as
    before, and the flag conflicts with `--repository`. Only a caller
    that passes the new flag can be given the nulls, so no caller built
    for the format before it reads one. Not taken: `boom query`
    answering nothing whenever nothing is declared, which would silence
    the refusal record 0008 keeps for a person; a refusal with a code a
    hook parses, which every hook would parse.

## Consequences

- Good: a build of a release's tag, by cargo or by boom's flake, says
  the release's version, given nothing, and the plugin installed from
  the same tag is at that version by construction.
- Good: a plugin installed for every session on a machine interrupts
  none outside a declared repository, and starts each session in one
  with the claims that stand for it.
- Bad: a release whose `gh release create` fails after the tag is
  pushed is published by hand from the run's artifacts, as for sluice,
  flume and crew.
- Bad: `plugins/claude.sh`, `set-version.sh`, the release checks and the
  offline evaluation are copies of timber's and sluice's, and a fix to
  one is no fix to the others; each file's opening comment names the
  revision it was taken from.
- Neutral: timber publishing boom (timber record 0011) retires the
  release commit of point 1.
