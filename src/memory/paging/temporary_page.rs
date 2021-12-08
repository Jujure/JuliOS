use super::super::frame_allocator::TinyAllocator;
use super::{
    Flags, Frame, FrameAllocator, Mapper, Page, PageTable, RecursivePageTable, Size4KiB, VirtAddr,
};

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> TemporaryPage
    where
        A: FrameAllocator<Size4KiB>,
    {
        TemporaryPage {
            page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    pub fn map(&mut self, frame: Frame, active_table: &mut RecursivePageTable) -> VirtAddr {
        unsafe {
            active_table
                .map_to(
                    self.page,
                    frame,
                    Flags::PRESENT | Flags::WRITABLE,
                    &mut self.allocator,
                )
                .expect("Failed to map temporary page")
                .flush();
        }
        self.page.start_address()
    }

    pub fn unmap(&mut self, active_table: &mut RecursivePageTable) {
        active_table
            .unmap(self.page)
            .expect("Failed to unmap")
            .1
            .flush()
    }

    pub fn map_table_frame(
        &mut self,
        frame: Frame,
        active_table: &mut RecursivePageTable,
    ) -> &mut PageTable {
        unsafe { &mut *(self.map(frame, active_table).as_u64() as *mut PageTable) }
    }
}
