#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod gdt;
mod interrupts;
mod memory;
mod serial;
mod vga;

extern crate multiboot2;

use core::panic::PanicInfo;
use vga::{Color, ColorCode};

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    vga::change_color(ColorCode::new(Color::LightRed, Color::Black));
    println!("{}", info);
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    vga::change_color(ColorCode::new(Color::LightCyan, Color::Black));
    println!("Starting init");
    gdt::init_gdt();
    interrupts::init_idt();
    vga::change_color(ColorCode::new(Color::LightGreen, Color::Black));
}

fn get_frame_allocator(multiboot_info_addr: usize) -> memory::AreaFrameAllocator {
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

    memory::AreaFrameAllocator::new( kernel_start, kernel_end, multiboot_start,
        multiboot_end, memory_map_tag.memory_areas())
}

#[no_mangle]
pub extern "C" fn julios_main(multiboot_info_addr: usize) -> ! {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };

    let mut frame_allocator = get_frame_allocator(multiboot_info_addr);

    memory::kernel_remap(&mut frame_allocator, boot_info);

    init();
    println!("***JuliOS V0.1.0***");
    serial_println!("Hello serial");
    memory::paging::test_paging(&mut frame_allocator);
    panic!("Kernel end of flow");
}
