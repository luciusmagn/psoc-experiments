use core::fmt::{self, Write};

#[cfg(feature = "uart-pin-probe")]
use cortex_m::delay::Delay;
use psoc6_pac::{Peripherals, SCB5};

pub const UART_BAUD: u32 = 115_200;
#[cfg(feature = "uart-pin-probe")]
pub const UART_PIN_PROBE_BAUD: u32 = 9_600;

const UART_OVERSAMPLE: u32 = 12;
const UART_DIVIDER_VALUE: u32 = 35; // 50 MHz / (35 + 1) / 12 = 115740 baud.
#[cfg(feature = "uart-pin-probe")]
const UART_PIN_PROBE_BIT_US: u32 = 1_000_000 / UART_PIN_PROBE_BAUD;

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

#[cfg(feature = "uart-pin-probe")]
const P5_1_CFG_MASK: u32 = 0xf0;
#[cfg(feature = "uart-pin-probe")]
const P5_1_STRONG_OUTPUT: u32 = 6 << 4;

pub struct Console<'a> {
    scb: &'a SCB5,
}

impl<'a> Console<'a> {
    pub fn new(scb: &'a SCB5) -> Self {
        Self { scb }
    }

    pub fn configure_hardware(p: &Peripherals) {
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
            w.bits(
                SCB_CTRL_ENABLED | SCB_CTRL_MODE_UART | SCB_CTRL_BYTE_MODE | (UART_OVERSAMPLE - 1),
            )
        });
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.scb.rx_fifo_status.read().bits() & FIFO_USED_MASK == 0 {
            return None;
        }

        Some((self.scb.rx_fifo_rd.read().bits() & 0xff) as u8)
    }

    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.write_byte(b'\r');
        }

        while self.scb.tx_fifo_status.read().bits() & (FIFO_USED_MASK | FIFO_SR_VALID) != 0 {}

        self.scb
            .tx_fifo_wr
            .write(|w| unsafe { w.bits(byte as u32) });
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    pub fn prompt(&mut self) {
        self.write_bytes(b"\nlisp> ");
    }

    pub fn continuation_prompt(&mut self) {
        self.write_bytes(b"\n....> ");
    }
}

#[cfg(feature = "uart-pin-probe")]
pub fn write_p5_1_bitbang_probe(p: &Peripherals, delay: &mut Delay, bytes: &[u8]) {
    configure_p5_1_gpio_tx(p);
    delay.delay_ms(50);
    for &byte in bytes {
        write_p5_1_bitbang_byte(p, delay, byte);
    }
    set_p5_1(p, true);
}

#[cfg(feature = "uart-pin-probe")]
fn configure_p5_1_gpio_tx(p: &Peripherals) {
    set_p5_1(p, true);
    p.HSIOM
        .prt5
        .port_sel0
        .modify(|_, w| unsafe { w.io1_sel().bits(0) });
    p.GPIO.prt5.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P5_1_CFG_MASK;
        bits |= P5_1_STRONG_OUTPUT;
        w.bits(bits)
    });
}

#[cfg(feature = "uart-pin-probe")]
fn write_p5_1_bitbang_byte(p: &Peripherals, delay: &mut Delay, byte: u8) {
    set_p5_1(p, false);
    delay.delay_us(UART_PIN_PROBE_BIT_US);
    for bit_index in 0..8 {
        set_p5_1(p, byte & (1 << bit_index) != 0);
        delay.delay_us(UART_PIN_PROBE_BIT_US);
    }
    set_p5_1(p, true);
    delay.delay_us(UART_PIN_PROBE_BIT_US);
}

#[cfg(feature = "uart-pin-probe")]
fn set_p5_1(p: &Peripherals, high: bool) {
    if high {
        p.GPIO.prt5.out_set.write(|w| w.out1().set_bit());
    } else {
        p.GPIO.prt5.out_clr.write(|w| w.out1().set_bit());
    }
}

impl Write for Console<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}
