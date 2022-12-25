pub mod iso;

use crate::fd::FDt;
use crate::utils::mutex::AsyncMutex;

use alloc::{boxed::Box, sync::Arc};
use async_trait::async_trait;
use core::cell::RefCell;
use lazy_static::lazy_static;

pub type FSt = Arc<RefCell<dyn FileSystem>>;

lazy_static! {
    pub static ref VIRTUAL_FS: AsyncMutex<VirtualFS> = AsyncMutex::new(VirtualFS::new());
}

#[async_trait(?Send)]
pub trait FileSystem {
    async fn open(&mut self, path: &str, flags: u32) -> Option<FDt>;
    async fn close(&mut self, fd: FDt);
}

pub struct VirtualFS {
    fs: FSt,
}

impl VirtualFS {
    fn new() -> Self {
        VirtualFS {
            fs: Arc::new(RefCell::new(iso::IsoFS {})),
        }
    }
}

#[async_trait(?Send)]
impl FileSystem for VirtualFS {
    async fn open(&mut self, path: &str, flags: u32) -> Option<FDt> {
        self.fs.borrow_mut().open(path, flags).await
    }
    
    async fn close(&mut self, fd: FDt) {
        self.fs.borrow_mut().close(fd).await
    }
}
