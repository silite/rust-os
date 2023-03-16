use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub mod simple_executor;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        // Since the poll method of the Future trait expects to be called on a Pin<&mut T> type,
        // use the Pin::as_mut method to convert the self.future field of type Pin<Box<T>> first
        let pin_mut_future = self.future.as_mut();
        pin_mut_future.poll(context)
    }
}
