use crate::{
    cxx_async_local_future::IntoCxxAsyncLocalFuture,
    ffi_utils::{get_dropper_const, get_dropper_noarg, get_fn_caller, PtrWrapper},
    get_count, spawn,
    submit_to::submit_to,
    this_shard_id,
};
use core::marker::PhantomData;
use cxx::SharedPtr;
use std::pin::Pin;
use std::{
    future::Future,
    sync::{Arc, RwLock},
};

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
    pub container: &'a mut Distributed<S>,
}

pub struct PeeringShardedServiceMut<'a, S>
where
    S: Service,
{
    pub instance: &'a mut S,
    pub container: &'a mut Distributed<S>,
}

/// A service distributed amongst all shards of a Seastar app.
///
/// You can use this to, for example, load balance a local storage.
/// One/many-to-one/many communication between instances of the service can be achieved
/// via the use of `PeeringShardedService` or its `mut` equivalent, `PeeringShardedService`.
///
/// # Panics
///
/// One can only delegate a "mutating map" on an instance if all other "mutating maps" on it have ceased.
/// In other words, you must ensure sequential acquisition of mutable borrows to an instance.
/// Failing to comply with this rule will lead to a `panic!`, much in the same way that breaking the rule for
/// a `RefCell` would do.
pub struct Distributed<S: Service> {
    _inner: SharedPtr<distributed>,
    _ty: PhantomData<S>,
    /// These allow only exclusive writes or shared reads to specific instances of the service.
    /// Don't fret - no thread trying to map over an instance will be hanged on any of these locks.
    /// They're not used for (blockingly) locking, but merely try-locking, which if failed will yield a panic.
    /// Comply with the `Distributed`'s ownership contract and all will be well.
    _locks: Vec<Arc<RwLock<()>>>,
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
                    _locks: vec![Default::default(); get_count() as usize],
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
        let lock = self._locks[shard_id as usize].clone();
        submit_to(shard_id, move || async move {
            let lock = lock.try_read();
            if lock.is_err() {
                panic!("instance {} already mutably borrowed", shard_id);
            }

            let instance = ffi::local(distr.as_ref().unwrap());
            let instance: &S = unsafe { &*(instance as *const S) };
            let _ = &container; // this is to avoid a partial move of the pointer
            let container = unsafe { &mut *(container.as_ptr_mut() as *mut Distributed<S>) };
            let pss = PeeringShardedService {
                instance,
                container,
            };
            func(pss).await
        })
    }

    fn submit_to_mut<'a, Func, Fut, Ret>(
        &'a self,
        shard_id: u32,
        func: Func,
        container: PtrWrapper,
    ) -> impl Future<Output = Ret>
    where
        Func: FnOnce(PeeringShardedServiceMut<'a, S>) -> Fut + Send + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        // Taking `self` by immutable reference here is intentional.
        // It enables us to delegate mutating maps over more than one instance at a time,
        // for example in a loop or an iterator. Thus, this serves the purpose of having
        // multiple mutable borrows to the `Distributed`'s data at the same time,
        // but each to a separate part of it, much like in this example:
        // https://doc.rust-lang.org/nomicon/borrow-splitting.html
        crate::assert_runtime_is_running();

        let distr = self._inner.clone();
        let lock = self._locks[shard_id as usize].clone();
        submit_to(shard_id, move || async move {
            let lock = lock.try_read();
            if lock.is_err() {
                panic!("instance {} already borrowed", shard_id);
            }

            let instance = ffi::local(distr.as_ref().unwrap());
            let instance = unsafe { &mut *(instance as *mut S) };
            let _ = &container; // this is to avoid a partial move of the pointer
            let container = unsafe { &mut *(container.as_ptr_mut() as *mut Distributed<S>) };
            let pss = PeeringShardedServiceMut {
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

        let mut res = vec![];
        for shard in shards.into_iter() {
            res.push(self.map_single(shard, func.clone()));
        }
        res
    }

    fn map_selected_mut<'a, Func, Fut, Ret, I>(
        &'a mut self,
        func: Func,
        shards: I,
    ) -> Vec<impl Future<Output = Ret>>
    where
        Func: FnOnce(PeeringShardedServiceMut<'a, S>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
        I: IntoIterator<Item = u32>,
    {
        crate::assert_runtime_is_running();

        let mut res = vec![];
        for shard in shards.into_iter() {
            let container = unsafe { PtrWrapper::new(self as *const Distributed<S> as _) };
            res.push(self.submit_to_mut(shard, func.clone(), container));
        }
        res
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

    /// Applies a mutating map function to all instances of the service and returns a vector of the results.
    ///
    /// Operates like `map_all` but mutates data along the way.
    pub fn map_all_mut<'a, Func, Ret, Fut>(
        &'a mut self,
        func: Func,
    ) -> Vec<impl Future<Output = Ret>>
    where
        Func: FnOnce(PeeringShardedServiceMut<'a, S>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        self.map_selected_mut(func, 0..get_count())
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

    /// Applies a map function to all instances of the service, except the one on the current shard, and returns a vector of the results.
    ///
    /// Operates like `map_others` but mutates data along the way.
    pub fn map_others_mut<'a, Func, Ret, Fut>(
        &'a mut self,
        func: Func,
    ) -> Vec<impl Future<Output = Ret>>
    where
        Func: FnOnce(PeeringShardedServiceMut<'a, S>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        let this_shard = this_shard_id();
        self.map_selected_mut(func, (0..get_count()).filter(move |sh| sh.ne(&this_shard)))
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

    /// Applies a map function only to the service instance on the provided shard.
    ///
    /// Operates like `map_single` but mutates data along the way.
    pub fn map_single_mut<'a, Func, Ret, Fut>(
        &'a mut self,
        shard_id: u32,
        func: Func,
    ) -> impl Future<Output = Ret>
    where
        Func: FnOnce(PeeringShardedServiceMut<'a, S>) -> Fut + Send + 'static,
        Fut: Future<Output = Ret>,
        Ret: Send + 'static,
    {
        let container = unsafe { PtrWrapper::new(self as *const Distributed<S> as _) };
        self.submit_to_mut(shard_id, func, container)
    }

    /// Like `map_single` but for the current shard.
    ///
    /// You can still use `map_single` to achieve the same,
    /// but then your function has to be `Send` for no reason.
    pub fn map_current<'a, Func, Ret, Fut>(&'a self, func: Func) -> impl Future<Output = Ret>
    where
        Func: FnOnce(PeeringShardedService<'a, S>) -> Fut + 'static,
        Fut: Future<Output = Ret>,
        Ret: 'static,
    {
        crate::assert_runtime_is_running();

        let distr = self._inner.clone();
        let container = unsafe { PtrWrapper::new(self as *const Distributed<S> as _) };
        let lock = self._locks[this_shard_id() as usize].clone();
        spawn(async move {
            let lock = lock.try_read();
            if lock.is_err() {
                panic!("instance {} already mutably borrowed", this_shard_id());
            }
            let instance = ffi::local(distr.as_ref().unwrap());
            let instance: &S = unsafe { &*(instance as *const S) };
            let _ = &container; // this is to avoid a partial move of the pointer
            let container = unsafe { &mut *(container.as_ptr_mut() as *mut Distributed<S>) };
            let pss = PeeringShardedService {
                instance,
                container,
            };
            func(pss).await
        })
    }

    /// Like `map_current` but modifies data along the way.
    ///
    pub fn map_current_mut<'a, Func, Ret, Fut>(
        &'a mut self,
        func: Func,
    ) -> impl Future<Output = Ret>
    where
        Func: FnOnce(PeeringShardedServiceMut<'a, S>) -> Fut + 'static,
        Fut: Future<Output = Ret>,
        Ret: 'static,
    {
        crate::assert_runtime_is_running();

        let distr = self._inner.clone();
        let container = unsafe { PtrWrapper::new(self as *const Distributed<S> as _) };
        let lock = self._locks[this_shard_id() as usize].clone();
        spawn(async move {
            let lock = lock.try_read();
            if lock.is_err() {
                panic!("instance {} already borrowed", this_shard_id());
            }

            let instance = ffi::local(distr.as_ref().unwrap());
            let instance = unsafe { &mut *(instance as *mut S) };
            let _ = &container; // this is to avoid a partial move of the pointer
            let container = unsafe { &mut *(container.as_ptr_mut() as *mut Distributed<S>) };
            let pss = PeeringShardedServiceMut {
                instance,
                container,
            };
            func(pss).await
        })
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

    struct BoolService(bool);

    impl BoolService {
        async fn set(&mut self) {
            self.0 = true;
        }
        async fn get(&self) -> bool {
            self.0
        }
    }

    impl Service for BoolService {}

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
    async fn test_map_all_mut() {
        let service_maker = move || BoolService(false);
        let mut distr = Distributed::start(service_maker).await;

        let futs = distr.map_all_mut(|pss| pss.instance.set());
        join_all(futs).await;

        let futs = distr.map_all(|pss| pss.instance.get());
        let count = join_all(futs)
            .await
            .into_iter()
            .filter(|x| x.eq(&false))
            .count();

        assert_eq!(count, 0);
        distr.stop().await;
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
    async fn test_map_others_mut() {
        let service_maker = move || BoolService(false);
        let mut distr = Distributed::start(service_maker).await;

        let futs = distr.map_others_mut(|pss| pss.instance.set());
        join_all(futs).await;

        let futs = distr.map_others(|pss| pss.instance.get());
        let count = join_all(futs)
            .await
            .into_iter()
            .filter(|x| x.eq(&false))
            .count();

        assert_eq!(count, 0);
        distr.stop().await;
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
    async fn test_map_current() {
        let counter: Arc<AtomicU32> = Default::default();
        let counter_clone = counter.clone();
        let service_maker = move || CounterService(counter_clone.clone());
        let distr = Distributed::start(service_maker).await;

        distr.map_current(|pss| pss.instance.inc()).await;
        assert_eq!(1, counter.load(Ordering::SeqCst));
        distr.stop().await;
    }

    #[seastar::test]
    async fn test_map_current_mut() {
        let service_maker = move || BoolService(false);
        let mut distr = Distributed::start(service_maker).await;

        distr.map_current_mut(|pss| pss.instance.set()).await;
        let res = distr.map_current(|pss| pss.instance.get()).await;
        assert_eq!(true, res);
        distr.stop().await;
    }

    #[seastar::test]
    async fn test_map_single_mut() {
        let service_maker = move || BoolService(false);
        let mut distr = Distributed::start(service_maker).await;

        for shard in 0..get_count() {
            distr.map_single_mut(shard, |pss| pss.instance.set()).await;
            let res = distr.map_single(shard, |pss| pss.instance.get()).await;
            assert_eq!(res, true);
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
