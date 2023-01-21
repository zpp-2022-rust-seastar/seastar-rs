use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct LocalBoxFuture<F>(pub Pin<Box<F>>)
where
    F: Future + 'static;

impl<F> LocalBoxFuture<F>
where
    F: Future + 'static,
{
    pub fn new(fut: F) -> Self {
        Self(Box::pin(fut))
    }
}

impl<F> Future for LocalBoxFuture<F>
where
    F: Future + 'static,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

unsafe impl<F> Send for LocalBoxFuture<F> where F: Future + 'static {}

unsafe impl<F> Sync for LocalBoxFuture<F> where F: Future + 'static {}
