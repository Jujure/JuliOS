use alloc::alloc::{alloc, dealloc, Layout};

const STACK_SIZE: usize = 4096 * 20;

pub struct Thread {
    rsp: u64
}

impl Thread {
    pub fn new() -> Self {
        unsafe {
            Thread {
                rsp: alloc(Layout::new::<[u8; STACK_SIZE]>()) as u64,
            }
        }
    }
}