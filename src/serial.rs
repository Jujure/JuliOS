use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

const COM1: u16 = 0x3f8;
#[allow(dead_code)]
const COM2: u16 = 0x2f8;
#[allow(dead_code)]
const COM3: u16 = 0x3e8;
#[allow(dead_code)]
const COM4: u16 = 0x2e8;

const ENABLE_TRANSMITTER: u8 = 0x1 << 1;

const FIFO: u8 = 0x1;
const TRIGGER_LVL_14: u8 = 0x3 << 6;
const CLEAR_TRANSMIT_FIFO: u8 = 0x1 << 2;
const CLEAR_REVEIVE_FIFO: u8 = 0x1 << 1;

const NO_PARITY: u8 = 0x0;
const EIGHT_BITS_LENGTH: u8 = 0x3;

const EMPTY_TRANSMITTER: u8 = 0x1 << 5;
const DLAB: u8 = 0x1 << 7;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = SerialPort::new(COM1);
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

pub struct SerialPort {
    base: Port<u8>,
    interrupt_enable: Port<u8>,
    fifo_control: Port<u8>,
    line_control: Port<u8>,
    line_status: Port<u8>,
}

impl SerialPort {
    pub fn new(port: u16) -> SerialPort {
        SerialPort {
            base: Port::new(port),
            interrupt_enable: Port::new(port + 1),
            fifo_control: Port::new(port + 2),
            line_control: Port::new(port + 3),
            line_status: Port::new(port + 5),
        }
    }

    fn write_byte(&mut self, byte: u8) -> bool {
        unsafe {
            let status: u8 = self.line_status.read();
            match status & EMPTY_TRANSMITTER {
                0 => false,
                _ => {
                    self.base.write(byte);
                    true
                }
            }
        }
    }

    fn write_string(&mut self, s: &str) -> usize {
        let mut len: usize = 0;
        for byte in s.bytes() {
            let written: bool = self.write_byte(byte);
            if !written {
                return len;
            }
            len += 1;
        }
        len
    }

    fn init_baud_rate(&mut self) {
        let line: u8;
        unsafe {
            line = self.line_control.read();
            self.line_control.write(line | DLAB);
            self.base.write(3);
            self.interrupt_enable.write(0);
            self.line_control.write(line);
        }
    }

    fn init(&mut self) {
        self.init_baud_rate();
        unsafe {
            self.line_control.write(NO_PARITY | EIGHT_BITS_LENGTH);
            self.fifo_control
                .write(FIFO | TRIGGER_LVL_14 | CLEAR_TRANSMIT_FIFO | CLEAR_REVEIVE_FIFO);
            self.interrupt_enable.write(ENABLE_TRANSMITTER);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let len: usize = self.write_string(s);
        match len {
            l if l == s.len() => Ok(()),
            _ => Err(fmt::Error),
        }
    }
}
