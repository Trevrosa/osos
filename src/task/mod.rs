use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(future)
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
