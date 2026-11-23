## MODIFIED Requirements

### Requirement: A version stores its edges

A version's entry SHALL store its edges: `amends`, its base; `source`, the claims it rests on; `changes`, the claims a decision changes; `origin`, the files and commits it came from; `includes`, the claims a narrative includes; `narratives`, the narratives an outline holds. No other record SHALL store an edge of a version. Boom SHALL refuse `changes` but on a decision, `includes` but on a narrative, and `narratives` but on an outline. Boom SHALL keep each edge one time, in the order given.

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

#### Scenario: An outline with its narratives

- **WHEN** the store holds the narratives `boom-end-to-end` and `change-records`, and `boom propose outline boomdrive-design-book "The design book of Boomdrive" --view book --narrative change-records --narrative boom-end-to-end --provenance run-1` runs
- **THEN** boom exits 0, and the entry of the proposal holds the edge `narratives` to `change-records`, then `boom-end-to-end`
