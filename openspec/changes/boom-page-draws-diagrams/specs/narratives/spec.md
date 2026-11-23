## ADDED Requirements

### Requirement: A diagram in a body is its source

A body SHALL give a diagram as a fenced code block whose language is `mermaid`. Boom SHALL count the block as one paragraph of kind `code`, and SHALL keep its source as the bytes of the body. `boom show` SHALL give the source.

#### Scenario: A diagram is a paragraph

- **WHEN** a narrative's body is a heading and a fenced `mermaid` code block that holds `flowchart LR` and `a --> b`, and `boom show <name>@<version> --json` runs
- **THEN** `paragraphs` gives two paragraphs, of the kinds `heading` and `code`, and the fingerprint of paragraph 2 is that of the bytes of the block in the body
