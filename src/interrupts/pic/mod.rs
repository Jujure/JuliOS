use crate::println;
pub use keyboard::keyboard_interrupt_handler;
use pic8259::ChainedPics;
pub use pit::timer_interrupt_handler;
pub use disk::{disk1_interrupt_handler, disk2_interrupt_handler};

pub mod keyboard;
pub mod pit;
pub mod disk;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    HardDisk1 = PIC_1_OFFSET + 14,
    HardDisk2 = PIC_1_OFFSET + 15,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init_pic() {
    println!("Initializing PIC");
    unsafe { PICS.lock().initialize() };
}
