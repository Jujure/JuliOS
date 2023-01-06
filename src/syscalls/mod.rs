use crate::println;
use crate::proc::thread::{resume_k_thread, RUNNING_THREAD};
use crate::proc::scheduler::SCHEDULER;
use crate::task::yield_executor::YieldExecutor;
use crate::task::Task;

pub use ids::*;

use alloc::sync::Arc;
use core::cell::RefCell;

pub mod ids;
pub mod io;
pub mod proc;

pub type SyscallContextT = Arc<RefCell<SyscallContext>>;

pub struct SyscallContext {
    id: SyscallId,
    res: u64,
    thread_id: crate::proc::thread::ThreadId,
}

impl SyscallContext {
    pub async fn run(&mut self) {
        println!("Running async syscall runner for {:?}", self.id);
        self.dispatch().await;
    }

    pub async fn dispatch(&mut self) {
        match self.id {
            EXIT_ID => proc::exit(self).await,
            _ => bad_syscall().await,
        }
    }
}


async fn syscall_runner(context: SyscallContextT) {
    context.borrow_mut().run().await;
}

pub fn syscall_routine(syscall_id: SyscallId) -> u64 {
    println!("Running syscall interrupt handler");
    let context: SyscallContextT = Arc::new(RefCell::new(SyscallContext {
        id: syscall_id,
        res: 0,
        thread_id: *RUNNING_THREAD.try_lock().unwrap(),
    }));
    
    println!("Spawning async syscall runner");
    let mut executor = YieldExecutor::new(context.borrow().thread_id);
    executor.spawn(Task::new(syscall_runner(context.clone())));
    executor.run();

    let res = context.borrow().res;
    res
}

async fn bad_syscall() {}
