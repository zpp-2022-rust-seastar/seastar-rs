use crate as seastar;
use crate::cxx_async_local_future::IntoCxxAsyncLocalFuture;
use core::cell::Cell;
use ffi::*;
use std::future::Future;
use std::rc::Rc;

#[cxx::bridge]
mod ffi {
    #[namespace = "seastar_ffi"]
    unsafe extern "C++" {
        type VoidFuture = crate::cxx_async_futures::VoidFuture;
    }

    #[namespace = "seastar_ffi::spawn"]
    unsafe extern "C++" {
        include!("seastar/src/spawn.hh");

        fn cpp_spawn(future: VoidFuture) -> VoidFuture;
    }
}

/// Spawns a new asynchronous task, returning an `Ret`.
///
/// The provided future will start running in the background immediately
/// when `spawn` is called.
///
/// Spawning a task enables the task to execute concurrently to other tasks.
///
/// This function must be called from the context of a Seastar runtime.
pub async fn spawn<T, Ret: 'static>(future: T) -> Ret
where
    T: Future<Output = Ret> + 'static,
{
    seastar::assert_runtime_is_running();

    let x: Rc<Cell<Option<Ret>>> = Default::default();

    let x_clone = x.clone();
    match cpp_spawn(VoidFuture::infallible_local(async move {
        x_clone.set(Some(future.await));
    }))
    .await
    {
        Ok(_) => x.take().unwrap(),
        Err(_) => panic!(),
    }
}
