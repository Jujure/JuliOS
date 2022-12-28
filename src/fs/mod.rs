pub mod iso;

use crate::fd::FDt;
use crate::utils::mutex::AsyncMutex;

use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use core::cell::RefCell;
use lazy_static::lazy_static;
use prefix_tree_map::{PrefixTreeMap, PrefixTreeMapBuilder};

pub type FSt = Arc<RefCell<dyn FileSystem>>;

lazy_static! {
    pub static ref VIRTUAL_FS: AsyncMutex<VirtualFS> = AsyncMutex::new(VirtualFS::new());
}

#[async_trait(?Send)]
pub trait FileSystem {
    async fn open(&mut self, path: &str, flags: u32) -> Option<FDt>;
}

pub struct VirtualFS {
    map_builder: PrefixTreeMapBuilder<String, String, FSt>,
    map: Option<PrefixTreeMap<String, String, FSt>>,
}

impl VirtualFS {
    fn new() -> Self {
        let mut res = VirtualFS {
            map_builder: PrefixTreeMapBuilder::new(),
            map: None,
        };
        let fs = Arc::new(RefCell::new(iso::IsoFS {}));
        let fs2 = Arc::new(RefCell::new(iso::IsoFS {}));
        res.mount("/", fs);
        res.mount("/mnt/iso", fs2);
        res
    }

    fn mount(&mut self, path: &str, fs: FSt) {
        let path_s: String = String::from(path);
        self.map_builder.insert_exact(
            path_s
                .split("/")
                .filter(|p| p != &"")
                .map(|s| String::from(s)),
            fs,
        );
        self.map = Some(self.map_builder.clone().build());
    }
}

#[async_trait(?Send)]
impl FileSystem for VirtualFS {
    async fn open(&mut self, path: &str, flags: u32) -> Option<FDt> {
        if let Some(map) = &self.map {
            let mut mnt_relative_path: String = String::from("");
            let path_s: String = String::from(path);
            let mut path_split: Vec<String> = path_s
                .split("/")
                .filter(|p| p != &"")
                .map(|s| String::from(s))
                .collect();
            loop {
                if let Some(fs) = map.find_exact(&path_split) {
                    // TODO, remove path prefix of the mount point
                    return fs.borrow_mut().open(mnt_relative_path.as_str(), flags).await;
                }
                else {
                    let component = path_split.remove(path_split.len() - 1);
                    mnt_relative_path = String::from("/") + component.as_str() + mnt_relative_path.as_str();
                }
            }
        } else {
            None
        }
    }
}
