use std::sync::{Mutex, MutexGuard};

static RUNNING_TEST_WITH_SEASTAR: Mutex<()> = Mutex::new(());

pub struct RunningTestWithSeastarGuard(MutexGuard<'static, ()>);

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
pub fn acquire_guard_for_seastar_test() -> RunningTestWithSeastarGuard {
    let guard = RUNNING_TEST_WITH_SEASTAR.lock().unwrap();
    RunningTestWithSeastarGuard(guard)
}
