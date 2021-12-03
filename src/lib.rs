#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod serial;
mod vga;
mod interrupts;

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
    interrupts::init_idt();
    vga::change_color(ColorCode::new(Color::LightGreen, Color::Black));
}

#[no_mangle]
pub extern "C" fn julios_main() -> ! {
    init();
    println!("***JuliOS V0.1.0***");
    serial_println!("Hello serial");
    x86_64::instructions::interrupts::int3();
    panic!("Kernel end of flow");
}
