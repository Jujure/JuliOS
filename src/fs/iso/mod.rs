mod fd;
pub mod iso9660;

use crate::drivers::atapi::DRIVE;
use crate::fd::{FDt, FD_TABLE};
use crate::println;
use crate::serial_println;
use crate::utils::unserialize;

use super::FileSystem;
use fd::IsoFD;
use iso9660::{IsoDir, IsoPrimVolDesc};

use alloc::{boxed::Box, sync::Arc, string::String, vec::Vec};
use async_trait::async_trait;

pub struct IsoFS {}

#[async_trait(?Send)]
impl FileSystem for IsoFS {
    async fn open(&mut self, path: &str, flags: u32) -> Option<FDt> {
        // ISO is a read only file system
        if flags != crate::syscalls::io::O_RDONLY {
            return None;
        }

        let voldesc = get_prim_vol_desc().await;

        // Invalid ISO
        if voldesc.std_identifier != "CD001".as_bytes() {
            return None;
        }

        let root: IsoDir = voldesc.root_dir;
        let mut curr_entry_block: [u8; iso9660::ISO_BLOCK_SIZE] = DRIVE
            .lock()
            .await
            .as_mut()
            .unwrap()
            .read_block(root.data_blk.le)
            .await;

        let mut curr_entry: &IsoDir = unserialize(curr_entry_block.as_ptr());

        let path_s: String = String::from(path);
        let path_it = path_s
            .split("/")
            .filter(|p| p != &"");

        for path_component in path_it {
            while curr_entry.idf_len != 0 {
                serial_println!("{:?}", curr_entry.idf_len);
                serial_println!("{:?}", alloc::str::from_utf8(curr_entry.get_idf()).unwrap());

                if curr_entry.get_idf() == path_component.as_bytes() {
                    serial_println!("Found {}", path_component);
                    curr_entry_block = DRIVE
                        .lock()
                        .await
                        .as_mut()
                        .unwrap()
                        .read_block(curr_entry.data_blk.le)
                        .await;
                    curr_entry = unserialize(curr_entry_block.as_ptr());
                    break;
                }

                // Next entry
                unsafe {
                    let curr_ptr: *const IsoDir = curr_entry;
                    curr_entry = &*curr_ptr.cast::<u8>().offset(curr_entry.dir_size as isize).cast::<IsoDir>();
                }
            }
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
