use multiboot2::BootInformation;
use x86_64::structures::paging::{FrameAllocator, Size4KiB, PageTable, RecursivePageTable, Page, PageTableFlags as Flags, Mapper};
use crate::println;
use x86_64::VirtAddr;

pub const P4: *mut PageTable = 0o177777_777_777_777_777_0000 as *mut _;

pub fn kernel_remap<A>(_allocator: &mut A, _boot_info: &BootInformation)
    where A: FrameAllocator<Size4KiB>
{
}

pub fn test_paging<A>(allocator: &mut A)
    where A: FrameAllocator<Size4KiB>
{
    let mut page_table = unsafe { RecursivePageTable::new(&mut *P4).expect("Could not create Page Table") };

    let addr = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = Page::containing_address(VirtAddr::new(addr));
    let frame = allocator.allocate_frame().expect("no more frames");
    println!("None = , map to {:?}", frame);
    unsafe { page_table.map_to(page, frame, Flags::PRESENT, allocator).expect("Could not map").flush() };
    println!("next free frame: {:?}", allocator.allocate_frame());

    let page_ptr: *mut u8 = page.start_address().as_mut_ptr();
    let frame_ptr: *mut u8 = frame.start_address().as_u64() as *mut u8;

    unsafe {
        println!("Page: {:#?}, Frame: {:#?}", *page_ptr, *frame_ptr);
        *frame_ptr = 42;
        println!("Page: {:#?}, Frame: {:#?}", *page_ptr, *frame_ptr);
    }
}
