//! Idiomatic bindings to the Seastar framework.
//!
//! Work in progress! Definitely not for use in production yet.

mod api_safety;
mod cxx_async_futures;
mod cxx_async_local_future;

mod config_and_start_seastar;
mod preempt;

#[cfg(test)]
pub(crate) mod seastar_test_guard;

#[cfg(test)]
pub(crate) use seastar_test_guard::acquire_guard_for_seastar_test;

pub use api_safety::*;
pub use config_and_start_seastar::*;
pub use preempt::*;

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

/// A macro that runs the `main` function's contents
/// in Seastar's runtime.
///
/// **Only** `main` is allowed to use this macro -
/// Seastar apps may only use one `AppTemplate`
/// instance at a time.
///
/// # Options
///
/// Currently, passing options is not supported.
/// The app is simply run with default parameters.
///
/// # Usage
///
/// ```rust
/// #[seastar::main]
/// async fn main() {
///     println!("Hello, world!");
/// }
/// ```
pub use seastar_macros::main;
