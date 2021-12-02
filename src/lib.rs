#![no_std]
#![no_main]

mod vga;
mod serial;
use core::panic::PanicInfo;
use vga::Color;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    vga::change_color(Color::LightRed, Color::Black);
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn julios_main() -> ! {
    println!("Hello World!");
    println!("{}", "***JuliOS***");
    serial_println!("Test serial");
    panic!("Kernel end of flow");
}
