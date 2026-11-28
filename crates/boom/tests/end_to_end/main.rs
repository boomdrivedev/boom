//! The end-to-end set: the tests that run the built binary, a shell or
//! git, one for each command's main path and one for each seam with a
//! real tool, in one test binary (logic-is-tested-as-units-and-few-tests-run-the-binary).
//! Each test ends each process that it starts, and the set runs at most
//! `SLOTS` tests at one time. The tests of boom's logic run in the crate,
//! in src/tests/.

#[path = "../../src/tests/support.rs"]
mod support;

#[path = "../../src/tests/hooks_store.rs"]
mod hooks_store;

mod command_line;
mod hooks;
mod mcp;
mod page;
mod signatures;
mod store;
mod terms;

use std::sync::{Condvar, Mutex};

/// How many tests of the set run at one time.
const SLOTS: usize = 4;

static TAKEN: Mutex<usize> = Mutex::new(0);
static FREED: Condvar = Condvar::new();

/// One of the set's slots, given back when it drops.
pub struct Slot;

/// Waits for a free slot and takes it: each test of the set takes one
/// before it starts a process.
pub fn slot() -> Slot {
    let mut taken = TAKEN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    while *taken >= SLOTS {
        taken = FREED
            .wait(taken)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
    *taken += 1;
    Slot
}

impl Drop for Slot {
    fn drop(&mut self) {
        let mut taken = TAKEN
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *taken -= 1;
        FREED.notify_one();
    }
}
