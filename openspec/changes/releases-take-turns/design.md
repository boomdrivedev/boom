## Context

The release workflow has four jobs: `landing`, `version`, `binaries` and
`release` (record 0010, design 1 and 3). The `version` job reckons the
next tag with `git-cliff --unreleased --bumped-version`, from the newest
`v*` tag. Only the `release` job has a concurrency group. Two runs that
start close together can each reckon the tag before either run pushes
it.

## Goals / Non-Goals

**Goals:**

- Each run reckons its tag after the run before it pushed its tag.
- `root:release` fails a workflow that loses this.

**Non-Goals:**

- A queue that keeps each waiting run. GitHub keeps one waiting run for
  each group. The next landing releases the commits of a cancelled run.

## Decisions

### 1. One concurrency group for the whole workflow

The workflow gets `concurrency: {group: release, cancel-in-progress:
false}` at its top level. GitHub then starts no job of a run while
another run of the group is in progress. The version job of the second
run reads the tags after the first run ends. Its checkout reads the
tags of the remote at that time.

Alternative: put the version job and the release job in one job-level
group. That does not work. The group holds one job at a time. The version job
of a second run can then run between the version job and the release
job of the first run.

Alternative: the release job reckons the tag again before it pushes.
That moves the version out of the job that the binaries build from, and
the binaries then can carry a version other than the tag.

### 2. No job-level group

A job whose group has the name of the group of its own workflow waits
for its own run, and the run never ends. The release job loses its
`concurrency: release`.

### 3. The check reads the top-level block

`check-release.sh` reads the lines of the workflow before `jobs:`. It
fails unless they hold `concurrency:` with `group: release` and
`cancel-in-progress: false`. It fails each job that has a `concurrency`
line. It reads YAML, never Markdown (core.5).

## Risks / Trade-offs

- [Three landings close together] → GitHub cancels the run that waits.
  The run after it releases the commits of both landings under one tag.
  No commit stays unreleased.
- [A run that fails holds nothing] → A failed run ends, and the next run
  starts. The failed landing waits for a person, as before.
