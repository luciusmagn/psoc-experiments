use psoc6_pac::{sdhc0, Peripherals};

const CARD_DETECT_MASK: u32 = 1 << 5;
const HSIOM_SEL_SDHC: u32 = 26;
const DRIVE_HIGHZ_INPUT: u32 = 0x08;
const DRIVE_STRONG_INPUT: u32 = 0x0e;
const CPU_CLOCK_HZ: u32 = 50_000_000;

const NORMAL_INT_ALL: u16 = 0x1fff;
const ERROR_INT_ALL: u16 = 0x07ff;
const NORMAL_INT_CMD_COMPLETE: u16 = 1 << 0;
const NORMAL_INT_ERROR: u16 = 1 << 15;
const PSTATE_CMD_INHIBIT: u32 = 1 << 0;
const CLK_CTRL_INTERNAL_CLK_EN: u16 = 1 << 0;
const CLK_CTRL_INTERNAL_CLK_STABLE: u16 = 1 << 1;
const CLK_CTRL_SD_CLK_EN: u16 = 1 << 2;
const CLK_CTRL_PLL_ENABLE: u16 = 1 << 3;
const HOST_CTRL1_CARD_DETECT_TEST_LEVEL: u8 = 1 << 6;
const HOST_CTRL1_CARD_DETECT_SIGNAL_SELECT: u8 = 1 << 7;
const HOST_CTRL2_HOST_VERSION_4_ENABLE: u16 = 1 << 12;
const GP_OUT_BASIC_SD: u32 = (1 << 0) | (1 << 1) | (1 << 3) | (1 << 4) | (1 << 5);

const SDHC_INPUT_CLOCK_HZ: u32 = 50_000_000;
const SD_INIT_CLOCK_HZ: u32 = 400_000;
const SD_INIT_CLOCK_DIVIDER: u16 = ((SDHC_INPUT_CLOCK_HZ / SD_INIT_CLOCK_HZ) >> 1) as u16;
const SD_BUS_RAMP_UP_MS: u32 = 1000;
const SD_CMD8_ARGUMENT: u32 = 0x0000_01aa;
const SD_CMD8_PATTERN_MASK: u32 = 0xff;
const SD_CMD8_PATTERN: u32 = 0xaa;
const SD_ACMD41_HCS: u32 = 1 << 30;
const SD_OCR_BUSY: u32 = 1 << 31;
const SD_OCR_CAPACITY: u32 = 1 << 30;
const SD_ACMD41_VOLTAGE_MASK: u32 = (1 << 23)
    | (1 << 22)
    | (1 << 21)
    | (1 << 20)
    | (1 << 19)
    | (1 << 18)
    | (1 << 17)
    | (1 << 16)
    | (1 << 15);
const SD_ACMD41_MAX_ATTEMPTS: u16 = 1000;

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

const CLK_ROOT_ENABLE: u32 = 1 << 31;
const CLK_ROOT_MUX_PATH0: u32 = 0;
const CLK_ROOT_DIV_BY_2: u32 = 1 << 4;
const SDHC1_HF_CLOCK_INDEX: usize = 2;
const SDHC1_HF_CLOCK_HZ: u32 = 50_000_000;

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

pub struct SdhcClockSnapshot {
    pub path0: u32,
    pub root0: u32,
    pub root2: u32,
    pub fll_config: u32,
    pub fll_config2: u32,
    pub fll_status: u32,
    pub selected_hf_hz: u32,
}

#[derive(Clone, Copy)]
pub enum CommandErrorCode {
    CommandLineBusy,
    CommandTimeout,
    CommandStatusError,
}

#[derive(Clone, Copy)]
pub struct CommandError {
    pub code: CommandErrorCode,
    pub normal_int: u16,
    pub error_int: u16,
    pub pstate: u32,
    pub command: u16,
    pub argument: u32,
    pub pstate_after_write: u32,
    pub normal_int_after_write: u16,
    pub error_int_after_write: u16,
}

#[derive(Clone, Copy)]
struct CommandTrace {
    command: u16,
    argument: u32,
    pstate_after_write: u32,
    normal_int_after_write: u16,
    error_int_after_write: u16,
}

#[derive(Clone, Copy)]
pub enum InitStatus {
    ReadySdhc,
    ReadySdsc,
    NoCardDetect,
    ClockNotStable,
    ResetTimeout,
    Cmd0Failed,
    Cmd8PatternMismatch,
    Acmd41Failed,
    Acmd41Busy,
}

pub struct InitReport {
    pub status: InitStatus,
    pub cmd8_response: u32,
    pub cmd8_error: Option<CommandError>,
    pub acmd41_ocr: u32,
    pub acmd41_attempts: u16,
    pub last_error: Option<CommandError>,
    pub gp_out: u32,
    pub gp_in: u32,
    pub host_ctrl1: u8,
    pub host_ctrl2: u16,
    pub xfer_mode: u16,
    pub tout_ctrl: u8,
    pub clk_ctrl: u16,
    pub pwr_ctrl: u8,
    pub sw_rst: u8,
    pub normal_int: u16,
    pub error_int: u16,
    pub normal_int_stat_en: u16,
    pub error_int_stat_en: u16,
    pub normal_int_signal_en: u16,
    pub error_int_signal_en: u16,
    pub pstate: u32,
    pub cmd: u16,
    pub argument: u32,
    pub response01: u32,
    pub response23: u32,
    pub response45: u32,
    pub response67: u32,
}

#[derive(Clone, Copy)]
enum ResponseType {
    None = 0,
    Len48 = 2,
}

#[derive(Clone, Copy)]
enum CommandType {
    Normal = 0,
    Abort = 3,
}

#[derive(Clone, Copy)]
struct Command {
    index: u8,
    argument: u32,
    response: ResponseType,
    command_type: CommandType,
    crc_check: bool,
    index_check: bool,
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
    configure_sdhc1_clock(p);
    p.SDHC0.wrap.ctl.write(|w| w.enable().set_bit());
    p.SDHC1.wrap.ctl.write(|w| w.enable().set_bit());
    for _ in 0..1024 {
        cortex_m::asm::nop();
    }
}

pub fn initialize_card(p: &Peripherals) -> InitReport {
    configure_sdhc1_clock(p);
    configure_card_detect(p);
    configure_sdhc1_pins(p);

    let card_detect = card_detect_snapshot(p);
    if !card_detect.is_low {
        return init_report(p, InitStatus::NoCardDetect, 0, None, 0, 0, None);
    }

    p.SDHC1.wrap.ctl.write(|w| w.enable().set_bit());
    let core = &p.SDHC1.core;

    if !enable_internal_clock(core) {
        return init_report(p, InitStatus::ClockNotStable, 0, None, 0, 0, None);
    }

    if !software_reset_all(core) {
        return init_report(p, InitStatus::ResetTimeout, 0, None, 0, 0, None);
    }

    configure_host_for_identification(core);
    enable_card_power(core);
    change_card_clock(core, SD_INIT_CLOCK_DIVIDER);
    delay_ms(SD_BUS_RAMP_UP_MS);
    clear_interrupts(core);

    if let Err(error) = send_command(
        core,
        Command {
            index: 0,
            argument: 0,
            response: ResponseType::None,
            command_type: CommandType::Abort,
            crc_check: false,
            index_check: false,
        },
    ) {
        return init_report(p, InitStatus::Cmd0Failed, 0, None, 0, 0, Some(error));
    }

    let _ = software_reset_command_line(core);

    let mut cmd8_error = None;
    let mut cmd8_is_valid = false;
    let cmd8_response = match send_command(
        core,
        Command {
            index: 8,
            argument: SD_CMD8_ARGUMENT,
            response: ResponseType::Len48,
            command_type: CommandType::Normal,
            crc_check: true,
            index_check: true,
        },
    ) {
        Ok(response) => {
            if response & SD_CMD8_PATTERN_MASK != SD_CMD8_PATTERN {
                return init_report(
                    p,
                    InitStatus::Cmd8PatternMismatch,
                    response,
                    None,
                    0,
                    0,
                    None,
                );
            }
            cmd8_is_valid = true;
            response
        }
        Err(error) => {
            cmd8_error = Some(error);
            let _ = software_reset_command_line(core);
            0
        }
    };

    let mut acmd41_argument = SD_ACMD41_VOLTAGE_MASK;
    if cmd8_is_valid {
        acmd41_argument |= SD_ACMD41_HCS;
    }
    let mut acmd41_ocr = 0;
    let mut acmd41_attempts = 0;

    while acmd41_attempts < SD_ACMD41_MAX_ATTEMPTS {
        acmd41_attempts += 1;

        if let Err(error) = send_app_command(core) {
            return init_report(
                p,
                InitStatus::Acmd41Failed,
                cmd8_response,
                cmd8_error,
                acmd41_ocr,
                acmd41_attempts,
                Some(error),
            );
        }

        acmd41_ocr = match send_command(
            core,
            Command {
                index: 41,
                argument: acmd41_argument,
                response: ResponseType::Len48,
                command_type: CommandType::Normal,
                crc_check: false,
                index_check: false,
            },
        ) {
            Ok(response) => response,
            Err(error) => {
                return init_report(
                    p,
                    InitStatus::Acmd41Failed,
                    cmd8_response,
                    cmd8_error,
                    acmd41_ocr,
                    acmd41_attempts,
                    Some(error),
                );
            }
        };

        if acmd41_ocr & SD_OCR_BUSY != 0 {
            let status = if acmd41_ocr & SD_OCR_CAPACITY != 0 {
                InitStatus::ReadySdhc
            } else {
                InitStatus::ReadySdsc
            };
            return init_report(
                p,
                status,
                cmd8_response,
                cmd8_error,
                acmd41_ocr,
                acmd41_attempts,
                None,
            );
        }

        delay_ms(1);
    }

    init_report(
        p,
        InitStatus::Acmd41Busy,
        cmd8_response,
        cmd8_error,
        acmd41_ocr,
        acmd41_attempts,
        None,
    )
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

pub fn sdhc1_clock_snapshot(p: &Peripherals) -> SdhcClockSnapshot {
    SdhcClockSnapshot {
        path0: p.SRSS.clk_path_select[0].read().bits(),
        root0: p.SRSS.clk_root_select[0].read().bits(),
        root2: p.SRSS.clk_root_select[SDHC1_HF_CLOCK_INDEX].read().bits(),
        fll_config: p.SRSS.clk_fll_config.read().bits(),
        fll_config2: p.SRSS.clk_fll_config2.read().bits(),
        fll_status: p.SRSS.clk_fll_status.read().bits(),
        selected_hf_hz: SDHC1_HF_CLOCK_HZ,
    }
}

fn configure_sdhc1_clock(p: &Peripherals) {
    p.SRSS.clk_root_select[SDHC1_HF_CLOCK_INDEX]
        .write(|w| unsafe { w.bits(CLK_ROOT_ENABLE | CLK_ROOT_MUX_PATH0 | CLK_ROOT_DIV_BY_2) });
}

fn init_report(
    p: &Peripherals,
    status: InitStatus,
    cmd8_response: u32,
    cmd8_error: Option<CommandError>,
    acmd41_ocr: u32,
    acmd41_attempts: u16,
    last_error: Option<CommandError>,
) -> InitReport {
    let core = &p.SDHC1.core;

    InitReport {
        status,
        cmd8_response,
        cmd8_error,
        acmd41_ocr,
        acmd41_attempts,
        last_error,
        gp_out: core.gp_out_r.read().bits(),
        gp_in: core.gp_in_r.read().bits(),
        host_ctrl1: core.host_ctrl1_r.read().bits(),
        host_ctrl2: core.host_ctrl2_r.read().bits(),
        xfer_mode: core.xfer_mode_r.read().bits(),
        tout_ctrl: core.tout_ctrl_r.read().bits(),
        clk_ctrl: core.clk_ctrl_r.read().bits(),
        pwr_ctrl: core.pwr_ctrl_r.read().bits(),
        sw_rst: core.sw_rst_r.read().bits(),
        normal_int: core.normal_int_stat_r.read().bits(),
        error_int: core.error_int_stat_r.read().bits(),
        normal_int_stat_en: core.normal_int_stat_en_r.read().bits(),
        error_int_stat_en: core.error_int_stat_en_r.read().bits(),
        normal_int_signal_en: core.normal_int_signal_en_r.read().bits(),
        error_int_signal_en: core.error_int_signal_en_r.read().bits(),
        pstate: core.pstate_reg.read().bits(),
        cmd: core.cmd_r.read().bits(),
        argument: core.argument_r.read().bits(),
        response01: core.resp01_r.read().bits(),
        response23: core.resp23_r.read().bits(),
        response45: core.resp45_r.read().bits(),
        response67: core.resp67_r.read().bits(),
    }
}

fn configure_host_for_identification(core: &sdhc0::CORE) {
    core.gp_out_r.write(|w| unsafe { w.bits(GP_OUT_BASIC_SD) });
    core.xfer_mode_r.write(|w| unsafe { w.bits(0) });
    core.host_ctrl1_r.write(|w| unsafe {
        w.bits(HOST_CTRL1_CARD_DETECT_TEST_LEVEL | HOST_CTRL1_CARD_DETECT_SIGNAL_SELECT)
    });
    core.tout_ctrl_r.write(|w| unsafe { w.bits(0x0e) });
    core.normal_int_stat_en_r
        .write(|w| unsafe { w.bits(NORMAL_INT_ALL) });
    core.error_int_stat_en_r
        .write(|w| unsafe { w.bits(ERROR_INT_ALL) });
    core.normal_int_signal_en_r.write(|w| unsafe { w.bits(0) });
    core.error_int_signal_en_r.write(|w| unsafe { w.bits(0) });
    core.host_ctrl2_r
        .write(|w| unsafe { w.bits(HOST_CTRL2_HOST_VERSION_4_ENABLE) });
}

fn enable_card_power(core: &sdhc0::CORE) {
    core.pwr_ctrl_r.write(|w| unsafe { w.bits(1) });
}

fn enable_internal_clock(core: &sdhc0::CORE) -> bool {
    core.clk_ctrl_r
        .modify(|r, w| unsafe { w.bits(r.bits() | CLK_CTRL_INTERNAL_CLK_EN) });
    wait_for_clock_stable(core)
}

fn wait_for_clock_stable(core: &sdhc0::CORE) -> bool {
    for _ in 0..1000 {
        if core.clk_ctrl_r.read().bits() & CLK_CTRL_INTERNAL_CLK_STABLE != 0 {
            return true;
        }
        delay_us(3);
    }

    false
}

fn software_reset_all(core: &sdhc0::CORE) -> bool {
    core.clk_ctrl_r.write(|w| unsafe { w.bits(0) });
    delay_us(10);
    core.sw_rst_r.write(|w| w.sw_rst_all().set_bit());

    for _ in 0..1000 {
        if core.sw_rst_r.read().sw_rst_all().bit_is_clear() {
            core.clk_ctrl_r
                .write(|w| unsafe { w.bits(CLK_CTRL_INTERNAL_CLK_EN) });
            return wait_for_clock_stable(core);
        }
        delay_us(3);
    }

    false
}

fn software_reset_command_line(core: &sdhc0::CORE) -> bool {
    core.sw_rst_r.write(|w| w.sw_rst_cmd().set_bit());

    for _ in 0..1000 {
        if core.sw_rst_r.read().sw_rst_cmd().bit_is_clear() {
            return true;
        }
        delay_us(3);
    }

    false
}

fn change_card_clock(core: &sdhc0::CORE, divider: u16) {
    let mut clk_ctrl = core.clk_ctrl_r.read().bits() & !(CLK_CTRL_SD_CLK_EN | CLK_CTRL_PLL_ENABLE);
    core.clk_ctrl_r.write(|w| unsafe { w.bits(clk_ctrl) });

    clk_ctrl &= !((0xff << 8) | (0x03 << 6));
    clk_ctrl |= (divider & 0xff) << 8;
    clk_ctrl |= ((divider >> 8) & 0x03) << 6;
    core.clk_ctrl_r.write(|w| unsafe { w.bits(clk_ctrl) });

    delay_us(10);
    core.clk_ctrl_r
        .write(|w| unsafe { w.bits(clk_ctrl | CLK_CTRL_PLL_ENABLE | CLK_CTRL_SD_CLK_EN) });
}

fn send_app_command(core: &sdhc0::CORE) -> Result<u32, CommandError> {
    send_command(
        core,
        Command {
            index: 55,
            argument: 0,
            response: ResponseType::Len48,
            command_type: CommandType::Normal,
            crc_check: true,
            index_check: true,
        },
    )
}

fn send_command(core: &sdhc0::CORE, command: Command) -> Result<u32, CommandError> {
    poll_command_line_free(core)?;
    clear_interrupts(core);

    let command_value = command_register_value(command);
    core.argument_r
        .write(|w| unsafe { w.bits(command.argument) });
    core.cmd_r.write(|w| unsafe { w.bits(command_value) });
    let trace = CommandTrace {
        command: command_value,
        argument: command.argument,
        pstate_after_write: core.pstate_reg.read().bits(),
        normal_int_after_write: core.normal_int_stat_r.read().bits(),
        error_int_after_write: core.error_int_stat_r.read().bits(),
    };
    delay_us(50);

    wait_command_complete(core, matches!(command.response, ResponseType::None), trace)?;
    delay_us(20);

    Ok(core.resp01_r.read().bits())
}

fn command_register_value(command: Command) -> u16 {
    let mut bits = ((command.index as u16) & 0x3f) << 8;
    bits |= (command.response as u16) & 0x03;
    bits |= ((command.command_type as u16) & 0x03) << 6;

    if command.crc_check {
        bits |= 1 << 3;
    }

    if command.index_check {
        bits |= 1 << 4;
    }

    bits
}

fn poll_command_line_free(core: &sdhc0::CORE) -> Result<(), CommandError> {
    for _ in 0..1000 {
        if core.pstate_reg.read().bits() & PSTATE_CMD_INHIBIT == 0 {
            return Ok(());
        }
        delay_us(3);
    }

    Err(command_error(
        core,
        CommandErrorCode::CommandLineBusy,
        CommandTrace::from_registers(core),
    ))
}

fn wait_command_complete(
    core: &sdhc0::CORE,
    allow_inhibit_fallback: bool,
    trace: CommandTrace,
) -> Result<(), CommandError> {
    for _ in 0..1000 {
        let normal_int = core.normal_int_stat_r.read().bits();
        let error_int = core.error_int_stat_r.read().bits();
        let pstate = core.pstate_reg.read().bits();

        if error_int != 0 || normal_int & NORMAL_INT_ERROR != 0 {
            let error = command_error(core, CommandErrorCode::CommandStatusError, trace);
            clear_interrupts(core);
            let _ = software_reset_command_line(core);
            return Err(error);
        }

        if normal_int & NORMAL_INT_CMD_COMPLETE != 0 {
            core.normal_int_stat_r
                .write(|w| unsafe { w.bits(NORMAL_INT_CMD_COMPLETE) });
            return Ok(());
        }

        if allow_inhibit_fallback && pstate & PSTATE_CMD_INHIBIT == 0 {
            delay_us(20);
            let normal_int = core.normal_int_stat_r.read().bits();
            let error_int = core.error_int_stat_r.read().bits();

            if error_int == 0 && normal_int & NORMAL_INT_ERROR == 0 {
                return Ok(());
            }
        }

        delay_us(3);
    }

    let error = command_error(core, CommandErrorCode::CommandTimeout, trace);
    let _ = software_reset_command_line(core);
    Err(error)
}

fn clear_interrupts(core: &sdhc0::CORE) {
    core.normal_int_stat_r
        .write(|w| unsafe { w.bits(NORMAL_INT_ALL) });
    core.error_int_stat_r
        .write(|w| unsafe { w.bits(ERROR_INT_ALL) });
}

fn command_error(core: &sdhc0::CORE, code: CommandErrorCode, trace: CommandTrace) -> CommandError {
    CommandError {
        code,
        normal_int: core.normal_int_stat_r.read().bits(),
        error_int: core.error_int_stat_r.read().bits(),
        pstate: core.pstate_reg.read().bits(),
        command: trace.command,
        argument: trace.argument,
        pstate_after_write: trace.pstate_after_write,
        normal_int_after_write: trace.normal_int_after_write,
        error_int_after_write: trace.error_int_after_write,
    }
}

impl CommandTrace {
    fn from_registers(core: &sdhc0::CORE) -> Self {
        Self {
            command: core.cmd_r.read().bits(),
            argument: core.argument_r.read().bits(),
            pstate_after_write: core.pstate_reg.read().bits(),
            normal_int_after_write: core.normal_int_stat_r.read().bits(),
            error_int_after_write: core.error_int_stat_r.read().bits(),
        }
    }
}

fn delay_ms(ms: u32) {
    for _ in 0..ms {
        delay_us(1000);
    }
}

fn delay_us(us: u32) {
    cortex_m::asm::delay((CPU_CLOCK_HZ / 1_000_000) * us);
}
