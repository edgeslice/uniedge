use core::fmt::Write;

use crate::console::Uart;
use crate::time::delay_ms;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[92m";
const DIM: &str = "\x1b[2m";

pub fn render() {
    let mut uart = Uart::new();

    uart.write_raw("\x1b[2J\x1b[H\x1b[?25l");
    let _ = writeln!(
        uart,
        "{CYAN}{BOLD}   __  __      _ ______    _            {RESET}"
    );
    let _ = writeln!(
        uart,
        "{CYAN}{BOLD}  / / / /___  (_) ____/___| | __ _  ___ {RESET}"
    );
    let _ = writeln!(
        uart,
        "{CYAN}{BOLD} / / / / __ \\/ / __/ / __| |/ _` |/ _ \\{RESET}"
    );
    let _ = writeln!(
        uart,
        "{CYAN}{BOLD}/ /_/ / / / / / /___\\__ \\ | (_| |  __/{RESET}"
    );
    let _ = writeln!(
        uart,
        "{CYAN}{BOLD}\\____/_/ /_/_/_____/___/_|\\__, |\\___|{RESET}"
    );
    let _ = writeln!(
        uart,
        "{CYAN}{BOLD}                          |___/      {RESET}"
    );
    let _ = writeln!(uart);

    phase(&mut uart, "priming bare-metal runtime");
    phase(&mut uart, "probing virtio-mmio network");
    phase(&mut uart, "arming tcp listener on :8080");

    let _ = writeln!(uart);
    let _ = writeln!(uart, "{GREEN}{BOLD}UniEdge is alive!{RESET}");
    let _ = writeln!(
        uart,
        "{DIM}serial: pl011 | web: http://127.0.0.1:8080{RESET}"
    );
    let _ = writeln!(uart);
    uart.write_raw("\x1b[?25h");
}

fn phase(uart: &mut Uart, label: &str) {
    const FRAMES: [&str; 6] = [
        "[=     ]", "[==    ]", "[===   ]", "[ ==== ]", "[  === ]", "[   == ]",
    ];

    for frame in FRAMES {
        let _ = write!(uart, "\r{CYAN}boot{RESET} {frame} {label}");
        delay_ms(70);
    }

    let _ = writeln!(uart, "\r{GREEN}ok  {RESET} {label}");
}
