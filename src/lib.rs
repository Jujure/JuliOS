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
use memory::paging::{FrameAllocator, Size4KiB};
use multiboot2::BootInformation;
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

pub fn init<A>(frame_allocator: &mut A, boot_info: &BootInformation)
where
    A: FrameAllocator<Size4KiB>,
{
    vga::change_color(ColorCode::new(Color::LightCyan, Color::Black));
    println!("Starting init");
    enable_nxe_bit();
    enable_write_protect_bit();
    memory::kernel_remap(frame_allocator, boot_info);
    gdt::init_gdt();
    interrupts::init_idt();
    vga::change_color(ColorCode::new(Color::LightGreen, Color::Black));
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

    memory::AreaFrameAllocator::new(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas(),
    )
}

#[no_mangle]
pub extern "C" fn julios_main(multiboot_info_addr: usize) -> ! {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };

    let mut frame_allocator = get_frame_allocator(multiboot_info_addr);

    init(&mut frame_allocator, &boot_info);
    println!("***JuliOS V0.1.0***");
    serial_println!("Hello serial");
    panic!("Kernel end of flow");
}
