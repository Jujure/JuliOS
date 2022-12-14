use crate::drivers::vga::{self, Color, ColorCode};
use crate::hlt_loop;
use crate::memory::gdt;
use crate::println;

use core::arch::asm;
use lazy_static::lazy_static;
use pic::{
    disk1_interrupt_handler, disk2_interrupt_handler, init_pic, keyboard_interrupt_handler,
    timer_interrupt_handler, InterruptIndex,
};
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub mod pic;

const SYSCALL_32_INTERRUPT_NUMBER: usize = 0x80;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::HardDisk1.as_usize()].set_handler_fn(disk1_interrupt_handler);
        idt[InterruptIndex::HardDisk2.as_usize()].set_handler_fn(disk2_interrupt_handler);
        idt[SYSCALL_32_INTERRUPT_NUMBER].set_handler_fn(syscall_handler_32);
        idt
    };
}

pub fn init_idt() {
    println!("Loading IDT");
    IDT.load();

    init_pic();

    println!("Enabling interrupts");
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    let color: vga::ColorCode = vga::get_color();
    vga::change_color(ColorCode::new(Color::Pink, Color::Black));
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    vga::change_color(color);
}

extern "x86-interrupt" fn syscall_handler_32(_stack_frame: InterruptStackFrame) {
    let rax: u64;
    let rbx: u64;
    let rcx: u64;
    let rdx: u64;
    let rsi: u64;
    let rdi: u64;
    let rbp: u64;
    unsafe {
        asm!(
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push rbp",

            "pop {rbp}",
            "pop {rdi}",
            "pop {rsi}",
            "pop {rdx}",
            "pop {rcx}",
            "pop {rbx}",
            "pop {rax}",
            rax = out(reg) rax,
            rbx = out(reg) rbx,
            rcx = out(reg) rcx,
            rdx = out(reg) rdx,
            rsi = out(reg) rsi,
            rdi = out(reg) rdi,
            rbp = out(reg) rbp,
        )
    }
    println!("Received syscall");
    crate::syscalls::syscall_routine(rax);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let color: vga::ColorCode = vga::get_color();
    vga::change_color(ColorCode::new(Color::LightRed, Color::Black));
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    vga::change_color(color);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
