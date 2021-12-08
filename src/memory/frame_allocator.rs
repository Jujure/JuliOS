pub use super::PAGE_SIZE;
use multiboot2::{MemoryArea, MemoryAreaIter};
pub use x86_64::structures::paging::{
    frame::PhysFrame as Frame, FrameAllocator, FrameDeallocator, Size4KiB,
};
use x86_64::PhysAddr;

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl AreaFrameAllocator {
    pub fn new(
        kernel_start: u64,
        kernel_end: u64,
        multiboot_start: u64,
        multiboot_end: u64,
        memory_areas: MemoryAreaIter,
    ) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(PhysAddr::new(0)),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(PhysAddr::new(kernel_start)),
            kernel_end: Frame::containing_address(PhysAddr::new(kernel_end)),
            multiboot_start: Frame::containing_address(PhysAddr::new(multiboot_start)),
            multiboot_end: Frame::containing_address(PhysAddr::new(multiboot_end)),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self
            .areas
            .clone()
            .filter(|area| {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(PhysAddr::new(address)) >= self.next_free_frame
            })
            .min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(PhysAddr::new(area.base_addr));
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let frame = Frame::containing_address(self.next_free_frame.start_address());

            // the last frame of the current area
            let current_area_last_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(PhysAddr::new(address))
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by the kernel
                self.next_free_frame =
                    Frame::containing_address(self.kernel_end.start_address() + PAGE_SIZE);
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame =
                    Frame::containing_address(self.multiboot_end.start_address() + PAGE_SIZE);
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame =
                    Frame::containing_address(self.next_free_frame.start_address() + PAGE_SIZE);
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }
}

impl FrameDeallocator<Size4KiB> for AreaFrameAllocator {
    unsafe fn deallocate_frame(&mut self, _frame: Frame) {
        unimplemented!()
    }
}

pub struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    pub fn new<A>(allocator: &mut A) -> TinyAllocator
    where
        A: FrameAllocator<Size4KiB>,
    {
        let mut f = || allocator.allocate_frame();
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}

unsafe impl FrameAllocator<Size4KiB> for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }
}
impl FrameDeallocator<Size4KiB> for TinyAllocator {
    unsafe fn deallocate_frame(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("Tiny allocator can hold only 3 frames.");
    }
}
