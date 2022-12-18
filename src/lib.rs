#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

mod drivers;
mod interrupts;
mod memory;
mod task;
mod fs;

//#[macro_use]
extern crate alloc;
extern crate multiboot2;

use core::panic::PanicInfo;
use drivers::vga::{self, Color, ColorCode};
use multiboot2::BootInformation;
use task::{executor::Executor, keyboard, Task};

use fs::iso::iso9660::{IsoDir, IsoPrimVolDesc, MultiEndian32};

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

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

pub fn init(boot_info: &BootInformation) {
    vga::change_color(ColorCode::new(Color::LightCyan, Color::Black));
    println!("Starting init");
    memory::init(boot_info);
    memory::gdt::init_gdt();
    interrupts::init_idt();
    drivers::atapi::init();
    vga::change_color(ColorCode::new(Color::LightGreen, Color::Black));
}

#[no_mangle]
pub extern "C" fn julios_main(multiboot_info_addr: usize) -> ! {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };

    init(&boot_info);
    println!("***JuliOS V0.1.0***");
    serial_println!("Hello serial");

    serial_println!("{}", core::mem::size_of::<IsoPrimVolDesc>());
    serial_println!("{}", core::mem::size_of::<IsoDir>());
    serial_println!("{}", core::mem::size_of::<MultiEndian32>());

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}
