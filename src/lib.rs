#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !
{
    loop {}
}

static HELLO: &[u8] = b"Welcome to the JuliOS";

static STAR: &[u8] = b"|/-\\";

#[no_mangle]
pub extern "C" fn julios_main() -> !
{
    let vga_buffer: *mut u8 = 0xb8000 as *mut u8;

    let mut k: usize = 0;
    for (i, &byte) in HELLO.iter().enumerate()
    {
        unsafe
        {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
        k = i * 2 + 2;
    }

    let mut j: usize = 0;
    loop
    {
        unsafe
        {
            *vga_buffer.offset(k as isize) = STAR[j];
            *vga_buffer.offset(k as isize + 1) = 0xb;
            j += 1;
            j %= 4;
        }
    }
}
