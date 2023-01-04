use crate::println;
use crate::proc::scheduler::SCHEDULER;

use super::SyscallContextT;

pub async fn exit(context: SyscallContextT) {
    println!("Exiting thread");
    let mut scheduler = SCHEDULER.lock().await;
    scheduler.exit(context.borrow().thread_id);
}
