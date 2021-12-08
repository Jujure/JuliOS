pub use self::frame_allocator::AreaFrameAllocator;
pub use paging::kernel_remap;
use crate::println;
use multiboot2::BootInformation;
use paging::{Size4KiB, RecursivePageTable, FrameAllocator, Page, Flags, Mapper};
use heap_alloc::{HEAP_START, HEAP_SIZE, ALLOCATOR};
use x86_64::VirtAddr;
use x86_64::structures::paging::{mapper::MapToError, page::PageRangeInclusive};

pub mod frame_allocator;
pub mod paging;
pub mod heap_alloc;

pub const PAGE_SIZE: usize = 4096;

pub fn init(boot_info: &BootInformation) {
    enable_nxe_bit();
    enable_write_protect_bit();
    let mut frame_allocator = get_frame_allocator(boot_info.start_address());
    let mut active_table = kernel_remap(&mut frame_allocator, boot_info);
    init_heap(&mut active_table, &mut frame_allocator)
        .expect("Heap initialization failed");
}

fn init_heap<A>(active_table: &mut RecursivePageTable, frame_allocator: &mut A)
    -> Result<(), MapToError<Size4KiB>>
    where A: FrameAllocator<Size4KiB>
{
    let page_range: PageRangeInclusive<Size4KiB> = {
        let heap_start = VirtAddr::new(HEAP_START);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = Flags::PRESENT | Flags::WRITABLE;
        unsafe {
            active_table.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as usize, HEAP_SIZE as usize);
    }

    Ok(())
}

fn get_frame_allocator(multiboot_info_addr: usize) -> AreaFrameAllocator {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

    let kernel_start: u64 = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end: u64 = elf_sections_tag
        .sections()
        .map(|s| s.addr + s.size)
        .max()
        .unwrap();

    let multiboot_start: u64 = multiboot_info_addr as u64;
    let multiboot_end: u64 = multiboot_start + (boot_info.total_size as u64);

    AreaFrameAllocator::new(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas(),
    )
}

fn enable_nxe_bit() {
    println!("Enabling nxe bit");
    use x86_64::registers::control::{Efer, EferFlags};
    unsafe { Efer::update(|efer| *efer |= EferFlags::NO_EXECUTE_ENABLE) }
}

fn enable_write_protect_bit() {
    println!("Enabling write protection bit");
    use x86_64::registers::control::{Cr0, Cr0Flags};

    unsafe { Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT) };
}
