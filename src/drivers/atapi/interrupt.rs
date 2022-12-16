
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};

use lazy_static::lazy_static;
use futures_util::task::AtomicWaker;

lazy_static! {
    pub static ref INTERRUPT_FUTURE: InterruptFuture = InterruptFuture::new();

    static ref INTERRUPT: AtomicBool = AtomicBool::new(false);
}

static WAKER: AtomicWaker = AtomicWaker::new();


pub(crate) fn mark_interrupt() {
    INTERRUPT.store(true, Ordering::Relaxed);
    WAKER.wake();
}

#[derive(Copy,Clone)]
pub struct InterruptFuture {
    _private:(),
}

impl InterruptFuture {
    pub fn new() -> Self {
        InterruptFuture { _private: () }
    }

    pub fn pop(&self) -> bool {
        let res = INTERRUPT.load(Ordering::Relaxed);
        INTERRUPT.store(false, Ordering::Relaxed);
        res
    }
}

impl Future for InterruptFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.pop() {
            return Poll::Ready(());
        }

        WAKER.register(&cx.waker());

        match self.pop() {
            true => {
                WAKER.take();
                Poll::Ready(())
            },
            false => Poll::Pending,
        }
    }
}