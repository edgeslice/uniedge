use core::fmt::{self, Write};
use core::ptr::{read_volatile, write_volatile};

const PL011_BASE: usize = 0x0900_0000;
const PL011_DR: *mut u32 = PL011_BASE as *mut u32;
const PL011_FR: *const u32 = (PL011_BASE + 0x18) as *const u32;
const PL011_TX_FULL: u32 = 1 << 5;

pub struct Uart;

impl Uart {
    pub const fn new() -> Self {
        Self
    }

    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            while read_volatile(PL011_FR) & PL011_TX_FULL != 0 {}
            write_volatile(PL011_DR, byte as u32);
        }
    }

    pub fn write_raw(&mut self, s: &str) {
        let _ = self.write_str(s);
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }

        Ok(())
    }
}
