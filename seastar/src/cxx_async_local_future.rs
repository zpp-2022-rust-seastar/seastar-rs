use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use cxx_async::{CxxAsyncResult, IntoCxxAsyncFuture};
use pin_project::pin_project;

#[pin_project]
struct LocalFuture<F>(#[pin] F);

impl<F> Future for LocalFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().0.poll(cx)
    }
}

unsafe impl<F> Send for LocalFuture<F> {}

pub(crate) trait IntoCxxAsyncLocalFuture: IntoCxxAsyncFuture {
    fn infallible_local<Fut>(future: Fut) -> Self
    where
        Fut: Future<Output = Self::Output> + 'static,
    {
        Self::infallible(LocalFuture(future))
    }

    fn fallible_local<Fut>(future: Fut) -> Self
    where
        Fut: Future<Output = CxxAsyncResult<Self::Output>> + 'static,
    {
        Self::fallible(LocalFuture(future))
    }
}

impl<F> IntoCxxAsyncLocalFuture for F where F: IntoCxxAsyncFuture {}
