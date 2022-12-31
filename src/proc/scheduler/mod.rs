use super::thread::{Thread, ThreadId};

use alloc::{collections::BTreeMap, sync::Arc};
use crossbeam_queue::ArrayQueue;

pub struct Scheduler {
    threads: BTreeMap<ThreadId, Arc<Thread>>,
    thread_queue: Arc<ArrayQueue<ThreadId>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            threads: BTreeMap::new(),
            thread_queue: Arc::new(ArrayQueue::new(100)),
        }
    }

    pub fn schedule(&mut self) -> Option<Arc<Thread>> {
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
}