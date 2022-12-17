use super::{InterruptIndex, PICS};
use x86_64::structures::idt::InterruptStackFrame;

fn disk_interrupt_handler(_disk: u16) {
    crate::drivers::atapi::interrupt::mark_interrupt();
}

pub extern "x86-interrupt" fn disk1_interrupt_handler(_stack_frame: InterruptStackFrame) {
    disk_interrupt_handler(1);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::HardDisk1.as_u8());
    }
}

pub extern "x86-interrupt" fn disk2_interrupt_handler(_stack_frame: InterruptStackFrame) {
    disk_interrupt_handler(2);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::HardDisk2.as_u8());
    }
}
