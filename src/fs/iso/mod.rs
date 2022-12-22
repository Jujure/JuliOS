pub mod iso9660;
mod fd;

use crate::println;
use crate::drivers::atapi::{DRIVE};
use crate::fd::{FDId, FD_TABLE, FDt};
use crate::utils::unserialize;

use iso9660::{IsoPrimVolDesc};
use fd::IsoFD;

use alloc::sync::Arc;

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

pub async fn open() -> FDt {
    let fd = IsoFD::new();
    FD_TABLE.lock().await.register_fd(fd.clone());
    fd
}