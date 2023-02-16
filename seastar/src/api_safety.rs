#[cxx::bridge(namespace = "seastar")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/core/reactor.hh");

        /// Checks whether the current thread is within a Seastar runtime.
        fn engine_is_ready() -> bool;
    }
}

pub use ffi::engine_is_ready;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppTemplate;
    use std::thread;

    #[test]
    fn test_engine_is_ready_only_for_thread_in_runtime() {
        assert!(!engine_is_ready());

        thread::spawn(|| {
            assert!(!engine_is_ready());
            let _guard = crate::acquire_guard_for_seastar_test();
            let mut app = AppTemplate::default();
            let fut = async {
                assert!(engine_is_ready());
                Ok(())
            };
            let args = vec!["test"];
            app.run_void(&args, fut);
            // Currently, Seastar does not perform a cleanup of its thread locals,
            // which unfortunately means the following assertion will pass.
            // Should it fail, we must:
            // 1) negate its condition
            // 2) review other code in the project that also follows
            //    this assumption and modify it accordingly.
            assert!(engine_is_ready());
        })
        .join()
        .unwrap();

        assert!(!engine_is_ready());
    }
}

/// Intended to be used in a runtime-dependent function.
/// Panics if called outside of a Seastar runtime.
pub fn assert_runtime_is_running() {
    if !engine_is_ready() {
        panic!("Attempting to call a runtime-dependent function outside of a Seastar runtime");
    }
}

/// Mainly intended to be used in [`seastar::AppTemplate::run_void`] and  [`seastar::AppTemplate::run_int`].
/// Panics if called within a Seastar runtime.
pub fn assert_runtime_is_not_running() {
    if engine_is_ready() {
        panic!("Attempting to call a function inside of a Seastar runtime that must be used outside of it");
    }
}
