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

#[cfg(test)]
mod tests {
    use super::*;

    #[seastar::test]
    async fn test_empty_spawn_void() {
        assert!(matches!(spawn(async move {}).await, ()));
    }

    #[seastar::test]
    async fn test_chained_spawn_void() {
        let res = spawn(async move {
            let _ = spawn(async move {}).await;
        })
        .await;
        assert!(matches!(res, ()));
    }

    #[seastar::test]
    async fn test_spawn_int() {
        let res = spawn(async move { 0 }).await;
        assert!(matches!(res, 0));
    }

    #[seastar::test]
    async fn test_two_spawn_int_and_void() {
        let mut res = spawn(async move { 0 }).await;
        assert!(matches!(res, 0));
        res = spawn(async move { 1 }).await;
        assert!(matches!(res, 1));
        assert!(matches!(spawn(async move {}).await, ()));
        assert!(matches!(spawn(async move {}).await, ()));
    }

    #[seastar::test]
    async fn test_chained_spawn_int() {
        let res = spawn(async move { spawn(async move { 2 }).await }).await;
        assert!(matches!(res, 2));
    }
}
