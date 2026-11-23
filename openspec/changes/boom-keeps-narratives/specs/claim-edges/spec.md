# Spec Delta

## MODIFIED Requirements

### Requirement: A version stores its edges

The entry of a version SHALL store its edges. `amends` is its base. `source` is the claims that it rests on. `changes` is the claims that a decision changes. `origin` is the files and the commits that the version came from. `includes` is the claims that a narrative includes. No other record SHALL store an edge of a version. Boom SHALL refuse `changes` on a version that is not a decision, and `includes` on a version that is not a narrative. Boom SHALL keep each edge one time, in the order given.

#### Scenario: A decision with its edges

- **WHEN** the store holds `flume-has-a-resident` and `one-github-client-per-host`, and `boom propose decision the-runner-serves-a-socket "<statement>" --body <file> --source flume-has-a-resident --changes one-github-client-per-host --origin mockup.svg --provenance run-1` runs
- **THEN** boom exits 0, and the entry of the proposal holds the edge `source`, the edge `changes` and the edge `origin`

#### Scenario: Changes on an invariant

- **WHEN** an agent proposes an invariant with `--changes one-github-client-per-host`
- **THEN** boom exits 2 and says that only a decision changes claims, and boom appends nothing

#### Scenario: One source given two times

- **WHEN** an agent proposes a claim with `--source flume-has-a-resident` two times
- **THEN** the entry of the proposal holds one edge `source` to `flume-has-a-resident`

#### Scenario: A narrative with its includes

- **WHEN** the store holds `status-read-from-events`, and `boom propose narrative boom-end-to-end "Boom, end to end" --body narrative.md --includes status-read-from-events --provenance run-1` runs
- **THEN** boom exits 0, and the entry of the proposal holds the edge `includes`
