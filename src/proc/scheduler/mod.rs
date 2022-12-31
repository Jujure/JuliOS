use crate::utils::mutex::AsyncMutex;

use super::thread::{Thread, ThreadId};

use alloc::{collections::BTreeMap, sync::Arc};
use core::cell::RefCell;
use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCHEDULER: AsyncMutex<Scheduler> = AsyncMutex::new(Scheduler::new());
}

pub type Threadt = Arc<RefCell<Thread>>;

pub struct Scheduler {
    pub threads: BTreeMap<ThreadId, Threadt>,
    thread_queue: Arc<ArrayQueue<ThreadId>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let mut res = Scheduler {
            threads: BTreeMap::new(),
            thread_queue: Arc::new(ArrayQueue::new(100)),
        };
        let k_thread: Thread = Thread {
            id: ThreadId(0),
            entry_point: 0,
            started: true,
            rsp: 0,
        };
        res.register(Arc::new(RefCell::new(k_thread)));
        res
    }

    pub fn schedule(&mut self) -> Option<Threadt> {
        if let Ok(thread_id) = self.thread_queue.pop() {
            self.thread_queue.push(thread_id);
            let thread = match self.threads.get_mut(&thread_id) {
                Some(thread) => thread,
                None => return None,
            };
            Some(thread.clone())
        } else {
            None
        }
    }

    pub fn register(&mut self, thread: Threadt) {
        let thread_id = thread.borrow().id;
        if self.threads.insert(thread_id, thread).is_some() {
            panic!("Duplicate thread ID")
        }
        self.thread_queue
            .push(thread_id)
            .expect("Thread queue full");
    }
}
