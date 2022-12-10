use crate::memory::gdt;
use crate::hlt_loop;
use crate::drivers::vga::{self, Color, ColorCode};
use crate::{println};
use lazy_static::lazy_static;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pic::{InterruptIndex, init_pic, keyboard_interrupt_handler, timer_interrupt_handler};

pub mod pic;

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


