use super::{InterruptIndex, PICS};

use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

pub const PS2_CONTROLLER_PORT: u16 = 0x60;

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(PS2_CONTROLLER_PORT);
    let scancode: u8 = unsafe { port.read() };

    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
