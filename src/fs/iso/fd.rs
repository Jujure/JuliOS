use crate::fd::{FDId, FDt, FileDescriptor};
use crate::println;
use crate::utils::mutex::AsyncMutex;

use alloc::{boxed::Box, sync::Arc};
use async_trait::async_trait;
use core::cell::RefCell;

pub struct IsoFD {
    pub fd: FDId,
}

impl IsoFD {
    pub fn new() -> FDt {
        Arc::new(RefCell::new(IsoFD { fd: FDId::new() }))
    }
}

#[async_trait]
impl FileDescriptor for IsoFD {
    fn get_fd(&self) -> FDId {
        self.fd
    }

    async fn write(&mut self, buf: &[u8], count: usize) -> isize {
        0
    }

    async fn read(&mut self, buf: &[u8], count: usize) -> isize {
        println!("Read from fd");
        0
    }
}
