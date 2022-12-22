use crate::println;
use crate::fd::{FDId, FileDescriptor, FDt};

use alloc::sync::Arc;
use core::cell::RefCell;

pub struct IsoFD {
    pub fd: FDId,
}

impl IsoFD {
    pub fn new() -> FDt {
        Arc::new(RefCell::new(IsoFD {
            fd: FDId::new(),
        }))
    }
}

impl FileDescriptor for IsoFD {
    fn write(&mut self, buf: *const u8, count: usize) -> isize {
        0
    }

    fn read(&mut self, buf: *mut u8, count: usize) -> isize {
        println!("Read from fd");
        0
    }
}