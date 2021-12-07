pub use self::frame_allocator::AreaFrameAllocator;
pub use paging::kernel_remap;

pub mod frame_allocator;
pub mod paging;

pub const PAGE_SIZE: usize = 4096;
