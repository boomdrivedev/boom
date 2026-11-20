## Why

On 2026-11-20, sluice landed #3 and #4 on main less than five minutes
apart. GitHub started a release run for each landing. Each version job
read the newest tag, `v0.1.0`, before either run pushed a tag. So both
runs reckoned `v0.2.0`. The run of #3 pushed `v0.2.0`. The release job
of #4 then failed with `tag 'v0.2.0' already exists`. A person ran the
run of #4 again by hand, and it released `v0.3.0`.

Only the release job takes turns now (`concurrency: release`). The
version job does not take turns, so two runs can reckon the same tag.

## What Changes

- The whole release workflow takes turns in one concurrency group,
  `release`, and does not cancel a run in progress. A run starts its
  version job only after the run before it ends. The version job then
  reads the tag that the run before it pushed.
- The release job no longer has a concurrency group of its own. A job
  in the same group as its workflow would wait for its own run.
- `check-release.sh` (`root:release`) fails a workflow without that
  group, a workflow that cancels a run in progress, and a job with a
  concurrency group of its own.
- When a third landing comes while one run waits, GitHub cancels the
  waiting run. The next run releases the commits of the cancelled run,
  because its range holds them. This is the behavior for a push that no
  landing made, and the spec says it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-version`: a landing with a new version is released one run
  at a time, after the release of each landing before it.

## Impact

- `.github/workflows/release.yml`: one `concurrency` block at the top,
  and none on the release job.
- `check-release.sh`: three new checks.
- No change to boom, its plugin or its flake. Record 0010 and core.8
  stay as they are: this change keeps their rules and closes a race in
  how the workflow follows them.
