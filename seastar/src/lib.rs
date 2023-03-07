//! Idiomatic bindings to the Seastar framework.
//!
//! Work in progress! Definitely not for use in production yet.

mod api_safety;
mod config_and_start_seastar;
mod cxx_async_futures;
mod cxx_async_local_future;
mod ffi_utils;
mod gate;
mod preempt;
#[cfg(test)]
pub(crate) mod seastar_test_guard;
mod spawn;
mod submit_to;

#[cfg(test)]
pub(crate) use seastar_test_guard::acquire_guard_for_seastar_test;

pub use api_safety::*;
pub use config_and_start_seastar::*;
pub use gate::*;
pub use preempt::*;
pub use spawn::*;
pub use submit_to::*;

/// A macro intended for running asynchronous tests.
///
/// Tests are spawned in a separate thread.
/// This is done to ensure thread_local cleanup between them
/// (at the time of writing, Seastar doesn't do it itself).
///
/// # Usage
///
/// ```rust
/// #[seastar::test]
/// async fn my_test() {
///     assert!(true);
/// }
/// ```
pub use seastar_macros::test;
