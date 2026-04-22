#![no_std]
#![no_main]

extern crate alloc;

mod allocator;
mod bootfx;
mod console;
mod net;
mod platform;
mod time;

use core::arch::global_asm;
use core::fmt::Write;
use core::panic::PanicInfo;

use console::Uart;

global_asm!(include_str!("boot.S"));

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    allocator::init();
    bootfx::render();

    let mut uart = Uart::new();
    if let Some(device) = platform::first_virtio_mmio() {
        let _ = writeln!(
            uart,
            "virtio-net mmio @ 0x{:016x} ({} bytes)",
            device.base, device.size
        );
        net::run(device);
    }

    let _ = writeln!(uart, "panic: virtio-mmio device not found");
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let mut uart = Uart::new();
    let _ = writeln!(uart, "\npanic: {info}");

    loop {
        core::hint::spin_loop();
    }
}
