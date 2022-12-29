#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

mod drivers;
mod fd;
mod fs;
mod interrupts;
mod memory;
mod proc;
mod syscalls;
mod task;
mod utils;

//#[macro_use]
extern crate alloc;
extern crate multiboot2;

use crate::fs::FileSystem;
use core::panic::PanicInfo;
use drivers::vga::{self, Color, ColorCode};
use multiboot2::BootInformation;
use task::{executor::Executor, keyboard, Task};

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
    vga::change_color(ColorCode::new(Color::LightGreen, Color::Black));
}

#[no_mangle]
pub extern "C" fn julios_main(multiboot_info_addr: usize) -> ! {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };

    init(&boot_info);
    println!("***JuliOS V0.1.0***");
    serial_println!("Hello serial");

    let mut executor = Executor::new();
    executor.spawn(Task::new(drivers::atapi::init()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(get_file()));
    executor.run();
}

async fn get_file() {
    let fd = fs::VIRTUAL_FS
        .lock()
        .await
        .open("/mnt/iso//boot/grub//grub.cfg", syscalls::io::O_RDONLY)
        .await
        .unwrap();
    let mut buf: [u8; 100] = [0; 100];
    let read = fd.borrow_mut().read(&mut buf, 100).await;

    serial_println!("{:?}", read);
    serial_println!("{}", alloc::str::from_utf8(&buf).unwrap());

    fd.borrow_mut().lseek(10, syscalls::io::SEEK_SET).await;

    fd.borrow_mut().read(&mut buf, 100).await;
    serial_println!("{}", alloc::str::from_utf8(&buf).unwrap());

    fd.borrow_mut().close().await;
}
