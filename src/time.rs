use core::arch::asm;

use smoltcp::time::Instant;

pub fn now() -> Instant {
    Instant::from_millis(uptime_millis() as i64)
}

pub fn delay_ms(ms: u64) {
    let start = counter();
    let ticks_per_ms = (frequency() / 1000).max(1);
    let wait_ticks = ticks_per_ms.saturating_mul(ms);

    while counter().wrapping_sub(start) < wait_ticks {
        core::hint::spin_loop();
    }
}

fn frequency() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {value}, cntfrq_el0", value = out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn counter() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {value}, cntpct_el0", value = out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn uptime_millis() -> u64 {
    let freq = frequency().max(1);
    counter().saturating_mul(1000) / freq
}
