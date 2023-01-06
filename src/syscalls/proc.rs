use crate::println;
use crate::proc::scheduler::SCHEDULER;
use crate::proc::thread::resume_k_thread;

use super::SyscallContext;

pub async fn exit(context: &SyscallContext) {
    println!("Running exit(2)");
    SCHEDULER.lock().await.exit(context.thread_id);
    resume_k_thread();
}
