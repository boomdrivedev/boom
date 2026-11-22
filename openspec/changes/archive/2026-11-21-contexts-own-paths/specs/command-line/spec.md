## MODIFIED Requirements

### Requirement: Exit codes

Each command SHALL exit 0 when it did what the caller asked. It SHALL exit 2 when it refuses. Examples are the accept of an agent, a name that a claim has, an amendment whose base is no longer accepted, an unknown name, and arguments that boom cannot read. It SHALL exit 3 when it cannot run. Examples are no store, an altered store, a blob that boom cannot read, and a signature that the program of the key did not make. On exit 2 and exit 3, boom SHALL write nothing.

#### Scenario: A refusal writes nothing

- **WHEN** boom refuses the accept of an agent
- **THEN** boom exits 2, and the log of the store does not change

#### Scenario: Could not run

- **WHEN** `boom accept` runs with a key whose signing program cannot run or exits with a code other than 0
- **THEN** boom exits 3 and says that the program of the key did not sign, and the log of the store does not change

#### Scenario: Arguments it cannot read

- **WHEN** `boom propose` runs with a kind that is not `invariant`, `decision`, `assumption`, `term` or `context`
- **THEN** boom exits 2 and names the kinds that it takes
