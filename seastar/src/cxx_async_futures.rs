use std::future::Future;

#[cxx_async::bridge(namespace = seastar_ffi)]
unsafe impl Future for VoidFuture {
    type Output = ();
}

#[cxx_async::bridge(namespace = seastar_ffi)]
unsafe impl Future for IntFuture {
    type Output = i32;
}
