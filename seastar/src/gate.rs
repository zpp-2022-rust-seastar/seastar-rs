use cxx::UniquePtr;
use std::marker::PhantomData;
use thiserror::Error;

#[cxx::bridge(namespace = "seastar_ffi::gate")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/gate.hh");

        type gate;
        type gate_holder;

        #[namespace = "seastar_ffi"]
        type VoidFuture = crate::cxx_async_futures::VoidFuture;

        fn new_gate() -> UniquePtr<gate>;
        fn new_gate_holder(gate: &UniquePtr<gate>) -> Result<UniquePtr<gate_holder>>;
        fn close_gate(gate: &UniquePtr<gate>) -> VoidFuture;
    }
}

use ffi::*;

/// Error returned by [`try_enter`](Gate::try_enter) when called on closed gate.
#[derive(Error, Debug)]
#[error("GateClosedError: gate closed")]
pub struct GateClosedError;

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

    /// Tries to enter the gate.
    ///
    /// If it succeeds, it returns [`GateHolder`] that will leave the gate when destroyed (RAII).
    ///
    /// If it fails, it returns [`GateClosedError`].
    pub fn try_enter(&self) -> Result<GateHolder, GateClosedError> {
        GateHolder::new(&self.inner)
    }

    /// Closes the gate.
    ///
    /// It must be called at most once (the underlying implementation aborts if it is closed twice).
    ///
    /// Returns a future that resolves after everybody have left the gate.
    pub async fn close(&self) {
        crate::assert_runtime_is_running();
        close_gate(&self.inner).await.unwrap();
    }
}

/// Facility to hold a gate opened using RAII.
///
/// A [`GateHolder`] is obtained when [`try_enter`](Gate::try_enter) succeeds.
///
/// The [`Gate`] is left when the [`GateHolder`] is dropped.
pub struct GateHolder<'a> {
    _inner: UniquePtr<gate_holder>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> GateHolder<'a> {
    fn new(gate: &'a UniquePtr<gate>) -> Result<Self, GateClosedError> {
        match new_gate_holder(gate) {
            Ok(holder) => Ok(GateHolder {
                _inner: holder,
                _phantom: PhantomData,
            }),
            Err(_) => Err(GateClosedError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as seastar;
    use futures::join;
    use std::{cell::RefCell, rc::Rc};

    #[seastar::test]
    async fn test_gate_only_close() {
        let gate = Gate::new();
        gate.close().await;
    }

    #[seastar::test]
    async fn test_gate_leave_then_close() {
        let gate = Gate::new();

        {
            let _ = gate.try_enter().unwrap();
        }

        gate.close().await;
    }

    #[seastar::test]
    async fn test_gate_close_then_leave() {
        let gate = Gate::new();

        let closing_started = Rc::new(RefCell::new(false));
        let closing_finished = Rc::new(RefCell::new(false));

        let handler = gate.try_enter().unwrap();
        let leave_future = async {
            drop(handler);
            assert!(*closing_started.borrow() && !*closing_finished.borrow());
        };

        let close_future = async {
            *closing_started.borrow_mut() = true;
            gate.close().await;
            *closing_finished.borrow_mut() = true;
        };

        // join! tries to finish close_future first.
        join!(close_future, leave_future);
        assert!(*closing_finished.borrow());
    }

    #[seastar::test]
    async fn test_gate_many_leave() {
        let gate = Gate::new();

        let closing_finished = Rc::new(RefCell::new(false));

        let handler1 = gate.try_enter().unwrap();
        let leave_future1 = async {
            drop(handler1);
            assert!(!*closing_finished.borrow());
        };

        let handler2 = gate.try_enter().unwrap();
        let leave_future2 = async {
            drop(handler2);
            assert!(!*closing_finished.borrow());
        };

        let handler3 = gate.try_enter().unwrap();
        let leave_future3 = async {
            drop(handler3);
            assert!(!*closing_finished.borrow());
        };

        let close_future = async {
            gate.close().await;
            *closing_finished.borrow_mut() = true;
        };

        join!(leave_future1, close_future, leave_future2, leave_future3);
        assert!(*closing_finished.borrow());
    }

    #[seastar::test]
    async fn test_gate_close_then_enter() {
        let gate = Gate::new();

        gate.close().await;

        match gate.try_enter() {
            Err(GateClosedError) => (),
            _ => panic!("gate.try_enter() should return Err(GateClosedError)."),
        }
    }
}
