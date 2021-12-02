#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::fmt::Write;
mod vga;

#[no_mangle]
pub extern "C" fn julios_main() -> ! {
    vga::WRITER.lock().write_str("Hello").unwrap();
    write!(vga::WRITER.lock(), " {}!\n{}", "World", "***JuliOS***").unwrap();
    loop {}
}
