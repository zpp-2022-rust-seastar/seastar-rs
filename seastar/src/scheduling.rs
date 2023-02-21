use cxx::UniquePtr;

#[cxx::bridge(namespace = "seastar_ffi::scheduling")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/scheduling.hh");

        type scheduling_group;

        fn new_sg() -> UniquePtr<scheduling_group>;
    }
}

use ffi::*;

/// Identifies function calls that are accounted as a group.
///
/// A `SchedulingGroup` is a tag that can be used to mark a function call.
/// Executions of such tagged calls are accounted as a group.
pub struct SchedulingGroup {
    pub(crate) inner: UniquePtr<scheduling_group>,
}

impl Default for SchedulingGroup {
    /// Creates a `SchedulingGroup` instance denoting the default group.
    fn default() -> Self {
        SchedulingGroup { inner: new_sg() }
    }
}
