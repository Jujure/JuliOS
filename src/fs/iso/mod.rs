mod fd;
pub mod iso9660;

use crate::drivers::atapi::read_block;
use crate::fd::FDt;
use crate::utils::unserialize;

use super::FileSystem;
use fd::IsoFD;
use iso9660::{IsoDir, IsoPrimVolDesc};

use alloc::{boxed::Box, string::String, vec::Vec};
use async_trait::async_trait;

pub struct IsoFS {}

#[async_trait(?Send)]
#[allow(unaligned_references)]
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

        let root: &IsoDir = &voldesc.root_dir;
        let mut curr_entry_block: [u8; iso9660::ISO_BLOCK_SIZE as usize] =
            read_block(root.data_blk.le).await;

        let mut curr_entry: &IsoDir = unserialize(curr_entry_block.as_ptr());

        let path_s: String = String::from(path);
        let path_split: Vec<String> = path_s
            .split("/")
            .filter(|p| p != &"")
            .map(|s| s.to_uppercase())
            .collect();
        let path_it = path_split.iter();

        for path_component in path_it {
            let mut found = false;
            while curr_entry.idf_len != 0 {
                // Found entry
                if curr_entry.matches(path_component.as_str()) {
                    found = true;

                    // Not the last component, go 1 directory deeper
                    if path_component.as_str() != path_split[path_split.len() - 1] {
                        // Not a directory
                        if curr_entry.file_type != iso9660::IsoFileType::ISDIR {
                            return None;
                        }
                        // Deeper entries
                        curr_entry_block = read_block(curr_entry.data_blk.le).await;
                        curr_entry = unserialize(curr_entry_block.as_ptr());
                    }
                    break;
                }

                // Next entry
                curr_entry = curr_entry.next_entry();
            }

            // File not found
            if !found {
                return None;
            }
        }

        Some(IsoFD::new(curr_entry).await)
    }
}

pub async fn get_prim_vol_desc() -> IsoPrimVolDesc {
    let desc_block = read_block(iso9660::ISO_PRIM_VOLDESC_BLOCK).await;
    *unserialize::<IsoPrimVolDesc>(desc_block.as_ptr())
}
