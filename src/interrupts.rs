use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;
use crate::vga::{self, Color, ColorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    println!("Load IDT");
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame) {
    let color: vga::ColorCode = vga::get_color();
    vga::change_color(ColorCode::new(Color::Pink, Color::Black));
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    vga::change_color(color);
}
