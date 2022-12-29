use super::thread::Thread;

use alloc::vec::Vec;

pub struct Scheduler {
    threads: Vec<Thread>,
}