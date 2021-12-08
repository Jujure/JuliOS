use multiboot2::BootInformation;
pub use x86_64::structures::paging::{FrameAllocator, Size4KiB, PageTable, RecursivePageTable, Page, PageTableFlags as Flags, Mapper, PhysFrame as Frame};
use crate::println;
use x86_64::{VirtAddr, PhysAddr};
use x86_64::registers::control;
use temporary_page::TemporaryPage;
use super::PAGE_SIZE;

mod temporary_page;

pub const P4: *mut PageTable = 0o177777_777_777_777_777_0000 as *mut _;

pub fn kernel_remap<A>(allocator: &mut A, boot_info: &BootInformation)
    where A: FrameAllocator<Size4KiB>
{
    let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtAddr::new(0xcafebabe)), allocator);
    let mut active_table = get_active_page_table();
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("No more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };
    new_table.under(&mut active_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf sections tag required");

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                // section is not loaded to memory
                continue;
            }
            println!("mapping section at addr: {:#x}, size: {:#x}",
                section.addr, section.size);
            assert!(section.start_address() % PAGE_SIZE == 0,
                    "sections need to be page aligned");


            let flags = Flags::WRITABLE | Flags::PRESENT;

            let start_frame = Frame::<Size4KiB>::containing_address(PhysAddr::new(section.start_address() as u64));
            let end_frame = Frame::containing_address(PhysAddr::new(section.end_address() as u64 - 1));
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                unsafe {
                    mapper.identity_map(frame, flags, allocator).expect("Failed to identity map kernel").flush();
                }
            }
        }
        let vga_buffer_frame = Frame::<Size4KiB>::containing_address(PhysAddr::new(0xb8000));
        unsafe {
            mapper.identity_map(vga_buffer_frame, Flags::PRESENT | Flags::WRITABLE, allocator).expect("Failed to identity map VGA buffer").flush();
        }

        let multiboot_start = Frame::<Size4KiB>::containing_address(PhysAddr::new(boot_info.start_address() as u64));
        let multiboot_end = Frame::containing_address(PhysAddr::new(boot_info.end_address() as u64 - 1));
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            unsafe {
                mapper.identity_map(frame, Flags::PRESENT, allocator).expect("Failed to identity map multiboot info struct").flush();
            }
        }
    });

    new_table.activate();
    println!("Loaded new page table!");
}

struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: & mut RecursivePageTable,
                                   temporary_page: &mut TemporaryPage) -> InactivePageTable {
        let table = temporary_page.map_table_frame(frame, active_table);
        table.zero();
        table[511].set_frame(frame.clone(), Flags::PRESENT | Flags::WRITABLE);
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }

    pub fn under<F>(&mut self, active_table: &mut RecursivePageTable,
                    temporary_page: &mut TemporaryPage, f: F)
        where F: FnOnce(&mut RecursivePageTable)
    {
        let backup = control::Cr3::read().0;
        let p4_table = temporary_page.map_table_frame(backup, active_table);
        unsafe {
            (*P4)[511].set_frame(self.p4_frame, Flags::PRESENT | Flags::WRITABLE);
        }
        x86_64::instructions::tlb::flush_all();

        f(active_table);

        p4_table[511].set_frame(backup, Flags::PRESENT | Flags::WRITABLE);
        x86_64::instructions::tlb::flush_all();

        temporary_page.unmap(active_table);
    }

    pub fn activate(&mut self) -> InactivePageTable {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(
                control::Cr3::read().0.start_address()
            ),
        };
        unsafe {
            control::Cr3::write(self.p4_frame, control::Cr3Flags::empty());
        }
        old_table
    }
}

pub fn get_active_page_table() -> RecursivePageTable<'static> {
    unsafe {
        RecursivePageTable::new(&mut *P4).expect("Could not create Page Table")
    }
}


pub fn test_paging<A>(allocator: &mut A)
    where A: FrameAllocator<Size4KiB>
{
    let mut page_table = get_active_page_table();

    let addr = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = Page::containing_address(VirtAddr::new(addr));
    let frame = allocator.allocate_frame().expect("no more frames");
    println!("None = , map to {:?}", frame);
    unsafe { page_table.map_to(page, frame, Flags::PRESENT, allocator).expect("Could not map").flush() };
    println!("next free frame: {:?}", allocator.allocate_frame());

    let page_ptr: *mut u8 = page.start_address().as_mut_ptr();
    let frame_ptr: *mut u8 = frame.start_address().as_u64() as *mut u8;

    unsafe {
        println!("Page: {:#?}, Frame: {:#?}", page_ptr, frame_ptr);
    }
}
