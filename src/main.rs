#![no_std]
#![no_main]

use core::arch::global_asm;
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

global_asm!(include_str!("boot.S"));

const PL011_BASE: usize = 0x0900_0000;
const PL011_DR: *mut u32 = PL011_BASE as *mut u32;
const PL011_FR: *const u32 = (PL011_BASE + 0x18) as *const u32;
const PL011_TX_FULL: u32 = 1 << 5;

struct Uart;

impl Uart {
    fn write_byte(&mut self, byte: u8) {
        unsafe {
            while read_volatile(PL011_FR) & PL011_TX_FULL != 0 {}
            write_volatile(PL011_DR, byte as u32);
        }
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

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    let mut uart = Uart;

    let _ = writeln!(uart, "\nuniEdge kernel booted");
    let _ = writeln!(uart, "target: qemu-system-aarch64 virt");
    let _ = writeln!(uart, "status: no_std / no_main / bare metal");

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let mut uart = Uart;
    let _ = writeln!(uart, "\npanic: {info}");

    loop {
        core::hint::spin_loop();
    }
}
