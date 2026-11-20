## MODIFIED Requirements

### Requirement: A landing with a new version is released

Each push to main that is a sluice landing SHALL be released when git-cliff finds a version that no tag names. The landing has `Sluice-Tested` on its head. git-cliff reads the Conventional Commits after the newest `v*` tag by version. The first version is `v0.1.0`. Each run SHALL reckon its tag only after the run before it ends. A push that no landing made, or a landing with no new version, SHALL release nothing. A later landing SHALL release its commits.

#### Scenario: Only a landing

- **WHEN** `root:release` checks the release workflow
- **THEN** the version job runs only where it reads the `Sluice-Tested` trailer of the head. It reckons the tag with `git-cliff --unreleased --bumped-version`, and it hands on no tag that exists

#### Scenario: One run at a time

- **WHEN** `root:release` checks the release workflow
- **THEN** the workflow has the concurrency group `release` at its top level, with `cancel-in-progress: false`, and no job has a concurrency group of its own

#### Scenario: Two landings close together

- **WHEN** sluice lands two changes on main less than five minutes apart, and each has a new version
- **THEN** the run of the second landing reckons its tag after the run of the first landing pushes its tag. The two runs push two different tags
