use super::{InterruptIndex, PICS};
use x86_64::structures::idt::InterruptStackFrame;

static mut TICKS: u64 = 0;

#[allow(dead_code)]
pub fn gettick() -> u64 {
    unsafe { return TICKS }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        TICKS += 1;
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
