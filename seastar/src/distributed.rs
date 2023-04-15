use crate::{
    cxx_async_local_future::IntoCxxAsyncLocalFuture,
    ffi_utils::{get_dropper_const, get_dropper_noarg, get_fn_caller, PtrWrapper},
    get_count,
    submit_to::submit_to,
    this_shard_id,
};
use core::marker::PhantomData;
use cxx::SharedPtr;
use std::future::Future;
use std::pin::Pin;

#[cxx::bridge(namespace = "seastar_ffi::distributed")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/distributed.hh");

        type distributed;

        #[namespace = "seastar_ffi"]
        type VoidFuture = crate::cxx_async_futures::VoidFuture;

        fn new_distributed() -> SharedPtr<distributed>;

        fn local(distr: &distributed) -> *const u8;

        unsafe fn start(
            distr: &distributed,
            raw_service_maker: *const u8,
            raw_service_maker_caller: unsafe fn(*const u8) -> *mut u8,
            raw_service_maker_droppper: unsafe fn(*const u8),
            stop_caller: unsafe fn(*mut u8) -> VoidFuture,
            dropper: unsafe fn(*mut u8),
        ) -> VoidFuture;

        unsafe fn start_single(
            distr: &distributed,
            raw_service_maker: *const u8,
            raw_service_maker_caller: unsafe fn(*const u8) -> *mut u8,
            raw_service_maker_droppper: unsafe fn(*const u8),
            stop_caller: unsafe fn(*mut u8) -> VoidFuture,
            dropper: unsafe fn(*mut u8),
        ) -> VoidFuture;

        fn stop(distr: &distributed) -> VoidFuture;
    }
}

fn stop_caller<S: Service>(raw_service: *mut u8) -> VoidFuture {
    VoidFuture::infallible_local(async move {
        let stop_future = unsafe { (raw_service as *mut S).as_mut().unwrap().stop() };
        Pin::from(stop_future).await;
    })
}

const fn get_stop_caller<S: Service>() -> fn(*mut u8) -> VoidFuture {
    stop_caller::<S>
}

use ffi::{distributed, VoidFuture};

unsafe impl Send for distributed {}
unsafe impl Sync for distributed {}

/// A trait which a service inside `Distributed` must implement.
///
/// Because of Rust not yet supporting `async` trait methods,
/// or trait methods that return an `impl` (`impl Future`) in this case,
/// the returned future must be `Box`ed.
///
/// # Examples
///
/// ```rust
/// use std::future::Future;
/// use seastar::Service;
///
/// struct FooService;
///
/// impl Service for FooService {
///     fn stop(&self) -> Box<dyn Future<Output = ()>> {
///         Box::new(async { println!("Shutting down!") })
///     }
/// }
/// ```
pub trait Service {
    /// The place to define what (possibly asynchronous) cleanup must be done for the service.
    ///
    /// If not implemented, defaults to a no-op.
    fn stop(&self) -> Box<dyn Future<Output = ()>> {
        Box::new(async {})
    }
}

/// An object on which `Distributed`'s mapping functions operate.
///
/// It provides access to the local instance of a service,
/// as well as its container, which can be used to communicate with other shards.
pub struct PeeringShardedService<'a, S>
where
    S: Service,
{
    pub instance: &'a S,
    pub container: &'a Distributed<S>,
}

/// A service distributed amongst all shards of a Seastar app.
///
/// You can use this to, for example, load balance a local storage.
pub struct Distributed<S: Service> {
    _inner: SharedPtr<distributed>,
    _ty: PhantomData<S>,
}

impl<S: Service> Distributed<S> {
    /// Returns a reference to the underlying service on the current shard.
    ///
    pub fn local(&self) -> &S {
        let local = ffi::local(self._inner.as_ref().unwrap());
        unsafe { &*(local as *const S) }
    }

    fn start_inner<Func>(service_maker: Func, single: bool) -> impl Future<Output = Self>
    where
        Func: Fn() -> S + Sync,
    {
        crate::assert_runtime_is_running();

        let stop_caller = get_stop_caller::<S>();
        let dropper = get_dropper_noarg::<S>();

        let raw_service_maker = move || Box::into_raw(Box::new(service_maker())) as *mut u8;
        let raw_service_maker_caller = get_fn_caller(&raw_service_maker);
        let raw_service_maker_dropper = get_dropper_const(&raw_service_maker);
        let boxed_raw_service_maker = Box::into_raw(Box::new(raw_service_maker)) as *const u8;

        let distr = ffi::new_distributed();
        let fut = unsafe {
            if single {
                ffi::start_single(
                    distr.as_ref().unwrap(),
                    boxed_raw_service_maker,
                    raw_service_maker_caller,
                    raw_service_maker_dropper,
                    stop_caller,
                    dropper,
                )
            } else {
                ffi::start(
                    distr.as_ref().unwrap(),
                    boxed_raw_service_maker,
                    raw_service_maker_caller,
                    raw_service_maker_dropper,
                    stop_caller,
                    dropper,
                )
            }
        };

        async move {
            match fut.await {
                Ok(_) => Distributed {
                    _inner: distr,
                    _ty: PhantomData,
                },
                Err(_) => panic!(),
            }
        }
    }

    /// Starts a single instance of the service on shard `0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::future::Future;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    /// use seastar::{Distributed, Service};
    ///
    /// struct CounterService(Arc<AtomicU32>);
    ///
    /// impl Service for CounterService {
    ///     fn stop(&self) -> Box<dyn Future<Output = ()>> {
    ///         let counter = self.0.clone();
    ///         Box::new(async move {
    ///             counter.fetch_add(1, Ordering::SeqCst);
    ///         })
    ///     }
    /// }
    ///
    /// #[seastar::test]
    /// async fn test_start_single_and_stop() {
    ///     let counter_clone = counter.clone();
    ///     let service_maker = move || CounterService(counter_clone.clone());
    ///     let distr = Distributed::start_single(service_maker).await;
    ///     distr.stop().await;
    ///     assert_eq!(1, counter.load(Ordering::SeqCst));
    /// }
    /// ```
    pub fn start_single<Func>(service_maker: Func) -> impl Future<Output = Self>
    where
        Func: Fn() -> S + Sync,
    {
        Distributed::start_inner(service_maker, true)
    }

    /// Starts an instance of the service on each shard.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::future::Future;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    /// use seastar::{get_count, Distributed, Service};
    ///
    /// struct CounterService(Arc<AtomicU32>);
    ///
    /// impl Service for CounterService {
    ///     fn stop(&self) -> Box<dyn Future<Output = ()>> {
    ///         let counter = self.0.clone();
    ///         Box::new(async move {
    ///             counter.fetch_add(1, Ordering::SeqCst);
    ///         })
    ///     }
    /// }
    ///
    /// #[seastar::test]
    /// async fn test_start_and_stop() {
    ///     let counter: Arc<AtomicU32> = Default::default();
    ///     let counter_clone = counter.clone();
    ///     let service_maker = move || CounterService(counter_clone.clone());
    ///     let distr = Distributed::start(service_maker).await;
    ///     distr.stop().await;
    ///     assert_eq!(get_count(), counter.load(Ordering::SeqCst));
    /// }
    /// ```
    pub fn start<Func>(service_maker: Func) -> impl Future<Output = Self>
    where
        Func: Fn() -> S + Sync,
    {
        Distributed::start_inner(service_maker, false)
    }

    /// Stops the service on all shards on which it was ran, freeing each instance's memory. Effectively an async destructor.
    ///
    /// This **must** be called when the distributed service is no longer to be used!.
    ///
    /// ```rust
    /// use std::future::Future;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    /// use seastar::{Distributed, Service};
    ///
    /// struct CounterService(Arc<AtomicU32>);
    ///
    /// impl Service for CounterService {
    ///     fn stop(&self) -> Box<dyn Future<Output = ()>> {
    ///         let counter = self.0.clone();
    ///         Box::new(async move {
    ///             counter.fetch_add(1, Ordering::SeqCst);
    ///         })
    ///     }
    /// }
    ///
    /// #[seastar::test]
    /// async fn test_start_single_and_stop() {
    ///     let counter_clone = counter.clone();
    ///     let service_maker = move || CounterService(counter_clone.clone());
    ///     let distr = Distributed::start_single(service_maker).await;
    ///     distr.stop().await;
    ///     assert_eq!(1, counter.load(Ordering::SeqCst));
    /// }
    /// ```
    pub async fn stop(&self) {
        crate::assert_runtime_is_running();
        ffi::stop(self._inner.as_ref().unwrap()).await.unwrap();
    }

    fn submit_to<'a, Func, Fut, Ret>(
        &'a self,
        shard_id: u32,
        func: Func,
        container: PtrWrapper,
    ) -> impl Future<Output = Ret>
    where
        Func: FnOnce(PeeringShardedService<'a, S>) -> Fut + Send + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        crate::assert_runtime_is_running();

        let distr = self._inner.clone();
        submit_to(shard_id, || async move {
            let instance = ffi::local(distr.as_ref().unwrap());
            let instance: &S = unsafe { &*(instance as *const S) };
            let _ = &container; // this is to avoid a partial move of the pointer
            let container = unsafe { &*(container.as_ptr_mut() as *const Distributed<S>) };
            let pss = PeeringShardedService {
                instance,
                container,
            };
            func(pss).await
        })
    }

    fn map_selected<'a, Func, Fut, Ret, I>(
        &'a self,
        func: Func,
        shards: I,
    ) -> Vec<impl Future<Output = Ret>>
    where
        Func: FnOnce(PeeringShardedService<'a, S>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
        I: IntoIterator<Item = u32>,
    {
        crate::assert_runtime_is_running();

        shards
            .into_iter()
            .map(|shard| {
                let distr = unsafe { PtrWrapper::new(self as *const Distributed<S> as _) };
                (shard, func.clone(), self._inner.clone(), distr)
            })
            .map(|(shard, func, inner, distr)| async move {
                submit_to(shard, || async move {
                    let local = ffi::local(inner.as_ref().unwrap());
                    let local = unsafe { &*(local as *const S) };
                    let _ = &distr; // this is to avoid a partial move of the pointer
                    let distr = unsafe { &*(distr.as_ptr_mut() as *const Distributed<S>) };
                    let pss = PeeringShardedService {
                        instance: local,
                        container: distr,
                    };
                    func(pss).await
                })
                .await
            })
            .collect()
    }

    /// Applies a map function to all instances of the service and returns a vector of the results.
    ///
    /// Equivalent to `seastar::distributed::map`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    /// use futures::future::join_all;
    /// use seastar::{get_count, Distributed, Service};
    ///
    /// struct CounterService(Arc<AtomicU32>);
    ///
    /// impl CounterService {
    ///     async fn inc(&self) {
    ///         self.0.fetch_add(1, Ordering::SeqCst);
    ///     }
    /// }
    ///
    /// impl Service for CounterService {}
    ///
    /// #[seastar::test]
    /// async fn test_map_all() {
    ///     let counter: Arc<AtomicU32> = Default::default();
    ///     let counter_clone = counter.clone();
    ///     let service_maker = move || CounterService(counter_clone.clone());
    ///     let distr = Distributed::start(service_maker).await;
    ///     
    ///     let futs = distr.map_all(|pss| pss.instance.inc());
    ///     join_all(futs).await;
    ///     distr.stop().await;
    ///     
    ///     assert_eq!(2 * get_count(), counter.load(Ordering::SeqCst));
    /// }
    /// ```
    pub fn map_all<'a, Func, Ret, Fut>(&'a self, func: Func) -> Vec<impl Future<Output = Ret>>
    where
        Func: FnOnce(PeeringShardedService<'a, S>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        self.map_selected(func, 0..get_count())
    }

    /// Applies a map function to all instances of the service, except the one on the current shard, and returns a vector of the results.
    ///
    /// Spiritually, a hybrid of `seastar::distributed::map` and `seastar::distributed::invoke_on_others`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    /// use futures::future::join_all;
    /// use seastar::{get_count, Distributed, Service};
    ///
    /// struct CounterService(Arc<AtomicU32>);
    ///
    /// impl CounterService {
    ///     async fn inc(&self) {
    ///         self.0.fetch_add(1, Ordering::SeqCst);
    ///     }
    /// }
    ///
    /// impl Service for CounterService {}
    ///
    /// #[seastar::test]
    /// async fn test_map_others() {
    ///     let counter: Arc<AtomicU32> = Default::default();
    ///     let counter_clone = counter.clone();
    ///     let service_maker = move || CounterService(counter_clone.clone());
    ///     let distr = Distributed::start(service_maker).await;
    ///     
    ///     let futs = distr.map_others(|pss| pss.instance.inc());
    ///     join_all(futs).await;
    ///     distr.stop().await;
    ///     
    ///     assert_eq!(2 * get_count() - 1, counter.load(Ordering::SeqCst));
    /// }
    /// ```
    pub fn map_others<'a, Func, Ret, Fut>(&'a self, func: Func) -> Vec<impl Future<Output = Ret>>
    where
        Func: FnOnce(PeeringShardedService<'a, S>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        let this_shard = this_shard_id();
        self.map_selected(func, (0..get_count()).filter(move |sh| sh.ne(&this_shard)))
    }

    /// Applies a map function only to the service instance on the provided shard.
    ///
    /// Spiritually, a hybrid of `seastar::distributed::map` and `seastar::distributed::invoke_on`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    /// use seastar::{get_count, Distributed, Service};
    ///
    /// struct CounterService(Arc<AtomicU32>);
    ///
    /// impl CounterService {
    ///     async fn inc(&self) {
    ///         self.0.fetch_add(1, Ordering::SeqCst);
    ///     }
    /// }
    ///
    /// impl Service for CounterService {}
    ///
    /// #[seastar::test]
    /// async fn test_map_single() {
    ///     let counter: Arc<AtomicU32> = Default::default();
    ///     let counter_clone = counter.clone();
    ///     let service_maker = move || CounterService(counter_clone.clone());
    ///     let distr = Distributed::start(service_maker).await;
    ///     
    ///     for shard in 0..get_count() {
    ///         distr.map_single(shard, |pss| pss.instance.inc()).await;
    ///         assert_eq!(shard + 1, counter.load(Ordering::SeqCst));
    ///     }
    ///     distr.stop().await;
    /// }
    /// ```
    pub fn map_single<'a, Func, Ret, Fut>(
        &'a self,
        shard_id: u32,
        func: Func,
    ) -> impl Future<Output = Ret>
    where
        Func: FnOnce(PeeringShardedService<'a, S>) -> Fut + Send + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        let container = unsafe { PtrWrapper::new(self as *const Distributed<S> as _) };
        self.submit_to(shard_id, func, container)
    }
}

#[cfg(test)]
mod tests {
    use futures::future::join_all;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    use super::*;
    use crate as seastar;

    struct CounterService(Arc<AtomicU32>);

    impl CounterService {
        async fn inc(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl Service for CounterService {
        fn stop(&self) -> Box<dyn Future<Output = ()>> {
            let counter = self.0.clone();
            Box::new(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        }
    }

    #[seastar::test]
    async fn test_start_single_and_stop() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start_single(service_maker).await;
        distr.stop().await;
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[seastar::test]
    async fn test_start_and_stop() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;
        distr.stop().await;
        assert_eq!(get_count(), counter.load(Ordering::SeqCst));
    }

    #[seastar::test]
    async fn test_map_all() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        let futs = distr.map_all(|pss| pss.instance.inc());
        join_all(futs).await;
        distr.stop().await;

        assert_eq!(2 * get_count(), counter.load(Ordering::SeqCst));
    }

    #[seastar::test]
    async fn test_map_others() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        let futs = distr.map_others(|pss| pss.instance.inc());
        join_all(futs).await;
        distr.stop().await;

        assert_eq!(2 * get_count() - 1, counter.load(Ordering::SeqCst));
    }

    #[seastar::test]
    async fn test_map_single() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        for shard in 0..get_count() {
            distr.map_single(shard, |pss| pss.instance.inc()).await;
            assert_eq!(shard + 1, counter.load(Ordering::SeqCst));
        }

        distr.stop().await;
    }

    #[seastar::test]
    async fn test_one_to_one_comm() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        let even_length = get_count() - get_count() % 2;
        let shards = (0..get_count()).take(even_length as usize);
        for shard in shards.filter(|s| s % 2 == 0) {
            distr
                .map_single(shard, move |pss| {
                    pss.container
                        .map_single(shard + 1, move |pss| pss.instance.inc())
                })
                .await;
            assert_eq!(shard / 2 + 1, counter.load(Ordering::SeqCst));
        }
        distr.stop().await;
    }

    #[seastar::test]
    async fn test_one_to_many_comm() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        distr
            .map_single(0, move |pss| async {
                let futs = pss.container.map_all(move |pss| pss.instance.inc());
                join_all(futs).await
            })
            .await;

        assert_eq!(get_count(), counter.load(Ordering::SeqCst));
        distr.stop().await;
    }

    #[seastar::test]
    async fn test_many_to_one_comm() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        let futs =
            distr.map_all(move |pss| pss.container.map_single(0, move |pss| pss.instance.inc()));
        join_all(futs).await;

        assert_eq!(get_count(), counter.load(Ordering::SeqCst));
        distr.stop().await;
    }

    #[seastar::test]
    async fn test_many_to_many_comm() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        let futs = distr.map_all(move |pss| async {
            let futs = pss.container.map_all(move |pss| pss.instance.inc());
            join_all(futs).await
        });
        join_all(futs).await;

        assert_eq!(get_count().pow(2), counter.load(Ordering::SeqCst));
        distr.stop().await;
    }
}
