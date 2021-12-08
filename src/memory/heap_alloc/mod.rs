use super::PAGE_SIZE;
use linked_list_allocator::LockedHeap;

pub const HEAP_START: u64 = 0x4444_4444_0000;
pub const HEAP_SIZE: u64 = PAGE_SIZE as u64 * 25;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();
