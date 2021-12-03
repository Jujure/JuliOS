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
}

impl SerialPort {
    pub fn new(port: u16) -> SerialPort {
        SerialPort {
            base: Port::new(port),
        }
    }

    fn write_byte(&mut self, byte: u8) {
        unsafe {
            self.base.write(byte);
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn init(&mut self) {}
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
