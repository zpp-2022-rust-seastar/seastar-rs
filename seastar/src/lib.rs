//! Idiomatic bindings to the Seastar framework.
//!
//! Work in progress! Definitely not for use in production yet.

mod cxx_async_local_future;

mod preempt;

pub use preempt::*;
