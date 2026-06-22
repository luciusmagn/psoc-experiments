use psoc6_pac::{sdhc0, Peripherals};

const HSIOM_SEL_SDHC: u32 = 26;
const DRIVE_STRONG: u32 = 0x06;
const DRIVE_STRONG_INPUT: u32 = 0x0e;

const NORMAL_INT_ALL: u16 = 0x1fff;
const ERROR_INT_ALL: u16 = 0x07ff;
const NORMAL_INT_CMD_COMPLETE: u16 = 1 << 0;
const NORMAL_INT_XFER_COMPLETE: u16 = 1 << 1;
const NORMAL_INT_BUF_RD_READY: u16 = 1 << 5;
const NORMAL_INT_ERROR: u16 = 1 << 15;
const PSTATE_CMD_INHIBIT: u32 = 1 << 0;
const PSTATE_CMD_INHIBIT_DAT: u32 = 1 << 1;
const PSTATE_DAT_LINE_ACTIVE: u32 = 1 << 2;
const PSTATE_BUF_RD_ENABLE: u32 = 1 << 11;
const CLK_CTRL_INTERNAL_CLK_EN: u16 = 1 << 0;
const CLK_CTRL_INTERNAL_CLK_STABLE: u16 = 1 << 1;
const CLK_CTRL_SD_CLK_EN: u16 = 1 << 2;
const CLK_CTRL_PLL_ENABLE: u16 = 1 << 3;
const HOST_CTRL1_CARD_DETECT_TEST_LEVEL: u8 = 1 << 6;
const HOST_CTRL1_CARD_DETECT_SIGNAL_SELECT: u8 = 1 << 7;
const HOST_CTRL1_DATA_TRANSFER_WIDTH_4BIT: u8 = 1 << 1;
const HOST_CTRL2_HOST_VERSION_4_ENABLE: u16 = 1 << 12;
const GP_OUT_BASIC_SD: u32 = (1 << 0) | (1 << 1) | (1 << 3) | (1 << 4) | (1 << 5);

const SDHC_INPUT_CLOCK_HZ: u32 = 50_000_000;
const SD_INIT_CLOCK_HZ: u32 = 400_000;
const SD_INIT_CLOCK_DIVIDER: u16 = ((SDHC_INPUT_CLOCK_HZ / SD_INIT_CLOCK_HZ) >> 1) as u16;
const SDIO_OCR_VOLTAGE_MASK: u32 = 0x00ff_8000;
const SDIO_OCR_BUSY: u32 = 1 << 31;
const SDIO_OCR_MEMORY_PRESENT: u32 = 1 << 27;
const SDIO_OCR_FUNCTIONS_SHIFT: u8 = 28;
const SD_RCA_SHIFT: u8 = 16;
const SDIO_CMD5_MAX_ATTEMPTS: u16 = 1000;
const SDIO_CMD52_RW_FLAG: u32 = 1 << 31;
const SDIO_CMD52_FUNCTION_SHIFT: u8 = 28;
const SDIO_CMD52_RAW_FLAG: u32 = 1 << 27;
const SDIO_CMD52_ADDRESS_SHIFT: u8 = 9;
const SDIO_CMD52_MAX_FUNCTION: u8 = 7;
const SDIO_CMD52_MAX_ADDRESS: u32 = 0x1ffff;
const SDIO_CMD53_RW_FLAG: u32 = 1 << 31;
const SDIO_CMD53_FUNCTION_SHIFT: u8 = 28;
const SDIO_CMD53_BLOCK_MODE: u32 = 1 << 27;
const SDIO_CMD53_INCREMENTING_ADDRESS: u32 = 1 << 26;
const SDIO_CMD53_ADDRESS_SHIFT: u8 = 9;
const SDIO_CMD53_MAX_COUNT: u16 = 16;
const SDIO_CMD53_WORD_BYTES: u16 = 4;
const SDIO_CCCR_IO_ENABLE: u32 = 0x02;
const SDIO_CCCR_IO_READY: u32 = 0x03;
const SDIO_CCCR_INTERRUPT_ENABLE: u32 = 0x04;
const SDIO_CCCR_BUS_INTERFACE_CONTROL: u32 = 0x07;
const SDIO_CCCR_BLOCK_SIZE_LOW: u32 = 0x10;
const SDIO_CCCR_BLOCK_SIZE_HIGH: u32 = 0x11;
const SDIO_CCCR_F1_BLOCK_SIZE_LOW: u32 = 0x110;
const SDIO_CCCR_F1_BLOCK_SIZE_HIGH: u32 = 0x111;
const SDIO_CCCR_F2_BLOCK_SIZE_LOW: u32 = 0x210;
const SDIO_CCCR_F2_BLOCK_SIZE_HIGH: u32 = 0x211;
const SDIO_CHIP_CLOCK_CSR: u32 = 0x1000e;
const SDIO_FUNCTION_ENABLE_1: u8 = 0x02;
const SDIO_FUNCTION_READY_1: u8 = 0x02;
const SDIO_INTERRUPT_MASTER_FUNC1_FUNC2: u8 = 0x07;
const SDIO_BUS_WIDTH_MASK: u8 = 0x03;
const SDIO_BUS_WIDTH_4BIT: u8 = 0x02;
const SDIO_BLOCK_SIZE_64: u8 = 64;
const SDIO_READY_MAX_ATTEMPTS: u16 = 1000;
const SDIO_BACKPLANE_SETUP_MAX_ATTEMPTS: u16 = 1000;
const SDIO_ALP_AVAIL_MAX_ATTEMPTS: u16 = 100;
const SBSDIO_FORCE_ALP: u8 = 0x01;
const SBSDIO_ALP_AVAIL_REQ: u8 = 0x08;
const SBSDIO_FORCE_HW_CLKREQ_OFF: u8 = 0x20;
const SBSDIO_ALP_AVAIL: u8 = 0x40;
const XFER_MODE_BLOCK_COUNT_ENABLE: u16 = 1 << 1;
const XFER_MODE_READ: u16 = 1 << 4;

const P2_SDIO_DATA_MASK: u32 = 0x0f | (0x0f << 4) | (0x0f << 8) | (0x0f << 12);
const P2_SDIO_DATA_CFG: u32 = DRIVE_STRONG_INPUT
    | (DRIVE_STRONG_INPUT << 4)
    | (DRIVE_STRONG_INPUT << 8)
    | (DRIVE_STRONG_INPUT << 12);
const P2_SDIO_CMD_CLK_MASK: u32 = (0x0f << 16) | (0x0f << 20);
const P2_SDIO_CMD_CLK_CFG: u32 = (DRIVE_STRONG_INPUT << 16) | (DRIVE_STRONG_INPUT << 20);
const P2_WIFI_REG_ON_MASK: u32 = 0x0f << 24;
const P2_WIFI_REG_ON_CFG: u32 = DRIVE_STRONG << 24;

const P2_SDIO_DATA_HSIOM_MASK: u32 = 0xff | (0xff << 8) | (0xff << 16) | (0xff << 24);
const P2_SDIO_DATA_HSIOM: u32 =
    HSIOM_SEL_SDHC | (HSIOM_SEL_SDHC << 8) | (HSIOM_SEL_SDHC << 16) | (HSIOM_SEL_SDHC << 24);
const P2_SDIO_CMD_CLK_HSIOM_MASK: u32 = 0xff | (0xff << 8);
const P2_SDIO_CMD_CLK_HSIOM: u32 = HSIOM_SEL_SDHC | (HSIOM_SEL_SDHC << 8);

const CLK_ROOT_ENABLE: u32 = 1 << 31;
const CLK_ROOT_MUX_PATH0: u32 = 0;
const CLK_ROOT_DIV_BY_2: u32 = 1 << 4;
const SDHC0_HF_CLOCK_INDEX: usize = 4;

#[derive(Clone, Copy)]
pub enum WifiSdioStatus {
    Ready,
    ClockNotStable,
    ResetTimeout,
    Cmd0Failed,
    Cmd5Failed,
    Cmd5Busy,
    Cmd3Failed,
    Cmd7Failed,
    SelectBusy,
}

#[derive(Clone, Copy)]
pub enum CommandErrorCode {
    CommandLineBusy,
    CommandTimeout,
    CommandStatusError,
}

#[derive(Clone, Copy)]
pub enum WifiSdioDirectStatus {
    Ready,
    InitFailed,
    InvalidFunction,
    InvalidAddress,
    Cmd52Failed,
}

#[derive(Clone, Copy)]
pub enum WifiSdioEnableStatus {
    Ready,
    InitFailed,
    WriteFailed,
    ReadyReadFailed,
    ReadyTimeout,
}

#[derive(Clone, Copy)]
pub enum WifiSdioBackplaneStatus {
    Ready,
    InitFailed,
    IoEnableWriteFailed,
    IoEnableReadFailed,
    IoEnableTimeout,
    BusControlReadFailed,
    BusControlWriteFailed,
    BlockSizeWriteFailed,
    BlockSizeReadFailed,
    BlockSizeTimeout,
    InterruptEnableWriteFailed,
    InterruptEnableReadFailed,
    ReadyReadFailed,
    ReadyTimeout,
}

#[derive(Clone, Copy)]
pub enum WifiSdioCmd53ReadStatus {
    Ready,
    SetupFailed,
    InvalidFunction,
    InvalidAddress,
    InvalidCount,
    DataSetupBusy,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    Cmd53Failed,
    BufferReadTimeout,
    BufferEnableTimeout,
    TransferTimeout,
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

pub struct WifiSdioHostSnapshot {
    pub wrap_ctl: u32,
    pub gp_out: u32,
    pub gp_in: u32,
    pub xfer_mode: u16,
    pub host_ctrl1: u8,
    pub host_ctrl2: u16,
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

pub struct WifiSdioPinSnapshot {
    pub p2_sel0: u32,
    pub p2_sel1: u32,
    pub p2_cfg: u32,
    pub p2_out: u32,
    pub p2_in: u32,
}

pub struct WifiSdioClockSnapshot {
    pub path0: u32,
    pub root0: u32,
    pub root1: u32,
    pub root2: u32,
    pub root3: u32,
    pub root4: u32,
    pub fll_config: u32,
    pub fll_config2: u32,
    pub fll_status: u32,
}

pub struct WifiSdioReport {
    pub status: WifiSdioStatus,
    pub cmd5_response: u32,
    pub cmd5_attempts: u16,
    pub rca: u16,
    pub function_count: u8,
    pub memory_present: bool,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
    pub pins: WifiSdioPinSnapshot,
    pub clock: WifiSdioClockSnapshot,
}

pub struct WifiSdioDirectReport {
    pub status: WifiSdioDirectStatus,
    pub init_status: WifiSdioStatus,
    pub function: u8,
    pub address: u32,
    pub write: bool,
    pub data: u8,
    pub response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioEnableReport {
    pub status: WifiSdioEnableStatus,
    pub init_status: WifiSdioStatus,
    pub requested: u8,
    pub ready: u8,
    pub attempts: u16,
    pub write_response: u32,
    pub ready_response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioBackplaneReport {
    pub status: WifiSdioBackplaneStatus,
    pub init_status: WifiSdioStatus,
    pub io_enable: u8,
    pub io_ready: u8,
    pub bus_control_before: u8,
    pub bus_control_after: u8,
    pub f0_block_size: u16,
    pub f1_block_size: u16,
    pub f2_block_size: u16,
    pub interrupt_enable: u8,
    pub attempts: u16,
    pub last_response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioCmd53ReadReport {
    pub status: WifiSdioCmd53ReadStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub function: u8,
    pub address: u32,
    pub count: u8,
    pub response: u32,
    pub bytes: [u8; SDIO_CMD53_MAX_COUNT as usize],
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

#[derive(Clone, Copy)]
enum ResponseType {
    None = 0,
    Len48 = 2,
    Len48Busy = 3,
}

#[derive(Clone, Copy)]
enum CommandType {
    Normal = 0,
}

#[derive(Clone, Copy)]
struct Command {
    index: u8,
    argument: u32,
    response: ResponseType,
    command_type: CommandType,
    data_present: bool,
    crc_check: bool,
    index_check: bool,
}

pub fn initialize(p: &Peripherals) -> WifiSdioReport {
    configure_wifi_power_pin(p);
    wifi_power_off(p);
    delay_ms(10);
    wifi_power_on(p);
    delay_ms(200);

    configure_sdhc0_clock(p);
    configure_sdio_pins(p);
    p.SDHC0.wrap.ctl.write(|w| w.enable().set_bit());
    let core = &p.SDHC0.core;

    if !enable_internal_clock(core) {
        return report(p, WifiSdioStatus::ClockNotStable, 0, 0, 0, None);
    }
    if !software_reset_all(core) {
        return report(p, WifiSdioStatus::ResetTimeout, 0, 0, 0, None);
    }

    configure_host_for_identification(core);
    enable_card_power(core);
    change_card_clock(core, SD_INIT_CLOCK_DIVIDER);
    delay_ms(1000);
    clear_interrupts(core);

    if let Err(error) = send_command(
        core,
        Command {
            index: 0,
            argument: 0,
            response: ResponseType::None,
            command_type: CommandType::Normal,
            data_present: false,
            crc_check: false,
            index_check: false,
        },
    ) {
        return report(p, WifiSdioStatus::Cmd0Failed, 0, 0, 0, Some(error));
    }

    let mut cmd5_response = 0;
    let mut cmd5_attempts = 0;
    while cmd5_attempts < SDIO_CMD5_MAX_ATTEMPTS {
        cmd5_attempts += 1;
        cmd5_response = match send_command(
            core,
            Command {
                index: 5,
                argument: SDIO_OCR_VOLTAGE_MASK,
                response: ResponseType::Len48,
                command_type: CommandType::Normal,
                data_present: false,
                crc_check: false,
                index_check: false,
            },
        ) {
            Ok(response) => response,
            Err(error) => {
                return report(
                    p,
                    WifiSdioStatus::Cmd5Failed,
                    cmd5_response,
                    cmd5_attempts,
                    0,
                    Some(error),
                );
            }
        };

        if cmd5_response & SDIO_OCR_BUSY != 0 {
            break;
        }
        delay_ms(1);
    }

    if cmd5_response & SDIO_OCR_BUSY == 0 {
        return report(
            p,
            WifiSdioStatus::Cmd5Busy,
            cmd5_response,
            cmd5_attempts,
            0,
            None,
        );
    }

    let rca_response = match send_command(
        core,
        Command {
            index: 3,
            argument: 0,
            response: ResponseType::Len48,
            command_type: CommandType::Normal,
            data_present: false,
            crc_check: true,
            index_check: true,
        },
    ) {
        Ok(response) => response,
        Err(error) => {
            return report(
                p,
                WifiSdioStatus::Cmd3Failed,
                cmd5_response,
                cmd5_attempts,
                0,
                Some(error),
            );
        }
    };
    let rca = (rca_response >> SD_RCA_SHIFT) as u16;

    if let Err(error) = send_command(
        core,
        Command {
            index: 7,
            argument: (rca as u32) << SD_RCA_SHIFT,
            response: ResponseType::Len48Busy,
            command_type: CommandType::Normal,
            data_present: false,
            crc_check: false,
            index_check: false,
        },
    ) {
        return report(
            p,
            WifiSdioStatus::Cmd7Failed,
            cmd5_response,
            cmd5_attempts,
            rca,
            Some(error),
        );
    }

    if !wait_transfer_complete(core) {
        return report(
            p,
            WifiSdioStatus::SelectBusy,
            cmd5_response,
            cmd5_attempts,
            rca,
            None,
        );
    }

    report(
        p,
        WifiSdioStatus::Ready,
        cmd5_response,
        cmd5_attempts,
        rca,
        None,
    )
}

pub fn cmd52_read(p: &Peripherals, function: u8, address: u32) -> WifiSdioDirectReport {
    cmd52_transfer(p, false, function, address, 0, false)
}

pub fn cmd52_write(p: &Peripherals, function: u8, address: u32, data: u8) -> WifiSdioDirectReport {
    cmd52_transfer(p, true, function, address, data, false)
}

pub fn enable_functions(p: &Peripherals, requested: u8) -> WifiSdioEnableReport {
    let init = initialize(p);
    if !matches!(init.status, WifiSdioStatus::Ready) {
        return enable_report(
            p,
            WifiSdioEnableStatus::InitFailed,
            init.status,
            requested,
            0,
            0,
            0,
            0,
            init.last_error,
        );
    }

    let write_response = match cmd52_selected(
        &p.SDHC0.core,
        true,
        0,
        SDIO_CCCR_IO_ENABLE,
        requested,
        false,
    ) {
        Ok(response) => response,
        Err(error) => {
            return enable_report(
                p,
                WifiSdioEnableStatus::WriteFailed,
                init.status,
                requested,
                0,
                0,
                0,
                0,
                Some(error),
            );
        }
    };

    let mut attempts = 0;
    let mut ready = 0;
    let mut ready_response = 0;
    while attempts < SDIO_READY_MAX_ATTEMPTS {
        attempts += 1;
        ready_response = match cmd52_selected(&p.SDHC0.core, false, 0, SDIO_CCCR_IO_READY, 0, false)
        {
            Ok(response) => response,
            Err(error) => {
                return enable_report(
                    p,
                    WifiSdioEnableStatus::ReadyReadFailed,
                    init.status,
                    requested,
                    ready,
                    attempts,
                    write_response,
                    ready_response,
                    Some(error),
                );
            }
        };
        ready = (ready_response & 0xff) as u8;
        if ready & requested == requested {
            return enable_report(
                p,
                WifiSdioEnableStatus::Ready,
                init.status,
                requested,
                ready,
                attempts,
                write_response,
                ready_response,
                None,
            );
        }
        delay_ms(1);
    }

    enable_report(
        p,
        WifiSdioEnableStatus::ReadyTimeout,
        init.status,
        requested,
        ready,
        attempts,
        write_response,
        ready_response,
        None,
    )
}

pub fn setup_backplane(p: &Peripherals) -> WifiSdioBackplaneReport {
    let init = initialize(p);
    if !matches!(init.status, WifiSdioStatus::Ready) {
        return backplane_report(
            p,
            WifiSdioBackplaneStatus::InitFailed,
            init.status,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            init.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let mut attempts = 0;
    let mut last_response = 0;
    let mut io_enable = 0;

    while attempts < SDIO_BACKPLANE_SETUP_MAX_ATTEMPTS {
        attempts += 1;
        if attempts > 1 {
            delay_ms(1);
        }

        last_response =
            match cmd52_write_byte_selected(core, SDIO_CCCR_IO_ENABLE, SDIO_FUNCTION_ENABLE_1) {
                Ok(response) => response,
                Err(error) => {
                    return backplane_report(
                        p,
                        WifiSdioBackplaneStatus::IoEnableWriteFailed,
                        init.status,
                        io_enable,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        attempts,
                        last_response,
                        Some(error),
                    )
                }
            };

        let read = match cmd52_read_byte_selected(core, SDIO_CCCR_IO_ENABLE) {
            Ok(read) => read,
            Err(error) => {
                return backplane_report(
                    p,
                    WifiSdioBackplaneStatus::IoEnableReadFailed,
                    init.status,
                    io_enable,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    attempts,
                    last_response,
                    Some(error),
                )
            }
        };
        io_enable = read.0;
        last_response = read.1;
        if io_enable & SDIO_FUNCTION_ENABLE_1 == SDIO_FUNCTION_ENABLE_1 {
            break;
        }
    }

    if io_enable & SDIO_FUNCTION_ENABLE_1 != SDIO_FUNCTION_ENABLE_1 {
        return backplane_report(
            p,
            WifiSdioBackplaneStatus::IoEnableTimeout,
            init.status,
            io_enable,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            attempts,
            last_response,
            None,
        );
    }

    let bus_control_before = match cmd52_read_byte_selected(core, SDIO_CCCR_BUS_INTERFACE_CONTROL) {
        Ok(read) => {
            last_response = read.1;
            read.0
        }
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::BusControlReadFailed,
                init.status,
                io_enable,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };

    let bus_control_requested = (bus_control_before & !SDIO_BUS_WIDTH_MASK) | SDIO_BUS_WIDTH_4BIT;
    last_response = match cmd52_write_byte_selected(
        core,
        SDIO_CCCR_BUS_INTERFACE_CONTROL,
        bus_control_requested,
    ) {
        Ok(response) => response,
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::BusControlWriteFailed,
                init.status,
                io_enable,
                0,
                bus_control_before,
                0,
                0,
                0,
                0,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };

    let bus_control_after = match cmd52_read_byte_selected(core, SDIO_CCCR_BUS_INTERFACE_CONTROL) {
        Ok(read) => {
            last_response = read.1;
            read.0
        }
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::BusControlReadFailed,
                init.status,
                io_enable,
                0,
                bus_control_before,
                0,
                0,
                0,
                0,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };
    enable_host_4bit(core);

    let mut f0_block_size = 0;
    attempts = 0;
    while attempts < SDIO_BACKPLANE_SETUP_MAX_ATTEMPTS {
        attempts += 1;
        last_response =
            match cmd52_write_byte_selected(core, SDIO_CCCR_BLOCK_SIZE_LOW, SDIO_BLOCK_SIZE_64) {
                Ok(response) => response,
                Err(error) => {
                    return backplane_report(
                        p,
                        WifiSdioBackplaneStatus::BlockSizeWriteFailed,
                        init.status,
                        io_enable,
                        0,
                        bus_control_before,
                        bus_control_after,
                        f0_block_size,
                        0,
                        0,
                        0,
                        attempts,
                        last_response,
                        Some(error),
                    )
                }
            };

        let read = match cmd52_read_byte_selected(core, SDIO_CCCR_BLOCK_SIZE_LOW) {
            Ok(read) => read,
            Err(error) => {
                return backplane_report(
                    p,
                    WifiSdioBackplaneStatus::BlockSizeReadFailed,
                    init.status,
                    io_enable,
                    0,
                    bus_control_before,
                    bus_control_after,
                    f0_block_size,
                    0,
                    0,
                    0,
                    attempts,
                    last_response,
                    Some(error),
                )
            }
        };
        f0_block_size = read.0 as u16;
        last_response = read.1;
        if read.0 == SDIO_BLOCK_SIZE_64 {
            break;
        }
        delay_ms(1);
    }

    if f0_block_size != SDIO_BLOCK_SIZE_64 as u16 {
        return backplane_report(
            p,
            WifiSdioBackplaneStatus::BlockSizeTimeout,
            init.status,
            io_enable,
            0,
            bus_control_before,
            bus_control_after,
            f0_block_size,
            0,
            0,
            0,
            attempts,
            last_response,
            None,
        );
    }

    if let Err(error) = write_block_size(core, SDIO_CCCR_BLOCK_SIZE_LOW, SDIO_CCCR_BLOCK_SIZE_HIGH)
    {
        return backplane_report(
            p,
            WifiSdioBackplaneStatus::BlockSizeWriteFailed,
            init.status,
            io_enable,
            0,
            bus_control_before,
            bus_control_after,
            f0_block_size,
            0,
            0,
            0,
            attempts,
            last_response,
            Some(error),
        );
    }
    if let Err(error) = write_block_size(
        core,
        SDIO_CCCR_F1_BLOCK_SIZE_LOW,
        SDIO_CCCR_F1_BLOCK_SIZE_HIGH,
    ) {
        return backplane_report(
            p,
            WifiSdioBackplaneStatus::BlockSizeWriteFailed,
            init.status,
            io_enable,
            0,
            bus_control_before,
            bus_control_after,
            f0_block_size,
            0,
            0,
            0,
            attempts,
            last_response,
            Some(error),
        );
    }
    if let Err(error) = write_block_size(
        core,
        SDIO_CCCR_F2_BLOCK_SIZE_LOW,
        SDIO_CCCR_F2_BLOCK_SIZE_HIGH,
    ) {
        return backplane_report(
            p,
            WifiSdioBackplaneStatus::BlockSizeWriteFailed,
            init.status,
            io_enable,
            0,
            bus_control_before,
            bus_control_after,
            f0_block_size,
            0,
            0,
            0,
            attempts,
            last_response,
            Some(error),
        );
    }

    f0_block_size = match read_block_size(core, SDIO_CCCR_BLOCK_SIZE_LOW, SDIO_CCCR_BLOCK_SIZE_HIGH)
    {
        Ok(read) => {
            last_response = read.1;
            read.0
        }
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::BlockSizeReadFailed,
                init.status,
                io_enable,
                0,
                bus_control_before,
                bus_control_after,
                f0_block_size,
                0,
                0,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };
    let f1_block_size = match read_block_size(
        core,
        SDIO_CCCR_F1_BLOCK_SIZE_LOW,
        SDIO_CCCR_F1_BLOCK_SIZE_HIGH,
    ) {
        Ok(read) => {
            last_response = read.1;
            read.0
        }
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::BlockSizeReadFailed,
                init.status,
                io_enable,
                0,
                bus_control_before,
                bus_control_after,
                f0_block_size,
                0,
                0,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };
    let f2_block_size = match read_block_size(
        core,
        SDIO_CCCR_F2_BLOCK_SIZE_LOW,
        SDIO_CCCR_F2_BLOCK_SIZE_HIGH,
    ) {
        Ok(read) => {
            last_response = read.1;
            read.0
        }
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::BlockSizeReadFailed,
                init.status,
                io_enable,
                0,
                bus_control_before,
                bus_control_after,
                f0_block_size,
                f1_block_size,
                0,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };

    last_response = match cmd52_write_byte_selected(
        core,
        SDIO_CCCR_INTERRUPT_ENABLE,
        SDIO_INTERRUPT_MASTER_FUNC1_FUNC2,
    ) {
        Ok(response) => response,
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::InterruptEnableWriteFailed,
                init.status,
                io_enable,
                0,
                bus_control_before,
                bus_control_after,
                f0_block_size,
                f1_block_size,
                f2_block_size,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };
    let interrupt_enable = match cmd52_read_byte_selected(core, SDIO_CCCR_INTERRUPT_ENABLE) {
        Ok(read) => {
            last_response = read.1;
            read.0
        }
        Err(error) => {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::InterruptEnableReadFailed,
                init.status,
                io_enable,
                0,
                bus_control_before,
                bus_control_after,
                f0_block_size,
                f1_block_size,
                f2_block_size,
                0,
                attempts,
                last_response,
                Some(error),
            )
        }
    };

    attempts = 0;
    let mut io_ready = 0;
    while attempts < SDIO_BACKPLANE_SETUP_MAX_ATTEMPTS {
        attempts += 1;
        let read = match cmd52_read_byte_selected(core, SDIO_CCCR_IO_READY) {
            Ok(read) => read,
            Err(error) => {
                return backplane_report(
                    p,
                    WifiSdioBackplaneStatus::ReadyReadFailed,
                    init.status,
                    io_enable,
                    io_ready,
                    bus_control_before,
                    bus_control_after,
                    f0_block_size,
                    f1_block_size,
                    f2_block_size,
                    interrupt_enable,
                    attempts,
                    last_response,
                    Some(error),
                )
            }
        };
        io_ready = read.0;
        last_response = read.1;
        if io_ready & SDIO_FUNCTION_READY_1 == SDIO_FUNCTION_READY_1 {
            return backplane_report(
                p,
                WifiSdioBackplaneStatus::Ready,
                init.status,
                io_enable,
                io_ready,
                bus_control_before,
                bus_control_after,
                f0_block_size,
                f1_block_size,
                f2_block_size,
                interrupt_enable,
                attempts,
                last_response,
                None,
            );
        }
        delay_ms(1);
    }

    backplane_report(
        p,
        WifiSdioBackplaneStatus::ReadyTimeout,
        init.status,
        io_enable,
        io_ready,
        bus_control_before,
        bus_control_after,
        f0_block_size,
        f1_block_size,
        f2_block_size,
        interrupt_enable,
        attempts,
        last_response,
        None,
    )
}

pub fn cmd53_read(
    p: &Peripherals,
    function: u8,
    address: u32,
    count: u8,
) -> WifiSdioCmd53ReadReport {
    if function == 0 || function > SDIO_CMD52_MAX_FUNCTION {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::InvalidFunction,
            WifiSdioBackplaneStatus::Ready,
            function,
            address,
            count,
            0,
            [0; SDIO_CMD53_MAX_COUNT as usize],
            None,
        );
    }

    if address > SDIO_CMD52_MAX_ADDRESS {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::InvalidAddress,
            WifiSdioBackplaneStatus::Ready,
            function,
            address,
            count,
            0,
            [0; SDIO_CMD53_MAX_COUNT as usize],
            None,
        );
    }

    if !is_valid_cmd53_read_count(count) {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::InvalidCount,
            WifiSdioBackplaneStatus::Ready,
            function,
            address,
            count,
            0,
            [0; SDIO_CMD53_MAX_COUNT as usize],
            None,
        );
    }

    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::SetupFailed,
            setup.status,
            function,
            address,
            count,
            0,
            [0; SDIO_CMD53_MAX_COUNT as usize],
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let alp_request = SBSDIO_FORCE_HW_CLKREQ_OFF | SBSDIO_ALP_AVAIL_REQ | SBSDIO_FORCE_ALP;
    let mut alp_response =
        match cmd52_write_function_byte_selected(core, 1, SDIO_CHIP_CLOCK_CSR, alp_request) {
            Ok(response) => response,
            Err(error) => {
                return cmd53_read_report(
                    p,
                    WifiSdioCmd53ReadStatus::AlpWriteFailed,
                    setup.status,
                    function,
                    address,
                    count,
                    0,
                    [0; SDIO_CMD53_MAX_COUNT as usize],
                    Some(error),
                )
            }
        };

    let mut alp_attempts = 0;
    let mut clock_csr = 0;
    while alp_attempts < SDIO_ALP_AVAIL_MAX_ATTEMPTS {
        alp_attempts += 1;
        let read = match cmd52_read_function_byte_selected(core, 1, SDIO_CHIP_CLOCK_CSR) {
            Ok(read) => read,
            Err(error) => {
                return cmd53_read_report(
                    p,
                    WifiSdioCmd53ReadStatus::AlpReadFailed,
                    setup.status,
                    function,
                    address,
                    count,
                    alp_response,
                    [0; SDIO_CMD53_MAX_COUNT as usize],
                    Some(error),
                )
            }
        };
        clock_csr = read.0;
        alp_response = read.1;
        if clock_csr & SBSDIO_ALP_AVAIL != 0 {
            break;
        }
        delay_ms(1);
    }

    if clock_csr & SBSDIO_ALP_AVAIL == 0 {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::AlpTimeout,
            setup.status,
            function,
            address,
            count,
            alp_response,
            [0; SDIO_CMD53_MAX_COUNT as usize],
            None,
        );
    }

    if let Err(error) = cmd52_write_function_byte_selected(core, 1, SDIO_CHIP_CLOCK_CSR, 0) {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::AlpClearFailed,
            setup.status,
            function,
            address,
            count,
            alp_response,
            [0; SDIO_CMD53_MAX_COUNT as usize],
            Some(error),
        );
    }

    if !configure_cmd53_byte_read(core, count) {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::DataSetupBusy,
            setup.status,
            function,
            address,
            count,
            0,
            [0; SDIO_CMD53_MAX_COUNT as usize],
            None,
        );
    }

    let argument = cmd53_argument(false, function, false, true, address, count as u16);
    let response = match send_command(
        core,
        Command {
            index: 53,
            argument,
            response: ResponseType::Len48,
            command_type: CommandType::Normal,
            data_present: true,
            crc_check: true,
            index_check: true,
        },
    ) {
        Ok(response) => response,
        Err(error) => {
            return cmd53_read_report(
                p,
                WifiSdioCmd53ReadStatus::Cmd53Failed,
                setup.status,
                function,
                address,
                count,
                0,
                [0; SDIO_CMD53_MAX_COUNT as usize],
                Some(error),
            )
        }
    };

    let bytes = match read_cmd53_bytes(core, count) {
        Ok(bytes) => bytes,
        Err(status) => {
            return cmd53_read_report(
                p,
                status,
                setup.status,
                function,
                address,
                count,
                response,
                [0; SDIO_CMD53_MAX_COUNT as usize],
                None,
            )
        }
    };

    cmd53_read_report(
        p,
        WifiSdioCmd53ReadStatus::Ready,
        setup.status,
        function,
        address,
        count,
        response,
        bytes,
        None,
    )
}

fn cmd52_transfer(
    p: &Peripherals,
    write: bool,
    function: u8,
    address: u32,
    data: u8,
    raw: bool,
) -> WifiSdioDirectReport {
    if function > SDIO_CMD52_MAX_FUNCTION {
        return direct_report(
            p,
            WifiSdioDirectStatus::InvalidFunction,
            WifiSdioStatus::Ready,
            function,
            address,
            write,
            data,
            0,
            None,
        );
    }

    if address > SDIO_CMD52_MAX_ADDRESS {
        return direct_report(
            p,
            WifiSdioDirectStatus::InvalidAddress,
            WifiSdioStatus::Ready,
            function,
            address,
            write,
            data,
            0,
            None,
        );
    }

    let init = initialize(p);
    if !matches!(init.status, WifiSdioStatus::Ready) {
        return direct_report(
            p,
            WifiSdioDirectStatus::InitFailed,
            init.status,
            function,
            address,
            write,
            data,
            0,
            init.last_error,
        );
    }

    match cmd52_selected(&p.SDHC0.core, write, function, address, data, raw) {
        Ok(response) => direct_report(
            p,
            WifiSdioDirectStatus::Ready,
            init.status,
            function,
            address,
            write,
            (response & 0xff) as u8,
            response,
            None,
        ),
        Err(error) => direct_report(
            p,
            WifiSdioDirectStatus::Cmd52Failed,
            init.status,
            function,
            address,
            write,
            data,
            0,
            Some(error),
        ),
    }
}

fn cmd52_selected(
    core: &sdhc0::CORE,
    write: bool,
    function: u8,
    address: u32,
    data: u8,
    raw: bool,
) -> Result<u32, CommandError> {
    let argument = cmd52_argument(write, function, raw, address, data);
    send_command(
        core,
        Command {
            index: 52,
            argument,
            response: ResponseType::Len48,
            command_type: CommandType::Normal,
            data_present: false,
            crc_check: true,
            index_check: true,
        },
    )
}

fn cmd52_argument(write: bool, function: u8, raw: bool, address: u32, data: u8) -> u32 {
    let mut argument = ((function as u32) & 0x07) << SDIO_CMD52_FUNCTION_SHIFT;
    argument |= (address & SDIO_CMD52_MAX_ADDRESS) << SDIO_CMD52_ADDRESS_SHIFT;
    argument |= data as u32;

    if write {
        argument |= SDIO_CMD52_RW_FLAG;
    }
    if raw {
        argument |= SDIO_CMD52_RAW_FLAG;
    }

    argument
}

fn cmd53_argument(
    write: bool,
    function: u8,
    block_mode: bool,
    incrementing_address: bool,
    address: u32,
    count: u16,
) -> u32 {
    let mut argument = ((function as u32) & 0x07) << SDIO_CMD53_FUNCTION_SHIFT;
    argument |= (address & SDIO_CMD52_MAX_ADDRESS) << SDIO_CMD53_ADDRESS_SHIFT;
    argument |= (count as u32) & 0x01ff;

    if write {
        argument |= SDIO_CMD53_RW_FLAG;
    }
    if block_mode {
        argument |= SDIO_CMD53_BLOCK_MODE;
    }
    if incrementing_address {
        argument |= SDIO_CMD53_INCREMENTING_ADDRESS;
    }

    argument
}

fn is_valid_cmd53_read_count(count: u8) -> bool {
    count > 0 && count as u16 <= SDIO_CMD53_MAX_COUNT && count as u16 % SDIO_CMD53_WORD_BYTES == 0
}

fn configure_cmd53_byte_read(core: &sdhc0::CORE, count: u8) -> bool {
    if !wait_command_and_data_lines_free(core) {
        return false;
    }

    core.blocksize_r.write(|w| unsafe { w.bits(count as u16) });
    core.blockcount_r.write(|w| unsafe { w.bits(1) });
    core.sdmasa_r.write(|w| unsafe { w.bits(1) });
    core.bgap_ctrl_r.write(|w| unsafe { w.bits(0) });
    core.tout_ctrl_r.write(|w| unsafe { w.bits(0x0e) });
    core.xfer_mode_r
        .write(|w| unsafe { w.bits(XFER_MODE_BLOCK_COUNT_ENABLE | XFER_MODE_READ) });
    clear_interrupts(core);

    true
}

fn wait_command_and_data_lines_free(core: &sdhc0::CORE) -> bool {
    const BUSY_MASK: u32 = PSTATE_CMD_INHIBIT | PSTATE_CMD_INHIBIT_DAT | PSTATE_DAT_LINE_ACTIVE;

    for _ in 0..1000 {
        if core.pstate_reg.read().bits() & BUSY_MASK == 0 {
            return true;
        }
        delay_us(3);
    }

    false
}

fn read_cmd53_bytes(
    core: &sdhc0::CORE,
    count: u8,
) -> Result<[u8; SDIO_CMD53_MAX_COUNT as usize], WifiSdioCmd53ReadStatus> {
    if !wait_buffer_read_ready(core) {
        let _ = software_reset_data_line(core);
        return Err(WifiSdioCmd53ReadStatus::BufferReadTimeout);
    }

    let mut bytes = [0u8; SDIO_CMD53_MAX_COUNT as usize];
    let word_count = count as usize / SDIO_CMD53_WORD_BYTES as usize;
    for word_index in 0..word_count {
        if !wait_buffer_read_enable(core) {
            let _ = software_reset_data_line(core);
            return Err(WifiSdioCmd53ReadStatus::BufferEnableTimeout);
        }

        let word = core.buf_data_r.read().bits();
        let offset = word_index * 4;
        bytes[offset] = word as u8;
        bytes[offset + 1] = (word >> 8) as u8;
        bytes[offset + 2] = (word >> 16) as u8;
        bytes[offset + 3] = (word >> 24) as u8;
    }

    if !wait_transfer_complete(core) {
        let _ = software_reset_data_line(core);
        return Err(WifiSdioCmd53ReadStatus::TransferTimeout);
    }

    Ok(bytes)
}

fn wait_buffer_read_ready(core: &sdhc0::CORE) -> bool {
    for _ in 0..1000 {
        let normal_int = core.normal_int_stat_r.read().bits();
        let error_int = core.error_int_stat_r.read().bits();
        if error_int != 0 || normal_int & NORMAL_INT_ERROR != 0 {
            return false;
        }
        if normal_int & NORMAL_INT_BUF_RD_READY != 0 {
            core.normal_int_stat_r
                .write(|w| unsafe { w.bits(NORMAL_INT_BUF_RD_READY) });
            return true;
        }
        delay_us(150);
    }

    false
}

fn wait_buffer_read_enable(core: &sdhc0::CORE) -> bool {
    for _ in 0..1000 {
        if core.pstate_reg.read().bits() & PSTATE_BUF_RD_ENABLE != 0 {
            return true;
        }
        delay_us(3);
    }

    false
}

fn cmd52_read_byte_selected(core: &sdhc0::CORE, address: u32) -> Result<(u8, u32), CommandError> {
    cmd52_read_function_byte_selected(core, 0, address)
}

fn cmd52_read_function_byte_selected(
    core: &sdhc0::CORE,
    function: u8,
    address: u32,
) -> Result<(u8, u32), CommandError> {
    let response = cmd52_selected(core, false, function, address, 0, false)?;
    Ok(((response & 0xff) as u8, response))
}

fn cmd52_write_byte_selected(
    core: &sdhc0::CORE,
    address: u32,
    data: u8,
) -> Result<u32, CommandError> {
    cmd52_write_function_byte_selected(core, 0, address, data)
}

fn cmd52_write_function_byte_selected(
    core: &sdhc0::CORE,
    function: u8,
    address: u32,
    data: u8,
) -> Result<u32, CommandError> {
    cmd52_selected(core, true, function, address, data, false)
}

fn write_block_size(
    core: &sdhc0::CORE,
    low_address: u32,
    high_address: u32,
) -> Result<(), CommandError> {
    cmd52_write_byte_selected(core, low_address, SDIO_BLOCK_SIZE_64)?;
    cmd52_write_byte_selected(core, high_address, 0)?;
    Ok(())
}

fn read_block_size(
    core: &sdhc0::CORE,
    low_address: u32,
    high_address: u32,
) -> Result<(u16, u32), CommandError> {
    let low = cmd52_read_byte_selected(core, low_address)?;
    let high = cmd52_read_byte_selected(core, high_address)?;
    Ok(((low.0 as u16) | ((high.0 as u16) << 8), high.1))
}

fn enable_host_4bit(core: &sdhc0::CORE) {
    core.host_ctrl1_r
        .modify(|r, w| unsafe { w.bits(r.bits() | HOST_CTRL1_DATA_TRANSFER_WIDTH_4BIT) });
}

fn backplane_report(
    p: &Peripherals,
    status: WifiSdioBackplaneStatus,
    init_status: WifiSdioStatus,
    io_enable: u8,
    io_ready: u8,
    bus_control_before: u8,
    bus_control_after: u8,
    f0_block_size: u16,
    f1_block_size: u16,
    f2_block_size: u16,
    interrupt_enable: u8,
    attempts: u16,
    last_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioBackplaneReport {
    WifiSdioBackplaneReport {
        status,
        init_status,
        io_enable,
        io_ready,
        bus_control_before,
        bus_control_after,
        f0_block_size,
        f1_block_size,
        f2_block_size,
        interrupt_enable,
        attempts,
        last_response,
        last_error,
        host: host_snapshot(p),
    }
}

fn cmd53_read_report(
    p: &Peripherals,
    status: WifiSdioCmd53ReadStatus,
    setup_status: WifiSdioBackplaneStatus,
    function: u8,
    address: u32,
    count: u8,
    response: u32,
    bytes: [u8; SDIO_CMD53_MAX_COUNT as usize],
    last_error: Option<CommandError>,
) -> WifiSdioCmd53ReadReport {
    WifiSdioCmd53ReadReport {
        status,
        setup_status,
        function,
        address,
        count,
        response,
        bytes,
        last_error,
        host: host_snapshot(p),
    }
}

fn enable_report(
    p: &Peripherals,
    status: WifiSdioEnableStatus,
    init_status: WifiSdioStatus,
    requested: u8,
    ready: u8,
    attempts: u16,
    write_response: u32,
    ready_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioEnableReport {
    WifiSdioEnableReport {
        status,
        init_status,
        requested,
        ready,
        attempts,
        write_response,
        ready_response,
        last_error,
        host: host_snapshot(p),
    }
}

fn direct_report(
    p: &Peripherals,
    status: WifiSdioDirectStatus,
    init_status: WifiSdioStatus,
    function: u8,
    address: u32,
    write: bool,
    data: u8,
    response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioDirectReport {
    WifiSdioDirectReport {
        status,
        init_status,
        function,
        address,
        write,
        data,
        response,
        last_error,
        host: host_snapshot(p),
    }
}

fn report(
    p: &Peripherals,
    status: WifiSdioStatus,
    cmd5_response: u32,
    cmd5_attempts: u16,
    rca: u16,
    last_error: Option<CommandError>,
) -> WifiSdioReport {
    let core = &p.SDHC0.core;
    WifiSdioReport {
        status,
        cmd5_response,
        cmd5_attempts,
        rca,
        function_count: ((cmd5_response >> SDIO_OCR_FUNCTIONS_SHIFT) & 0x07) as u8,
        memory_present: cmd5_response & SDIO_OCR_MEMORY_PRESENT != 0,
        last_error,
        host: host_snapshot(p),
        pins: WifiSdioPinSnapshot {
            p2_sel0: p.HSIOM.prt2.port_sel0.read().bits(),
            p2_sel1: p.HSIOM.prt2.port_sel1.read().bits(),
            p2_cfg: p.GPIO.prt2.cfg.read().bits(),
            p2_out: p.GPIO.prt2.out.read().bits(),
            p2_in: p.GPIO.prt2.in_.read().bits(),
        },
        clock: WifiSdioClockSnapshot {
            path0: p.SRSS.clk_path_select[0].read().bits(),
            root0: p.SRSS.clk_root_select[0].read().bits(),
            root1: p.SRSS.clk_root_select[1].read().bits(),
            root2: p.SRSS.clk_root_select[2].read().bits(),
            root3: p.SRSS.clk_root_select[3].read().bits(),
            root4: p.SRSS.clk_root_select[4].read().bits(),
            fll_config: p.SRSS.clk_fll_config.read().bits(),
            fll_config2: p.SRSS.clk_fll_config2.read().bits(),
            fll_status: p.SRSS.clk_fll_status.read().bits(),
        },
    }
}

fn host_snapshot(p: &Peripherals) -> WifiSdioHostSnapshot {
    let core = &p.SDHC0.core;
    WifiSdioHostSnapshot {
        wrap_ctl: p.SDHC0.wrap.ctl.read().bits(),
        gp_out: core.gp_out_r.read().bits(),
        gp_in: core.gp_in_r.read().bits(),
        xfer_mode: core.xfer_mode_r.read().bits(),
        host_ctrl1: core.host_ctrl1_r.read().bits(),
        host_ctrl2: core.host_ctrl2_r.read().bits(),
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

fn configure_wifi_power_pin(p: &Peripherals) {
    p.GPIO.prt2.out_clr.write(|w| w.out6().set_bit());
    p.GPIO.prt2.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P2_WIFI_REG_ON_MASK;
        bits |= P2_WIFI_REG_ON_CFG;
        w.bits(bits)
    });
}

fn wifi_power_off(p: &Peripherals) {
    p.GPIO.prt2.out_clr.write(|w| w.out6().set_bit());
}

fn wifi_power_on(p: &Peripherals) {
    p.GPIO.prt2.out_set.write(|w| w.out6().set_bit());
}

fn configure_sdio_pins(p: &Peripherals) {
    p.GPIO.prt2.out_set.write(|w| unsafe { w.bits(0x30) });

    p.HSIOM.prt2.port_sel0.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P2_SDIO_DATA_HSIOM_MASK;
        bits |= P2_SDIO_DATA_HSIOM;
        w.bits(bits)
    });
    p.HSIOM.prt2.port_sel1.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !P2_SDIO_CMD_CLK_HSIOM_MASK;
        bits |= P2_SDIO_CMD_CLK_HSIOM;
        w.bits(bits)
    });

    p.GPIO.prt2.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !(P2_SDIO_DATA_MASK | P2_SDIO_CMD_CLK_MASK);
        bits |= P2_SDIO_DATA_CFG | P2_SDIO_CMD_CLK_CFG;
        w.bits(bits)
    });
}

fn configure_sdhc0_clock(p: &Peripherals) {
    p.SRSS.clk_root_select[SDHC0_HF_CLOCK_INDEX]
        .write(|w| unsafe { w.bits(CLK_ROOT_ENABLE | CLK_ROOT_MUX_PATH0 | CLK_ROOT_DIV_BY_2) });
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

fn software_reset_data_line(core: &sdhc0::CORE) -> bool {
    core.sw_rst_r.write(|w| w.sw_rst_dat().set_bit());
    for _ in 0..1000 {
        if core.sw_rst_r.read().sw_rst_dat().bit_is_clear() {
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

fn wait_transfer_complete(core: &sdhc0::CORE) -> bool {
    for _ in 0..1000 {
        let normal_int = core.normal_int_stat_r.read().bits();
        let error_int = core.error_int_stat_r.read().bits();
        if error_int != 0 || normal_int & NORMAL_INT_ERROR != 0 {
            return false;
        }
        if normal_int & NORMAL_INT_XFER_COMPLETE != 0 {
            core.normal_int_stat_r
                .write(|w| unsafe { w.bits(NORMAL_INT_XFER_COMPLETE) });
            return true;
        }
        delay_us(250);
    }
    false
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

    wait_command_complete(core, trace)?;
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
    if command.data_present {
        bits |= 1 << 5;
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

fn wait_command_complete(core: &sdhc0::CORE, trace: CommandTrace) -> Result<(), CommandError> {
    for _ in 0..1000 {
        let normal_int = core.normal_int_stat_r.read().bits();
        let error_int = core.error_int_stat_r.read().bits();
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
    for _ in 0..us {
        cortex_m::asm::delay(50);
    }
}
