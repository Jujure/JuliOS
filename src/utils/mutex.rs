use core::future::Future;
use core::pin::Pin;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use alloc::sync::Arc;

use futures_util::task::AtomicWaker;

#[derive(Clone)]
struct Lock {
    lock: Arc<AtomicBool>,
    waker: Arc<AtomicWaker>,
}

pub struct AsyncMutex<T> {
    lock: Lock,
    inner: UnsafeCell<T>,
}

pub struct AsyncMutexGuard<'a, T>
where
    T: 'a,
{
    mutex: &'a AsyncMutex<T>
}

impl Lock {
    fn new() -> Self {
        Lock {
            lock: Arc::new(AtomicBool::new(false)),
            waker: Arc::new(AtomicWaker::new()),
        }
    }

    fn try_lock(&self) -> bool {
        self.lock.swap(true, Ordering::Acquire)
    }

    fn drop(&self) {
        self.lock.swap(false, Ordering::Release);
        self.waker.wake();
    }
}

impl Future for Lock {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.try_lock() {
            return Poll::Ready(());
        }

        self.waker.register(&cx.waker());

        match self.try_lock() {
            true => {
                self.waker.take();
                Poll::Ready(())
            },
            false => Poll::Pending,
        }
    }
}

impl<T> AsyncMutex<T> {
    pub fn new(val: T) -> Self {
        AsyncMutex {
            lock: Lock::new(),
            inner: UnsafeCell::new(val),
        }
    }

    pub fn try_lock(&self) -> Option<AsyncMutexGuard<T>> {
        if self.lock.try_lock() {
            Some(AsyncMutexGuard { mutex: self })
        } else {
            None
        }
    }

    pub async fn lock(&self) -> AsyncMutexGuard<'_, T> {
        self.lock.clone().await;
        AsyncMutexGuard { mutex: self }
    }
}

unsafe impl<T> Send for AsyncMutex<T> {}
unsafe impl<T> Sync for AsyncMutex<T> {}

impl<T> Drop for AsyncMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.lock.drop();
    }
}

impl<T> Deref for AsyncMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.mutex.inner.get()
        }
    }
}

impl<T> DerefMut for AsyncMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.mutex.inner.get()
        }
    }
}