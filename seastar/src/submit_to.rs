use crate as seastar;
use crate::ffi_utils::{get_dropper, get_fn_once_caller};
use ffi::*;
use std::future::Future;

use crate::cxx_async_local_future::IntoCxxAsyncLocalFuture;

#[cxx::bridge]
mod ffi {
    #[namespace = "seastar_ffi"]
    unsafe extern "C++" {
        type VoidFuture = crate::cxx_async_futures::VoidFuture;
    }

    #[namespace = "seastar_ffi::submit_to"]
    unsafe extern "C++" {
        include!("seastar/src/submit_to.hh");

        unsafe fn submit_to(
            shard_id: u32,
            closure: *mut u8,
            caller: unsafe fn(*mut u8) -> VoidFuture,
        ) -> VoidFuture;
    }
}

/// Runs a function `func` on a `shard_id` shard.
///
/// # Example
///
/// ```rust
/// #[seastar::test]
/// async fn submit_to_example() {
///     let ret = submit_to(0, || async { 42 }).await;
///     assert!(matches!(ret, 42));
/// }
pub async fn submit_to<Func, Fut, Ret>(shard_id: u32, func: Func) -> Ret
where
    Func: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Ret> + 'static,
    Ret: Send + 'static,
{
    let (tx, rx) = futures::channel::oneshot::channel::<Ret>();

    let closure = move || {
        VoidFuture::infallible_local(async {
            tx.send(func().await).ok();
        })
    };

    let closure_caller = get_fn_once_caller(&closure);
    let dropper = get_dropper(&closure);
    let boxed_closure = Box::into_raw(Box::new(closure)) as *mut u8;

    unsafe {
        match ffi::submit_to(shard_id, boxed_closure, closure_caller).await {
            Ok(_) => rx.await.unwrap(),
            Err(_) => {
                dropper(boxed_closure);
                panic!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[seastar::test]
    async fn test_submit_to() {
        let ret = submit_to(0, || async { 42 }).await;
        assert!(matches!(ret, 42));
    }

    #[seastar::test]
    async fn test_submit_to_nested() {
        let ret = submit_to(0, || async { submit_to(0, || async { 42 }).await }).await;
        assert!(matches!(ret, 42));
    }

    #[seastar::test]
    async fn test_submit_to_two_shards() {
        let ret = submit_to(0, || async { 17 }).await;
        assert!(matches!(ret, 17));
        let ret = submit_to(1, || async { 25 }).await;
        assert!(matches!(ret, 25));
    }

    #[seastar::test]
    async fn test_submit_to_two_shards_nested() {
        let ret = submit_to(1, || async { submit_to(0, || async { 42 }).await }).await;
        assert!(matches!(ret, 42));
    }
}
