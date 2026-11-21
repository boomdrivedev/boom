# Spec Delta

## Purpose

The first edges of the graph of Boom. A version amends its base, rests
on its sources, changes claims, and comes from its origin. The
entry of the version stores each edge with the version that it meant.
Boom builds the graph from the edges when asked, and keeps no graph as
the truth (`a-link-remembers-the-version-it-meant`,
`a-claim-links-to-its-origin`, `claims-cite-claims-by-name`,
`names-use-the-domains-terms`).

## ADDED Requirements

### Requirement: A version stores its edges

The entry of a version SHALL store its edges. `amends` is its base. `source` is the claims that it rests on. `changes` is the claims that a decision changes. `origin` is the files and the commits that the version came from. No other record SHALL store an edge of a version. Boom SHALL refuse `changes` on a version that is not a decision. Boom SHALL keep each edge one time, in the order given.

#### Scenario: A decision with its edges

- **WHEN** the store holds `flume-has-a-resident` and `one-github-client-per-host`, and `boom propose decision the-runner-serves-a-socket "<statement>" --body <file> --source flume-has-a-resident --changes one-github-client-per-host --origin mockup.svg --provenance run-1` runs
- **THEN** boom exits 0, and the entry of the proposal holds the edge `source`, the edge `changes` and the edge `origin`

#### Scenario: Changes on an invariant

- **WHEN** an agent proposes an invariant with `--changes one-github-client-per-host`
- **THEN** boom exits 2 and says that only a decision changes claims, and boom appends nothing

#### Scenario: One source given two times

- **WHEN** an agent proposes a claim with `--source flume-has-a-resident` two times
- **THEN** the entry of the proposal holds one edge `source` to `flume-has-a-resident`

### Requirement: An edge to a claim names the version that it meant

A `source` edge and a `changes` edge SHALL be `<name>` or `<name>@<version>` of a claim of the store. For `<name>`, boom SHALL store the accepted version of that claim, or its newest version when none is accepted. Boom SHALL refuse a name that no claim has, a version that its claim does not have, and an edge of any other form. A URL, a path of a file and the number of an old record are such forms.

#### Scenario: A name stores its accepted version

- **WHEN** an agent proposes a claim with `--source status-read-from-events`, and `status-read-from-events@<v1>` is accepted
- **THEN** the entry stores the edge `status-read-from-events@<v1>`

#### Scenario: A name with no accepted version

- **WHEN** an agent proposes a claim with `--source money-has-currency`, and no version of `money-has-currency` is accepted
- **THEN** the entry stores the edge to the newest version of `money-has-currency`

#### Scenario: A claim that the store does not hold

- **WHEN** an agent proposes a claim with `--source record-0001`, and no claim has that name
- **THEN** boom exits 2 and names `record-0001`, and boom appends nothing

#### Scenario: A version that its claim does not have

- **WHEN** an agent proposes a claim with `--source status-read-from-events@000000000000`, and that claim has no such version
- **THEN** boom exits 2 and names `status-read-from-events@000000000000`, and boom appends nothing

#### Scenario: A URL

- **WHEN** an agent proposes a claim with `--source https://example.com/model`
- **THEN** boom exits 2 and names the edge. It says that an edge names a claim of the store. Boom appends nothing

### Requirement: The origin of a claim

An `origin` edge SHALL name a file that boom keeps, or a file in a commit. For `--origin <file>`, boom SHALL keep the bytes of the file under their SHA-256 fingerprint, with the name of the file. Boom SHALL store that fingerprint. For `--origin git:<repository>@<commit>:<path>`, boom SHALL store that reference. The commit SHALL be 40 hexadecimal characters. The path SHALL be relative, with no `..`. Boom SHALL NOT read a repository.

#### Scenario: A mockup kept by its fingerprint

- **WHEN** an agent proposes a claim with `--origin mockup.svg`
- **THEN** boom exits 0, and the entry stores the name `mockup.svg` and the fingerprint of its bytes

#### Scenario: A file in a commit

- **WHEN** an agent proposes a claim with `--origin git:sluice@<40 hexadecimal characters>:design/overview.svg`
- **THEN** boom exits 0, and the entry stores that reference. `boom show` gives the repository, the commit and the path

#### Scenario: A short commit

- **WHEN** an agent proposes a claim with `--origin git:sluice@1a2b3c:design/overview.svg`
- **THEN** boom exits 2 and says that a commit has 40 hexadecimal characters, and boom appends nothing

#### Scenario: A commit with no path

- **WHEN** an agent proposes a claim with `--origin git:sluice@<40 hexadecimal characters>`
- **THEN** boom exits 2 and says that a reference to a commit names the path of a file in it, and boom appends nothing

#### Scenario: A path out of the tree

- **WHEN** an agent proposes a claim with `--origin git:sluice@<40 hexadecimal characters>:../secret.txt`
- **THEN** boom exits 2 and says that the path is relative, with no `..`, and boom appends nothing

### Requirement: boom show writes an artifact

`boom show --artifact <fingerprint>` SHALL write the bytes of the artifact that boom keeps under that fingerprint to standard output. `boom show` SHALL take a target or an artifact. Boom SHALL refuse both, and SHALL refuse neither. Boom SHALL refuse a fingerprint of no artifact that it keeps.

#### Scenario: A mockup read back

- **WHEN** an agent proposes a claim with `--origin mockup.svg`, and then `boom show --artifact <fingerprint>` runs
- **THEN** the second command writes the bytes of `mockup.svg` to standard output, and exits 0

#### Scenario: A fingerprint of no artifact

- **WHEN** `boom show --artifact <fingerprint>` runs, and boom keeps no artifact with that fingerprint
- **THEN** boom exits 2 and says that no artifact has that fingerprint

#### Scenario: A target and an artifact

- **WHEN** `boom show money-has-currency --artifact <fingerprint>` runs
- **THEN** boom exits 2 and says that `boom show` takes a target or an artifact, and not both

### Requirement: boom show gives the edges of a version

`boom show <name>@<version>` SHALL give each edge of the version, in its order. For an edge to a claim, it SHALL give the version that the edge meant and its statement. It SHALL mark the edge out of date where another version of that claim is accepted now. For each `changes` edge, it SHALL give `Before:` with that statement, and `Now:` with the statement of the decision. For an `origin` edge, it SHALL give the name and the fingerprint, or the repository, the commit and the path.

#### Scenario: An edge out of date

- **WHEN** a decision names `flume-has-a-resident@<v1>` as a source, a person then accepts `flume-has-a-resident@<v2>`, and `boom show <decision>@<version>` runs
- **THEN** boom gives the edge to `flume-has-a-resident@<v1>` with its statement. Boom marks it out of date, with `<v2>` as the accepted version

#### Scenario: A change, before and now

- **WHEN** a decision with the statement "The runner serves a socket for status and tokens" changes `one-github-client-per-host`, and `boom show <decision>@<version>` runs
- **THEN** boom gives `Before:` with the statement of `one-github-client-per-host`. Boom gives `Now:` with "The runner serves a socket for status and tokens"

#### Scenario: The edges answered as JSON

- **WHEN** `boom show <decision>@<version> --json` runs for the decision with its edges
- **THEN** the answer gives `source`, `changes` and `origin`, each a list in the order given

### Requirement: boom show gives the claims that rest on a claim

`boom show <name>` SHALL give each claim whose version names the claim as a source, under `source of:`. It SHALL give each decision whose version changes the claim, under `changed by:`. A version counts when it is the accepted version of its claim, or the newest version when none is accepted. Boom SHALL build this answer from the edges in the views when asked, and SHALL keep no graph. With `--json`, the answer SHALL give `source_of` and `changed_by`.

#### Scenario: A claim named as a source

- **WHEN** the accepted version of `the-runner-serves-a-socket` names `flume-has-a-resident` as a source, and `boom show flume-has-a-resident --json` runs
- **THEN** `source_of` gives `the-runner-serves-a-socket` and its version

#### Scenario: A claim changed

- **WHEN** the accepted version of the decision `the-runner-serves-a-socket` changes `one-github-client-per-host`, and `boom show one-github-client-per-host --json` runs
- **THEN** `changed_by` gives `the-runner-serves-a-socket` and its version

#### Scenario: A name in a body

- **WHEN** a body names `flume-has-a-resident` in its text, and no edge names it
- **THEN** `source_of` of `flume-has-a-resident` is empty

#### Scenario: The same answer after the views are built again

- **WHEN** `boom show flume-has-a-resident --json` runs two times, and `boom verify` drops and builds the views between the two runs
- **THEN** both runs print the same bytes
