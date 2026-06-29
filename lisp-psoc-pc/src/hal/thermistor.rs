use cortex_m::asm;
use psoc6_pac::Peripherals;

const THERM_GND_PIN: u32 = 0;
const THERM_OUT0_PIN: u8 = 1;
const THERM_OUT1_PIN: u8 = 2;
const THERM_VDD_PIN: u32 = 3;

const DRIVE_ANALOG: u32 = 0x00;
const DRIVE_STRONG_IN_OFF: u32 = 0x06;

const PCLK_PASS_CLOCK_SAR: usize = 0x34;
const SAR_CLOCK_DIVIDER: usize = 1;
const SAR_INT8_DIV: u32 = 7;
const DIV_CMD_PHASE_ALIGN_CLK_PERI: u32 = (0xff << 16) | (3 << 24);
const DIV_CMD_ENABLE_8BIT_SAR: u32 =
    (1 << 31) | DIV_CMD_PHASE_ALIGN_CLK_PERI | SAR_CLOCK_DIVIDER as u32;
const DIV_CMD_DISABLE_8BIT_SAR: u32 =
    (1 << 30) | DIV_CMD_PHASE_ALIGN_CLK_PERI | SAR_CLOCK_DIVIDER as u32;

const SAR_REFERENCE_MV: u16 = 3300;
const SAR_MAX_COUNTS: u32 = 4095;
const SAR_SAMPLE_TIME: u32 = 128;
const SAR_CHANNEL0_ENABLE: u32 = 1;
const SAR_EOS_DSI_OUT_ENABLE: u32 = 1 << 31;
const SAR_CLEAR_ALL_SWITCHES: u32 = 0x3fff_ffff;
const SAR_INTR_CLEAR_ALL: u32 = 0xff;
const SAR_SARMUX_P1_P2_MASK: u32 = (1 << THERM_OUT0_PIN) | (1 << THERM_OUT1_PIN);
const SAR_CONVERSION_TIMEOUT_POLLS: u32 = 100_000;

#[derive(Clone, Copy)]
pub enum ReadStatus {
    Ready,
    Timeout,
}

#[derive(Clone, Copy)]
pub struct ReadReport {
    pub status: ReadStatus,
    pub reference_mv: u16,
    pub out0_counts: u16,
    pub out1_counts: u16,
    pub out0_mv: u16,
    pub out1_mv: u16,
    pub delta_counts: u16,
    pub out0_poll_count: u32,
    pub out1_poll_count: u32,
    pub sar_ctrl: u32,
    pub sar_sample_ctrl: u32,
    pub sar_chan_config0: u32,
    pub sar_chan_en: u32,
    pub sar_intr: u32,
    pub sar_status: u32,
    pub sar_mux_switch0: u32,
    pub sar_mux_switch_sq_ctrl: u32,
    pub peri_clock_sar: u32,
    pub peri_div8_sar: u32,
    pub gpio_prt10_cfg: u32,
    pub gpio_prt10_out: u32,
    pub gpio_prt10_in: u32,
    pub hsiom_prt10_sel0: u32,
}

#[derive(Clone, Copy)]
struct Sample {
    ready: bool,
    counts: u16,
    poll_count: u32,
}

pub fn read(p: &Peripherals) -> ReadReport {
    configure_thermistor_pins(p);
    configure_sar_clock(p);
    configure_sar(p);

    power_thermistor(p, true);
    settle();

    let out0 = sample_sarmux_pin(p, THERM_OUT0_PIN);
    let out1 = sample_sarmux_pin(p, THERM_OUT1_PIN);

    power_thermistor(p, false);
    shutdown_sar(p);

    let status = if out0.ready && out1.ready {
        ReadStatus::Ready
    } else {
        ReadStatus::Timeout
    };

    ReadReport {
        status,
        reference_mv: SAR_REFERENCE_MV,
        out0_counts: out0.counts,
        out1_counts: out1.counts,
        out0_mv: counts_to_millivolts(out0.counts),
        out1_mv: counts_to_millivolts(out1.counts),
        delta_counts: abs_diff(out0.counts, out1.counts),
        out0_poll_count: out0.poll_count,
        out1_poll_count: out1.poll_count,
        sar_ctrl: p.SAR.ctrl.read().bits(),
        sar_sample_ctrl: p.SAR.sample_ctrl.read().bits(),
        sar_chan_config0: p.SAR.chan_config[0].read().bits(),
        sar_chan_en: p.SAR.chan_en.read().bits(),
        sar_intr: p.SAR.intr.read().bits(),
        sar_status: p.SAR.status.read().bits(),
        sar_mux_switch0: p.SAR.mux_switch0.read().bits(),
        sar_mux_switch_sq_ctrl: p.SAR.mux_switch_sq_ctrl.read().bits(),
        peri_clock_sar: p.PERI.clock_ctl[PCLK_PASS_CLOCK_SAR].read().bits(),
        peri_div8_sar: p.PERI.div_8_ctl[SAR_CLOCK_DIVIDER].read().bits(),
        gpio_prt10_cfg: p.GPIO.prt10.cfg.read().bits(),
        gpio_prt10_out: p.GPIO.prt10.out.read().bits(),
        gpio_prt10_in: p.GPIO.prt10.in_.read().bits(),
        hsiom_prt10_sel0: p.HSIOM.prt10.port_sel0.read().bits(),
    }
}

fn configure_thermistor_pins(p: &Peripherals) {
    p.GPIO
        .prt10
        .out_clr
        .write(|w| unsafe { w.bits((1 << THERM_GND_PIN) | (1 << THERM_VDD_PIN)) });

    p.HSIOM.prt10.port_sel0.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !0xffff;
        w.bits(bits)
    });

    p.GPIO.prt10.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !0xffff;
        bits |= DRIVE_STRONG_IN_OFF << (THERM_GND_PIN * 4);
        bits |= DRIVE_ANALOG << ((THERM_OUT0_PIN as u32) * 4);
        bits |= DRIVE_ANALOG << ((THERM_OUT1_PIN as u32) * 4);
        bits |= DRIVE_STRONG_IN_OFF << (THERM_VDD_PIN * 4);
        w.bits(bits)
    });
}

fn configure_sar_clock(p: &Peripherals) {
    p.PERI
        .div_cmd
        .write(|w| unsafe { w.bits(DIV_CMD_DISABLE_8BIT_SAR) });
    while p.PERI.div_cmd.read().disable().bit_is_set() {}

    p.PERI.div_8_ctl[SAR_CLOCK_DIVIDER].write(|w| unsafe { w.bits(SAR_INT8_DIV << 8) });
    p.PERI.clock_ctl[PCLK_PASS_CLOCK_SAR].write(|w| unsafe { w.bits(SAR_CLOCK_DIVIDER as u32) });

    p.PERI
        .div_cmd
        .write(|w| unsafe { w.bits(DIV_CMD_ENABLE_8BIT_SAR) });
    while p.PERI.div_cmd.read().enable().bit_is_set() {}
}

fn configure_sar(p: &Peripherals) {
    p.SAR.ctrl.modify(|_, w| w.enabled().clear_bit());
    while p.SAR.status.read().busy().bit_is_set() {}

    p.SAR
        .sample_ctrl
        .write(|w| unsafe { w.bits(SAR_EOS_DSI_OUT_ENABLE) });
    p.SAR
        .sample_time01
        .write(|w| unsafe { w.bits(SAR_SAMPLE_TIME | (SAR_SAMPLE_TIME << 16)) });
    p.SAR.sample_time23.write(|w| unsafe { w.bits(0) });
    p.SAR.range_thres.write(|w| unsafe { w.bits(0) });
    p.SAR.range_cond.write(|w| w.range_cond().below());
    p.SAR.intr_mask.write(|w| unsafe { w.bits(0) });
    p.SAR.saturate_intr_mask.write(|w| unsafe { w.bits(0) });
    p.SAR.range_intr_mask.write(|w| unsafe { w.bits(0) });
    p.SAR.intr.write(|w| unsafe { w.bits(SAR_INTR_CLEAR_ALL) });
    p.SAR
        .saturate_intr
        .write(|w| unsafe { w.bits(SAR_CHANNEL0_ENABLE) });
    p.SAR
        .range_intr
        .write(|w| unsafe { w.bits(SAR_CHANNEL0_ENABLE) });
    p.SAR
        .mux_switch_clear0
        .write(|w| unsafe { w.bits(SAR_CLEAR_ALL_SWITCHES) });
    p.SAR
        .mux_switch0
        .write(|w| unsafe { w.bits(SAR_SARMUX_P1_P2_MASK) });
    p.SAR
        .mux_switch_sq_ctrl
        .modify(|r, w| unsafe { w.bits(r.bits() | SAR_SARMUX_P1_P2_MASK) });

    p.SAR.ctrl.write(|w| {
        w.pwr_ctrl_vref()
            .pwr_100()
            .vref_sel()
            .vdda()
            .neg_sel()
            .vssa_kelvin()
            .sar_hw_ctrl_negvref()
            .set_bit()
            .comp_dly()
            .d12()
            .refbuf_en()
            .set_bit()
            .comp_pwr()
            .p60()
    });
    p.SAR.ctrl.modify(|_, w| w.enabled().set_bit());
}

fn sample_sarmux_pin(p: &Peripherals, pin: u8) -> Sample {
    p.SAR.chan_config[0].write(|w| unsafe {
        w.pos_pin_addr()
            .bits(pin)
            .pos_port_addr()
            .sarmux()
            .differential_en()
            .clear_bit()
            .avg_en()
            .clear_bit()
            .sample_time_sel()
            .bits(0)
            .neg_addr_en()
            .clear_bit()
    });
    p.SAR
        .chan_en
        .write(|w| unsafe { w.bits(SAR_CHANNEL0_ENABLE) });
    p.SAR.intr.write(|w| unsafe { w.bits(SAR_INTR_CLEAR_ALL) });
    p.SAR.start_ctrl.write(|w| w.fw_trigger().set_bit());

    let mut poll_count = 0;
    while poll_count < SAR_CONVERSION_TIMEOUT_POLLS {
        if p.SAR.intr.read().eos_intr().bit_is_set() {
            let counts = p.SAR.chan_result[0].read().result().bits() & 0x0fff;
            p.SAR.intr.write(|w| unsafe { w.bits(SAR_INTR_CLEAR_ALL) });
            return Sample {
                ready: true,
                counts,
                poll_count,
            };
        }
        poll_count += 1;
    }

    Sample {
        ready: false,
        counts: 0,
        poll_count,
    }
}

fn power_thermistor(p: &Peripherals, powered: bool) {
    p.GPIO
        .prt10
        .out_clr
        .write(|w| unsafe { w.bits(1 << THERM_GND_PIN) });

    if powered {
        p.GPIO
            .prt10
            .out_set
            .write(|w| unsafe { w.bits(1 << THERM_VDD_PIN) });
    } else {
        p.GPIO
            .prt10
            .out_clr
            .write(|w| unsafe { w.bits(1 << THERM_VDD_PIN) });
    }
}

fn shutdown_sar(p: &Peripherals) {
    p.SAR.ctrl.modify(|_, w| w.enabled().clear_bit());
    while p.SAR.status.read().busy().bit_is_set() {}
    p.SAR.chan_en.write(|w| unsafe { w.bits(0) });
    p.SAR
        .mux_switch_clear0
        .write(|w| unsafe { w.bits(SAR_CLEAR_ALL_SWITCHES) });
    p.SAR
        .mux_switch_sq_ctrl
        .modify(|r, w| unsafe { w.bits(r.bits() & !SAR_SARMUX_P1_P2_MASK) });
}

fn settle() {
    for _ in 0..10_000 {
        asm::nop();
    }
}

fn counts_to_millivolts(counts: u16) -> u16 {
    (((counts as u32) * (SAR_REFERENCE_MV as u32) + (SAR_MAX_COUNTS / 2)) / SAR_MAX_COUNTS) as u16
}

fn abs_diff(left: u16, right: u16) -> u16 {
    if left >= right {
        left - right
    } else {
        right - left
    }
}
