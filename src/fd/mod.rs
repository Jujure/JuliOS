use crate::println;
use crate::utils::AsyncMutex;

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use async_trait::async_trait;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use lazy_static::lazy_static;

pub type FDt = Arc<RefCell<dyn FileDescriptor>>;

lazy_static! {
    pub static ref FD_TABLE: AsyncMutex<FDTable> = AsyncMutex::new(FDTable::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FDId(u32);

impl FDId {
    pub fn new() -> Self {
        // TODO: search for first available fd
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        FDId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct FDTable {
    table: BTreeMap<FDId, FDt>,
}

impl FDTable {
    pub fn new() -> Self {
        FDTable {
            table: BTreeMap::new(),
        }
    }

    pub fn unregister_fd(&mut self, fd: FDt) {
        self.table.remove(&fd.borrow().get_fd());
        println!(
            "Unregistered fd: {:?}",
            fd.borrow().get_fd()
        );
    }

    pub fn register_fd(&mut self, fd: FDt) {
        self.table.insert(fd.borrow().get_fd(), fd.clone());
        println!(
            "Registered fd: {:?}",
            self.table.get(&FDId(0)).unwrap().borrow().get_fd()
        );
    }
}

#[async_trait]
pub trait FileDescriptor {
    fn get_fd(&self) -> FDId;
    async fn write(&mut self, buf: &[u8], count: usize) -> isize;
    async fn read(&mut self, buf: &mut [u8], count: usize) -> isize;
}
