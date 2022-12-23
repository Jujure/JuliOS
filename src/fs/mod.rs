pub mod iso;

use crate::fd::FDt;

use alloc::{sync::Arc, boxed::Box};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait FileSystem {
    async fn open(path: &str, flags: u32) -> Option<FDt>;
}