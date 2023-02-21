use crate::assert_runtime_is_running;
use cxx::SharedPtr;

#[cxx::bridge(namespace = "seastar_ffi::scheduling")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/scheduling.hh");

        type scheduling_group;

        #[namespace = "seastar_ffi"]
        type VoidFuture = crate::cxx_async_futures::VoidFuture;

        fn new_sg() -> SharedPtr<scheduling_group>;

        fn sg_active(sg: &scheduling_group) -> bool;

        fn sg_name(sg: &scheduling_group) -> &str;

        fn sg_is_main(sg: &scheduling_group) -> bool;

        fn sg_set_shares(sg: &SharedPtr<scheduling_group>, shares: f32);

        fn sg_equal(sg1: &scheduling_group, sg2: &scheduling_group) -> bool;

        fn create_sg(sg: &mut SharedPtr<scheduling_group>, name: &str, shares: f32) -> VoidFuture;

        fn destroy_sg(sg: &SharedPtr<scheduling_group>) -> VoidFuture;

        fn rename_sg(sg: &SharedPtr<scheduling_group>, new_name: &str) -> VoidFuture;

        fn max_sg() -> u32;

        fn current_sg() -> SharedPtr<scheduling_group>;
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

impl PartialEq for SchedulingGroup {
    fn eq(&self, other: &Self) -> bool {
        sg_equal(&self.inner, &other.inner)
    }
}

impl Eq for SchedulingGroup {}

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

    /// Creates a scheduling group with a specified number of shares.
    ///
    /// The operation is global and affects all shards. The returned scheduling
    /// group can then be used in any shard.
    ///
    /// # Arguments
    /// * `name` - A name that identifiers the group; will be used as a label in the
    ///    group's metrics.
    /// * `shares` - The number of shares of the CPU time allotted to the group;
    ///    Use numbers in the 1-1000 range (but can go above).
    pub async fn create(name: &str, shares: f32) -> Self {
        assert_runtime_is_running();
        let mut sg = SharedPtr::null();
        create_sg(&mut sg, name, shares).await.unwrap();
        SchedulingGroup { inner: sg }
    }

    /// Destroys a scheduling group.
    ///
    /// Destroys a scheduling group previously created with
    /// [`create`](SchedulingGroup::create).
    ///
    /// The destroyed group must not be currently in use
    /// and must not be used or destroyed again later.
    ///
    /// The operation is global and affects all shards.
    ///
    /// Returns a future that is ready when the scheduling group has been torn down.
    pub async unsafe fn destroy(&self) {
        assert_runtime_is_running();
        destroy_sg(&self.inner).await.unwrap();
    }

    /// Renames scheduling group.
    ///
    /// Renames a scheduling group previously created with
    /// [`create`](SchedulingGroup::create). The operation is global and affects all
    /// shards. The operation affects the exported statistics labels.
    ///
    /// Returns a future that is ready when the scheduling group has been renamed.
    ///
    /// # Arguments
    /// * `new_name` - The new name for the scheduling group.
    pub async fn rename(&self, new_name: &str) {
        assert_runtime_is_running();
        rename_sg(&self.inner, new_name).await.unwrap();
    }

    /// Returns the maximal number of scheduling groups defined by
    /// `SEASTAR_SCHEDULING_GROUPS_COUNT`.
    pub fn max() -> u32 {
        max_sg()
    }

    /// Returns the current scheduling group.
    pub fn current() -> Self {
        SchedulingGroup {
            inner: current_sg(),
        }
    }
}
