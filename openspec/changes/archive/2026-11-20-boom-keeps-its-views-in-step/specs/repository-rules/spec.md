## MODIFIED Requirements

### Requirement: As the person accepted it

`boom query` SHALL take its answer from the views, once they are at the log's last entry, as the judge built them from the log. An entry that fails boom's checks SHALL change nothing an agent gets. A view edited by hand SHALL be found and built again by `boom verify`, after which `boom query` gives what the person accepted.

#### Scenario: A statement edited in the views

- **WHEN** a standing version's statement in the `versions` table is changed by another program, `views_head` left as it was, and `boom verify` runs
- **THEN** `boom query` gives the statement the person accepted

#### Scenario: A status edited in the views

- **WHEN** a proposed version attached to `shop` is marked standing in the `versions` and `claims` tables by another program, `views_head` left as it was, and `boom verify` runs
- **THEN** `boom query --repository shop` does not give it

#### Scenario: An accept written by hand

- **WHEN** an `accepted` entry with no signature, for a proposed version attached to `shop`, is appended to the log by another program, chained correctly
- **THEN** `boom query --repository shop` does not give that version

#### Scenario: The views and a full replay agree

- **WHEN** a store's views are kept in step through 200 proposals, accepts and rejects, and `boom query --repository shop --json` runs, then runs again after the views are dropped
- **THEN** both print the same bytes
