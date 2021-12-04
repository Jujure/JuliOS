#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod gdt;
mod interrupts;
mod serial;
mod vga;

use core::panic::PanicInfo;
use vga::{Color, ColorCode};

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    vga::change_color(ColorCode::new(Color::LightRed, Color::Black));
    println!("{}", info);
    loop {}
}

pub fn init() {
    vga::change_color(ColorCode::new(Color::LightCyan, Color::Black));
    println!("Starting init");
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    vga::change_color(ColorCode::new(Color::LightGreen, Color::Black));
}

#[no_mangle]
pub extern "C" fn julios_main() -> ! {
    init();
    println!("***JuliOS V0.1.0***");
    serial_println!("Hello serial");

    panic!("Kernel end of flow");
}
