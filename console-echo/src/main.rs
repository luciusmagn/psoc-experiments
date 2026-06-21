#![no_std]
#![no_main]

use core::fmt::{self, Write};

use cortex_m_rt::entry;
use panic_halt as _;
use psoc6_pac::{Peripherals, SCB5};

mod micro_sd;

const SYSCLK_HZ: u32 = 50_000_000;
const UART_BAUD: u32 = 115_200;
const UART_OVERSAMPLE: u32 = 12;
const UART_DIVIDER_VALUE: u32 = 35; // 50 MHz / (35 + 1) / 12 = 115740 baud.

const SCB5_CLOCK: usize = 5;
const UART_CLOCK_DIVIDER: usize = 0;
const DIV_CMD_ENABLE_8BIT_0: u32 = (1 << 31) | (0xff << 16) | (3 << 24);
const DIV_CMD_DISABLE_8BIT_0: u32 = (1 << 30) | (0xff << 16) | (3 << 24);

const SCB_CTRL_ENABLED: u32 = 1 << 31;
const SCB_CTRL_MODE_UART: u32 = 2 << 24;
const SCB_CTRL_BYTE_MODE: u32 = 1 << 11;

const SCB_UART_STD: u32 = 0 << 24;
const SCB_UART_STOP_BITS_1: u32 = 1;
const SCB_UART_BREAK_WIDTH_11_BITS: u32 = 10 << 16;
const SCB_DATA_WIDTH_8: u32 = 7;

const FIFO_USED_MASK: u32 = 0x01ff;
const FIFO_SR_VALID: u32 = 1 << 15;
const FIFO_CLEAR: u32 = 1 << 16;

struct Console<'a> {
    scb: &'a SCB5,
}

impl<'a> Console<'a> {
    fn new(scb: &'a SCB5) -> Self {
        Self { scb }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.scb.rx_fifo_status.read().bits() & FIFO_USED_MASK == 0 {
            return None;
        }

        Some((self.scb.rx_fifo_rd.read().bits() & 0xff) as u8)
    }

    fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.write_byte(b'\r');
        }

        while self.scb.tx_fifo_status.read().bits() & (FIFO_USED_MASK | FIFO_SR_VALID) != 0 {}

        self.scb
            .tx_fifo_wr
            .write(|w| unsafe { w.bits(byte as u32) });
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    fn prompt(&mut self) {
        self.write_bytes(b"\npsoc6> ");
    }
}

impl Write for Console<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

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

    configure_led(&p);
    micro_sd::configure_card_detect(&p);
    micro_sd::configure_sdhc1_pins(&p);
    configure_uart(&p);

    let mut delay = cortex_m::delay::Delay::new(cp.SYST, SYSCLK_HZ);
    let mut console = Console::new(&p.SCB5);

    led_off(&p);

    writeln!(console, "\nPSoC6 console-echo").ok();
    writeln!(console, "UART: SCB5 P5.1 TX / P5.0 RX, {} 8N1", UART_BAUD).ok();
    writeln!(
        console,
        "Try: help, regs, led on, led off, heartbeat on, (+ 1 2 3)"
    )
    .ok();
    console.prompt();

    let mut line = [0u8; 96];
    let mut line_len = 0usize;
    let mut led_state = false;
    let mut heartbeat_enabled = false;
    let mut heartbeat_ms = 0u16;

    loop {
        if let Some(byte) = console.read_byte() {
            match byte {
                b'\r' | b'\n' => {
                    console.write_bytes(b"\n");
                    handle_line(
                        &line[..line_len],
                        &mut console,
                        &p,
                        &mut led_state,
                        &mut heartbeat_enabled,
                    );
                    line_len = 0;
                    console.prompt();
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
        if heartbeat_enabled {
            heartbeat_ms += 1;
            if heartbeat_ms >= 500 {
                heartbeat_ms = 0;
                led_toggle(&p, &mut led_state);
            }
        } else {
            heartbeat_ms = 0;
        }
    }
}

fn configure_led(p: &Peripherals) {
    p.GPIO.prt13.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !(0x0f << 28);
        bits |= 0x06 << 28;
        w.bits(bits)
    });
    led_off(p);
}

fn write_sdhc_registers(console: &mut Console<'_>, name: &str, snapshot: &micro_sd::SdhcSnapshot) {
    writeln!(
        console,
        "{}.WRAP.CTL      = 0x{:08x}",
        name, snapshot.wrap_ctl
    )
    .ok();
    writeln!(
        console,
        "{}.HOST_VERSION  = 0x{:04x}",
        name, snapshot.host_version
    )
    .ok();
    writeln!(console, "{}.CAP1          = 0x{:08x}", name, snapshot.cap1).ok();
    writeln!(console, "{}.CAP2          = 0x{:08x}", name, snapshot.cap2).ok();
    writeln!(
        console,
        "{}.PSTATE        = 0x{:08x}",
        name, snapshot.pstate
    )
    .ok();
}

fn write_micro_sd_pins(console: &mut Console<'_>, snapshot: &micro_sd::PinSnapshot) {
    writeln!(
        console,
        "microSD HSIOM P12.SEL1=0x{:08x} P13.SEL0=0x{:08x}",
        snapshot.p12_sel1, snapshot.p13_sel0
    )
    .ok();
    writeln!(
        console,
        "microSD GPIO  P12.CFG =0x{:08x} P13.CFG =0x{:08x}",
        snapshot.p12_cfg, snapshot.p13_cfg
    )
    .ok();
}

fn led_on(p: &Peripherals) {
    p.GPIO.prt13.out_clr.write(|w| w.out7().set_bit());
}

fn led_off(p: &Peripherals) {
    p.GPIO.prt13.out_set.write(|w| w.out7().set_bit());
}

fn led_set(p: &Peripherals, on: bool) {
    if on {
        led_on(p);
    } else {
        led_off(p);
    }
}

fn led_toggle(p: &Peripherals, state: &mut bool) {
    *state = !*state;
    led_set(p, *state);
}

fn configure_uart(p: &Peripherals) {
    p.SCB5.ctrl.write(|w| unsafe { w.bits(0) });

    // Route the hard-wired KitProg3 USB-UART bridge pins to SCB5.
    p.GPIO.prt5.out_set.write(|w| w.out1().set_bit());
    p.HSIOM
        .prt5
        .port_sel0
        .modify(|_, w| w.io0_sel().act_6().io1_sel().variant(18));

    p.GPIO.prt5.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !0xff;
        bits |= 1 << 3; // P5.0 RX: high-Z with input buffer enabled.
        bits |= 6 << 4; // P5.1 TX: strong drive, input buffer disabled.
        w.bits(bits)
    });

    p.PERI
        .div_cmd
        .write(|w| unsafe { w.bits(DIV_CMD_DISABLE_8BIT_0) });
    while p.PERI.div_cmd.read().disable().bit_is_set() {}

    p.PERI.div_8_ctl[UART_CLOCK_DIVIDER].write(|w| unsafe { w.bits(UART_DIVIDER_VALUE << 8) });
    p.PERI.clock_ctl[SCB5_CLOCK].write(|w| unsafe { w.bits(0) });
    p.PERI
        .div_cmd
        .write(|w| unsafe { w.bits(DIV_CMD_ENABLE_8BIT_0) });
    while p.PERI.div_cmd.read().enable().bit_is_set() {}

    p.SCB5.uart_ctrl.write(|w| unsafe { w.bits(SCB_UART_STD) });
    p.SCB5
        .uart_tx_ctrl
        .write(|w| unsafe { w.bits(SCB_UART_STOP_BITS_1) });
    p.SCB5
        .uart_rx_ctrl
        .write(|w| unsafe { w.bits(SCB_UART_STOP_BITS_1 | SCB_UART_BREAK_WIDTH_11_BITS) });
    p.SCB5
        .tx_ctrl
        .write(|w| unsafe { w.bits(SCB_DATA_WIDTH_8) });
    p.SCB5
        .rx_ctrl
        .write(|w| unsafe { w.bits(SCB_DATA_WIDTH_8) });

    p.SCB5.tx_fifo_ctrl.write(|w| unsafe { w.bits(FIFO_CLEAR) });
    p.SCB5.tx_fifo_ctrl.write(|w| unsafe { w.bits(0) });
    p.SCB5.rx_fifo_ctrl.write(|w| unsafe { w.bits(FIFO_CLEAR) });
    p.SCB5.rx_fifo_ctrl.write(|w| unsafe { w.bits(0) });

    p.SCB5.ctrl.write(|w| unsafe {
        w.bits(SCB_CTRL_ENABLED | SCB_CTRL_MODE_UART | SCB_CTRL_BYTE_MODE | (UART_OVERSAMPLE - 1))
    });
}

fn handle_line(
    line: &[u8],
    console: &mut Console<'_>,
    p: &Peripherals,
    led_state: &mut bool,
    heartbeat_enabled: &mut bool,
) {
    let line = trim_ascii(line);

    if line.is_empty() {
        return;
    }

    if eq_ascii(line, b"help") || eq_ascii(line, b"?") {
        writeln!(console, "commands:").ok();
        writeln!(console, "  help").ok();
        writeln!(console, "  regs").ok();
        writeln!(console, "  led on | led off | led toggle | led status").ok();
        writeln!(console, "  heartbeat on | heartbeat off").ok();
        writeln!(console, "  sd status").ok();
        writeln!(console, "  sd pins | sd pinmux").ok();
        writeln!(console, "  sd init").ok();
        writeln!(console, "  sdhc regs").ok();
        writeln!(console, "  reboot").ok();
        writeln!(console, "  (+ 1 2 3) ; also -, *, /, flat integer args").ok();
        return;
    }

    if eq_ascii(line, b"regs") {
        writeln!(
            console,
            "SCB5.CTRL       = 0x{:08x}",
            p.SCB5.ctrl.read().bits()
        )
        .ok();
        writeln!(
            console,
            "SCB5.UART_CTRL  = 0x{:08x}",
            p.SCB5.uart_ctrl.read().bits()
        )
        .ok();
        writeln!(
            console,
            "SCB5.RX_STATUS  = 0x{:08x}",
            p.SCB5.rx_fifo_status.read().bits()
        )
        .ok();
        writeln!(
            console,
            "SCB5.TX_STATUS  = 0x{:08x}",
            p.SCB5.tx_fifo_status.read().bits()
        )
        .ok();
        writeln!(
            console,
            "PERI.CLOCK[5]   = 0x{:08x}",
            p.PERI.clock_ctl[SCB5_CLOCK].read().bits()
        )
        .ok();
        writeln!(
            console,
            "PERI.DIV8[0]    = 0x{:08x}",
            p.PERI.div_8_ctl[UART_CLOCK_DIVIDER].read().bits()
        )
        .ok();
        writeln!(
            console,
            "HSIOM.PRT5.SEL0 = 0x{:08x}",
            p.HSIOM.prt5.port_sel0.read().bits()
        )
        .ok();
        writeln!(
            console,
            "GPIO.PRT5.CFG   = 0x{:08x}",
            p.GPIO.prt5.cfg.read().bits()
        )
        .ok();
        writeln!(
            console,
            "GPIO.PRT13.OUT   = 0x{:08x}",
            p.GPIO.prt13.out.read().bits()
        )
        .ok();
        writeln!(
            console,
            "GPIO.PRT13.CFG   = 0x{:08x}",
            p.GPIO.prt13.cfg.read().bits()
        )
        .ok();
        return;
    }

    if eq_ascii(line, b"led on") {
        *heartbeat_enabled = false;
        *led_state = true;
        led_set(p, *led_state);
        writeln!(console, "ok; heartbeat off").ok();
        return;
    }

    if eq_ascii(line, b"led off") {
        *heartbeat_enabled = false;
        *led_state = false;
        led_set(p, *led_state);
        writeln!(console, "ok; heartbeat off").ok();
        return;
    }

    if eq_ascii(line, b"led toggle") {
        *heartbeat_enabled = false;
        *led_state = !*led_state;
        led_set(p, *led_state);
        writeln!(console, "ok; heartbeat off").ok();
        return;
    }

    if eq_ascii(line, b"led status") {
        writeln!(
            console,
            "led={} heartbeat={} GPIO.PRT13.OUT=0x{:08x}",
            if *led_state { "on" } else { "off" },
            if *heartbeat_enabled { "on" } else { "off" },
            p.GPIO.prt13.out.read().bits()
        )
        .ok();
        return;
    }

    if eq_ascii(line, b"heartbeat on") {
        *heartbeat_enabled = true;
        writeln!(console, "ok").ok();
        return;
    }

    if eq_ascii(line, b"heartbeat off") {
        *heartbeat_enabled = false;
        writeln!(console, "ok").ok();
        return;
    }

    if eq_ascii(line, b"sd status") {
        let snapshot = micro_sd::card_detect_snapshot(p);

        writeln!(
            console,
            "microSD CD_L(P13.5)={} GPIO.PRT13.IN=0x{:08x} GPIO.PRT13.CFG=0x{:08x}",
            if snapshot.is_low { "low" } else { "high" },
            snapshot.prt13_in,
            snapshot.prt13_cfg
        )
        .ok();
        return;
    }

    if eq_ascii(line, b"sd pins") {
        write_micro_sd_pins(console, &micro_sd::pin_snapshot(p));
        return;
    }

    if eq_ascii(line, b"sd pinmux") {
        micro_sd::configure_sdhc1_pins(p);
        writeln!(console, "ok").ok();
        write_micro_sd_pins(console, &micro_sd::pin_snapshot(p));
        return;
    }

    if eq_ascii(line, b"sd init") {
        let report = micro_sd::initialize_card(p);
        write_micro_sd_init_report(console, &report);
        return;
    }

    if eq_ascii(line, b"sdhc regs") {
        micro_sd::enable_sdhc_controllers(p);

        write_sdhc_registers(console, "SDHC0", &micro_sd::sdhc0_snapshot(p));
        write_sdhc_registers(console, "SDHC1", &micro_sd::sdhc1_snapshot(p));
        write_micro_sd_pins(console, &micro_sd::pin_snapshot(p));
        return;
    }

    if eq_ascii(line, b"reboot") {
        writeln!(console, "resetting").ok();
        cortex_m::peripheral::SCB::sys_reset();
    }

    if line.starts_with(b"(") {
        match eval_flat_lisp(line) {
            Ok(value) => writeln!(console, "=> {}", value).ok(),
            Err(err) => writeln!(console, "error: {}", err).ok(),
        };
        return;
    }

    writeln!(console, "unknown command; try help").ok();
}

fn write_micro_sd_init_report(console: &mut Console<'_>, report: &micro_sd::InitReport) {
    writeln!(console, "sd init: {}", micro_sd_init_status(report.status)).ok();
    writeln!(
        console,
        "CMD8=0x{:08x} ACMD41_OCR=0x{:08x} attempts={}",
        report.cmd8_response, report.acmd41_ocr, report.acmd41_attempts
    )
    .ok();
    writeln!(
        console,
        "SDHC1.CLK_CTRL=0x{:04x} PWR_CTRL=0x{:02x}",
        report.clk_ctrl, report.pwr_ctrl
    )
    .ok();
    writeln!(
        console,
        "SDHC1.NORM_INT=0x{:04x} ERR_INT=0x{:04x} PSTATE=0x{:08x}",
        report.normal_int, report.error_int, report.pstate
    )
    .ok();

    if let Some(error) = report.last_error {
        writeln!(
            console,
            "last error: {} NORM_INT=0x{:04x} ERR_INT=0x{:04x} PSTATE=0x{:08x}",
            micro_sd_command_error(error.code),
            error.normal_int,
            error.error_int,
            error.pstate
        )
        .ok();
    }
}

fn micro_sd_init_status(status: micro_sd::InitStatus) -> &'static str {
    match status {
        micro_sd::InitStatus::ReadySdhc => "ready SDHC/SDXC",
        micro_sd::InitStatus::ReadySdsc => "ready SDSC",
        micro_sd::InitStatus::NoCardDetect => "no card on CD_L",
        micro_sd::InitStatus::ClockNotStable => "internal clock not stable",
        micro_sd::InitStatus::ResetTimeout => "host reset timeout",
        micro_sd::InitStatus::Cmd0Failed => "CMD0 failed",
        micro_sd::InitStatus::Cmd8Failed => "CMD8 failed",
        micro_sd::InitStatus::Cmd8PatternMismatch => "CMD8 pattern mismatch",
        micro_sd::InitStatus::Acmd41Failed => "ACMD41 failed",
        micro_sd::InitStatus::Acmd41Busy => "ACMD41 busy timeout",
    }
}

fn micro_sd_command_error(code: micro_sd::CommandErrorCode) -> &'static str {
    match code {
        micro_sd::CommandErrorCode::CommandLineBusy => "command line busy",
        micro_sd::CommandErrorCode::CommandTimeout => "command timeout",
        micro_sd::CommandErrorCode::CommandStatusError => "command status error",
    }
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

fn eq_ascii(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(&l, &r)| l.to_ascii_lowercase() == r.to_ascii_lowercase())
}

fn eval_flat_lisp(input: &[u8]) -> Result<i32, &'static str> {
    let mut parser = Parser { input, pos: 0 };
    parser.skip_ws();
    parser.expect(b'(')?;
    parser.skip_ws();
    let op = parser.take_op()?;
    parser.skip_ws();

    let first = parser.take_i32()?;
    let mut result = first;
    let mut args = 1u32;

    loop {
        parser.skip_ws();
        if parser.try_take(b')') {
            break;
        }

        let next = parser.take_i32()?;
        args += 1;
        result = match op {
            b'+' => result.checked_add(next).ok_or("integer overflow")?,
            b'-' => result.checked_sub(next).ok_or("integer overflow")?,
            b'*' => result.checked_mul(next).ok_or("integer overflow")?,
            b'/' => {
                if next == 0 {
                    return Err("division by zero");
                }
                result.checked_div(next).ok_or("integer overflow")?
            }
            _ => return Err("unsupported operator"),
        };
    }

    parser.skip_ws();
    if !parser.is_done() {
        return Err("trailing input");
    }

    if args == 1 {
        result = match op {
            b'-' => result.checked_neg().ok_or("integer overflow")?,
            b'/' => return Err("division needs at least two args"),
            _ => result,
        };
    }

    Ok(result)
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn is_done(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn expect(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.try_take(byte) {
            Ok(())
        } else {
            Err("unexpected syntax")
        }
    }

    fn try_take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn take_op(&mut self) -> Result<u8, &'static str> {
        match self.peek() {
            Some(op @ (b'+' | b'-' | b'*' | b'/')) => {
                self.pos += 1;
                Ok(op)
            }
            Some(_) => Err("expected operator"),
            None => Err("unexpected end"),
        }
    }

    fn take_i32(&mut self) -> Result<i32, &'static str> {
        self.skip_ws();
        if self.peek() == Some(b'(') {
            return Err("nested expressions not implemented yet");
        }

        let mut negative = false;
        if self.peek() == Some(b'-') {
            negative = true;
            self.pos += 1;
        }

        let mut value: i32 = 0;
        let mut digits = 0u32;
        while let Some(byte) = self.peek() {
            if !byte.is_ascii_digit() {
                break;
            }

            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add((byte - b'0') as i32))
                .ok_or("integer overflow")?;
            self.pos += 1;
            digits += 1;
        }

        if digits == 0 {
            return Err("expected integer");
        }

        if negative {
            value = value.checked_neg().ok_or("integer overflow")?;
        }

        Ok(value)
    }
}
