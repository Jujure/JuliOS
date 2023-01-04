use crate::println;
use crate::proc::scheduler::SCHEDULER;

use super::SyscallContext;

pub async fn exit(context: &SyscallContext) {
    println!("Running exit(2)");
    let mut scheduler = SCHEDULER.lock().await;
    scheduler.exit(context.thread_id);
}
