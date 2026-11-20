## 1. The workflow

- [x] 1.1 Add `concurrency: {group: release, cancel-in-progress: false}` at the top level of `.github/workflows/release.yml`, with a comment that says why
- [x] 1.2 Remove `concurrency: release` from the release job

## 2. The check

- [x] 2.1 `check-release.sh` fails a workflow whose lines before `jobs:` do not hold `concurrency:`, `group: release` and `cancel-in-progress: false`
- [x] 2.2 `check-release.sh` fails each job that has a `concurrency` line
- [x] 2.3 Run `root:release` on the changed workflow. Run it also on three copies: one with no block, one that cancels a run in progress, and one whose release job keeps its group. Each copy fails with its own line

## 3. The gate

- [x] 3.1 Run `root:format`, `root:release`, `root:specs` and `root:changes` through `sluice exec main`
