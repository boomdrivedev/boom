## ADDED Requirements

### Requirement: The plugin teaches an agent to use Boom

The plugin SHALL hold three skills, each a folder of `skills/` with its `SKILL.md`: `using-boom`, `proposing-claims` and `writing-narratives`. `proposing-claims` SHALL also hold `kinds.md`. The pinned Claude Code's strict validation of the plugin SHALL pass with the skills. No task SHALL read the words of a skill.

#### Scenario: The skills validated

- **WHEN** the pinned Claude Code validates the plugin's folder strictly
- **THEN** it passes, with the folders `skills/using-boom`, `skills/proposing-claims` and `skills/writing-narratives` in the plugin

#### Scenario: Each skill at its path

- **WHEN** the plugin's test lists the files of `skills/`
- **THEN** it finds `using-boom/SKILL.md`, `proposing-claims/SKILL.md`, `proposing-claims/kinds.md` and `writing-narratives/SKILL.md`, and it reads none of them
