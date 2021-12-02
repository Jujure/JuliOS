#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

mod vga;

#[no_mangle]
pub extern "C" fn julios_main() -> ! {
    println!("Hello World!");
    println!("{}", "***JuliOS***");
    panic!("Test panick");
    loop {}
}
