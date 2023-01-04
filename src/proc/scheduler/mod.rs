use crate::utils::mutex::AsyncMutex;

use super::thread::{Thread, ThreadId};

use alloc::{collections::BTreeMap, sync::Arc};
use core::cell::RefCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCHEDULER: AsyncMutex<Scheduler> = AsyncMutex::new(Scheduler::new());
}

pub type Threadt = Arc<RefCell<Thread>>;

pub const K_THREAD_ID: ThreadId = ThreadId(0); // Kernel main thread identifier

struct ThreadStream {
    ids: ArrayQueue<ThreadId>,
    waker: AtomicWaker,
}

impl ThreadStream {
    pub fn new() -> Self {
        ThreadStream {
            ids: ArrayQueue::new(100),
            waker: AtomicWaker::new(),
        }
    }

    pub fn register(&mut self, id: ThreadId) {
        self.ids.push(id).expect("Thread queue full");
        self.waker.wake();
    }
}

impl Stream for ThreadStream {
    type Item = ThreadId;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if let Ok(id) = self.ids.pop() {
            return Poll::Ready(Some(id));
        }

        self.waker.register(&cx.waker());

        match self.ids.pop() {
            Ok(id) => {
                self.waker.take();
                Poll::Ready(Some(id))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub struct Scheduler {
    pub threads: BTreeMap<ThreadId, Threadt>,
    thread_queue: ThreadStream,
}

impl Scheduler {
    pub fn new() -> Self {
        let mut res = Scheduler {
            threads: BTreeMap::new(),
            thread_queue: ThreadStream::new(),
        };
        let k_thread: Thread = Thread {
            id: K_THREAD_ID,
            entry_point: 0,
            started: true,
            rsp: 0,
            base_stack: 0,
        };
        res.register(Arc::new(RefCell::new(k_thread)));
        res
    }

    pub async fn run(&mut self) {
        while let Some(id) = self.thread_queue.next().await {
            if let Some(thread) = self.get_thread(id) {
                // Thread still exists
                unsafe {
                    (&mut *thread.as_ptr()).run();
                }
                self.thread_queue.register(id);
            }
        }
    }

    pub fn register(&mut self, thread: Threadt) {
        let thread_id = thread.borrow().id;
        if self.threads.insert(thread_id, thread).is_some() {
            panic!("Duplicate thread ID")
        }
        if thread_id != K_THREAD_ID {
            self.thread_queue.register(thread_id);
        }
    }

    pub fn exit(&mut self, id: ThreadId) {
        self.threads.remove(&id).unwrap().borrow().exit();
    }

    pub fn get_thread(&mut self, id: ThreadId) -> Option<Threadt> {
        if let Some(thread) = self.threads.get_mut(&id) {
            Some(thread.clone())
        } else {
            None
        }
    }
}

pub async fn scheduler_run() {
    let mut scheduler = SCHEDULER.lock().await;
    SCHEDULER.force_unlock();
    scheduler.run().await;
}
