pub mod iso9660;

use crate::println;
use crate::drivers::atapi::{DRIVE};
use crate::utils::unserialize;
use iso9660::{IsoPrimVolDesc};

pub async fn init_prim_vol_desc() {
    let mut guard = DRIVE
        .lock()
        .await;
    let desc_block = guard.as_mut()
        .unwrap()
        .read_block(iso9660::ISO_PRIM_VOLDESC_BLOCK)
        .await;
    let prim_vol_desc: &IsoPrimVolDesc = unserialize::<IsoPrimVolDesc>(desc_block.as_ptr());

    println!("{:?}", alloc::string::String::from_utf8_lossy(&prim_vol_desc.std_identifier));
}