//! Idiomatic bindings to the Seastar framework.
//!
//! Work in progress! Definitely not for use in production yet.

mod cxx_async_futures;
mod local_box_future;

mod config_and_start_seastar;
mod preempt;

pub use config_and_start_seastar::*;
pub use preempt::*;
