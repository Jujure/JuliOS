use crate::println;
use crate::fd::{FDId, FileDescriptor, FDt};

use alloc::{sync::Arc, boxed::Box};
use async_trait::async_trait;
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

#[async_trait]
impl FileDescriptor for IsoFD {
    async fn write(&mut self, buf: &[u8], count: usize) -> isize {
        0
    }

    async fn read(&mut self, buf: &[u8], count: usize) -> isize {
        println!("Read from fd");
        0
    }
}