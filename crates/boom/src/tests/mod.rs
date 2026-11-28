//! The tests of boom's logic, in its own process: each command runs
//! through `cli::run` on a store in a scratch folder, signing with keys
//! made in software, and starts no process, no git and no network
//! (logic-is-tested-as-units-and-few-tests-run-the-binary). The tests
//! that run the built binary, git or a shell are in `tests/`, and take
//! the same support by its path.

mod support;

mod batches;
mod claims;
mod contexts;
mod edges;
mod import;
mod narratives;
mod notes;
mod persons;
mod query;
mod search;
mod supersede;
mod views;
mod withdraw;
