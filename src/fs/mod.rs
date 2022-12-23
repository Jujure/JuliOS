pub mod iso;

use crate::fd::FDt;

use alloc::{sync::Arc, boxed::Box};
use async_trait::async_trait;

#[async_trait]
pub trait FileSystem {
    async fn open(&mut self, path: &str, flags: u32) -> Option<FDt>;
}