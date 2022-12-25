use crate::drivers::atapi::read_block;
use crate::fd::{FDId, FDt, FileDescriptor, FD_TABLE};

use super::iso9660::{IsoDir, ISO_BLOCK_SIZE};

use alloc::{boxed::Box, sync::Arc};
use async_trait::async_trait;
use core::cell::RefCell;

pub struct IsoFD {
    pub fd: FDId,
    offset: u32,
    lba: u32,
    size: u32
}

impl IsoFD {
    pub async fn new(entry: &IsoDir) -> FDt {
        let fd = Arc::new(RefCell::new(IsoFD {
            fd: FDId::new(),
            offset: 0,
            lba: entry.data_blk.le,
            size: entry.file_size.le,
        }));

        FD_TABLE.lock().await.register_fd(fd.clone());
        fd
    }
}

#[async_trait]
impl FileDescriptor for IsoFD {
    fn get_fd(&self) -> FDId {
        self.fd
    }

    async fn write(&mut self, _buf: &[u8], _count: usize) -> isize {
        -1
    }

    #[allow(unaligned_references)]
    async fn read(&mut self, buf: &mut [u8], count: usize) -> isize {
        let mut block_offset = self.offset / ISO_BLOCK_SIZE;
        let mut content = read_block(self.lba + block_offset).await;
        let mut read: isize = 0;
        for _ in 0..count {
            if self.offset >= self.size {
                break;
            }

            buf[read as usize] = content[(self.offset % ISO_BLOCK_SIZE) as usize];
            read += 1;
            self.offset += 1;

            if self.offset % ISO_BLOCK_SIZE == 0 {
                block_offset += 1;
                content = read_block(self.lba + block_offset).await;
            }
        }

        read
    }
}
