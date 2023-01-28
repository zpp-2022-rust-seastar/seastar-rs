//! Idiomatic bindings to the Seastar framework.
//!
//! Work in progress! Definitely not for use in production yet.

mod cxx_async_futures;
mod cxx_async_local_future;

mod config_and_start_seastar;
mod preempt;

#[cfg(test)]
pub(crate) mod seastar_test_guard;

#[cfg(test)]
pub(crate) use seastar_test_guard::acquire_guard_for_seastar_test;

pub use config_and_start_seastar::*;
pub use preempt::*;
