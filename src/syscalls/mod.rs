use crate::proc::thread::{resume_k_thread, RUNNING_THREAD};
use crate::task::executor::EXECUTOR;
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

pub fn syscall_dispatcher(context: SyscallContextT) -> Task {
    match context.borrow().id {
        EXIT_ID => Task::new(proc::exit(context.clone())),
        _ => Task::new(bad_syscall()),
    }
}

pub fn syscall_routine(syscall_id: SyscallId) -> u64 {
    let context: SyscallContextT = Arc::new(RefCell::new(SyscallContext {
        id: syscall_id,
        res: 0,
        thread_id: *RUNNING_THREAD.try_lock().unwrap(),
    }));
    
    EXECUTOR
        .try_lock()
        .unwrap()
        .spawn(syscall_dispatcher(context.clone()));

    resume_k_thread();

    let res = context.borrow().res;
    res
}

async fn bad_syscall() {}
