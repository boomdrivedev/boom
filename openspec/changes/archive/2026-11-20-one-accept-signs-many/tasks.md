## 1. The entry

- [x] 1.1 `Act::Accepted` names its versions. An accept of one version keeps the form `name` and `version`, and an accept of several names `versions`
- [x] 1.2 Read both forms, and refuse `versions` that holds one version

## 2. The judge, the views and the replay

- [x] 2.1 Refuse an accept that names no version, or one claim two times
- [x] 2.2 Judge each version of the act. The act counts when each version counts, and repeats when each repeats
- [x] 2.3 Apply the accept to each version, in the state, the views and the replay

## 3. The commands

- [x] 3.1 `boom accept` leaves out each version that the person accepted already, writes one entry, judges it whole, and signs it one time
- [x] 3.2 `boom log` gives each version of an act, and `targets`
- [x] 3.3 `boom show <number>` refuses an entry of several versions and lists them

## 4. The tests and the gate

- [x] 4.1 Test three versions accepted together with one signature, and accepted again
- [x] 4.2 Test one claim two times, an entry with one bad version, the views and a replay, and the log and the show
- [x] 4.3 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs` and `root:changes` through `sluice exec main`
