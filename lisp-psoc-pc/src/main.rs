#![no_std]
#![no_main]

use core::fmt::{self, Write};
use core::ptr::{read_volatile, write_volatile};

use cortex_m_rt::entry;
use panic_halt as _;
use psoc6_pac::{Peripherals, SCB5};

mod lisp;
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

const BUTTON0_MASK: u32 = 1 << 4;
const PERIPHERAL_REGISTER_START: u32 = 0x4000_0000;
const PERIPHERAL_REGISTER_END: u32 = 0x40ff_fffc;

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
        self.write_bytes(b"\nlisp> ");
    }

    fn continuation_prompt(&mut self) {
        self.write_bytes(b"\n....> ");
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
    configure_button(&p);
    micro_sd::configure_card_detect(&p);
    micro_sd::configure_sdhc1_pins(&p);
    configure_uart(&p);

    let mut delay = cortex_m::delay::Delay::new(cp.SYST, SYSCLK_HZ);
    let mut console = Console::new(&p.SCB5);
    let mut machine = lisp::Machine::new();

    led_off(&p);
    if let Err(error) = machine.bootstrap() {
        writeln!(console, "\nLisp bootstrap failed: {}", error.message()).ok();
    }

    writeln!(console, "\nPSoC6 lisp-psoc-pc").ok();
    writeln!(console, "UART: SCB5 P5.1 TX / P5.0 RX, {} 8N1", UART_BAUD).ok();
    writeln!(console, "Try: (help), (led off), (regs), (+ 1 2 3)").ok();
    console.prompt();

    let mut line = [0u8; 384];
    let mut line_len = 0usize;
    let mut led_state = false;
    let mut heartbeat_enabled = false;
    let mut heartbeat_ms = 0u16;
    let mut uptime_ms = 0u32;

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
                        let mut board = PsocBoard {
                            p: &p,
                            led_state: &mut led_state,
                            heartbeat_enabled: &mut heartbeat_enabled,
                            uptime_ms,
                        };
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
        uptime_ms = uptime_ms.wrapping_add(1);
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

fn configure_button(p: &Peripherals) {
    p.GPIO.prt0.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !(0x0f << 16);
        bits |= 0x08 << 16;
        w.bits(bits)
    });
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

    p.GPIO.prt5.out_set.write(|w| w.out1().set_bit());
    p.HSIOM
        .prt5
        .port_sel0
        .modify(|_, w| w.io0_sel().act_6().io1_sel().variant(18));

    p.GPIO.prt5.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !0xff;
        bits |= 1 << 3;
        bits |= 6 << 4;
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

struct PsocBoard<'a> {
    p: &'a Peripherals,
    led_state: &'a mut bool,
    heartbeat_enabled: &'a mut bool,
    uptime_ms: u32,
}

impl lisp::Board for PsocBoard<'_> {
    fn led(&mut self, action: lisp::LedAction) -> bool {
        match action {
            lisp::LedAction::On => {
                *self.heartbeat_enabled = false;
                *self.led_state = true;
                led_set(self.p, true);
            }
            lisp::LedAction::Off => {
                *self.heartbeat_enabled = false;
                *self.led_state = false;
                led_set(self.p, false);
            }
            lisp::LedAction::Toggle => {
                *self.heartbeat_enabled = false;
                *self.led_state = !*self.led_state;
                led_set(self.p, *self.led_state);
            }
            lisp::LedAction::Status => {}
        }

        *self.led_state
    }

    fn heartbeat(&mut self, enabled: bool) -> bool {
        *self.heartbeat_enabled = enabled;
        *self.heartbeat_enabled
    }

    fn button_pressed(&mut self, index: i32) -> Result<bool, lisp::Error> {
        if index != 0 {
            return Err(lisp::Error::new("unknown button"));
        }

        Ok(self.p.GPIO.prt0.in_.read().bits() & BUTTON0_MASK == 0)
    }

    fn millis(&mut self) -> u32 {
        self.uptime_ms
    }

    fn read32(&mut self, address: u32) -> Result<u32, lisp::Error> {
        if !is_peripheral_register_address(address) {
            return Err(lisp::Error::new(
                "address outside peripheral register range",
            ));
        }

        Ok(unsafe { read_volatile(address as *const u32) })
    }

    fn write32(&mut self, address: u32, value: u32) -> Result<(), lisp::Error> {
        if !is_peripheral_register_address(address) {
            return Err(lisp::Error::new(
                "address outside peripheral register range",
            ));
        }

        unsafe {
            write_volatile(address as *mut u32, value);
        }
        Ok(())
    }

    fn registers(&mut self) -> lisp::RegisterReport {
        lisp::RegisterReport {
            scb5_ctrl: self.p.SCB5.ctrl.read().bits(),
            scb5_uart_ctrl: self.p.SCB5.uart_ctrl.read().bits(),
            scb5_rx_status: self.p.SCB5.rx_fifo_status.read().bits(),
            scb5_tx_status: self.p.SCB5.tx_fifo_status.read().bits(),
            peri_clock5: self.p.PERI.clock_ctl[SCB5_CLOCK].read().bits(),
            peri_div8_0: self.p.PERI.div_8_ctl[UART_CLOCK_DIVIDER].read().bits(),
            hsiom_prt5_sel0: self.p.HSIOM.prt5.port_sel0.read().bits(),
            gpio_prt5_cfg: self.p.GPIO.prt5.cfg.read().bits(),
            gpio_prt13_out: self.p.GPIO.prt13.out.read().bits(),
            gpio_prt13_cfg: self.p.GPIO.prt13.cfg.read().bits(),
        }
    }

    fn sd_status(&mut self) -> lisp::SdStatusReport {
        let snapshot = micro_sd::card_detect_snapshot(self.p);
        lisp::SdStatusReport {
            cd_low: snapshot.is_low,
            prt13_in: snapshot.prt13_in,
            prt13_cfg: snapshot.prt13_cfg,
        }
    }

    fn sd_pins(&mut self) -> lisp::SdPinsReport {
        sd_pins_report(micro_sd::pin_snapshot(self.p))
    }

    fn sd_pinmux(&mut self) -> lisp::SdPinsReport {
        micro_sd::configure_sdhc1_pins(self.p);
        sd_pins_report(micro_sd::pin_snapshot(self.p))
    }

    fn sd_clock(&mut self) -> lisp::SdClockReport {
        micro_sd::enable_sdhc_controllers(self.p);
        let snapshot = micro_sd::sdhc1_clock_snapshot(self.p);
        lisp::SdClockReport {
            path0: snapshot.path0,
            root0: snapshot.root0,
            root2: snapshot.root2,
            fll_config: snapshot.fll_config,
            fll_config2: snapshot.fll_config2,
            fll_status: snapshot.fll_status,
            selected_hf_hz: snapshot.selected_hf_hz,
        }
    }

    fn sd_init(&mut self) -> lisp::SdInitReport {
        let report = micro_sd::initialize_card(self.p);
        lisp::SdInitReport {
            status: sd_init_status(report.status),
            cmd8_response: report.cmd8_response,
            cmd8_error: report.cmd8_error.map(sd_command_error_report),
            acmd41_ocr: report.acmd41_ocr,
            acmd41_attempts: report.acmd41_attempts,
            gp_out: report.gp_out,
            gp_in: report.gp_in,
            host_ctrl1: report.host_ctrl1,
            host_ctrl2: report.host_ctrl2,
            xfer_mode: report.xfer_mode,
            tout_ctrl: report.tout_ctrl,
            clk_ctrl: report.clk_ctrl,
            pwr_ctrl: report.pwr_ctrl,
            sw_rst: report.sw_rst,
            normal_int: report.normal_int,
            error_int: report.error_int,
            normal_int_stat_en: report.normal_int_stat_en,
            error_int_stat_en: report.error_int_stat_en,
            normal_int_signal_en: report.normal_int_signal_en,
            error_int_signal_en: report.error_int_signal_en,
            pstate: report.pstate,
            cmd: report.cmd,
            argument: report.argument,
            response01: report.response01,
            response23: report.response23,
            response45: report.response45,
            response67: report.response67,
            last_error: report.last_error.map(sd_command_error_report),
        }
    }

    fn sd_read0(&mut self) -> lisp::SdRead0Report {
        let report = micro_sd::read_sector_zero(self.p);
        lisp::SdRead0Report {
            status: sd_read_status(report.status),
            init_status: sd_init_status(report.init_status),
            rca: report.rca,
            ocr: report.ocr,
            acmd41_attempts: report.acmd41_attempts,
            command_response: report.command_response,
            last_error: report.last_error.map(sd_command_error_report),
            first_words: report.first_words,
            mbr_signature: report.mbr_signature,
            partition_type: report.partition_type,
            normal_int: report.normal_int,
            error_int: report.error_int,
            pstate: report.pstate,
            block_size: report.block_size,
            block_count: report.block_count,
            xfer_mode: report.xfer_mode,
            cmd: report.cmd,
            argument: report.argument,
        }
    }

    fn sdhc_registers(&mut self) -> lisp::SdhcReport {
        micro_sd::enable_sdhc_controllers(self.p);

        lisp::SdhcReport {
            sdhc0: sdhc_core_report(micro_sd::sdhc0_snapshot(self.p)),
            sdhc1: sdhc_core_report(micro_sd::sdhc1_snapshot(self.p)),
            pins: sd_pins_report(micro_sd::pin_snapshot(self.p)),
        }
    }

    fn reboot(&mut self) -> ! {
        cortex_m::peripheral::SCB::sys_reset();
    }
}

fn is_peripheral_register_address(address: u32) -> bool {
    address & 0x03 == 0
        && address >= PERIPHERAL_REGISTER_START
        && address <= PERIPHERAL_REGISTER_END
}

fn sd_pins_report(snapshot: micro_sd::PinSnapshot) -> lisp::SdPinsReport {
    lisp::SdPinsReport {
        p12_sel1: snapshot.p12_sel1,
        p13_sel0: snapshot.p13_sel0,
        p12_cfg: snapshot.p12_cfg,
        p13_cfg: snapshot.p13_cfg,
    }
}

fn sdhc_core_report(snapshot: micro_sd::SdhcSnapshot) -> lisp::SdhcCoreReport {
    lisp::SdhcCoreReport {
        wrap_ctl: snapshot.wrap_ctl,
        host_version: snapshot.host_version,
        cap1: snapshot.cap1,
        cap2: snapshot.cap2,
        pstate: snapshot.pstate,
    }
}

fn sd_init_status(status: micro_sd::InitStatus) -> &'static [u8] {
    match status {
        micro_sd::InitStatus::ReadySdhc => b"ready-sdhc",
        micro_sd::InitStatus::ReadySdsc => b"ready-sdsc",
        micro_sd::InitStatus::NoCardDetect => b"no-card-detect",
        micro_sd::InitStatus::ClockNotStable => b"clock-not-stable",
        micro_sd::InitStatus::ResetTimeout => b"reset-timeout",
        micro_sd::InitStatus::Cmd0Failed => b"cmd0-failed",
        micro_sd::InitStatus::Cmd8PatternMismatch => b"cmd8-pattern-mismatch",
        micro_sd::InitStatus::Acmd41Failed => b"acmd41-failed",
        micro_sd::InitStatus::Acmd41Busy => b"acmd41-busy",
    }
}

fn sd_read_status(status: micro_sd::ReadStatus) -> &'static [u8] {
    match status {
        micro_sd::ReadStatus::Ready => b"ready",
        micro_sd::ReadStatus::InitFailed => b"init-failed",
        micro_sd::ReadStatus::Cmd2Failed => b"cmd2-failed",
        micro_sd::ReadStatus::Cmd3Failed => b"cmd3-failed",
        micro_sd::ReadStatus::Cmd7Failed => b"cmd7-failed",
        micro_sd::ReadStatus::Cmd16Failed => b"cmd16-failed",
        micro_sd::ReadStatus::DataSetupBusy => b"data-setup-busy",
        micro_sd::ReadStatus::Cmd17Failed => b"cmd17-failed",
        micro_sd::ReadStatus::BufferReadTimeout => b"buffer-read-timeout",
        micro_sd::ReadStatus::BufferEnableTimeout => b"buffer-enable-timeout",
        micro_sd::ReadStatus::TransferTimeout => b"transfer-timeout",
    }
}

fn sd_command_error(code: micro_sd::CommandErrorCode) -> &'static [u8] {
    match code {
        micro_sd::CommandErrorCode::CommandLineBusy => b"command-line-busy",
        micro_sd::CommandErrorCode::CommandTimeout => b"command-timeout",
        micro_sd::CommandErrorCode::CommandStatusError => b"command-status-error",
    }
}

fn sd_command_error_report(error: micro_sd::CommandError) -> lisp::SdCommandErrorReport {
    lisp::SdCommandErrorReport {
        code: sd_command_error(error.code),
        normal_int: error.normal_int,
        error_int: error.error_int,
        pstate: error.pstate,
        command: error.command,
        argument: error.argument,
        pstate_after_write: error.pstate_after_write,
        normal_int_after_write: error.normal_int_after_write,
        error_int_after_write: error.error_int_after_write,
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
