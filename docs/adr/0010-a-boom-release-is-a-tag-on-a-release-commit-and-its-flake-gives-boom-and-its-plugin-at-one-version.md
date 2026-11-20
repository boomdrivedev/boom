---
number: 10
title: A boom release is a tag on a release commit that writes its version, and boom's flake gives boom and its Claude Code plugin at one version
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-is-released-by-version's construct stage
---

# A boom release is a tag on a release commit that writes its version, and boom's flake gives boom and its Claude Code plugin at one version

Status: accepted. These are the choices of the construct stage. They are
cheap to change. No host holds boom yet, and nothing pins a release. The
first release is the landing of the bolt that carries this change.

Deciders: the construct stage of the change
`boom-is-released-by-version`.

Sources:

- Record 0009. It lets a repository move into boom only when two things
  hold. boom is on the machine at a release that its callers pin. Each
  agent session there starts with the claims that stand for it.
- boomdrivedev/blueprints principle 14: a build of a release says its
  version, given nothing.
- boomdrivedev/blueprints principle 17: the plugins of a tool come from
  its flake at one revision with its command line.
- boomdrivedev/boomdrive record 0023 (the flake of Boomdrive takes the
  package of each tool from the flake of that tool at a release tag).
- The record 0111 of sluice as amended on 2026-11-18, the release of
  flume and the `code.10` of crew. This record copies their form.
- The records 0031 and 0035 of timber, and its record 0011, its
  statement 24 and C.11.
- `interview.md`, the answers of the user of 2026-11-01:
  `plugins-in-this-repository`, `callers-pin-boom` and
  `claude-code-plugin-three-moments`.
- Record 0008.
- core.3 to core.7.

## Context and problem statement

The tree of boom names `0.0.0` in the workspace of `Cargo.toml`. Nothing
tags a commit. No flake or archive gives boom. Thus a machine builds the
commit that it checks out, whatever it is. An agent session gets its
claims only where the settings of a repository name `boom query` as a
hook, written by hand.

Record 0009 waits on two things:

- boom installed at a release that its callers pin.
- Each session that starts with the claims of its repository.

The family already releases its tools in one way. boom copies that form
and does not invent one.

## Considered options

For what a release is:

1. A tag on a release commit off main. The commit writes the version of
   the release into each file that carries it (chosen).
2. A tag on the landing commit, with the version given to each build.
3. A release commit pushed to main.
4. A binary that says only its revision.

For how a session gets its claims:

5. A Claude Code plugin whose session hook runs
   `boom query --if-declared` (chosen).
6. A script beside the hook that finds the declaration itself.
7. The hook written in the settings of each repository, as the README
   had it.

## Decision outcome

1. **A release commit off main, tagged.** The tag `v<x.y.z>` of a
   release names a commit with one parent: the landing that it releases.
   The only change of that commit writes the version of the release into
   the files that point 2 lists. The tree of main names `0.0.0`. A build
   takes its version from its tree alone: no environment variable, no
   `--impure`, no argument. `boom --version` stays the version flag of
   clap.

   Not taken:
   - The tag on the landing commit, with the version given to the build
     (option 2). Each builder except the workflow of boom would have to
     know the version (principle 14).
   - The release commit pushed to main (option 3). Only sluice lands on
     main, and its gate never saw that commit.
   - A binary that says only its revision (option 4). That is the design
     of timber for a binary that timber publishes (timber record 0011).

   When timber publishes boom, the MVP of timber retires the release
   commit, and this point goes with it.
2. **Three files carry the version, and one writer writes them.** The
   three files are these:
   - The `version` of `[workspace.package]` in `Cargo.toml`.
   - The entry for `boom` in `Cargo.lock`, the package with no `source`.
   - The manifest of the plugin.

   The marketplace names no version, so the version of the plugin is in
   one place. `set-version.sh`, the writer of sluice, writes each file
   beside itself. It moves them over their originals only after it
   writes all of them. Thus a refusal leaves the tree as it was.
   `--files` lists the three. `root:versions` holds each file to `0.0.0`,
   and checks on a scratch copy that the writer does what it says.

   Not taken:
   - A `VERSION` file. That puts one fact in two places.
   - A version in the marketplace. That is a second home for the version
     of the plugin.
3. **A landing with a new version is released.** On each push to main,
   the workflow reads the `Sluice-Tested` trailer of the head. Only
   where the trailer is present does git-cliff calculate the next
   version from the Conventional Commits since the newest `v*` tag.
   `v0.1.0` is the first.

   The release job does these steps, under `concurrency: release` and
   with the token of the run (core.3):
   - It writes the notes of git-cliff.
   - It runs the writer.
   - It commits the files that the writer lists, by path, as
     `chore(release): <tag>`.
   - It tags that commit.
   - It pushes the tag alone.
   - It publishes the GitHub release, verified against the tag.

   The gate of sluice is the one place where the checks of boom run.
   Thus, if no landing made a push, the next landing releases it, because
   the range of that landing holds it.

   Not taken: the CI tiers of sluice, run again on GitHub before a
   release. The gate of boom already ran them.
4. **The binaries are the binaries of cargo, and they run with no Nix.**
   For `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and
   `aarch64-apple-darwin`, the job runs
   `cargo build --release --locked -p boom` under the toolchain that
   `rust-toolchain.toml` names, after the writer writes the version of
   the release. The Linux binaries are static against musl. The job
   installs no Nix. It fails unless the built `boom --version` says the
   version of the release. Thus a binary that loads anything from a Nix
   store fails where the job built it.

   Not taken:
   - `nix build`. Its binaries load from `/nix/store` and fail on a host
     without it, as the binaries of sluice v0.97.1 did.
   - x86_64-darwin. The family releases no tool for it.
5. **`root:release` holds the workflow to its steps.** It reads the YAML
   job by job. It fails, and names each missing step, on any of these:
   - A version job that does not run from a landing, or that calculates
     the version in another way.
   - A binaries job that builds Linux against anything except musl, runs
     Nix, or skips the writer or the version test.
   - A release job that commits other than by path, pushes anything
     except the tag, or publishes without verification.
   - Any `--impure` or `BOOM_VERSION`.

   It reads no Markdown file (core.5).

   Not taken: actionlint. It reads the keys and shell of a workflow, and
   it cannot say what the steps of a release must be.
6. **The flake of boom gives boom and its plugin.** For aarch64-darwin,
   aarch64-linux and x86_64-linux, it gives two packages:
   - `packages.<system>.boom`, also `default`. The Rust that
     `.prototools` pins builds it from the tree of the flake, at the
     workspace version of `Cargo.toml`.
   - `packages.<system>.boom-claude-plugin`. This is `plugins/claude-code`
     of the tree, copied whole, at the version of the `boom` package.

   A machine installs both from one tag. The flake gives no module. boom
   writes no file on a host, so the configuration of a host installs the
   package, as the set of Boomdrive will (boomdrive record 0023).

   Not taken:
   - A `services.boom` module that only installs the package.
   - The plugin built by Boomdrive from the source of boom. Principle 17
     gives that to the flake of boom.
7. **The `nix` project checks the flake offline.** statix lints it. The
   nix that `devenv.nix` pins evaluates the two packages of each system
   and the cases of `nix/tests`, read-only, against a dummy store. It
   reads the inputs of the flake where the devenv shell fetched them, as
   timber does (record 0035).

   Not taken: `nix flake check`. It needs the network and the daemon,
   which the sandbox of the gate denies.
8. **The session hook of the plugin runs `boom query --if-declared`.**
   `plugins/claude-code` holds the manifest, at the version of the tree,
   and one `SessionStart` hook. Claude Code runs the hook in the folder
   of the session, and puts its standard output in the context of the
   agent. The hook calls `boom` from `PATH`, the boom of the same release
   beside it (`plugins-in-this-repository`). A marketplace at the root of
   the tree lists the plugin by its folder. Thus a person without Nix
   adds the plugin from the tag of the release.

   Not taken:
   - A script that finds the declaration itself (option 6). It repeats
     the discovery of boom in shell.
   - `|| true` on the command. It hides a declaration that boom cannot
     read.
   - The hook in the settings of each repository as the only way
     (option 7). Each repository writes it by hand.
9. **The plugin is a moon project of its own.** `claude-code` does these
   things:
   - It formats and lints its JSON with Biome.
   - It validates the manifest and its hooks with the pinned Claude Code
     through `plugins/claude.sh`, the script of timber. The script gives
     Claude Code a scratch home, and none of the configuration or
     credentials of the user (core.3).
   - It tests the hook as Claude Code runs it, with a `boom` of its own
     and with the boom that the tree builds.

   `devenv.nix` pins Claude Code, jq, nix and statix, and
   `root:toolchain` asserts them (core.6). The validation of the
   marketplace by Claude Code reads the README for advice that changes
   no verdict. That is the reading of the tool itself, as the reading of
   the pages of the site by Astro is. core.5 does not rule it out.

   Not taken: proof in shell of the claims that a declared repository
   gets. Shell cannot make the keys of the tests.
   `crates/boom/tests/query.rs` proves them (record 0006).
10. **`boom query --if-declared` answers nothing where nothing is
    declared.** Where the working folder declares a repository, it
    answers as `boom query` does, byte for byte. Where the working
    folder declares none, it does these things:
    - It opens no store.
    - It writes nothing to standard error.
    - It exits 0.
    - With `--json`, it gives `boom-query/1`, with `repository` and
      `head` null and `claims` empty.

    It refuses a declaration that it cannot read, as before. The flag
    conflicts with `--repository`. Only a caller that gives the new flag
    can get the nulls. Thus no caller built for the format before the
    flag reads one.

    Not taken:
    - `boom query` that answers nothing each time nothing is declared.
      That silences the refusal that record 0008 keeps for a person.
    - A refusal with a code that a hook parses. Each hook then parses
      it.

## Consequences

- Good: a build of the tag of a release, by cargo or by the flake of
  boom, says the version of the release, given nothing. The plugin
  installed from the same tag is at that version by construction.
- Good: a plugin installed for each session on a machine interrupts no
  session outside a declared repository. It starts each session in a
  declared repository with the claims that stand for it.
- Bad: when `gh release create` fails after the tag is pushed, a person
  publishes the release by hand from the artifacts of the run, as for
  sluice, flume and crew.
- Bad: `plugins/claude.sh`, `set-version.sh`, the release checks and the
  offline evaluation are copies of the files of timber and sluice. A fix
  to one is no fix to the others. The opening comment of each file names
  the revision that it was taken from.
- Neutral: when timber publishes boom (timber record 0011), it retires
  the release commit of point 1.
