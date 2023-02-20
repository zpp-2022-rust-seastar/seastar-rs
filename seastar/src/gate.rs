use cxx::UniquePtr;

#[cxx::bridge(namespace = "seastar_ffi::gate")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/gate.hh");

        type gate;

        fn new_gate() -> UniquePtr<gate>;
    }
}

use ffi::*;

/// Facility to stop new requests, and to tell when existing requests are done.
///
/// When stopping a service that serves asynchronous requests, we are faced with
/// two problems: preventing new requests from coming in, and knowing when existing
/// requests have completed. The `Gate` class provides a solution.
pub struct Gate {
    inner: UniquePtr<gate>,
}

impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate {
    /// Creates a new gate.
    pub fn new() -> Self {
        Gate { inner: new_gate() }
    }
}
