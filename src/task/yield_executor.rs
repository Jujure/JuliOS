
use crate::println;
use crate::proc::thread::{ThreadId, resume_k_thread};
use crate::proc::scheduler::SCHEDULER;

use super::{Task, TaskId};

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

pub struct YieldExecutor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
    thread_id: ThreadId,
}

impl YieldExecutor {
    pub fn new(thread_id: ThreadId) -> Self {
        YieldExecutor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
            thread_id: thread_id,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_ready_tasks();
            if self.done() {
                break;
            } else {
                println!("Blocking thread");
                SCHEDULER
                    .try_lock()
                    .unwrap()
                    .block(self.thread_id);
                println!("Returning to scheduler");
                resume_k_thread();
            }
        }
    }

    fn done(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("Duplicate task ID");
        }
        self.task_queue.push(task_id).expect("Task queue full");
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
            thread_id,
        } = self; // Executor destructuring

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // Task does not exist anymore
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| YieldTaskWaker::new(task_id, task_queue.clone(), *thread_id));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task is done
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}

struct YieldTaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
    thread_id: ThreadId,
}

impl YieldTaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>, thread_id: ThreadId) -> Waker {
        Waker::from(Arc::new(YieldTaskWaker {
            task_id,
            task_queue,
            thread_id
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("Task queue full");
        SCHEDULER
            .try_lock()
            .unwrap()
            .unblock(self.thread_id);
    }
}

impl Wake for YieldTaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
