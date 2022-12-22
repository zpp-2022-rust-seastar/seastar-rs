#[cxx::bridge(namespace = "seastar")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/core/preempt.hh");

        /// Checks whether the current task exhausted its time quota and should
        /// yield to the runtime as soon as possible.
        fn need_preempt() -> bool;
    }
}

pub use ffi::need_preempt;

#[test]
fn test_preempt_smoke_test() {
    // The need_preempt function "works" even if there is no Seastar runtime
    // present. This test was only used to make sure that linking with Seastar
    // works properly. It can be removed later after we get some proper tests.
    assert!(!need_preempt());
    assert!(!need_preempt());
    assert!(!need_preempt());
}
