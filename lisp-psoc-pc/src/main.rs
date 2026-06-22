#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;
use hal::{board, console};
use panic_halt as _;
use psoc6_pac::Peripherals;

mod hal;
mod lisp;
mod lisp_store;

const SYSCLK_HZ: u32 = 50_000_000;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = unsafe { Peripherals::steal() };

    extern "C" {
        static _svectors: u32;
    }
    unsafe {
        cp.SCB.vtor.write(&_svectors as *const u32 as u32);
    }

    board::State::configure_hardware(&p);
    console::Console::configure_hardware(&p);

    let mut delay = cortex_m::delay::Delay::new(cp.SYST, SYSCLK_HZ);
    let mut console = console::Console::new(&p.SCB5);
    let mut machine = lisp::Machine::new();
    let mut board_state = board::State::new();

    board_state.led_off(&p);
    if let Err(error) = machine.bootstrap() {
        writeln!(console, "\nLisp bootstrap failed: {}", error.message()).ok();
    }

    writeln!(console, "\nPSoC6 lisp-psoc-pc").ok();
    writeln!(
        console,
        "UART: SCB5 P5.1 TX / P5.0 RX, {} 8N1",
        console::UART_BAUD
    )
    .ok();
    writeln!(console, "Try: (help), (led off), (regs), (+ 1 2 3)").ok();
    console.prompt();

    let mut line = [0u8; 384];
    let mut line_len = 0usize;

    loop {
        if let Some(byte) = console.read_byte() {
            match byte {
                b'\r' | b'\n' => {
                    console.write_bytes(b"\n");
                    let input = trim_ascii(&line[..line_len]);
                    if input.is_empty() {
                        line_len = 0;
                        console.prompt();
                    } else if input_needs_more(input) {
                        if line_len < line.len() {
                            line[line_len] = b'\n';
                            line_len += 1;
                            console.continuation_prompt();
                        } else {
                            line_len = 0;
                            writeln!(console, "error: input line too long").ok();
                            console.prompt();
                        }
                    } else {
                        let mut board = board_state.lisp_board(&p);
                        machine.eval_line(input, &mut board, &mut console).ok();
                        line_len = 0;
                        console.prompt();
                    }
                }
                0x03 => {
                    line_len = 0;
                    console.write_bytes(b"^C");
                    console.prompt();
                }
                0x08 | 0x7f => {
                    if line_len > 0 {
                        line_len -= 1;
                        console.write_bytes(b"\x08 \x08");
                    }
                }
                b if b.is_ascii_graphic() || b == b' ' => {
                    if line_len < line.len() {
                        line[line_len] = b;
                        line_len += 1;
                        console.write_byte(b);
                    } else {
                        console.write_byte(b'\x07');
                    }
                }
                _ => {}
            }
        }

        delay.delay_ms(1);
        board_state.tick_ms(&p);
    }
}

fn input_needs_more(input: &[u8]) -> bool {
    let mut depth = 0u16;
    let mut in_comment = false;

    for &byte in input {
        if in_comment {
            if byte == b'\n' {
                in_comment = false;
            }
            continue;
        }

        match byte {
            b';' => in_comment = true,
            b'(' => depth = depth.saturating_add(1),
            b')' => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    depth != 0
}

fn trim_ascii(mut input: &[u8]) -> &[u8] {
    while matches!(input.first(), Some(b' ' | b'\t')) {
        input = &input[1..];
    }
    while matches!(input.last(), Some(b' ' | b'\t')) {
        input = &input[..input.len() - 1];
    }
    input
}
