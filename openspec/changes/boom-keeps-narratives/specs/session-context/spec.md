# Spec Delta

## ADDED Requirements

### Requirement: Session start gives the narratives by name

After the contexts and before the menu, the text SHALL give the narratives that stand for the repository as a count and a line `narrative <name>` for each. The line of a narrative with open notes SHALL end with `(<count> open notes)`, and SHALL count the open notes of each version of the narrative. The menu SHALL also name `boom page <name>`. The text SHALL NOT give a line of the body of a narrative.

#### Scenario: A narrative with open notes

- **WHEN** the narrative `boom-end-to-end` is accepted for `boom`, a proposed version of it has two open notes, and a session starts in a worktree of `boom`
- **THEN** the text gives the line `narrative boom-end-to-end (2 open notes)` under the count of narratives, and no line of its body

#### Scenario: The menu names the page

- **WHEN** a session starts in a worktree of `boom`
- **THEN** the menu names `boom page <name>`
