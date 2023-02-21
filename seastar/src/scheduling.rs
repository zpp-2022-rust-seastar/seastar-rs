use crate::assert_runtime_is_running;
use cxx::SharedPtr;

#[cxx::bridge(namespace = "seastar_ffi::scheduling")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/scheduling.hh");

        type scheduling_group;

        fn new_sg() -> SharedPtr<scheduling_group>;

        fn sg_active(sg: &scheduling_group) -> bool;

        fn sg_name(sg: &scheduling_group) -> &str;

        fn sg_is_main(sg: &scheduling_group) -> bool;

        fn sg_set_shares(sg: &SharedPtr<scheduling_group>, shares: f32);
    }
}

use ffi::*;

/// Identifies function calls that are accounted as a group.
///
/// A `SchedulingGroup` is a tag that can be used to mark a function call.
/// Executions of such tagged calls are accounted as a group.
#[derive(Clone)]
pub struct SchedulingGroup {
    pub(crate) inner: SharedPtr<scheduling_group>,
}

impl Default for SchedulingGroup {
    /// Creates a `SchedulingGroup` instance denoting the default group.
    fn default() -> Self {
        SchedulingGroup { inner: new_sg() }
    }
}

impl SchedulingGroup {
    /// Checks if the group is active.
    pub fn active(&self) -> bool {
        sg_active(&self.inner)
    }

    /// Returns the name of the group.
    pub fn name(&self) -> &str {
        assert_runtime_is_running();
        sg_name(&self.inner)
    }

    /// Checks if the group is main (default).
    pub fn is_main(&self) -> bool {
        sg_is_main(&self.inner)
    }

    /// Adjusts the number of shares allotted to the group.
    ///
    /// Dynamically adjusts the number of shares allotted to the group, increasing or
    /// decreasing the amount of CPU bandwidth it gets. The adjustment is local to
    /// the shard.
    ///
    /// This can be used to reduce a background job's interference with a foreground
    /// load: the shares can be started at a low value, increased when the background
    /// job's backlog increases, and reduced again when the backlog decreases.
    ///
    /// # Arguments
    /// * `shares` - The number of shares allotted to the group. Use numbers in the
    ///   1-1000 range.
    pub fn set_shares(&self, shares: f32) {
        assert_runtime_is_running();
        sg_set_shares(&self.inner, shares);
    }
}
