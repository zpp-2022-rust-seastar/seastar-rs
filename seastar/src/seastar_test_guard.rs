use std::sync::{Mutex, MutexGuard};

static RUNNING_TEST_WITH_SEASTAR: Mutex<()> = Mutex::new(());

pub(crate) struct RunningTestWithSeastarGuard(MutexGuard<'static, ()>);

/// Acquires a global mutex for the purpose of running a test with the
/// seastar runtime.
///
/// Although seastar seems to support starting and stopping the runtime
/// multiple times in a sequence, it doesn't like (and doesn't detect)
/// when multiple runtimes are trying to run in parallel. `cargo test` runs
/// tests in parallel by default, so it's easy to write tests which will
/// make seastar angry (the anger manifests itself in form of segfaults).
///
/// The mutex should be taken by all tests that create a seastar runtime
/// and held until the test finishes.
pub(crate) fn acquire_guard_for_seastar_test() -> RunningTestWithSeastarGuard {
    // If a test panics, we assume that the runtime has been stopped
    // properly in that test, so we can ignore that the lock is poisoned.
    let guard = RUNNING_TEST_WITH_SEASTAR
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    RunningTestWithSeastarGuard(guard)
}
