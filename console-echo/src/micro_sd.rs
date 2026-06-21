use psoc6_pac::Peripherals;

const CARD_DETECT_MASK: u32 = 1 << 5;
const HSIOM_SEL_SDHC: u32 = 26;
const DRIVE_HIGHZ_INPUT: u32 = 0x08;
const DRIVE_STRONG_INPUT: u32 = 0x0e;

const P12_SDHC_PINS_MASK: u32 = (0x0f << 16) | (0x0f << 20);
const P12_SDHC_PINS_CFG: u32 = (DRIVE_STRONG_INPUT << 16) | (DRIVE_STRONG_INPUT << 20);
const P13_SDHC_DATA_MASK: u32 = 0x0f | (0x0f << 4) | (0x0f << 8) | (0x0f << 12);
const P13_SDHC_DATA_CFG: u32 = DRIVE_STRONG_INPUT
    | (DRIVE_STRONG_INPUT << 4)
    | (DRIVE_STRONG_INPUT << 8)
    | (DRIVE_STRONG_INPUT << 12);

const P12_SDHC_HSIOM_MASK: u32 = 0xff | (0xff << 8);
const P12_SDHC_HSIOM: u32 = HSIOM_SEL_SDHC | (HSIOM_SEL_SDHC << 8);
const P13_SDHC_HSIOM_MASK: u32 = 0xff | (0xff << 8) | (0xff << 16) | (0xff << 24);
const P13_SDHC_HSIOM: u32 =
    HSIOM_SEL_SDHC | (HSIOM_SEL_SDHC << 8) | (HSIOM_SEL_SDHC << 16) | (HSIOM_SEL_SDHC << 24);

pub struct CardDetectSnapshot {
    pub is_low: bool,
    pub prt13_in: u32,
    pub prt13_cfg: u32,
}

pub struct PinSnapshot {
    pub p12_sel1: u32,
    pub p13_sel0: u32,
    pub p12_cfg: u32,
    pub p13_cfg: u32,
}

pub struct SdhcSnapshot {
    pub wrap_ctl: u32,
    pub host_version: u16,
    pub cap1: u32,
    pub cap2: u32,
    pub pstate: u32,
}

pub fn configure_card_detect(p: &Peripherals) {
    p.GPIO.prt13.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !(0x0f << 20);
        bits |= DRIVE_HIGHZ_INPUT << 20;
        w.bits(bits)
    });
}

pub fn configure_sdhc1_pins(p: &Peripherals) {
    p.GPIO.prt12.out_set.write(|w| unsafe { w.bits(0x30) });
    p.GPIO.prt13.out_set.write(|w| unsafe { w.bits(0x0f) });

    p.HSIOM.prt12.port_sel1.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P12_SDHC_HSIOM_MASK;
        bits |= P12_SDHC_HSIOM;
        w.bits(bits)
    });

    p.HSIOM.prt13.port_sel0.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P13_SDHC_HSIOM_MASK;
        bits |= P13_SDHC_HSIOM;
        w.bits(bits)
    });

    p.GPIO.prt12.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P12_SDHC_PINS_MASK;
        bits |= P12_SDHC_PINS_CFG;
        w.bits(bits)
    });

    p.GPIO.prt13.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P13_SDHC_DATA_MASK;
        bits |= P13_SDHC_DATA_CFG;
        w.bits(bits)
    });
}

pub fn card_detect_snapshot(p: &Peripherals) -> CardDetectSnapshot {
    let prt13_in = p.GPIO.prt13.in_.read().bits();

    CardDetectSnapshot {
        is_low: prt13_in & CARD_DETECT_MASK == 0,
        prt13_in,
        prt13_cfg: p.GPIO.prt13.cfg.read().bits(),
    }
}

pub fn pin_snapshot(p: &Peripherals) -> PinSnapshot {
    PinSnapshot {
        p12_sel1: p.HSIOM.prt12.port_sel1.read().bits(),
        p13_sel0: p.HSIOM.prt13.port_sel0.read().bits(),
        p12_cfg: p.GPIO.prt12.cfg.read().bits(),
        p13_cfg: p.GPIO.prt13.cfg.read().bits(),
    }
}

pub fn enable_sdhc_controllers(p: &Peripherals) {
    p.SDHC0.wrap.ctl.write(|w| w.enable().set_bit());
    p.SDHC1.wrap.ctl.write(|w| w.enable().set_bit());
    for _ in 0..1024 {
        cortex_m::asm::nop();
    }
}

pub fn sdhc0_snapshot(p: &Peripherals) -> SdhcSnapshot {
    SdhcSnapshot {
        wrap_ctl: p.SDHC0.wrap.ctl.read().bits(),
        host_version: p.SDHC0.core.host_cntrl_vers_r.read().bits(),
        cap1: p.SDHC0.core.capabilities1_r.read().bits(),
        cap2: p.SDHC0.core.capabilities2_r.read().bits(),
        pstate: p.SDHC0.core.pstate_reg.read().bits(),
    }
}

pub fn sdhc1_snapshot(p: &Peripherals) -> SdhcSnapshot {
    SdhcSnapshot {
        wrap_ctl: p.SDHC1.wrap.ctl.read().bits(),
        host_version: p.SDHC1.core.host_cntrl_vers_r.read().bits(),
        cap1: p.SDHC1.core.capabilities1_r.read().bits(),
        cap2: p.SDHC1.core.capabilities2_r.read().bits(),
        pstate: p.SDHC1.core.pstate_reg.read().bits(),
    }
}
