# release-version Specification

## Purpose
What a boom release is and how a machine gets one: a tag naming a tree
that says its version, made from a landing on main, whose binaries run
anywhere and whose flake gives boom and its plugin at that one version
(blueprints principles 14 and 17; boomdrive record 0023; record 0010).

## Requirements

### Requirement: main's tree names no release

Every file of the tree that carries boom's version SHALL give `0.0.0`: the workspace's version in `Cargo.toml`, the lock's entry for `boom`, and the Claude Code plugin's manifest. Every plugin manifest under `plugins/` SHALL be among them, and the marketplace SHALL name no version for a plugin, so each plugin's version is said once.

#### Scenario: main's files

- **WHEN** the versions check runs on main's tree
- **THEN** it passes, each of the three files giving `0.0.0`

#### Scenario: A file that gives another version

- **WHEN** the plugin's manifest gives `0.1.0` while the workspace gives `0.0.0`
- **THEN** the versions check fails naming the manifest, what it gives and the workspace's version

#### Scenario: A version in the marketplace

- **WHEN** the marketplace names a version for the plugin `boom`
- **THEN** the versions check fails naming the marketplace

### Requirement: One writer writes a release's version, all or none

`set-version.sh <major.minor.patch>` SHALL write the version into every file that carries boom's version and change nothing else, and `set-version.sh --files` SHALL list those files. It SHALL refuse a version that is not `major.minor.patch`, and a file in which it finds no version, naming it, and on a refusal SHALL leave every file as it was.

#### Scenario: A version written

- **WHEN** `set-version.sh 98.76.54` runs on a copy of the tree
- **THEN** each file `--files` lists gives `98.76.54`, and differs from the tree's only in the lines that give the version

#### Scenario: Not a version

- **WHEN** `set-version.sh 0.98` runs on a copy of the tree
- **THEN** it exits non-zero saying `0.98` is not a version, and no file is changed and no scratch file is left

#### Scenario: A file with no version

- **WHEN** `set-version.sh 98.76.54` runs on a copy whose plugin manifest holds no `version`
- **THEN** it exits non-zero naming the manifest, and no file is changed and no scratch file is left

### Requirement: A landing with a new version is released

Each push to main that is a sluice landing, its head carrying `Sluice-Tested`, SHALL be released when git-cliff, from the Conventional Commits since the newest `v*` tag by version, finds a version no tag names; the first is `v0.1.0`. A push no landing made, or a landing in which git-cliff finds no new version, SHALL release nothing.

#### Scenario: Only a landing

- **WHEN** the release workflow is checked
- **THEN** its version job runs only where the head's `Sluice-Tested` trailer is read, and reckons the tag with `git-cliff --unreleased --bumped-version`, handing on none that already exists

### Requirement: A release's tag names a release commit off main

A release SHALL be the tag `v<x.y.z>` on a commit whose one parent is the landing released and whose only change is the files the writer lists, written with the release's version and committed by path as `chore(release): v<x.y.z>`. The tag SHALL be pushed alone and the GitHub release published verified against it; nothing SHALL be committed to main or pushed to a branch.

#### Scenario: The release job

- **WHEN** the release workflow is checked
- **THEN** its release job runs the writer with the release's version, commits the files `--files` lists by path as `chore(release): <tag>`, tags that commit, pushes `refs/tags/<tag>` alone and publishes with `gh release create --verify-tag`, and the workflow pushes no branch

### Requirement: A build takes its version from its tree alone

boom's version SHALL come from the tree it is built from and nothing else: no environment variable, argument or impure evaluation names it. A build of main says `0.0.0`, and a build of a release's tag says the release's version, by cargo and by boom's flake alike.

#### Scenario: The flake's package

- **WHEN** the flake's `boom` package is evaluated for each system it gives
- **THEN** its version is `Cargo.toml`'s workspace version

#### Scenario: Nothing handed to the build

- **WHEN** the release workflow is checked
- **THEN** it holds no `--impure` and no `BOOM_VERSION`

### Requirement: A release's binaries run with no Nix

A release's binaries SHALL be built by cargo, `--locked`, with the Rust the tree pins, from the landing with the release's version written: x86_64 and aarch64 Linux static against musl, and aarch64 macOS. Each SHALL be run where it was built, on a runner holding no Nix, and SHALL say `boom <x.y.z>` before the tag is pushed.

#### Scenario: The binaries job

- **WHEN** the release workflow is checked
- **THEN** its binaries job names only `-linux-musl` Linux targets, runs the writer before `cargo build --release --locked -p boom --target`, runs the built binary's `--version` against the tag, and installs no Nix and runs no `nix` command

### Requirement: The flake gives a release's boom and plugin at one version

boom's flake SHALL give, for each of aarch64-darwin, aarch64-linux and x86_64-linux, the package `boom`, also its default, and `boom-claude-plugin`, the plugin's folder of the same tree, at the `boom` package's version, so a machine installs boom and its plugin from one tag. It SHALL give no other system.

#### Scenario: Every system evaluates

- **WHEN** the flake is evaluated with no store
- **THEN** both packages of each of the three systems evaluate to a derivation, and no other system is given

#### Scenario: The plugin at boom's version

- **WHEN** the flake's `boom-claude-plugin` is evaluated for each system
- **THEN** its version is that system's `boom` package's, and its source is the tree's `plugins/claude-code`
