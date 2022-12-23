pub mod iso9660;
mod fd;

use crate::println;
use crate::drivers::atapi::{DRIVE};
use crate::fd::{FD_TABLE, FDt};
use crate::utils::unserialize;

use super::FileSystem;
use iso9660::{IsoPrimVolDesc, IsoDir};
use fd::IsoFD;

use alloc::{sync::Arc, boxed::Box};
use async_trait::async_trait;

pub struct IsoFS {
}

#[async_trait(?Send)]
impl FileSystem for IsoFS {
    async fn open(path: &str, flags: u32) -> Option<FDt> {
        if flags != crate::syscalls::io::O_RDONLY {
            return None;
        }

        let voldesc = get_prim_vol_desc().await;

        if voldesc.std_identifier != "CD001".as_bytes() {
            return None;
        }

        let fd = IsoFD::new();
        FD_TABLE.lock().await.register_fd(fd.clone());
        Some(fd.clone())
    }
}

pub async fn get_prim_vol_desc() -> IsoPrimVolDesc {
    let desc_block = DRIVE
        .lock()
        .await
        .as_mut()
        .unwrap()
        .read_block(iso9660::ISO_PRIM_VOLDESC_BLOCK)
        .await;
    *unserialize::<IsoPrimVolDesc>(desc_block.as_ptr())
}