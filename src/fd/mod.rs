use crate::println;
use crate::utils::AsyncMutex;

use alloc::{collections::BTreeMap, sync::Arc, boxed::Box};
use async_trait::async_trait;
use core::cell::RefCell;
use lazy_static::lazy_static;

pub type FDt = Arc<RefCell<dyn FileDescriptor>>;

lazy_static! {
    pub static ref FD_TABLE: AsyncMutex<FDTable> = {
        AsyncMutex::new(FDTable::new())
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FDId(u64);

impl FDId {
    pub fn new() -> Self {
        // TODO: search for first available fd
        FDId(1)
    }
}

pub struct FDTable {
    table: BTreeMap<FDId, FDt>,
}

impl FDTable {
    pub fn new() -> Self {
        FDTable { table: BTreeMap::new() }
    }

    pub fn register_fd(&mut self, fd: FDt) {
        // TODO
        self.table.insert(fd.borrow().get_fd(), fd.clone());
        println!("Registered fd: {:?}", self.table.get(&FDId(1)).unwrap().borrow().get_fd());
    }
}

#[async_trait]
pub trait FileDescriptor {
    fn get_fd(&self) -> FDId;
    async fn write(&mut self, buf: &[u8], count: usize) -> isize;
    async fn read(&mut self, buf: &[u8], count: usize) -> isize;
}