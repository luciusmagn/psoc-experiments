use psoc6_pac::{sdhc0, Peripherals};

const HSIOM_SEL_SDHC: u32 = 26;
const DRIVE_STRONG: u32 = 0x06;
const DRIVE_STRONG_INPUT: u32 = 0x0e;

const NORMAL_INT_ALL: u16 = 0x1fff;
const ERROR_INT_ALL: u16 = 0x07ff;
const NORMAL_INT_CMD_COMPLETE: u16 = 1 << 0;
const NORMAL_INT_XFER_COMPLETE: u16 = 1 << 1;
const NORMAL_INT_BUF_WR_READY: u16 = 1 << 4;
const NORMAL_INT_BUF_RD_READY: u16 = 1 << 5;
const NORMAL_INT_ERROR: u16 = 1 << 15;
const PSTATE_CMD_INHIBIT: u32 = 1 << 0;
const PSTATE_CMD_INHIBIT_DAT: u32 = 1 << 1;
const PSTATE_DAT_LINE_ACTIVE: u32 = 1 << 2;
const PSTATE_BUF_WR_ENABLE: u32 = 1 << 10;
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
const SDIO_CMD53_PREVIEW_BYTES: usize = 16;
const SDIO_CMD53_MAX_READ_BYTES: u16 = 64;
const SDIO_CMD53_WORD_BYTES: u16 = 4;
const SDIO_CMD53_BLOCK_BYTES: u16 = 64;
const SDIO_CMD53_BLOCK_WORDS: usize = 16;
const SDPCM_HEADER_BYTES: u8 = 12;
const SOCSRAM_BLOCK_PROBE_NO_MISMATCH: u32 = 0xffff_ffff;
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
const SDIO_BACKPLANE_FUNCTION: u8 = 1;
const SDIO_FUNCTION_ENABLE_1_2: u8 = 0x06;
const SDIO_FUNCTION_READY_2: u8 = 0x04;
const SDIO_BACKPLANE_ADDRESS_LOW: u32 = 0x1000a;
const SDIO_BACKPLANE_ADDRESS_MID: u32 = 0x1000b;
const SDIO_BACKPLANE_ADDRESS_HIGH: u32 = 0x1000c;
const SDIO_FUNCTION2_WATERMARK: u32 = 0x10008;
const SDIO_CHIP_CLOCK_CSR: u32 = 0x1000e;
const SDIO_PULL_UP: u32 = 0x1000f;
const SDIO_FUNCTION_ENABLE_1: u8 = 0x02;
const SDIO_FUNCTION_READY_1: u8 = 0x02;
const SDIO_INTERRUPT_MASTER_FUNC1_FUNC2: u8 = 0x07;
const SDIO_INTERRUPT_MASTER_FUNC2: u8 = 0x05;
const SDIO_FUNCTION_MASK_F1_F2: u8 = 0x03;
const SDIO_BUS_WIDTH_MASK: u8 = 0x03;
const SDIO_BUS_WIDTH_4BIT: u8 = 0x02;
const SDIO_BLOCK_SIZE_64: u8 = 64;
const SDIO_READY_MAX_ATTEMPTS: u16 = 1000;
const SDIO_BACKPLANE_SETUP_MAX_ATTEMPTS: u16 = 1000;
const SDIO_ALP_AVAIL_MAX_ATTEMPTS: u16 = 100;
const SDIO_HT_AVAIL_MAX_ATTEMPTS: u16 = 2500;
const SDIO_F2_READY_MAX_ATTEMPTS: u16 = 1000;
const SBSDIO_FORCE_ALP: u8 = 0x01;
const SBSDIO_ALP_AVAIL_REQ: u8 = 0x08;
const SBSDIO_FORCE_HW_CLKREQ_OFF: u8 = 0x20;
const SBSDIO_ALP_AVAIL: u8 = 0x40;
const SBSDIO_HT_AVAIL: u8 = 0x80;
const SBSDIO_SB_OFT_ADDR_MASK: u32 = 0x07fff;
const SBSDIO_SB_ACCESS_2_4B_FLAG: u32 = 0x08000;
const SDIO_F2_WATERMARK: u8 = 8;
const HOST_INTERRUPT_MASK: u32 = 0x0000_00f0;
const AI_IOCTRL_OFFSET: u32 = 0x408;
const AI_RESETCTRL_OFFSET: u32 = 0x800;
const AI_RESETSTATUS_OFFSET: u32 = 0x804;
const SICF_CLOCK_EN: u8 = 0x01;
const SICF_FGC: u8 = 0x02;
const AIRC_RESET: u8 = 0x01;
const CORE_CONTROL_NONE: u8 = 0;
const CORE_CONTROL_RESET: u8 = AIRC_RESET;
const CORE_CONTROL_CLOCKED: u8 = SICF_CLOCK_EN;
const CORE_CONTROL_FORCE_CLOCKED: u8 = SICF_FGC | SICF_CLOCK_EN;
const CYW43430_SDIO_CORE_BASE: u32 = 0x1800_2000;
const CYW43430_SDIO_INT_HOST_MASK: u32 = CYW43430_SDIO_CORE_BASE + 0x24;
const CYW43430_SDIO_FUNCTION_INT_MASK: u32 = CYW43430_SDIO_CORE_BASE + 0x34;
const CYW43430_ARM_WRAPPER_BASE: u32 = 0x1810_3000;
const CYW43430_SOCSRAM_BASE: u32 = 0x1800_4000;
const CYW43430_SOCSRAM_WRAPPER_BASE: u32 = 0x1810_4000;
const CYW43430_SOCSRAM_BYTES: usize = 512 * 1024;
const CYW43430_SOCSRAM_BANKX_INDEX: u32 = CYW43430_SOCSRAM_BASE + 0x10;
const CYW43430_SOCSRAM_BANKX_PDA: u32 = CYW43430_SOCSRAM_BASE + 0x44;
const HOST_CTRL1_DMA_SELECT_MASK: u8 = 0x03 << 3;
const HOST_CTRL1_DMA_SELECT_ADMA2: u8 = 0x02 << 3;
const MBIU_CTRL_ALL_BURSTS: u8 = 0x0f;
const XFER_MODE_DMA_ENABLE: u16 = 1 << 0;
const XFER_MODE_BLOCK_COUNT_ENABLE: u16 = 1 << 1;
const XFER_MODE_READ: u16 = 1 << 4;
const ADMA_ATTR_VALID: u32 = 1 << 0;
const ADMA_ATTR_END: u32 = 1 << 1;
const ADMA_ACT_TRAN: u32 = 4 << 3;
const ADMA_LEN_SHIFT: u32 = 16;
const FNV_OFFSET_BASIS: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;
const NVRAM_IMAGE_SIZE_ALIGNMENT: usize = 4;

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
const SDHC_WRAP_CTL_ENABLE: u32 = 1 << 31;

#[repr(align(8))]
struct AdmaDescriptorTable([u32; 2]);

static mut CMD53_WRITE_ADMA_DESCRIPTOR: AdmaDescriptorTable = AdmaDescriptorTable([0; 2]);
static mut CMD53_WRITE_WORD: u32 = 0;
static mut CMD53_WRITE_BLOCK_WORDS: [u32; SDIO_CMD53_BLOCK_WORDS] = [0; SDIO_CMD53_BLOCK_WORDS];

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
pub enum WifiSdioF2FrameStatus {
    Ready,
    HeaderReadFailed,
    InvalidHeader,
    FrameTooShort,
    FrameTooLarge,
    UnsupportedLength,
    BodyReadFailed,
}

#[derive(Clone, Copy)]
pub enum WifiSdioBackplaneReadStatus {
    Ready,
    SetupFailed,
    InvalidAddress,
    InvalidCount,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    WindowHighWriteFailed,
    WindowMidWriteFailed,
    WindowLowWriteFailed,
    DataSetupBusy,
    Cmd52Failed,
    Cmd53Failed,
    BufferReadTimeout,
    BufferEnableTimeout,
    TransferTimeout,
}

#[derive(Clone, Copy)]
pub enum WifiSdioBackplaneWrite8Status {
    Ready,
    SetupFailed,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    WindowHighWriteFailed,
    WindowMidWriteFailed,
    WindowLowWriteFailed,
    Cmd52Failed,
}

#[derive(Clone, Copy)]
pub enum WifiSdioBackplaneWrite32Status {
    Ready,
    SetupFailed,
    InvalidAddress,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    WindowHighWriteFailed,
    WindowMidWriteFailed,
    WindowLowWriteFailed,
    DataSetupBusy,
    Cmd52Failed,
    Cmd53Failed,
    TransferTimeout,
    DataLineBusy,
    ReadbackCmd52Failed,
}

#[derive(Clone, Copy)]
pub enum WifiSdioSocramProbeStatus {
    Ready,
    SetupFailed,
    InvalidAddress,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    ArmDisableFailed,
    SocramDisableFailed,
    SocramResetFailed,
    BankIndexWriteFailed,
    BankPdaWriteFailed,
    OriginalReadFailed,
    ProbeWriteFailed,
    ProbeReadbackMismatch,
    RestoreWriteFailed,
    RestoreReadbackMismatch,
}

#[derive(Clone, Copy)]
pub enum WifiSdioSocramBlockProbeStatus {
    Ready,
    SetupFailed,
    InvalidAddress,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    ArmDisableFailed,
    SocramDisableFailed,
    SocramResetFailed,
    BankIndexWriteFailed,
    BankPdaWriteFailed,
    OriginalReadFailed,
    ProbeWriteFailed,
    ProbeReadFailed,
    ProbeReadbackMismatch,
    RestoreWriteFailed,
    RestoreReadFailed,
    RestoreReadbackMismatch,
}

#[derive(Clone, Copy)]
pub enum WifiSdioFirmwareLoadStatus {
    Ready,
    BlobMissing,
    BlobTooLarge,
    SetupFailed,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    ArmDisableFailed,
    SocramDisableFailed,
    SocramResetFailed,
    BankIndexWriteFailed,
    BankPdaWriteFailed,
    WriteFailed,
    VerifyReadFailed,
    VerifyMismatch,
}

#[derive(Clone, Copy)]
pub enum WifiSdioFirmwareStartStatus {
    Ready,
    FirmwareFailed,
    NvramMissing,
    NvramTooLarge,
    NvramWriteFailed,
    NvramVerifyReadFailed,
    NvramVerifyMismatch,
    NvramSizeWriteFailed,
    PullUpWriteFailed,
    IoEnableWriteFailed,
    InterruptEnableWriteFailed,
    ArmResetFailed,
    ArmStateReadFailed,
    HtReadFailed,
    HtTimeout,
    HostInterruptMaskWriteFailed,
    FunctionInterruptMaskWriteFailed,
    WatermarkWriteFailed,
    F2ReadyReadFailed,
    F2ReadyTimeout,
}

#[derive(Clone, Copy)]
pub enum WifiSdioCoreStateStatus {
    Ready,
    SetupFailed,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    WindowHighWriteFailed,
    WindowMidWriteFailed,
    WindowLowWriteFailed,
    IoctrlReadFailed,
    ResetctrlReadFailed,
    ResetstatusReadFailed,
}

#[derive(Clone, Copy)]
pub enum WifiSdioCoreResetStatus {
    Ready,
    SetupFailed,
    AlpWriteFailed,
    AlpReadFailed,
    AlpTimeout,
    AlpClearFailed,
    WindowHighWriteFailed,
    WindowMidWriteFailed,
    WindowLowWriteFailed,
    BeforeIoctrlReadFailed,
    BeforeResetctrlReadFailed,
    BeforeResetstatusReadFailed,
    DisableResetctrlReadFailed,
    DisableIoctrlWriteFailed,
    DisableIoctrlReadFailed,
    DisableResetctrlWriteFailed,
    ResetIoctrlWriteFailed,
    ResetIoctrlReadFailed,
    ResetctrlWriteFailed,
    FinalIoctrlWriteFailed,
    AfterIoctrlReadFailed,
    AfterResetctrlReadFailed,
    AfterResetstatusReadFailed,
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
enum AlpError {
    Write(CommandError),
    Read { response: u32, error: CommandError },
    Timeout { response: u32 },
    Clear { response: u32, error: CommandError },
}

#[derive(Clone, Copy)]
enum BackplaneWindowError {
    High(CommandError),
    Mid(CommandError),
    Low(CommandError),
}

#[derive(Clone, Copy)]
enum BackplaneByteReadError {
    Window(BackplaneWindowError),
    Cmd52(CommandError),
}

#[derive(Clone, Copy)]
enum BackplaneByteWriteError {
    Window(BackplaneWindowError),
    Cmd52(CommandError),
}

enum ClockWaitError {
    Read { attempts: u16, error: CommandError },
    Timeout { value: u8, attempts: u16 },
}

enum ReadyWaitError {
    Read { attempts: u16, error: CommandError },
    Timeout { value: u8, attempts: u16 },
}

#[derive(Clone, Copy)]
struct BackplaneByteRead {
    value: u8,
    response: u32,
}

#[derive(Clone, Copy)]
struct BackplaneByteWrite {
    response: u32,
}

#[derive(Clone, Copy)]
struct BackplaneWordRead {
    value: u32,
    response: u32,
}

struct BackplaneWordWrite {
    window_base: u32,
    window_address: u32,
    response: u32,
    readback: u32,
}

struct BackplaneWordWriteFailure {
    status: WifiSdioBackplaneWrite32Status,
    window_base: u32,
    window_address: u32,
    response: u32,
    last_error: Option<CommandError>,
}

struct BackplaneBlockRead {
    words: [u32; SDIO_CMD53_BLOCK_WORDS],
    response: u32,
}

struct BackplaneBlockWrite {
    response: u32,
}

struct NvramWrite {
    address: u32,
    rounded_bytes: u32,
    size_word: u32,
    checksum: u32,
    last_response: u32,
}

struct NvramVerify {
    checksum: u32,
    mismatch_offset: u32,
    mismatch_expected: u32,
    mismatch_actual: u32,
    last_response: u32,
}

enum NvramWriteError {
    TooLarge,
    Block(BackplaneBlockWriteError),
    Word(BackplaneWordWriteFailure),
}

enum BackplaneBlockReadError {
    InvalidAddress,
    Window(BackplaneWindowError),
    DataSetupBusy,
    Cmd53(CommandError),
    Data {
        response: u32,
        status: WifiSdioCmd53ReadStatus,
    },
}

enum BackplaneBlockWriteError {
    InvalidAddress,
    Window(BackplaneWindowError),
    DataSetupBusy,
    Cmd53(CommandError),
    Data {
        response: u32,
        status: WifiSdioBackplaneWrite32Status,
    },
}

struct SocramPrepared {
    setup_status: WifiSdioBackplaneStatus,
    last_response: u32,
}

enum SocramPrepareError {
    Setup {
        setup_status: WifiSdioBackplaneStatus,
        last_error: Option<CommandError>,
    },
    Alp {
        setup_status: WifiSdioBackplaneStatus,
        error: AlpError,
    },
    ArmDisable {
        setup_status: WifiSdioBackplaneStatus,
        last_response: u32,
        error: CoreResetStepError,
    },
    SocramDisable {
        setup_status: WifiSdioBackplaneStatus,
        last_response: u32,
        error: CoreResetStepError,
    },
    SocramReset {
        setup_status: WifiSdioBackplaneStatus,
        last_response: u32,
        error: CoreResetStepError,
    },
    BankIndexWrite {
        setup_status: WifiSdioBackplaneStatus,
        error: BackplaneWordWriteFailure,
    },
    BankPdaWrite {
        setup_status: WifiSdioBackplaneStatus,
        error: BackplaneWordWriteFailure,
    },
}

#[derive(Clone, Copy)]
enum CoreRegisterAccess {
    Ioctrl,
    Resetctrl,
    Resetstatus,
}

#[derive(Clone, Copy)]
enum CoreStateReadError {
    Window(BackplaneWindowError),
    Cmd52 {
        access: CoreRegisterAccess,
        error: CommandError,
    },
}

#[derive(Clone, Copy)]
enum CoreResetStepError {
    DisableResetctrlRead(BackplaneByteReadError),
    DisableIoctrlWrite(BackplaneByteWriteError),
    DisableIoctrlRead(BackplaneByteReadError),
    DisableResetctrlWrite(BackplaneByteWriteError),
    ResetIoctrlWrite(BackplaneByteWriteError),
    ResetIoctrlRead(BackplaneByteReadError),
    ResetctrlWrite(BackplaneByteWriteError),
    FinalIoctrlWrite(BackplaneByteWriteError),
}

pub struct WifiSdioHostSnapshot {
    pub wrap_ctl: u32,
    pub gp_out: u32,
    pub gp_in: u32,
    pub xfer_mode: u16,
    pub block_size: u16,
    pub block_count: u16,
    pub sdmasa: u32,
    pub adma_sa_low: u32,
    pub adma_id_low: u32,
    pub adma_err_stat: u8,
    pub bgap_ctrl: u8,
    pub host_ctrl1: u8,
    pub host_ctrl2: u16,
    pub capabilities1: u32,
    pub capabilities2: u32,
    pub mbiu_ctrl: u8,
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
    pub bytes: [u8; SDIO_CMD53_PREVIEW_BYTES],
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioBackplaneReadReport {
    pub status: WifiSdioBackplaneReadStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub address: u32,
    pub count: u8,
    pub window_base: u32,
    pub window_address: u32,
    pub response: u32,
    pub bytes: [u8; SDIO_CMD53_PREVIEW_BYTES],
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioBackplaneWrite8Report {
    pub status: WifiSdioBackplaneWrite8Status,
    pub setup_status: WifiSdioBackplaneStatus,
    pub address: u32,
    pub value: u8,
    pub window_base: u32,
    pub window_address: u32,
    pub response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioBackplaneWrite32Report {
    pub status: WifiSdioBackplaneWrite32Status,
    pub setup_status: WifiSdioBackplaneStatus,
    pub address: u32,
    pub value: u32,
    pub window_base: u32,
    pub window_address: u32,
    pub response: u32,
    pub readback: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioSocramProbeReport {
    pub status: WifiSdioSocramProbeStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub write_status: WifiSdioBackplaneWrite32Status,
    pub address: u32,
    pub pattern: u32,
    pub original: u32,
    pub readback: u32,
    pub restored: u32,
    pub last_response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioSocramBlockProbeReport {
    pub status: WifiSdioSocramBlockProbeStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub read_status: WifiSdioBackplaneReadStatus,
    pub write_status: WifiSdioBackplaneWrite32Status,
    pub address: u32,
    pub seed: u32,
    pub original_checksum: u32,
    pub readback_checksum: u32,
    pub restored_checksum: u32,
    pub mismatch_index: u32,
    pub mismatch_expected: u32,
    pub mismatch_actual: u32,
    pub last_response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioFirmwareLoadReport {
    pub status: WifiSdioFirmwareLoadStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub read_status: WifiSdioBackplaneReadStatus,
    pub write_status: WifiSdioBackplaneWrite32Status,
    pub firmware_bytes: u32,
    pub processed_bytes: u32,
    pub chunk_count: u32,
    pub firmware_checksum: u32,
    pub verify_checksum: u32,
    pub mismatch_offset: u32,
    pub mismatch_expected: u32,
    pub mismatch_actual: u32,
    pub last_response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioFirmwareStartReport {
    pub status: WifiSdioFirmwareStartStatus,
    pub firmware_status: WifiSdioFirmwareLoadStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub read_status: WifiSdioBackplaneReadStatus,
    pub write_status: WifiSdioBackplaneWrite32Status,
    pub firmware_bytes: u32,
    pub nvram_bytes: u32,
    pub nvram_rounded_bytes: u32,
    pub nvram_address: u32,
    pub nvram_size_word: u32,
    pub firmware_checksum: u32,
    pub nvram_checksum: u32,
    pub nvram_verify_checksum: u32,
    pub mismatch_offset: u32,
    pub mismatch_expected: u32,
    pub mismatch_actual: u32,
    pub arm_before: WifiSdioCoreSnapshot,
    pub arm_after: WifiSdioCoreSnapshot,
    pub ht_clock_csr: u8,
    pub ht_attempts: u16,
    pub io_enable: u8,
    pub io_ready: u8,
    pub f2_attempts: u16,
    pub last_response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioF2HeaderReport {
    pub status: WifiSdioCmd53ReadStatus,
    pub response: u32,
    pub bytes: [u8; 4],
    pub length: u16,
    pub checksum: u16,
    pub valid: bool,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioF2FrameReport {
    pub status: WifiSdioF2FrameStatus,
    pub header_status: WifiSdioCmd53ReadStatus,
    pub body_status: WifiSdioCmd53ReadStatus,
    pub header_response: u32,
    pub body_response: u32,
    pub bytes: [u8; SDIO_CMD53_PREVIEW_BYTES],
    pub byte_count: u8,
    pub length: u16,
    pub checksum: u16,
    pub valid: bool,
    pub sequence: u8,
    pub channel_and_flags: u8,
    pub channel: u8,
    pub flags: u8,
    pub next_length: u8,
    pub header_length: u8,
    pub wireless_flow_control: u8,
    pub bus_data_credit: u8,
    pub reserved0: u8,
    pub reserved1: u8,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

#[derive(Clone, Copy)]
pub struct WifiSdioCoreSnapshot {
    pub ioctrl: u8,
    pub resetctrl: u8,
    pub resetstatus: u8,
    pub clock_enabled: bool,
    pub force_gated: bool,
    pub in_reset: bool,
    pub reset_busy: bool,
    pub core_up: bool,
}

pub struct WifiSdioCoreStateReport {
    pub status: WifiSdioCoreStateStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub base: u32,
    pub ioctrl: u8,
    pub resetctrl: u8,
    pub resetstatus: u8,
    pub clock_enabled: bool,
    pub force_gated: bool,
    pub in_reset: bool,
    pub reset_busy: bool,
    pub core_up: bool,
    pub last_response: u32,
    pub last_error: Option<CommandError>,
    pub host: WifiSdioHostSnapshot,
}

pub struct WifiSdioCoreResetReport {
    pub status: WifiSdioCoreResetStatus,
    pub setup_status: WifiSdioBackplaneStatus,
    pub base: u32,
    pub before: WifiSdioCoreSnapshot,
    pub after: WifiSdioCoreSnapshot,
    pub last_response: u32,
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

pub fn backplane_read(p: &Peripherals, address: u32, count: u8) -> WifiSdioBackplaneReadReport {
    if !is_valid_backplane_read_count(count) {
        return backplane_read_report(
            p,
            WifiSdioBackplaneReadStatus::InvalidCount,
            WifiSdioBackplaneStatus::Ready,
            address,
            count,
            0,
            0,
            0,
            [0; SDIO_CMD53_PREVIEW_BYTES],
            None,
        );
    }

    if backplane_read_crosses_window(address, count) {
        return backplane_read_report(
            p,
            WifiSdioBackplaneReadStatus::InvalidAddress,
            WifiSdioBackplaneStatus::Ready,
            address,
            count,
            0,
            0,
            0,
            [0; SDIO_CMD53_PREVIEW_BYTES],
            None,
        );
    }

    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return backplane_read_report(
            p,
            WifiSdioBackplaneReadStatus::SetupFailed,
            setup.status,
            address,
            count,
            0,
            0,
            0,
            [0; SDIO_CMD53_PREVIEW_BYTES],
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let alp_response = match request_alp_clock(core) {
        Ok(response) => response,
        Err(error) => {
            let (status, response, last_error) = backplane_read_alp_error(error);
            return backplane_read_report(
                p,
                status,
                setup.status,
                address,
                count,
                0,
                0,
                response,
                [0; SDIO_CMD53_PREVIEW_BYTES],
                last_error,
            );
        }
    };

    let window_base = match set_backplane_window(core, address) {
        Ok(window_base) => window_base,
        Err(error) => {
            let (status, last_error) = backplane_window_error(error);
            return backplane_read_report(
                p,
                status,
                setup.status,
                address,
                count,
                backplane_window_base(address),
                0,
                alp_response,
                [0; SDIO_CMD53_PREVIEW_BYTES],
                Some(last_error),
            );
        }
    };
    let window_address = backplane_window_address(address, count);

    if count == 1 {
        return match cmd52_read_function_byte_selected(
            core,
            SDIO_BACKPLANE_FUNCTION,
            window_address,
        ) {
            Ok((byte, response)) => {
                let mut bytes = [0; SDIO_CMD53_PREVIEW_BYTES];
                bytes[0] = byte;
                backplane_read_report(
                    p,
                    WifiSdioBackplaneReadStatus::Ready,
                    setup.status,
                    address,
                    count,
                    window_base,
                    window_address,
                    response,
                    bytes,
                    None,
                )
            }
            Err(error) => backplane_read_report(
                p,
                WifiSdioBackplaneReadStatus::Cmd52Failed,
                setup.status,
                address,
                count,
                window_base,
                window_address,
                0,
                [0; SDIO_CMD53_PREVIEW_BYTES],
                Some(error),
            ),
        };
    }

    if !configure_cmd53_read(core, count, false) {
        return backplane_read_report(
            p,
            WifiSdioBackplaneReadStatus::DataSetupBusy,
            setup.status,
            address,
            count,
            window_base,
            window_address,
            0,
            [0; SDIO_CMD53_PREVIEW_BYTES],
            None,
        );
    }

    let argument = cmd53_argument(
        false,
        SDIO_BACKPLANE_FUNCTION,
        false,
        true,
        window_address,
        count as u16,
    );
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
            return backplane_read_report(
                p,
                WifiSdioBackplaneReadStatus::Cmd53Failed,
                setup.status,
                address,
                count,
                window_base,
                window_address,
                0,
                [0; SDIO_CMD53_PREVIEW_BYTES],
                Some(error),
            )
        }
    };

    let bytes = match read_cmd53_bytes(core, count) {
        Ok(bytes) => bytes,
        Err(status) => {
            return backplane_read_report(
                p,
                backplane_read_data_status(status),
                setup.status,
                address,
                count,
                window_base,
                window_address,
                response,
                [0; SDIO_CMD53_PREVIEW_BYTES],
                None,
            )
        }
    };

    backplane_read_report(
        p,
        WifiSdioBackplaneReadStatus::Ready,
        setup.status,
        address,
        count,
        window_base,
        window_address,
        response,
        bytes,
        None,
    )
}

pub fn backplane_write8(p: &Peripherals, address: u32, value: u8) -> WifiSdioBackplaneWrite8Report {
    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return backplane_write8_report(
            p,
            WifiSdioBackplaneWrite8Status::SetupFailed,
            setup.status,
            address,
            value,
            0,
            0,
            0,
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let alp_response = match request_alp_clock(core) {
        Ok(response) => response,
        Err(error) => {
            let (status, response, last_error) = backplane_write8_alp_error(error);
            return backplane_write8_report(
                p,
                status,
                setup.status,
                address,
                value,
                0,
                0,
                response,
                last_error,
            );
        }
    };

    let window_base = match set_backplane_window(core, address) {
        Ok(window_base) => window_base,
        Err(error) => {
            let (status, last_error) = backplane_write8_window_error(error);
            return backplane_write8_report(
                p,
                status,
                setup.status,
                address,
                value,
                backplane_window_base(address),
                0,
                alp_response,
                Some(last_error),
            );
        }
    };
    let window_address = backplane_window_address(address, 1);

    match cmd52_write_function_byte_selected(core, SDIO_BACKPLANE_FUNCTION, window_address, value) {
        Ok(response) => backplane_write8_report(
            p,
            WifiSdioBackplaneWrite8Status::Ready,
            setup.status,
            address,
            value,
            window_base,
            window_address,
            response,
            None,
        ),
        Err(error) => backplane_write8_report(
            p,
            WifiSdioBackplaneWrite8Status::Cmd52Failed,
            setup.status,
            address,
            value,
            window_base,
            window_address,
            0,
            Some(error),
        ),
    }
}

pub fn backplane_write32(
    p: &Peripherals,
    address: u32,
    value: u32,
) -> WifiSdioBackplaneWrite32Report {
    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return backplane_write32_report(
            p,
            WifiSdioBackplaneWrite32Status::SetupFailed,
            setup.status,
            address,
            value,
            0,
            0,
            0,
            0,
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let alp_response = match request_alp_clock(core) {
        Ok(response) => response,
        Err(error) => {
            let (status, response, last_error) = backplane_write32_alp_error(error);
            return backplane_write32_report(
                p,
                status,
                setup.status,
                address,
                value,
                0,
                0,
                response,
                0,
                last_error,
            );
        }
    };

    backplane_write32_after_setup_report(p, setup.status, address, value, alp_response)
}

fn backplane_write32_after_setup_report(
    p: &Peripherals,
    setup_status: WifiSdioBackplaneStatus,
    address: u32,
    value: u32,
    last_response: u32,
) -> WifiSdioBackplaneWrite32Report {
    match write_backplane_u32_after_setup(&p.SDHC0.core, address, value, last_response) {
        Ok(write) => backplane_write32_report(
            p,
            WifiSdioBackplaneWrite32Status::Ready,
            setup_status,
            address,
            value,
            write.window_base,
            write.window_address,
            write.response,
            write.readback,
            None,
        ),
        Err(error) => backplane_write32_report(
            p,
            error.status,
            setup_status,
            address,
            value,
            error.window_base,
            error.window_address,
            error.response,
            0,
            error.last_error,
        ),
    }
}

fn write_backplane_u32_after_setup(
    core: &sdhc0::CORE,
    address: u32,
    value: u32,
    last_response: u32,
) -> Result<BackplaneWordWrite, BackplaneWordWriteFailure> {
    if address & 0x03 != 0 || backplane_read_crosses_window(address, 4) {
        return Err(BackplaneWordWriteFailure {
            status: WifiSdioBackplaneWrite32Status::InvalidAddress,
            window_base: 0,
            window_address: 0,
            response: 0,
            last_error: None,
        });
    }

    let window_base = match set_backplane_window(core, address) {
        Ok(window_base) => window_base,
        Err(error) => {
            let (status, last_error) = backplane_write32_window_error(error);
            return Err(BackplaneWordWriteFailure {
                status,
                window_base: backplane_window_base(address),
                window_address: 0,
                response: last_response,
                last_error: Some(last_error),
            });
        }
    };
    let window_address = backplane_window_address(address, 4);

    let descriptor_address = prepare_cmd53_write_word(value, 4);
    if !configure_cmd53_write_adma(core, 4, false, descriptor_address) {
        return Err(BackplaneWordWriteFailure {
            status: WifiSdioBackplaneWrite32Status::DataSetupBusy,
            window_base,
            window_address,
            response: 0,
            last_error: None,
        });
    }

    let argument = cmd53_argument(
        true,
        SDIO_BACKPLANE_FUNCTION,
        false,
        true,
        window_address,
        4,
    );
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
            return Err(BackplaneWordWriteFailure {
                status: WifiSdioBackplaneWrite32Status::Cmd53Failed,
                window_base,
                window_address,
                response: 0,
                last_error: Some(error),
            })
        }
    };

    if let Err(status) = wait_cmd53_adma_write_complete(core) {
        return Err(BackplaneWordWriteFailure {
            status,
            window_base,
            window_address,
            response,
            last_error: None,
        });
    }

    let readback = match read_backplane_u32_bytes(core, address) {
        Ok(readback) => readback,
        Err(error) => {
            let (status, last_error) = backplane_write32_byte_readback_error(error);
            return Err(BackplaneWordWriteFailure {
                status,
                window_base,
                window_address,
                response,
                last_error,
            });
        }
    };

    Ok(BackplaneWordWrite {
        window_base,
        window_address,
        response,
        readback: readback.value,
    })
}

fn read_backplane_block_after_setup(
    core: &sdhc0::CORE,
    address: u32,
) -> Result<BackplaneBlockRead, BackplaneBlockReadError> {
    if address & 0x03 != 0 || backplane_read_crosses_window(address, SDIO_CMD53_BLOCK_BYTES as u8) {
        return Err(BackplaneBlockReadError::InvalidAddress);
    }

    set_backplane_window(core, address).map_err(BackplaneBlockReadError::Window)?;
    let window_address = backplane_window_address(address, SDIO_CMD53_BLOCK_BYTES as u8);

    if !configure_cmd53_read(core, SDIO_CMD53_BLOCK_BYTES as u8, true) {
        return Err(BackplaneBlockReadError::DataSetupBusy);
    }

    let argument = cmd53_argument(
        false,
        SDIO_BACKPLANE_FUNCTION,
        true,
        true,
        window_address,
        1,
    );
    let response = send_command(
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
    )
    .map_err(BackplaneBlockReadError::Cmd53)?;

    let words = read_cmd53_words(core)
        .map_err(|status| BackplaneBlockReadError::Data { response, status })?;
    Ok(BackplaneBlockRead { words, response })
}

fn write_backplane_block_after_setup(
    core: &sdhc0::CORE,
    address: u32,
    words: &[u32; SDIO_CMD53_BLOCK_WORDS],
) -> Result<BackplaneBlockWrite, BackplaneBlockWriteError> {
    if address & 0x03 != 0 || backplane_read_crosses_window(address, SDIO_CMD53_BLOCK_BYTES as u8) {
        return Err(BackplaneBlockWriteError::InvalidAddress);
    }

    set_backplane_window(core, address).map_err(BackplaneBlockWriteError::Window)?;
    let window_address = backplane_window_address(address, SDIO_CMD53_BLOCK_BYTES as u8);
    let descriptor_address = prepare_cmd53_write_words(words, SDIO_CMD53_BLOCK_BYTES);
    if !configure_cmd53_write_adma(core, SDIO_CMD53_BLOCK_BYTES as u8, true, descriptor_address) {
        return Err(BackplaneBlockWriteError::DataSetupBusy);
    }

    let argument = cmd53_argument(true, SDIO_BACKPLANE_FUNCTION, true, true, window_address, 1);
    let response = send_command(
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
    )
    .map_err(BackplaneBlockWriteError::Cmd53)?;

    wait_cmd53_adma_write_complete(core)
        .map_err(|status| BackplaneBlockWriteError::Data { response, status })?;
    Ok(BackplaneBlockWrite { response })
}

pub fn backplane_write32_bytes(
    p: &Peripherals,
    address: u32,
    value: u32,
) -> WifiSdioBackplaneWrite32Report {
    if address & 0x03 != 0 || backplane_read_crosses_window(address, 4) {
        return backplane_write32_report(
            p,
            WifiSdioBackplaneWrite32Status::InvalidAddress,
            WifiSdioBackplaneStatus::Ready,
            address,
            value,
            0,
            0,
            0,
            0,
            None,
        );
    }

    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return backplane_write32_report(
            p,
            WifiSdioBackplaneWrite32Status::SetupFailed,
            setup.status,
            address,
            value,
            0,
            0,
            0,
            0,
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let alp_response = match request_alp_clock(core) {
        Ok(response) => response,
        Err(error) => {
            let (status, response, last_error) = backplane_write32_alp_error(error);
            return backplane_write32_report(
                p,
                status,
                setup.status,
                address,
                value,
                0,
                0,
                response,
                0,
                last_error,
            );
        }
    };

    let window_base = match set_backplane_window(core, address) {
        Ok(window_base) => window_base,
        Err(error) => {
            let (status, last_error) = backplane_write32_window_error(error);
            return backplane_write32_report(
                p,
                status,
                setup.status,
                address,
                value,
                backplane_window_base(address),
                0,
                alp_response,
                0,
                Some(last_error),
            );
        }
    };

    let window_address = backplane_window_address(address, 1);
    let mut last_response = alp_response;
    let bytes = [
        value as u8,
        (value >> 8) as u8,
        (value >> 16) as u8,
        (value >> 24) as u8,
    ];
    for (offset, byte) in bytes.iter().enumerate() {
        last_response = match cmd52_write_function_byte_selected(
            core,
            SDIO_BACKPLANE_FUNCTION,
            window_address + offset as u32,
            *byte,
        ) {
            Ok(response) => response,
            Err(error) => {
                return backplane_write32_report(
                    p,
                    WifiSdioBackplaneWrite32Status::Cmd52Failed,
                    setup.status,
                    address,
                    value,
                    window_base,
                    window_address,
                    last_response,
                    0,
                    Some(error),
                );
            }
        };
    }

    let readback = match read_backplane_u32_bytes(core, address) {
        Ok(readback) => readback,
        Err(error) => {
            let (status, last_error) = backplane_write32_byte_readback_error(error);
            return backplane_write32_report(
                p,
                status,
                setup.status,
                address,
                value,
                window_base,
                window_address,
                last_response,
                0,
                last_error,
            );
        }
    };

    backplane_write32_report(
        p,
        WifiSdioBackplaneWrite32Status::Ready,
        setup.status,
        address,
        value,
        window_base,
        window_address,
        last_response,
        readback.value,
        None,
    )
}

pub fn socram_probe(p: &Peripherals, address: u32, pattern: u32) -> WifiSdioSocramProbeReport {
    if address & 0x03 != 0 || backplane_read_crosses_window(address, 4) {
        return socram_probe_report(
            p,
            WifiSdioSocramProbeStatus::InvalidAddress,
            WifiSdioBackplaneStatus::Ready,
            WifiSdioBackplaneWrite32Status::InvalidAddress,
            address,
            pattern,
            0,
            0,
            0,
            0,
            None,
        );
    }

    let prepared = match prepare_cyw43430_socram(p) {
        Ok(prepared) => prepared,
        Err(error) => {
            let (status, setup_status, write_status, last_response, last_error) =
                socram_probe_prepare_error(error);
            return socram_probe_report(
                p,
                status,
                setup_status,
                write_status,
                address,
                pattern,
                0,
                0,
                0,
                last_response,
                last_error,
            );
        }
    };

    let core = &p.SDHC0.core;
    let setup_status = prepared.setup_status;
    let mut last_response = prepared.last_response;
    let original = match read_backplane_u32_bytes(core, address) {
        Ok(read) => {
            last_response = read.response;
            read.value
        }
        Err(error) => {
            let (write_status, last_error) = backplane_write32_byte_readback_error(error);
            return socram_probe_report(
                p,
                WifiSdioSocramProbeStatus::OriginalReadFailed,
                setup_status,
                write_status,
                address,
                pattern,
                0,
                0,
                0,
                last_response,
                last_error,
            );
        }
    };

    let probe = match write_backplane_u32_after_setup(core, address, pattern, last_response) {
        Ok(write) => {
            last_response = write.response;
            write.readback
        }
        Err(error) => {
            return socram_probe_report(
                p,
                WifiSdioSocramProbeStatus::ProbeWriteFailed,
                setup_status,
                error.status,
                address,
                pattern,
                original,
                0,
                0,
                error.response,
                error.last_error,
            );
        }
    };

    if probe != pattern {
        return socram_probe_report(
            p,
            WifiSdioSocramProbeStatus::ProbeReadbackMismatch,
            setup_status,
            WifiSdioBackplaneWrite32Status::Ready,
            address,
            pattern,
            original,
            probe,
            0,
            last_response,
            None,
        );
    }

    let restored = match write_backplane_u32_after_setup(core, address, original, last_response) {
        Ok(write) => {
            last_response = write.response;
            write.readback
        }
        Err(error) => {
            return socram_probe_report(
                p,
                WifiSdioSocramProbeStatus::RestoreWriteFailed,
                setup_status,
                error.status,
                address,
                pattern,
                original,
                probe,
                0,
                error.response,
                error.last_error,
            );
        }
    };

    if restored != original {
        return socram_probe_report(
            p,
            WifiSdioSocramProbeStatus::RestoreReadbackMismatch,
            setup_status,
            WifiSdioBackplaneWrite32Status::Ready,
            address,
            pattern,
            original,
            probe,
            restored,
            last_response,
            None,
        );
    }

    socram_probe_report(
        p,
        WifiSdioSocramProbeStatus::Ready,
        setup_status,
        WifiSdioBackplaneWrite32Status::Ready,
        address,
        pattern,
        original,
        probe,
        restored,
        last_response,
        None,
    )
}

pub fn socram_block_probe(
    p: &Peripherals,
    address: u32,
    seed: u32,
) -> WifiSdioSocramBlockProbeReport {
    if address & 0x03 != 0 || backplane_read_crosses_window(address, SDIO_CMD53_BLOCK_BYTES as u8) {
        return socram_block_probe_report(
            p,
            WifiSdioSocramBlockProbeStatus::InvalidAddress,
            WifiSdioBackplaneStatus::Ready,
            WifiSdioBackplaneReadStatus::InvalidAddress,
            WifiSdioBackplaneWrite32Status::InvalidAddress,
            address,
            seed,
            0,
            0,
            0,
            SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
            0,
            0,
            0,
            None,
        );
    }

    let prepared = match prepare_cyw43430_socram(p) {
        Ok(prepared) => prepared,
        Err(error) => {
            let (status, setup_status, write_status, last_response, last_error) =
                socram_block_probe_prepare_error(error);
            return socram_block_probe_report(
                p,
                status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                write_status,
                address,
                seed,
                0,
                0,
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                last_response,
                last_error,
            );
        }
    };

    let core = &p.SDHC0.core;
    let setup_status = prepared.setup_status;
    let mut last_response = prepared.last_response;
    let original = match read_backplane_block_after_setup(core, address) {
        Ok(read) => {
            last_response = read.response;
            read.words
        }
        Err(error) => {
            let (read_status, response, last_error) = backplane_block_read_error(error);
            return socram_block_probe_report(
                p,
                WifiSdioSocramBlockProbeStatus::OriginalReadFailed,
                setup_status,
                read_status,
                WifiSdioBackplaneWrite32Status::Ready,
                address,
                seed,
                0,
                0,
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                response_or_last(response, last_response),
                last_error,
            );
        }
    };

    let original_checksum = checksum_words(&original);
    let probe_words = socram_block_probe_words(seed);
    let mut read_status = WifiSdioBackplaneReadStatus::Ready;
    let mut readback_checksum = 0;
    let mut pending_status = WifiSdioSocramBlockProbeStatus::Ready;
    let mut pending_last_response = 0;
    let mut pending_last_error = None;
    let mut mismatch_index = SOCSRAM_BLOCK_PROBE_NO_MISMATCH;
    let mut mismatch_expected = 0;
    let mut mismatch_actual = 0;

    match write_backplane_block_after_setup(core, address, &probe_words) {
        Ok(write) => {
            last_response = write.response;
        }
        Err(error) => {
            let (write_status, response, last_error) = backplane_block_write_error(error);
            return socram_block_probe_report(
                p,
                WifiSdioSocramBlockProbeStatus::ProbeWriteFailed,
                setup_status,
                read_status,
                write_status,
                address,
                seed,
                original_checksum,
                0,
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                response_or_last(response, last_response),
                last_error,
            );
        }
    }

    match read_backplane_block_after_setup(core, address) {
        Ok(read) => {
            last_response = read.response;
            let readback = read.words;
            readback_checksum = checksum_words(&readback);
            let mismatch = mismatch_words(&probe_words, &readback);
            if mismatch.0 != SOCSRAM_BLOCK_PROBE_NO_MISMATCH {
                pending_status = WifiSdioSocramBlockProbeStatus::ProbeReadbackMismatch;
                mismatch_index = mismatch.0;
                mismatch_expected = mismatch.1;
                mismatch_actual = mismatch.2;
            }
        }
        Err(error) => {
            let (status, response, last_error) = backplane_block_read_error(error);
            read_status = status;
            pending_status = WifiSdioSocramBlockProbeStatus::ProbeReadFailed;
            pending_last_response = response_or_last(response, last_response);
            pending_last_error = last_error;
        }
    }

    match write_backplane_block_after_setup(core, address, &original) {
        Ok(write) => {
            last_response = write.response;
        }
        Err(error) => {
            let (write_status, response, last_error) = backplane_block_write_error(error);
            return socram_block_probe_report(
                p,
                WifiSdioSocramBlockProbeStatus::RestoreWriteFailed,
                setup_status,
                read_status,
                write_status,
                address,
                seed,
                original_checksum,
                readback_checksum,
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                response_or_last(response, last_response),
                last_error,
            );
        }
    }

    let restored = match read_backplane_block_after_setup(core, address) {
        Ok(read) => {
            last_response = read.response;
            read.words
        }
        Err(error) => {
            let (status, response, last_error) = backplane_block_read_error(error);
            return socram_block_probe_report(
                p,
                WifiSdioSocramBlockProbeStatus::RestoreReadFailed,
                setup_status,
                status,
                WifiSdioBackplaneWrite32Status::Ready,
                address,
                seed,
                original_checksum,
                readback_checksum,
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                response_or_last(response, last_response),
                last_error,
            );
        }
    };

    let restored_checksum = checksum_words(&restored);
    let restore_mismatch = mismatch_words(&original, &restored);
    if restore_mismatch.0 != SOCSRAM_BLOCK_PROBE_NO_MISMATCH {
        return socram_block_probe_report(
            p,
            WifiSdioSocramBlockProbeStatus::RestoreReadbackMismatch,
            setup_status,
            WifiSdioBackplaneReadStatus::Ready,
            WifiSdioBackplaneWrite32Status::Ready,
            address,
            seed,
            original_checksum,
            readback_checksum,
            restored_checksum,
            restore_mismatch.0,
            restore_mismatch.1,
            restore_mismatch.2,
            last_response,
            None,
        );
    }

    if !matches!(pending_status, WifiSdioSocramBlockProbeStatus::Ready) {
        return socram_block_probe_report(
            p,
            pending_status,
            setup_status,
            read_status,
            WifiSdioBackplaneWrite32Status::Ready,
            address,
            seed,
            original_checksum,
            readback_checksum,
            restored_checksum,
            mismatch_index,
            mismatch_expected,
            mismatch_actual,
            response_or_last(pending_last_response, last_response),
            pending_last_error,
        );
    }

    socram_block_probe_report(
        p,
        WifiSdioSocramBlockProbeStatus::Ready,
        setup_status,
        WifiSdioBackplaneReadStatus::Ready,
        WifiSdioBackplaneWrite32Status::Ready,
        address,
        seed,
        original_checksum,
        readback_checksum,
        restored_checksum,
        SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
        0,
        0,
        last_response,
        None,
    )
}

pub fn load_firmware(p: &Peripherals, firmware: &[u8]) -> WifiSdioFirmwareLoadReport {
    if firmware.is_empty() {
        return firmware_load_report(
            p,
            WifiSdioFirmwareLoadStatus::BlobMissing,
            WifiSdioBackplaneStatus::Ready,
            WifiSdioBackplaneReadStatus::Ready,
            WifiSdioBackplaneWrite32Status::Ready,
            0,
            0,
            0,
            0,
            0,
            SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
            0,
            0,
            0,
            None,
        );
    }

    if firmware.len() > CYW43430_SOCSRAM_BYTES {
        return firmware_load_report(
            p,
            WifiSdioFirmwareLoadStatus::BlobTooLarge,
            WifiSdioBackplaneStatus::Ready,
            WifiSdioBackplaneReadStatus::Ready,
            WifiSdioBackplaneWrite32Status::Ready,
            firmware.len() as u32,
            0,
            firmware_chunk_count(firmware.len()),
            checksum_bytes(firmware),
            0,
            SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
            0,
            0,
            0,
            None,
        );
    }

    let firmware_bytes = firmware.len() as u32;
    let chunk_count = firmware_chunk_count(firmware.len());
    let firmware_checksum = checksum_bytes(firmware);
    let prepared = match prepare_cyw43430_socram(p) {
        Ok(prepared) => prepared,
        Err(error) => {
            let (status, setup_status, write_status, last_response, last_error) =
                firmware_load_prepare_error(error);
            return firmware_load_report(
                p,
                status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                write_status,
                firmware_bytes,
                0,
                chunk_count,
                firmware_checksum,
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                last_response,
                last_error,
            );
        }
    };

    let core = &p.SDHC0.core;
    let setup_status = prepared.setup_status;
    let mut last_response = prepared.last_response;
    let mut offset = 0usize;

    while offset < firmware.len() {
        let chunk_len = firmware_chunk_len(firmware.len(), offset);
        let words = firmware_chunk_words(firmware, offset, chunk_len);
        match write_backplane_block_after_setup(core, offset as u32, &words) {
            Ok(write) => {
                last_response = write.response;
            }
            Err(error) => {
                let (write_status, response, last_error) = backplane_block_write_error(error);
                return firmware_load_report(
                    p,
                    WifiSdioFirmwareLoadStatus::WriteFailed,
                    setup_status,
                    WifiSdioBackplaneReadStatus::Ready,
                    write_status,
                    firmware_bytes,
                    offset as u32,
                    chunk_count,
                    firmware_checksum,
                    0,
                    SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                    0,
                    0,
                    response_or_last(response, last_response),
                    last_error,
                );
            }
        }
        offset += SDIO_CMD53_BLOCK_BYTES as usize;
    }

    let mut verify_checksum = FNV_OFFSET_BASIS;
    let mut mismatch_offset = SOCSRAM_BLOCK_PROBE_NO_MISMATCH;
    let mut mismatch_expected = 0;
    let mut mismatch_actual = 0;
    offset = 0;

    while offset < firmware.len() {
        let chunk_len = firmware_chunk_len(firmware.len(), offset);
        let read = match read_backplane_block_after_setup(core, offset as u32) {
            Ok(read) => read,
            Err(error) => {
                let (read_status, response, last_error) = backplane_block_read_error(error);
                return firmware_load_report(
                    p,
                    WifiSdioFirmwareLoadStatus::VerifyReadFailed,
                    setup_status,
                    read_status,
                    WifiSdioBackplaneWrite32Status::Ready,
                    firmware_bytes,
                    offset as u32,
                    chunk_count,
                    firmware_checksum,
                    verify_checksum,
                    mismatch_offset,
                    mismatch_expected,
                    mismatch_actual,
                    response_or_last(response, last_response),
                    last_error,
                );
            }
        };
        last_response = read.response;

        for chunk_offset in 0..chunk_len {
            let expected = firmware[offset + chunk_offset];
            let actual = word_byte(&read.words, chunk_offset);
            verify_checksum = checksum_byte(verify_checksum, actual);
            if mismatch_offset == SOCSRAM_BLOCK_PROBE_NO_MISMATCH && expected != actual {
                mismatch_offset = (offset + chunk_offset) as u32;
                mismatch_expected = expected as u32;
                mismatch_actual = actual as u32;
            }
        }

        offset += SDIO_CMD53_BLOCK_BYTES as usize;
    }

    let status = if mismatch_offset == SOCSRAM_BLOCK_PROBE_NO_MISMATCH {
        WifiSdioFirmwareLoadStatus::Ready
    } else {
        WifiSdioFirmwareLoadStatus::VerifyMismatch
    };

    firmware_load_report(
        p,
        status,
        setup_status,
        WifiSdioBackplaneReadStatus::Ready,
        WifiSdioBackplaneWrite32Status::Ready,
        firmware_bytes,
        firmware_bytes,
        chunk_count,
        firmware_checksum,
        verify_checksum,
        mismatch_offset,
        mismatch_expected,
        mismatch_actual,
        last_response,
        None,
    )
}

pub fn start_firmware(
    p: &Peripherals,
    firmware: &[u8],
    nvram: &[u8],
) -> WifiSdioFirmwareStartReport {
    let firmware_report = load_firmware(p, firmware);
    if !matches!(firmware_report.status, WifiSdioFirmwareLoadStatus::Ready) {
        return firmware_start_report(
            p,
            WifiSdioFirmwareStartStatus::FirmwareFailed,
            firmware_report.status,
            firmware_report.setup_status,
            firmware_report.read_status,
            firmware_report.write_status,
            firmware_report.firmware_bytes,
            0,
            0,
            0,
            0,
            firmware_report.firmware_checksum,
            0,
            0,
            firmware_report.mismatch_offset,
            firmware_report.mismatch_expected,
            firmware_report.mismatch_actual,
            empty_core_snapshot(),
            empty_core_snapshot(),
            0,
            0,
            0,
            0,
            0,
            firmware_report.last_response,
            firmware_report.last_error,
        );
    }

    if nvram.is_empty() {
        return firmware_start_report(
            p,
            WifiSdioFirmwareStartStatus::NvramMissing,
            firmware_report.status,
            firmware_report.setup_status,
            WifiSdioBackplaneReadStatus::Ready,
            WifiSdioBackplaneWrite32Status::Ready,
            firmware_report.firmware_bytes,
            0,
            0,
            0,
            0,
            firmware_report.firmware_checksum,
            0,
            0,
            SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
            0,
            0,
            empty_core_snapshot(),
            empty_core_snapshot(),
            0,
            0,
            0,
            0,
            0,
            firmware_report.last_response,
            None,
        );
    }

    let core = &p.SDHC0.core;
    let setup_status = firmware_report.setup_status;
    let mut last_response = firmware_report.last_response;

    let nvram_write = match write_nvram_image_after_setup(core, nvram, &mut last_response) {
        Ok(write) => {
            last_response = write.last_response;
            write
        }
        Err(error) => {
            let (status, write_status, response, last_error) = nvram_write_error(error);
            return firmware_start_report(
                p,
                status,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                write_status,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                rounded_nvram_len(nvram.len()) as u32,
                nvram_download_address(rounded_nvram_len(nvram.len())),
                nvram_size_word(rounded_nvram_len(nvram.len())),
                firmware_report.firmware_checksum,
                checksum_padded_bytes(nvram, rounded_nvram_len(nvram.len())),
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                empty_core_snapshot(),
                empty_core_snapshot(),
                0,
                0,
                0,
                0,
                0,
                response_or_last(response, last_response),
                last_error,
            );
        }
    };

    let nvram_verify = match verify_nvram_image_after_setup(core, nvram, &mut last_response) {
        Ok(verify) => {
            last_response = verify.last_response;
            verify
        }
        Err(error) => {
            let (read_status, response, last_error) = backplane_block_read_error(error);
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::NvramVerifyReadFailed,
                firmware_report.status,
                setup_status,
                read_status,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                0,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                empty_core_snapshot(),
                empty_core_snapshot(),
                0,
                0,
                0,
                0,
                0,
                response_or_last(response, last_response),
                last_error,
            );
        }
    };

    if nvram_verify.mismatch_offset != SOCSRAM_BLOCK_PROBE_NO_MISMATCH {
        return firmware_start_report(
            p,
            WifiSdioFirmwareStartStatus::NvramVerifyMismatch,
            firmware_report.status,
            setup_status,
            WifiSdioBackplaneReadStatus::Ready,
            WifiSdioBackplaneWrite32Status::Ready,
            firmware_report.firmware_bytes,
            nvram.len() as u32,
            nvram_write.rounded_bytes,
            nvram_write.address,
            nvram_write.size_word,
            firmware_report.firmware_checksum,
            nvram_write.checksum,
            nvram_verify.checksum,
            nvram_verify.mismatch_offset,
            nvram_verify.mismatch_expected,
            nvram_verify.mismatch_actual,
            empty_core_snapshot(),
            empty_core_snapshot(),
            0,
            0,
            0,
            0,
            0,
            last_response,
            None,
        );
    }

    match write_backplane_u32_after_setup(
        core,
        CYW43430_SOCSRAM_BYTES as u32 - 4,
        nvram_write.size_word,
        last_response,
    ) {
        Ok(write) => {
            last_response = write.response;
        }
        Err(error) => {
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::NvramSizeWriteFailed,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                error.status,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                empty_core_snapshot(),
                empty_core_snapshot(),
                0,
                0,
                0,
                0,
                0,
                error.response,
                error.last_error,
            );
        }
    }

    if let Err(error) =
        cmd52_write_function_byte_selected(core, SDIO_BACKPLANE_FUNCTION, SDIO_PULL_UP, 0)
    {
        return firmware_start_command_error_report(
            p,
            WifiSdioFirmwareStartStatus::PullUpWriteFailed,
            &firmware_report,
            &nvram_write,
            &nvram_verify,
            nvram.len() as u32,
            last_response,
            error,
        );
    }

    let io_enable =
        match cmd52_write_byte_selected(core, SDIO_CCCR_IO_ENABLE, SDIO_FUNCTION_ENABLE_1_2) {
            Ok(response) => {
                last_response = response;
                SDIO_FUNCTION_ENABLE_1_2
            }
            Err(error) => {
                return firmware_start_command_error_report(
                    p,
                    WifiSdioFirmwareStartStatus::IoEnableWriteFailed,
                    &firmware_report,
                    &nvram_write,
                    &nvram_verify,
                    nvram.len() as u32,
                    last_response,
                    error,
                );
            }
        };

    if let Err(error) = cmd52_write_byte_selected(
        core,
        SDIO_CCCR_INTERRUPT_ENABLE,
        SDIO_INTERRUPT_MASTER_FUNC2,
    ) {
        return firmware_start_command_error_report(
            p,
            WifiSdioFirmwareStartStatus::InterruptEnableWriteFailed,
            &firmware_report,
            &nvram_write,
            &nvram_verify,
            nvram.len() as u32,
            last_response,
            error,
        );
    }

    let arm_before = match read_core_snapshot(core, CYW43430_ARM_WRAPPER_BASE) {
        Ok((snapshot, response)) => {
            last_response = response;
            snapshot
        }
        Err(error) => {
            let (_, last_error) = before_core_snapshot_error(error);
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::ArmStateReadFailed,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                empty_core_snapshot(),
                empty_core_snapshot(),
                0,
                0,
                io_enable,
                0,
                0,
                last_response,
                Some(last_error),
            );
        }
    };

    if let Err(error) = reset_core_registers(core, CYW43430_ARM_WRAPPER_BASE, &mut last_response) {
        let (_, last_error) = core_reset_step_error(error);
        return firmware_start_report(
            p,
            WifiSdioFirmwareStartStatus::ArmResetFailed,
            firmware_report.status,
            setup_status,
            WifiSdioBackplaneReadStatus::Ready,
            WifiSdioBackplaneWrite32Status::Ready,
            firmware_report.firmware_bytes,
            nvram.len() as u32,
            nvram_write.rounded_bytes,
            nvram_write.address,
            nvram_write.size_word,
            firmware_report.firmware_checksum,
            nvram_write.checksum,
            nvram_verify.checksum,
            SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
            0,
            0,
            arm_before,
            empty_core_snapshot(),
            0,
            0,
            io_enable,
            0,
            0,
            last_response,
            Some(last_error),
        );
    }

    let arm_after = match read_core_snapshot(core, CYW43430_ARM_WRAPPER_BASE) {
        Ok((snapshot, response)) => {
            last_response = response;
            snapshot
        }
        Err(error) => {
            let (_, last_error) = after_core_snapshot_error(error);
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::ArmStateReadFailed,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                arm_before,
                empty_core_snapshot(),
                0,
                0,
                io_enable,
                0,
                0,
                last_response,
                Some(last_error),
            );
        }
    };

    let (ht_clock_csr, ht_attempts) = match wait_ht_available(core, &mut last_response) {
        Ok(result) => result,
        Err(ClockWaitError::Read { attempts, error }) => {
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::HtReadFailed,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                arm_before,
                arm_after,
                0,
                attempts,
                io_enable,
                0,
                0,
                last_response,
                Some(error),
            );
        }
        Err(ClockWaitError::Timeout { value, attempts }) => {
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::HtTimeout,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                arm_before,
                arm_after,
                value,
                attempts,
                io_enable,
                0,
                0,
                last_response,
                None,
            );
        }
    };

    match write_backplane_u32_after_setup(
        core,
        CYW43430_SDIO_INT_HOST_MASK,
        HOST_INTERRUPT_MASK,
        last_response,
    ) {
        Ok(write) => {
            last_response = write.response;
        }
        Err(error) => {
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::HostInterruptMaskWriteFailed,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                error.status,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                arm_before,
                arm_after,
                ht_clock_csr,
                ht_attempts,
                io_enable,
                0,
                0,
                error.response,
                error.last_error,
            );
        }
    }

    match write_backplane_u8(
        core,
        CYW43430_SDIO_FUNCTION_INT_MASK,
        SDIO_FUNCTION_MASK_F1_F2,
    ) {
        Ok(write) => {
            last_response = write.response;
        }
        Err(error) => {
            let (_, last_error) = backplane_write8_window_or_cmd_error(error);
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::FunctionInterruptMaskWriteFailed,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                arm_before,
                arm_after,
                ht_clock_csr,
                ht_attempts,
                io_enable,
                0,
                0,
                last_response,
                Some(last_error),
            );
        }
    }

    if let Err(error) = cmd52_write_function_byte_selected(
        core,
        SDIO_BACKPLANE_FUNCTION,
        SDIO_FUNCTION2_WATERMARK,
        SDIO_F2_WATERMARK,
    ) {
        return firmware_start_command_error_report(
            p,
            WifiSdioFirmwareStartStatus::WatermarkWriteFailed,
            &firmware_report,
            &nvram_write,
            &nvram_verify,
            nvram.len() as u32,
            last_response,
            error,
        );
    }

    let (io_ready, f2_attempts) = match wait_f2_ready(core, &mut last_response) {
        Ok(result) => result,
        Err(ReadyWaitError::Read { attempts, error }) => {
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::F2ReadyReadFailed,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                arm_before,
                arm_after,
                ht_clock_csr,
                ht_attempts,
                io_enable,
                0,
                attempts,
                last_response,
                Some(error),
            );
        }
        Err(ReadyWaitError::Timeout { value, attempts }) => {
            return firmware_start_report(
                p,
                WifiSdioFirmwareStartStatus::F2ReadyTimeout,
                firmware_report.status,
                setup_status,
                WifiSdioBackplaneReadStatus::Ready,
                WifiSdioBackplaneWrite32Status::Ready,
                firmware_report.firmware_bytes,
                nvram.len() as u32,
                nvram_write.rounded_bytes,
                nvram_write.address,
                nvram_write.size_word,
                firmware_report.firmware_checksum,
                nvram_write.checksum,
                nvram_verify.checksum,
                SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
                0,
                0,
                arm_before,
                arm_after,
                ht_clock_csr,
                ht_attempts,
                io_enable,
                value,
                attempts,
                last_response,
                None,
            );
        }
    };

    firmware_start_report(
        p,
        WifiSdioFirmwareStartStatus::Ready,
        firmware_report.status,
        setup_status,
        WifiSdioBackplaneReadStatus::Ready,
        WifiSdioBackplaneWrite32Status::Ready,
        firmware_report.firmware_bytes,
        nvram.len() as u32,
        nvram_write.rounded_bytes,
        nvram_write.address,
        nvram_write.size_word,
        firmware_report.firmware_checksum,
        nvram_write.checksum,
        nvram_verify.checksum,
        SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
        0,
        0,
        arm_before,
        arm_after,
        ht_clock_csr,
        ht_attempts,
        io_enable,
        io_ready,
        f2_attempts,
        last_response,
        None,
    )
}

pub fn f2_read_header(p: &Peripherals) -> WifiSdioF2HeaderReport {
    let transfer = f2_read_bytes(p, 4);
    let bytes = [
        transfer.bytes[0],
        transfer.bytes[1],
        transfer.bytes[2],
        transfer.bytes[3],
    ];
    f2_header_report(
        p,
        transfer.status,
        transfer.response,
        bytes,
        transfer.last_error,
    )
}

pub fn f2_read_frame(p: &Peripherals) -> WifiSdioF2FrameReport {
    let header = f2_read_bytes(p, 4);
    let mut bytes = [0; SDIO_CMD53_PREVIEW_BYTES];
    copy_bytes(&mut bytes, 0, &header.bytes, 4);
    let length = u16::from_le_bytes([bytes[0], bytes[1]]);
    let checksum = u16::from_le_bytes([bytes[2], bytes[3]]);
    let valid = length != 0 && (length ^ checksum) == 0xffff;

    if !matches!(header.status, WifiSdioCmd53ReadStatus::Ready) {
        return f2_frame_report(
            p,
            WifiSdioF2FrameStatus::HeaderReadFailed,
            header.status,
            WifiSdioCmd53ReadStatus::Ready,
            header.response,
            0,
            bytes,
            4,
            header.last_error,
        );
    }

    if !valid {
        return f2_frame_report(
            p,
            WifiSdioF2FrameStatus::InvalidHeader,
            header.status,
            WifiSdioCmd53ReadStatus::Ready,
            header.response,
            0,
            bytes,
            4,
            None,
        );
    }

    if length < SDPCM_HEADER_BYTES as u16 {
        return f2_frame_report(
            p,
            WifiSdioF2FrameStatus::FrameTooShort,
            header.status,
            WifiSdioCmd53ReadStatus::Ready,
            header.response,
            0,
            bytes,
            4,
            None,
        );
    }

    if length as usize > SDIO_CMD53_PREVIEW_BYTES {
        return f2_frame_report(
            p,
            WifiSdioF2FrameStatus::FrameTooLarge,
            header.status,
            WifiSdioCmd53ReadStatus::Ready,
            header.response,
            0,
            bytes,
            4,
            None,
        );
    }

    let body_count = (length - 4) as u8;
    if body_count % SDIO_CMD53_WORD_BYTES as u8 != 0 {
        return f2_frame_report(
            p,
            WifiSdioF2FrameStatus::UnsupportedLength,
            header.status,
            WifiSdioCmd53ReadStatus::Ready,
            header.response,
            0,
            bytes,
            4,
            None,
        );
    }

    let body = f2_read_bytes(p, body_count);
    if !matches!(body.status, WifiSdioCmd53ReadStatus::Ready) {
        return f2_frame_report(
            p,
            WifiSdioF2FrameStatus::BodyReadFailed,
            header.status,
            body.status,
            header.response,
            body.response,
            bytes,
            4,
            body.last_error,
        );
    }

    copy_bytes(&mut bytes, 4, &body.bytes, body_count);

    f2_frame_report(
        p,
        WifiSdioF2FrameStatus::Ready,
        header.status,
        body.status,
        header.response,
        body.response,
        bytes,
        length as u8,
        None,
    )
}

fn prepare_cyw43430_socram(p: &Peripherals) -> Result<SocramPrepared, SocramPrepareError> {
    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return Err(SocramPrepareError::Setup {
            setup_status: setup.status,
            last_error: setup.last_error,
        });
    }

    let core = &p.SDHC0.core;
    let mut last_response = request_alp_clock(core).map_err(|error| SocramPrepareError::Alp {
        setup_status: setup.status,
        error,
    })?;

    disable_core_registers(core, CYW43430_ARM_WRAPPER_BASE, &mut last_response).map_err(
        |error| SocramPrepareError::ArmDisable {
            setup_status: setup.status,
            last_response,
            error,
        },
    )?;
    disable_core_registers(core, CYW43430_SOCSRAM_WRAPPER_BASE, &mut last_response).map_err(
        |error| SocramPrepareError::SocramDisable {
            setup_status: setup.status,
            last_response,
            error,
        },
    )?;
    reset_core_registers(core, CYW43430_SOCSRAM_WRAPPER_BASE, &mut last_response).map_err(
        |error| SocramPrepareError::SocramReset {
            setup_status: setup.status,
            last_response,
            error,
        },
    )?;

    match write_backplane_u32_after_setup(core, CYW43430_SOCSRAM_BANKX_INDEX, 3, last_response) {
        Ok(write) => {
            last_response = write.response;
        }
        Err(error) => {
            return Err(SocramPrepareError::BankIndexWrite {
                setup_status: setup.status,
                error,
            });
        }
    }

    match write_backplane_u32_after_setup(core, CYW43430_SOCSRAM_BANKX_PDA, 0, last_response) {
        Ok(write) => {
            last_response = write.response;
        }
        Err(error) => {
            return Err(SocramPrepareError::BankPdaWrite {
                setup_status: setup.status,
                error,
            });
        }
    }

    Ok(SocramPrepared {
        setup_status: setup.status,
        last_response,
    })
}

fn socram_probe_prepare_error(
    error: SocramPrepareError,
) -> (
    WifiSdioSocramProbeStatus,
    WifiSdioBackplaneStatus,
    WifiSdioBackplaneWrite32Status,
    u32,
    Option<CommandError>,
) {
    match error {
        SocramPrepareError::Setup {
            setup_status,
            last_error,
        } => (
            WifiSdioSocramProbeStatus::SetupFailed,
            setup_status,
            WifiSdioBackplaneWrite32Status::Ready,
            0,
            last_error,
        ),
        SocramPrepareError::Alp {
            setup_status,
            error,
        } => {
            let (status, response, last_error) = socram_probe_alp_error(error);
            (
                status,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                response,
                last_error,
            )
        }
        SocramPrepareError::ArmDisable {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioSocramProbeStatus::ArmDisableFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::SocramDisable {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioSocramProbeStatus::SocramDisableFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::SocramReset {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioSocramProbeStatus::SocramResetFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::BankIndexWrite {
            setup_status,
            error,
        } => (
            WifiSdioSocramProbeStatus::BankIndexWriteFailed,
            setup_status,
            error.status,
            error.response,
            error.last_error,
        ),
        SocramPrepareError::BankPdaWrite {
            setup_status,
            error,
        } => (
            WifiSdioSocramProbeStatus::BankPdaWriteFailed,
            setup_status,
            error.status,
            error.response,
            error.last_error,
        ),
    }
}

fn socram_block_probe_prepare_error(
    error: SocramPrepareError,
) -> (
    WifiSdioSocramBlockProbeStatus,
    WifiSdioBackplaneStatus,
    WifiSdioBackplaneWrite32Status,
    u32,
    Option<CommandError>,
) {
    match error {
        SocramPrepareError::Setup {
            setup_status,
            last_error,
        } => (
            WifiSdioSocramBlockProbeStatus::SetupFailed,
            setup_status,
            WifiSdioBackplaneWrite32Status::Ready,
            0,
            last_error,
        ),
        SocramPrepareError::Alp {
            setup_status,
            error,
        } => {
            let (status, response, last_error) = socram_block_probe_alp_error(error);
            (
                status,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                response,
                last_error,
            )
        }
        SocramPrepareError::ArmDisable {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioSocramBlockProbeStatus::ArmDisableFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::SocramDisable {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioSocramBlockProbeStatus::SocramDisableFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::SocramReset {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioSocramBlockProbeStatus::SocramResetFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::BankIndexWrite {
            setup_status,
            error,
        } => (
            WifiSdioSocramBlockProbeStatus::BankIndexWriteFailed,
            setup_status,
            error.status,
            error.response,
            error.last_error,
        ),
        SocramPrepareError::BankPdaWrite {
            setup_status,
            error,
        } => (
            WifiSdioSocramBlockProbeStatus::BankPdaWriteFailed,
            setup_status,
            error.status,
            error.response,
            error.last_error,
        ),
    }
}

fn firmware_load_prepare_error(
    error: SocramPrepareError,
) -> (
    WifiSdioFirmwareLoadStatus,
    WifiSdioBackplaneStatus,
    WifiSdioBackplaneWrite32Status,
    u32,
    Option<CommandError>,
) {
    match error {
        SocramPrepareError::Setup {
            setup_status,
            last_error,
        } => (
            WifiSdioFirmwareLoadStatus::SetupFailed,
            setup_status,
            WifiSdioBackplaneWrite32Status::Ready,
            0,
            last_error,
        ),
        SocramPrepareError::Alp {
            setup_status,
            error,
        } => {
            let (status, response, last_error) = firmware_load_alp_error(error);
            (
                status,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                response,
                last_error,
            )
        }
        SocramPrepareError::ArmDisable {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioFirmwareLoadStatus::ArmDisableFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::SocramDisable {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioFirmwareLoadStatus::SocramDisableFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::SocramReset {
            setup_status,
            last_response,
            error,
        } => {
            let (_, last_error) = core_reset_step_error(error);
            (
                WifiSdioFirmwareLoadStatus::SocramResetFailed,
                setup_status,
                WifiSdioBackplaneWrite32Status::Ready,
                last_response,
                Some(last_error),
            )
        }
        SocramPrepareError::BankIndexWrite {
            setup_status,
            error,
        } => (
            WifiSdioFirmwareLoadStatus::BankIndexWriteFailed,
            setup_status,
            error.status,
            error.response,
            error.last_error,
        ),
        SocramPrepareError::BankPdaWrite {
            setup_status,
            error,
        } => (
            WifiSdioFirmwareLoadStatus::BankPdaWriteFailed,
            setup_status,
            error.status,
            error.response,
            error.last_error,
        ),
    }
}

fn socram_block_probe_words(seed: u32) -> [u32; SDIO_CMD53_BLOCK_WORDS] {
    let mut words = [0u32; SDIO_CMD53_BLOCK_WORDS];
    let mut state = seed ^ 0x9e37_79b9;
    for word in words.iter_mut() {
        state = state.wrapping_add(0x9e37_79b9);
        let mut mixed = state;
        mixed ^= mixed >> 16;
        mixed = mixed.wrapping_mul(0x7feb_352d);
        mixed ^= mixed >> 15;
        mixed = mixed.wrapping_mul(0x846c_a68b);
        mixed ^= mixed >> 16;
        *word = mixed;
    }
    words
}

fn checksum_words(words: &[u32; SDIO_CMD53_BLOCK_WORDS]) -> u32 {
    let mut checksum = FNV_OFFSET_BASIS;
    for word in words.iter() {
        checksum ^= *word;
        checksum = checksum.wrapping_mul(FNV_PRIME);
    }
    checksum
}

fn checksum_byte(checksum: u32, byte: u8) -> u32 {
    (checksum ^ byte as u32).wrapping_mul(FNV_PRIME)
}

fn checksum_bytes(bytes: &[u8]) -> u32 {
    let mut checksum = FNV_OFFSET_BASIS;
    for &byte in bytes {
        checksum = checksum_byte(checksum, byte);
    }
    checksum
}

fn checksum_padded_bytes(bytes: &[u8], rounded_len: usize) -> u32 {
    let mut checksum = FNV_OFFSET_BASIS;
    for index in 0..rounded_len {
        let byte = if index < bytes.len() { bytes[index] } else { 0 };
        checksum = checksum_byte(checksum, byte);
    }
    checksum
}

fn firmware_chunk_count(byte_count: usize) -> u32 {
    ((byte_count + SDIO_CMD53_BLOCK_BYTES as usize - 1) / SDIO_CMD53_BLOCK_BYTES as usize) as u32
}

fn firmware_chunk_len(byte_count: usize, offset: usize) -> usize {
    let remaining = byte_count - offset;
    if remaining < SDIO_CMD53_BLOCK_BYTES as usize {
        remaining
    } else {
        SDIO_CMD53_BLOCK_BYTES as usize
    }
}

fn firmware_chunk_words(
    firmware: &[u8],
    offset: usize,
    chunk_len: usize,
) -> [u32; SDIO_CMD53_BLOCK_WORDS] {
    let mut words = [0u32; SDIO_CMD53_BLOCK_WORDS];
    for chunk_offset in 0..chunk_len {
        let word_index = chunk_offset / 4;
        let shift = (chunk_offset & 0x03) * 8;
        words[word_index] |= (firmware[offset + chunk_offset] as u32) << shift;
    }
    words
}

fn word_byte(words: &[u32; SDIO_CMD53_BLOCK_WORDS], byte_index: usize) -> u8 {
    ((words[byte_index / 4] >> ((byte_index & 0x03) * 8)) & 0xff) as u8
}

fn wait_ht_available(
    core: &sdhc0::CORE,
    last_response: &mut u32,
) -> Result<(u8, u16), ClockWaitError> {
    let mut attempts = 0u16;
    let mut value = 0u8;
    while attempts < SDIO_HT_AVAIL_MAX_ATTEMPTS {
        attempts += 1;
        let read =
            cmd52_read_function_byte_selected(core, SDIO_BACKPLANE_FUNCTION, SDIO_CHIP_CLOCK_CSR)
                .map_err(|error| ClockWaitError::Read { attempts, error })?;
        value = read.0;
        *last_response = read.1;
        if value & SBSDIO_HT_AVAIL != 0 {
            return Ok((value, attempts));
        }
        delay_ms(1);
    }

    Err(ClockWaitError::Timeout { value, attempts })
}

fn wait_f2_ready(core: &sdhc0::CORE, last_response: &mut u32) -> Result<(u8, u16), ReadyWaitError> {
    let mut attempts = 0u16;
    let mut value = 0u8;
    while attempts < SDIO_F2_READY_MAX_ATTEMPTS {
        attempts += 1;
        let read = cmd52_read_byte_selected(core, SDIO_CCCR_IO_READY)
            .map_err(|error| ReadyWaitError::Read { attempts, error })?;
        value = read.0;
        *last_response = read.1;
        if value & SDIO_FUNCTION_READY_2 != 0 {
            return Ok((value, attempts));
        }
        delay_ms(1);
    }

    Err(ReadyWaitError::Timeout { value, attempts })
}

fn rounded_nvram_len(byte_count: usize) -> usize {
    let remainder = byte_count % NVRAM_IMAGE_SIZE_ALIGNMENT;
    if remainder == 0 {
        byte_count
    } else {
        byte_count + NVRAM_IMAGE_SIZE_ALIGNMENT - remainder
    }
}

fn nvram_download_address(rounded_len: usize) -> u32 {
    CYW43430_SOCSRAM_BYTES as u32 - 4 - rounded_len as u32
}

fn nvram_size_word(rounded_len: usize) -> u32 {
    let word_count = (rounded_len / 4) as u32;
    (!word_count << 16) | word_count
}

fn padded_byte(bytes: &[u8], index: usize) -> u8 {
    if index < bytes.len() {
        bytes[index]
    } else {
        0
    }
}

fn padded_word(bytes: &[u8], offset: usize) -> u32 {
    let b0 = padded_byte(bytes, offset) as u32;
    let b1 = padded_byte(bytes, offset + 1) as u32;
    let b2 = padded_byte(bytes, offset + 2) as u32;
    let b3 = padded_byte(bytes, offset + 3) as u32;
    b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
}

fn write_nvram_image_after_setup(
    core: &sdhc0::CORE,
    nvram: &[u8],
    last_response: &mut u32,
) -> Result<NvramWrite, NvramWriteError> {
    let rounded_len = rounded_nvram_len(nvram.len());
    if rounded_len == 0 {
        return Err(NvramWriteError::TooLarge);
    }
    if rounded_len > CYW43430_SOCSRAM_BYTES - 4 {
        return Err(NvramWriteError::TooLarge);
    }

    let address = nvram_download_address(rounded_len);
    let mut offset = 0usize;
    while rounded_len - offset >= SDIO_CMD53_BLOCK_BYTES as usize {
        let chunk_len = SDIO_CMD53_BLOCK_BYTES as usize;
        let words = firmware_chunk_words(nvram, offset, chunk_len);
        let write = write_backplane_block_after_setup(core, address + offset as u32, &words)
            .map_err(NvramWriteError::Block)?;
        *last_response = write.response;
        offset += SDIO_CMD53_BLOCK_BYTES as usize;
    }

    while offset < rounded_len {
        let value = padded_word(nvram, offset);
        let write =
            write_backplane_u32_after_setup(core, address + offset as u32, value, *last_response)
                .map_err(NvramWriteError::Word)?;
        *last_response = write.response;
        offset += 4;
    }

    Ok(NvramWrite {
        address,
        rounded_bytes: rounded_len as u32,
        size_word: nvram_size_word(rounded_len),
        checksum: checksum_padded_bytes(nvram, rounded_len),
        last_response: *last_response,
    })
}

fn verify_nvram_image_after_setup(
    core: &sdhc0::CORE,
    nvram: &[u8],
    last_response: &mut u32,
) -> Result<NvramVerify, BackplaneBlockReadError> {
    let rounded_len = rounded_nvram_len(nvram.len());
    let address = nvram_download_address(rounded_len);
    let mut verify_checksum = FNV_OFFSET_BASIS;
    let mut mismatch_offset = SOCSRAM_BLOCK_PROBE_NO_MISMATCH;
    let mut mismatch_expected = 0;
    let mut mismatch_actual = 0;
    let mut offset = 0usize;

    while rounded_len - offset >= SDIO_CMD53_BLOCK_BYTES as usize {
        let read = read_backplane_block_after_setup(core, address + offset as u32)?;
        *last_response = read.response;
        for chunk_offset in 0..SDIO_CMD53_BLOCK_BYTES as usize {
            let expected = padded_byte(nvram, offset + chunk_offset);
            let actual = word_byte(&read.words, chunk_offset);
            verify_checksum = checksum_byte(verify_checksum, actual);
            if mismatch_offset == SOCSRAM_BLOCK_PROBE_NO_MISMATCH && expected != actual {
                mismatch_offset = (offset + chunk_offset) as u32;
                mismatch_expected = expected as u32;
                mismatch_actual = actual as u32;
            }
        }
        offset += SDIO_CMD53_BLOCK_BYTES as usize;
    }

    while offset < rounded_len {
        let read =
            read_backplane_u32_bytes(core, address + offset as u32).map_err(
                |error| match error {
                    BackplaneByteReadError::Window(error) => BackplaneBlockReadError::Window(error),
                    BackplaneByteReadError::Cmd52(error) => BackplaneBlockReadError::Cmd53(error),
                },
            )?;
        *last_response = read.response;
        for byte_index in 0..4 {
            let expected = padded_byte(nvram, offset + byte_index);
            let actual = ((read.value >> (byte_index * 8)) & 0xff) as u8;
            verify_checksum = checksum_byte(verify_checksum, actual);
            if mismatch_offset == SOCSRAM_BLOCK_PROBE_NO_MISMATCH && expected != actual {
                mismatch_offset = (offset + byte_index) as u32;
                mismatch_expected = expected as u32;
                mismatch_actual = actual as u32;
            }
        }
        offset += 4;
    }

    Ok(NvramVerify {
        checksum: verify_checksum,
        mismatch_offset,
        mismatch_expected,
        mismatch_actual,
        last_response: *last_response,
    })
}

fn mismatch_words(
    expected: &[u32; SDIO_CMD53_BLOCK_WORDS],
    actual: &[u32; SDIO_CMD53_BLOCK_WORDS],
) -> (u32, u32, u32) {
    for index in 0..SDIO_CMD53_BLOCK_WORDS {
        if expected[index] != actual[index] {
            return (index as u32, expected[index], actual[index]);
        }
    }

    (SOCSRAM_BLOCK_PROBE_NO_MISMATCH, 0, 0)
}

fn response_or_last(response: u32, last_response: u32) -> u32 {
    if response == 0 {
        last_response
    } else {
        response
    }
}

pub fn core_state(p: &Peripherals, base: u32) -> WifiSdioCoreStateReport {
    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return core_state_report(
            p,
            WifiSdioCoreStateStatus::SetupFailed,
            setup.status,
            base,
            0,
            0,
            0,
            0,
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let mut last_response = match request_alp_clock(core) {
        Ok(response) => response,
        Err(error) => {
            let (status, response, last_error) = core_state_alp_error(error);
            return core_state_report(p, status, setup.status, base, 0, 0, 0, response, last_error);
        }
    };

    let ioctrl = match read_backplane_u8(core, base + AI_IOCTRL_OFFSET) {
        Ok(read) => {
            last_response = read.response;
            read.value
        }
        Err(error) => {
            let (status, last_error) =
                core_state_backplane_byte_error(error, WifiSdioCoreStateStatus::IoctrlReadFailed);
            return core_state_report(
                p,
                status,
                setup.status,
                base,
                0,
                0,
                0,
                last_response,
                Some(last_error),
            );
        }
    };

    let resetctrl = match read_backplane_u8(core, base + AI_RESETCTRL_OFFSET) {
        Ok(read) => {
            last_response = read.response;
            read.value
        }
        Err(error) => {
            let (status, last_error) = core_state_backplane_byte_error(
                error,
                WifiSdioCoreStateStatus::ResetctrlReadFailed,
            );
            return core_state_report(
                p,
                status,
                setup.status,
                base,
                ioctrl,
                0,
                0,
                last_response,
                Some(last_error),
            );
        }
    };

    let resetstatus = match read_backplane_u8(core, base + AI_RESETSTATUS_OFFSET) {
        Ok(read) => {
            last_response = read.response;
            read.value
        }
        Err(error) => {
            let (status, last_error) = core_state_backplane_byte_error(
                error,
                WifiSdioCoreStateStatus::ResetstatusReadFailed,
            );
            return core_state_report(
                p,
                status,
                setup.status,
                base,
                ioctrl,
                resetctrl,
                0,
                last_response,
                Some(last_error),
            );
        }
    };

    core_state_report(
        p,
        WifiSdioCoreStateStatus::Ready,
        setup.status,
        base,
        ioctrl,
        resetctrl,
        resetstatus,
        last_response,
        None,
    )
}

pub fn reset_core(p: &Peripherals, base: u32) -> WifiSdioCoreResetReport {
    let setup = setup_backplane(p);
    if !matches!(setup.status, WifiSdioBackplaneStatus::Ready) {
        return core_reset_report(
            p,
            WifiSdioCoreResetStatus::SetupFailed,
            setup.status,
            base,
            empty_core_snapshot(),
            empty_core_snapshot(),
            0,
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    let mut last_response = match request_alp_clock(core) {
        Ok(response) => response,
        Err(error) => {
            let (status, response, last_error) = core_reset_alp_error(error);
            return core_reset_report(
                p,
                status,
                setup.status,
                base,
                empty_core_snapshot(),
                empty_core_snapshot(),
                response,
                last_error,
            );
        }
    };

    let before = match read_core_snapshot(core, base) {
        Ok((snapshot, response)) => {
            last_response = response;
            snapshot
        }
        Err(error) => {
            let (status, last_error) = before_core_snapshot_error(error);
            return core_reset_report(
                p,
                status,
                setup.status,
                base,
                empty_core_snapshot(),
                empty_core_snapshot(),
                last_response,
                Some(last_error),
            );
        }
    };

    if let Err(error) = reset_core_registers(core, base, &mut last_response) {
        let (status, last_error) = core_reset_step_error(error);
        return core_reset_report(
            p,
            status,
            setup.status,
            base,
            before,
            empty_core_snapshot(),
            last_response,
            Some(last_error),
        );
    }

    let after = match read_core_snapshot(core, base) {
        Ok((snapshot, response)) => {
            last_response = response;
            snapshot
        }
        Err(error) => {
            let (status, last_error) = after_core_snapshot_error(error);
            return core_reset_report(
                p,
                status,
                setup.status,
                base,
                before,
                empty_core_snapshot(),
                last_response,
                Some(last_error),
            );
        }
    };

    core_reset_report(
        p,
        WifiSdioCoreResetStatus::Ready,
        setup.status,
        base,
        before,
        after,
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
            [0; SDIO_CMD53_PREVIEW_BYTES],
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
            [0; SDIO_CMD53_PREVIEW_BYTES],
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
            [0; SDIO_CMD53_PREVIEW_BYTES],
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
            [0; SDIO_CMD53_PREVIEW_BYTES],
            setup.last_error,
        );
    }

    let core = &p.SDHC0.core;
    if let Err(error) = request_alp_clock(core) {
        let (status, response, last_error) = cmd53_read_alp_error(error);
        return cmd53_read_report(
            p,
            status,
            setup.status,
            function,
            address,
            count,
            response,
            [0; SDIO_CMD53_PREVIEW_BYTES],
            last_error,
        );
    }

    let block_mode = count as u16 == SDIO_CMD53_BLOCK_BYTES;
    if !configure_cmd53_read(core, count, block_mode) {
        return cmd53_read_report(
            p,
            WifiSdioCmd53ReadStatus::DataSetupBusy,
            setup.status,
            function,
            address,
            count,
            0,
            [0; SDIO_CMD53_PREVIEW_BYTES],
            None,
        );
    }

    let command_count = if block_mode { 1 } else { count as u16 };
    let transfer_bytes = if block_mode {
        SDIO_CMD53_BLOCK_BYTES as u8
    } else {
        count
    };
    let argument = cmd53_argument(false, function, block_mode, true, address, command_count);
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
                [0; SDIO_CMD53_PREVIEW_BYTES],
                Some(error),
            )
        }
    };

    let bytes = match read_cmd53_bytes(core, transfer_bytes) {
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
                [0; SDIO_CMD53_PREVIEW_BYTES],
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

fn request_alp_clock(core: &sdhc0::CORE) -> Result<u32, AlpError> {
    let alp_request = SBSDIO_FORCE_HW_CLKREQ_OFF | SBSDIO_ALP_AVAIL_REQ | SBSDIO_FORCE_ALP;
    let mut alp_response = cmd52_write_function_byte_selected(
        core,
        SDIO_BACKPLANE_FUNCTION,
        SDIO_CHIP_CLOCK_CSR,
        alp_request,
    )
    .map_err(AlpError::Write)?;

    let mut attempts = 0;
    let mut clock_csr = 0;
    while attempts < SDIO_ALP_AVAIL_MAX_ATTEMPTS {
        attempts += 1;
        let read =
            cmd52_read_function_byte_selected(core, SDIO_BACKPLANE_FUNCTION, SDIO_CHIP_CLOCK_CSR)
                .map_err(|error| AlpError::Read {
                response: alp_response,
                error,
            })?;
        clock_csr = read.0;
        alp_response = read.1;
        if clock_csr & SBSDIO_ALP_AVAIL != 0 {
            break;
        }
        delay_ms(1);
    }

    if clock_csr & SBSDIO_ALP_AVAIL == 0 {
        return Err(AlpError::Timeout {
            response: alp_response,
        });
    }

    cmd52_write_function_byte_selected(core, SDIO_BACKPLANE_FUNCTION, SDIO_CHIP_CLOCK_CSR, 0)
        .map_err(|error| AlpError::Clear {
            response: alp_response,
            error,
        })?;

    Ok(alp_response)
}

fn cmd53_read_alp_error(error: AlpError) -> (WifiSdioCmd53ReadStatus, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (WifiSdioCmd53ReadStatus::AlpWriteFailed, 0, Some(error)),
        AlpError::Read { response, error } => (
            WifiSdioCmd53ReadStatus::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => (WifiSdioCmd53ReadStatus::AlpTimeout, response, None),
        AlpError::Clear { response, error } => (
            WifiSdioCmd53ReadStatus::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn backplane_read_alp_error(
    error: AlpError,
) -> (WifiSdioBackplaneReadStatus, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (WifiSdioBackplaneReadStatus::AlpWriteFailed, 0, Some(error)),
        AlpError::Read { response, error } => (
            WifiSdioBackplaneReadStatus::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => (WifiSdioBackplaneReadStatus::AlpTimeout, response, None),
        AlpError::Clear { response, error } => (
            WifiSdioBackplaneReadStatus::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn backplane_write8_alp_error(
    error: AlpError,
) -> (WifiSdioBackplaneWrite8Status, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (
            WifiSdioBackplaneWrite8Status::AlpWriteFailed,
            0,
            Some(error),
        ),
        AlpError::Read { response, error } => (
            WifiSdioBackplaneWrite8Status::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => {
            (WifiSdioBackplaneWrite8Status::AlpTimeout, response, None)
        }
        AlpError::Clear { response, error } => (
            WifiSdioBackplaneWrite8Status::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn backplane_write32_alp_error(
    error: AlpError,
) -> (WifiSdioBackplaneWrite32Status, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (
            WifiSdioBackplaneWrite32Status::AlpWriteFailed,
            0,
            Some(error),
        ),
        AlpError::Read { response, error } => (
            WifiSdioBackplaneWrite32Status::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => {
            (WifiSdioBackplaneWrite32Status::AlpTimeout, response, None)
        }
        AlpError::Clear { response, error } => (
            WifiSdioBackplaneWrite32Status::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn socram_probe_alp_error(
    error: AlpError,
) -> (WifiSdioSocramProbeStatus, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (WifiSdioSocramProbeStatus::AlpWriteFailed, 0, Some(error)),
        AlpError::Read { response, error } => (
            WifiSdioSocramProbeStatus::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => (WifiSdioSocramProbeStatus::AlpTimeout, response, None),
        AlpError::Clear { response, error } => (
            WifiSdioSocramProbeStatus::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn socram_block_probe_alp_error(
    error: AlpError,
) -> (WifiSdioSocramBlockProbeStatus, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (
            WifiSdioSocramBlockProbeStatus::AlpWriteFailed,
            0,
            Some(error),
        ),
        AlpError::Read { response, error } => (
            WifiSdioSocramBlockProbeStatus::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => {
            (WifiSdioSocramBlockProbeStatus::AlpTimeout, response, None)
        }
        AlpError::Clear { response, error } => (
            WifiSdioSocramBlockProbeStatus::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn firmware_load_alp_error(
    error: AlpError,
) -> (WifiSdioFirmwareLoadStatus, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (WifiSdioFirmwareLoadStatus::AlpWriteFailed, 0, Some(error)),
        AlpError::Read { response, error } => (
            WifiSdioFirmwareLoadStatus::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => (WifiSdioFirmwareLoadStatus::AlpTimeout, response, None),
        AlpError::Clear { response, error } => (
            WifiSdioFirmwareLoadStatus::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn nvram_write_error(
    error: NvramWriteError,
) -> (
    WifiSdioFirmwareStartStatus,
    WifiSdioBackplaneWrite32Status,
    u32,
    Option<CommandError>,
) {
    match error {
        NvramWriteError::TooLarge => (
            WifiSdioFirmwareStartStatus::NvramTooLarge,
            WifiSdioBackplaneWrite32Status::InvalidAddress,
            0,
            None,
        ),
        NvramWriteError::Block(error) => {
            let (status, response, last_error) = backplane_block_write_error(error);
            (
                WifiSdioFirmwareStartStatus::NvramWriteFailed,
                status,
                response,
                last_error,
            )
        }
        NvramWriteError::Word(error) => (
            WifiSdioFirmwareStartStatus::NvramWriteFailed,
            error.status,
            error.response,
            error.last_error,
        ),
    }
}

fn backplane_write8_window_or_cmd_error(
    error: BackplaneByteWriteError,
) -> (WifiSdioBackplaneWrite8Status, CommandError) {
    match error {
        BackplaneByteWriteError::Window(error) => backplane_write8_window_error(error),
        BackplaneByteWriteError::Cmd52(error) => {
            (WifiSdioBackplaneWrite8Status::Cmd52Failed, error)
        }
    }
}

fn core_state_alp_error(error: AlpError) -> (WifiSdioCoreStateStatus, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (WifiSdioCoreStateStatus::AlpWriteFailed, 0, Some(error)),
        AlpError::Read { response, error } => (
            WifiSdioCoreStateStatus::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => (WifiSdioCoreStateStatus::AlpTimeout, response, None),
        AlpError::Clear { response, error } => (
            WifiSdioCoreStateStatus::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn core_reset_alp_error(error: AlpError) -> (WifiSdioCoreResetStatus, u32, Option<CommandError>) {
    match error {
        AlpError::Write(error) => (WifiSdioCoreResetStatus::AlpWriteFailed, 0, Some(error)),
        AlpError::Read { response, error } => (
            WifiSdioCoreResetStatus::AlpReadFailed,
            response,
            Some(error),
        ),
        AlpError::Timeout { response } => (WifiSdioCoreResetStatus::AlpTimeout, response, None),
        AlpError::Clear { response, error } => (
            WifiSdioCoreResetStatus::AlpClearFailed,
            response,
            Some(error),
        ),
    }
}

fn read_core_snapshot(
    core: &sdhc0::CORE,
    base: u32,
) -> Result<(WifiSdioCoreSnapshot, u32), CoreStateReadError> {
    let ioctrl = read_backplane_u8(core, base + AI_IOCTRL_OFFSET)
        .map_err(|error| core_state_read_error(error, CoreRegisterAccess::Ioctrl))?;
    let resetctrl = read_backplane_u8(core, base + AI_RESETCTRL_OFFSET)
        .map_err(|error| core_state_read_error(error, CoreRegisterAccess::Resetctrl))?;
    let resetstatus = read_backplane_u8(core, base + AI_RESETSTATUS_OFFSET)
        .map_err(|error| core_state_read_error(error, CoreRegisterAccess::Resetstatus))?;

    Ok((
        core_snapshot_from_registers(ioctrl.value, resetctrl.value, resetstatus.value),
        resetstatus.response,
    ))
}

fn core_state_read_error(
    error: BackplaneByteReadError,
    access: CoreRegisterAccess,
) -> CoreStateReadError {
    match error {
        BackplaneByteReadError::Window(error) => CoreStateReadError::Window(error),
        BackplaneByteReadError::Cmd52(error) => CoreStateReadError::Cmd52 { access, error },
    }
}

fn reset_core_registers(
    core: &sdhc0::CORE,
    base: u32,
    last_response: &mut u32,
) -> Result<(), CoreResetStepError> {
    disable_core_registers(core, base, last_response)?;

    *last_response = write_backplane_u8(core, base + AI_IOCTRL_OFFSET, CORE_CONTROL_FORCE_CLOCKED)
        .map_err(CoreResetStepError::ResetIoctrlWrite)?
        .response;
    *last_response = read_backplane_u8(core, base + AI_IOCTRL_OFFSET)
        .map_err(CoreResetStepError::ResetIoctrlRead)?
        .response;
    *last_response = write_backplane_u8(core, base + AI_RESETCTRL_OFFSET, CORE_CONTROL_NONE)
        .map_err(CoreResetStepError::ResetctrlWrite)?
        .response;
    delay_ms(1);
    *last_response = write_backplane_u8(core, base + AI_IOCTRL_OFFSET, CORE_CONTROL_CLOCKED)
        .map_err(CoreResetStepError::FinalIoctrlWrite)?
        .response;
    Ok(())
}

fn disable_core_registers(
    core: &sdhc0::CORE,
    base: u32,
    last_response: &mut u32,
) -> Result<(), CoreResetStepError> {
    let resetctrl = read_backplane_u8(core, base + AI_RESETCTRL_OFFSET)
        .map_err(CoreResetStepError::DisableResetctrlRead)?;
    *last_response = resetctrl.response;
    let resetctrl = read_backplane_u8(core, base + AI_RESETCTRL_OFFSET)
        .map_err(CoreResetStepError::DisableResetctrlRead)?;
    *last_response = resetctrl.response;
    if resetctrl.value & CORE_CONTROL_RESET != 0 {
        return Ok(());
    }

    *last_response = write_backplane_u8(core, base + AI_IOCTRL_OFFSET, CORE_CONTROL_NONE)
        .map_err(CoreResetStepError::DisableIoctrlWrite)?
        .response;
    *last_response = read_backplane_u8(core, base + AI_IOCTRL_OFFSET)
        .map_err(CoreResetStepError::DisableIoctrlRead)?
        .response;
    delay_ms(1);
    *last_response = write_backplane_u8(core, base + AI_RESETCTRL_OFFSET, CORE_CONTROL_RESET)
        .map_err(CoreResetStepError::DisableResetctrlWrite)?
        .response;
    delay_ms(1);
    Ok(())
}

fn before_core_snapshot_error(
    error: CoreStateReadError,
) -> (WifiSdioCoreResetStatus, CommandError) {
    match error {
        CoreStateReadError::Window(error) => core_reset_window_error(error),
        CoreStateReadError::Cmd52 { access, error } => {
            let status = match access {
                CoreRegisterAccess::Ioctrl => WifiSdioCoreResetStatus::BeforeIoctrlReadFailed,
                CoreRegisterAccess::Resetctrl => WifiSdioCoreResetStatus::BeforeResetctrlReadFailed,
                CoreRegisterAccess::Resetstatus => {
                    WifiSdioCoreResetStatus::BeforeResetstatusReadFailed
                }
            };
            (status, error)
        }
    }
}

fn after_core_snapshot_error(error: CoreStateReadError) -> (WifiSdioCoreResetStatus, CommandError) {
    match error {
        CoreStateReadError::Window(error) => core_reset_window_error(error),
        CoreStateReadError::Cmd52 { access, error } => {
            let status = match access {
                CoreRegisterAccess::Ioctrl => WifiSdioCoreResetStatus::AfterIoctrlReadFailed,
                CoreRegisterAccess::Resetctrl => WifiSdioCoreResetStatus::AfterResetctrlReadFailed,
                CoreRegisterAccess::Resetstatus => {
                    WifiSdioCoreResetStatus::AfterResetstatusReadFailed
                }
            };
            (status, error)
        }
    }
}

fn core_reset_step_error(error: CoreResetStepError) -> (WifiSdioCoreResetStatus, CommandError) {
    match error {
        CoreResetStepError::DisableResetctrlRead(error) => {
            core_reset_byte_read_error(error, WifiSdioCoreResetStatus::DisableResetctrlReadFailed)
        }
        CoreResetStepError::DisableIoctrlWrite(error) => {
            core_reset_byte_write_error(error, WifiSdioCoreResetStatus::DisableIoctrlWriteFailed)
        }
        CoreResetStepError::DisableIoctrlRead(error) => {
            core_reset_byte_read_error(error, WifiSdioCoreResetStatus::DisableIoctrlReadFailed)
        }
        CoreResetStepError::DisableResetctrlWrite(error) => {
            core_reset_byte_write_error(error, WifiSdioCoreResetStatus::DisableResetctrlWriteFailed)
        }
        CoreResetStepError::ResetIoctrlWrite(error) => {
            core_reset_byte_write_error(error, WifiSdioCoreResetStatus::ResetIoctrlWriteFailed)
        }
        CoreResetStepError::ResetIoctrlRead(error) => {
            core_reset_byte_read_error(error, WifiSdioCoreResetStatus::ResetIoctrlReadFailed)
        }
        CoreResetStepError::ResetctrlWrite(error) => {
            core_reset_byte_write_error(error, WifiSdioCoreResetStatus::ResetctrlWriteFailed)
        }
        CoreResetStepError::FinalIoctrlWrite(error) => {
            core_reset_byte_write_error(error, WifiSdioCoreResetStatus::FinalIoctrlWriteFailed)
        }
    }
}

fn core_reset_byte_read_error(
    error: BackplaneByteReadError,
    cmd52_status: WifiSdioCoreResetStatus,
) -> (WifiSdioCoreResetStatus, CommandError) {
    match error {
        BackplaneByteReadError::Window(error) => core_reset_window_error(error),
        BackplaneByteReadError::Cmd52(error) => (cmd52_status, error),
    }
}

fn core_reset_byte_write_error(
    error: BackplaneByteWriteError,
    cmd52_status: WifiSdioCoreResetStatus,
) -> (WifiSdioCoreResetStatus, CommandError) {
    match error {
        BackplaneByteWriteError::Window(error) => core_reset_window_error(error),
        BackplaneByteWriteError::Cmd52(error) => (cmd52_status, error),
    }
}

fn core_reset_window_error(error: BackplaneWindowError) -> (WifiSdioCoreResetStatus, CommandError) {
    match error {
        BackplaneWindowError::High(error) => {
            (WifiSdioCoreResetStatus::WindowHighWriteFailed, error)
        }
        BackplaneWindowError::Mid(error) => (WifiSdioCoreResetStatus::WindowMidWriteFailed, error),
        BackplaneWindowError::Low(error) => (WifiSdioCoreResetStatus::WindowLowWriteFailed, error),
    }
}

fn backplane_write32_byte_readback_error(
    error: BackplaneByteReadError,
) -> (WifiSdioBackplaneWrite32Status, Option<CommandError>) {
    match error {
        BackplaneByteReadError::Window(error) => {
            let (status, error) = backplane_write32_window_error(error);
            (status, Some(error))
        }
        BackplaneByteReadError::Cmd52(error) => (
            WifiSdioBackplaneWrite32Status::ReadbackCmd52Failed,
            Some(error),
        ),
    }
}

fn backplane_block_read_error(
    error: BackplaneBlockReadError,
) -> (WifiSdioBackplaneReadStatus, u32, Option<CommandError>) {
    match error {
        BackplaneBlockReadError::InvalidAddress => {
            (WifiSdioBackplaneReadStatus::InvalidAddress, 0, None)
        }
        BackplaneBlockReadError::Window(error) => {
            let (status, error) = backplane_window_error(error);
            (status, 0, Some(error))
        }
        BackplaneBlockReadError::DataSetupBusy => {
            (WifiSdioBackplaneReadStatus::DataSetupBusy, 0, None)
        }
        BackplaneBlockReadError::Cmd53(error) => {
            (WifiSdioBackplaneReadStatus::Cmd53Failed, 0, Some(error))
        }
        BackplaneBlockReadError::Data { response, status } => {
            (backplane_read_data_status(status), response, None)
        }
    }
}

fn backplane_block_write_error(
    error: BackplaneBlockWriteError,
) -> (WifiSdioBackplaneWrite32Status, u32, Option<CommandError>) {
    match error {
        BackplaneBlockWriteError::InvalidAddress => {
            (WifiSdioBackplaneWrite32Status::InvalidAddress, 0, None)
        }
        BackplaneBlockWriteError::Window(error) => {
            let (status, error) = backplane_write32_window_error(error);
            (status, 0, Some(error))
        }
        BackplaneBlockWriteError::DataSetupBusy => {
            (WifiSdioBackplaneWrite32Status::DataSetupBusy, 0, None)
        }
        BackplaneBlockWriteError::Cmd53(error) => {
            (WifiSdioBackplaneWrite32Status::Cmd53Failed, 0, Some(error))
        }
        BackplaneBlockWriteError::Data { response, status } => (status, response, None),
    }
}

fn read_backplane_u8(
    core: &sdhc0::CORE,
    address: u32,
) -> Result<BackplaneByteRead, BackplaneByteReadError> {
    set_backplane_window(core, address).map_err(BackplaneByteReadError::Window)?;
    let window_address = backplane_window_address(address, 1);
    let read = cmd52_read_function_byte_selected(core, SDIO_BACKPLANE_FUNCTION, window_address)
        .map_err(BackplaneByteReadError::Cmd52)?;
    Ok(BackplaneByteRead {
        value: read.0,
        response: read.1,
    })
}

fn read_backplane_u32_bytes(
    core: &sdhc0::CORE,
    address: u32,
) -> Result<BackplaneWordRead, BackplaneByteReadError> {
    set_backplane_window(core, address).map_err(BackplaneByteReadError::Window)?;
    let window_address = backplane_window_address(address, 1);
    let mut bytes = [0u8; 4];
    let mut response = 0;
    for (offset, byte) in bytes.iter_mut().enumerate() {
        let read = cmd52_read_function_byte_selected(
            core,
            SDIO_BACKPLANE_FUNCTION,
            window_address + offset as u32,
        )
        .map_err(BackplaneByteReadError::Cmd52)?;
        *byte = read.0;
        response = read.1;
    }

    let value = (bytes[0] as u32)
        | ((bytes[1] as u32) << 8)
        | ((bytes[2] as u32) << 16)
        | ((bytes[3] as u32) << 24);

    Ok(BackplaneWordRead { value, response })
}

fn write_backplane_u8(
    core: &sdhc0::CORE,
    address: u32,
    value: u8,
) -> Result<BackplaneByteWrite, BackplaneByteWriteError> {
    set_backplane_window(core, address).map_err(BackplaneByteWriteError::Window)?;
    let window_address = backplane_window_address(address, 1);
    let response =
        cmd52_write_function_byte_selected(core, SDIO_BACKPLANE_FUNCTION, window_address, value)
            .map_err(BackplaneByteWriteError::Cmd52)?;
    Ok(BackplaneByteWrite { response })
}

fn set_backplane_window(core: &sdhc0::CORE, address: u32) -> Result<u32, BackplaneWindowError> {
    let base = backplane_window_base(address);
    cmd52_write_function_byte_selected(
        core,
        SDIO_BACKPLANE_FUNCTION,
        SDIO_BACKPLANE_ADDRESS_HIGH,
        (base >> 24) as u8,
    )
    .map_err(BackplaneWindowError::High)?;
    cmd52_write_function_byte_selected(
        core,
        SDIO_BACKPLANE_FUNCTION,
        SDIO_BACKPLANE_ADDRESS_MID,
        (base >> 16) as u8,
    )
    .map_err(BackplaneWindowError::Mid)?;
    cmd52_write_function_byte_selected(
        core,
        SDIO_BACKPLANE_FUNCTION,
        SDIO_BACKPLANE_ADDRESS_LOW,
        (base >> 8) as u8,
    )
    .map_err(BackplaneWindowError::Low)?;
    Ok(base)
}

fn backplane_window_error(
    error: BackplaneWindowError,
) -> (WifiSdioBackplaneReadStatus, CommandError) {
    match error {
        BackplaneWindowError::High(error) => {
            (WifiSdioBackplaneReadStatus::WindowHighWriteFailed, error)
        }
        BackplaneWindowError::Mid(error) => {
            (WifiSdioBackplaneReadStatus::WindowMidWriteFailed, error)
        }
        BackplaneWindowError::Low(error) => {
            (WifiSdioBackplaneReadStatus::WindowLowWriteFailed, error)
        }
    }
}

fn backplane_write8_window_error(
    error: BackplaneWindowError,
) -> (WifiSdioBackplaneWrite8Status, CommandError) {
    match error {
        BackplaneWindowError::High(error) => {
            (WifiSdioBackplaneWrite8Status::WindowHighWriteFailed, error)
        }
        BackplaneWindowError::Mid(error) => {
            (WifiSdioBackplaneWrite8Status::WindowMidWriteFailed, error)
        }
        BackplaneWindowError::Low(error) => {
            (WifiSdioBackplaneWrite8Status::WindowLowWriteFailed, error)
        }
    }
}

fn backplane_write32_window_error(
    error: BackplaneWindowError,
) -> (WifiSdioBackplaneWrite32Status, CommandError) {
    match error {
        BackplaneWindowError::High(error) => {
            (WifiSdioBackplaneWrite32Status::WindowHighWriteFailed, error)
        }
        BackplaneWindowError::Mid(error) => {
            (WifiSdioBackplaneWrite32Status::WindowMidWriteFailed, error)
        }
        BackplaneWindowError::Low(error) => {
            (WifiSdioBackplaneWrite32Status::WindowLowWriteFailed, error)
        }
    }
}

fn core_state_backplane_byte_error(
    error: BackplaneByteReadError,
    read_failed_status: WifiSdioCoreStateStatus,
) -> (WifiSdioCoreStateStatus, CommandError) {
    match error {
        BackplaneByteReadError::Window(error) => core_state_window_error(error),
        BackplaneByteReadError::Cmd52(error) => (read_failed_status, error),
    }
}

fn core_state_window_error(error: BackplaneWindowError) -> (WifiSdioCoreStateStatus, CommandError) {
    match error {
        BackplaneWindowError::High(error) => {
            (WifiSdioCoreStateStatus::WindowHighWriteFailed, error)
        }
        BackplaneWindowError::Mid(error) => (WifiSdioCoreStateStatus::WindowMidWriteFailed, error),
        BackplaneWindowError::Low(error) => (WifiSdioCoreStateStatus::WindowLowWriteFailed, error),
    }
}

fn backplane_window_base(address: u32) -> u32 {
    address & !SBSDIO_SB_OFT_ADDR_MASK
}

fn backplane_window_address(address: u32, count: u8) -> u32 {
    let mut window_address = address & SBSDIO_SB_OFT_ADDR_MASK;
    if count == 4 {
        window_address |= SBSDIO_SB_ACCESS_2_4B_FLAG;
    }
    window_address
}

fn is_valid_backplane_read_count(count: u8) -> bool {
    count == 1 || count == 2 || count == 4
}

fn backplane_read_crosses_window(address: u32, count: u8) -> bool {
    (address & SBSDIO_SB_OFT_ADDR_MASK) + count as u32 > SBSDIO_SB_OFT_ADDR_MASK + 1
}

fn is_valid_cmd53_read_count(count: u8) -> bool {
    count == 2
        || (count as u16 <= SDIO_CMD53_MAX_READ_BYTES && count as u16 % SDIO_CMD53_WORD_BYTES == 0)
}

fn configure_cmd53_read(core: &sdhc0::CORE, count: u8, block_mode: bool) -> bool {
    if !wait_command_and_data_lines_free(core) {
        return false;
    }

    let block_size = if block_mode {
        SDIO_CMD53_BLOCK_BYTES
    } else {
        count as u16
    };
    core.blocksize_r.write(|w| unsafe { w.bits(0) });
    core.xfer_mode_r.write(|w| unsafe { w.bits(0) });
    core.sdmasa_r.write(|w| unsafe { w.bits(1) });
    core.blocksize_r.write(|w| unsafe { w.bits(block_size) });
    core.blockcount_r.write(|w| unsafe { w.bits(1) });
    core.bgap_ctrl_r.write(|w| unsafe { w.bits(0) });
    core.tout_ctrl_r.write(|w| unsafe { w.bits(0x0e) });
    core.xfer_mode_r
        .write(|w| unsafe { w.bits(XFER_MODE_BLOCK_COUNT_ENABLE | XFER_MODE_READ) });
    clear_interrupts(core);

    true
}

fn configure_cmd53_write_adma(
    core: &sdhc0::CORE,
    count: u8,
    block_mode: bool,
    descriptor_address: u32,
) -> bool {
    if !wait_command_and_data_lines_free(core) {
        return false;
    }

    let block_size = if block_mode {
        SDIO_CMD53_BLOCK_BYTES
    } else {
        count as u16
    };
    select_adma2(core);
    core.blocksize_r.write(|w| unsafe { w.bits(0) });
    core.xfer_mode_r.write(|w| unsafe { w.bits(0) });
    core.adma_sa_low_r
        .write(|w| unsafe { w.bits(descriptor_address) });
    core.sdmasa_r.write(|w| unsafe { w.bits(0) });
    core.blocksize_r.write(|w| unsafe { w.bits(block_size) });
    core.blockcount_r.write(|w| unsafe { w.bits(1) });
    core.bgap_ctrl_r.write(|w| unsafe { w.bits(0) });
    core.mbiu_ctrl_r
        .write(|w| unsafe { w.bits(MBIU_CTRL_ALL_BURSTS) });
    core.tout_ctrl_r.write(|w| unsafe { w.bits(0x0d) });
    core.xfer_mode_r
        .write(|w| unsafe { w.bits(XFER_MODE_DMA_ENABLE | XFER_MODE_BLOCK_COUNT_ENABLE) });
    clear_interrupts(core);

    true
}

fn select_adma2(core: &sdhc0::CORE) {
    core.host_ctrl1_r.modify(|r, w| unsafe {
        let bits = (r.bits() & !HOST_CTRL1_DMA_SELECT_MASK) | HOST_CTRL1_DMA_SELECT_ADMA2;
        w.bits(bits)
    });
}

fn prepare_cmd53_write_word(value: u32, byte_count: u16) -> u32 {
    let descriptor =
        ADMA_ATTR_VALID | ADMA_ATTR_END | ADMA_ACT_TRAN | ((byte_count as u32) << ADMA_LEN_SHIFT);

    unsafe {
        let buffer = core::ptr::addr_of_mut!(CMD53_WRITE_WORD);
        core::ptr::write_volatile(buffer, value);

        let descriptor_table = core::ptr::addr_of_mut!(CMD53_WRITE_ADMA_DESCRIPTOR) as *mut u32;
        core::ptr::write_volatile(descriptor_table, descriptor);
        core::ptr::write_volatile(descriptor_table.add(1), buffer as u32);
        cortex_m::asm::dsb();

        descriptor_table as u32
    }
}

fn prepare_cmd53_write_words(words: &[u32; SDIO_CMD53_BLOCK_WORDS], byte_count: u16) -> u32 {
    let descriptor =
        ADMA_ATTR_VALID | ADMA_ATTR_END | ADMA_ACT_TRAN | ((byte_count as u32) << ADMA_LEN_SHIFT);

    unsafe {
        let buffer = core::ptr::addr_of_mut!(CMD53_WRITE_BLOCK_WORDS) as *mut u32;
        for (index, word) in words.iter().enumerate() {
            core::ptr::write_volatile(buffer.add(index), *word);
        }

        let descriptor_table = core::ptr::addr_of_mut!(CMD53_WRITE_ADMA_DESCRIPTOR) as *mut u32;
        core::ptr::write_volatile(descriptor_table, descriptor);
        core::ptr::write_volatile(descriptor_table.add(1), buffer as u32);
        cortex_m::asm::dsb();

        descriptor_table as u32
    }
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
) -> Result<[u8; SDIO_CMD53_PREVIEW_BYTES], WifiSdioCmd53ReadStatus> {
    if !wait_buffer_read_ready(core) {
        let _ = software_reset_data_line(core);
        return Err(WifiSdioCmd53ReadStatus::BufferReadTimeout);
    }

    let mut bytes = [0u8; SDIO_CMD53_PREVIEW_BYTES];
    let word_count =
        (count as usize + SDIO_CMD53_WORD_BYTES as usize - 1) / SDIO_CMD53_WORD_BYTES as usize;
    for word_index in 0..word_count {
        if !wait_buffer_read_enable(core) {
            let _ = software_reset_data_line(core);
            return Err(WifiSdioCmd53ReadStatus::BufferEnableTimeout);
        }

        let word = core.buf_data_r.read().bits();
        let offset = word_index * 4;
        if offset < SDIO_CMD53_PREVIEW_BYTES {
            bytes[offset] = word as u8;
            bytes[offset + 1] = (word >> 8) as u8;
            bytes[offset + 2] = (word >> 16) as u8;
            bytes[offset + 3] = (word >> 24) as u8;
        }
    }

    if !wait_transfer_complete(core) {
        let _ = software_reset_data_line(core);
        return Err(WifiSdioCmd53ReadStatus::TransferTimeout);
    }

    Ok(bytes)
}

fn read_cmd53_words(
    core: &sdhc0::CORE,
) -> Result<[u32; SDIO_CMD53_BLOCK_WORDS], WifiSdioCmd53ReadStatus> {
    if !wait_buffer_read_ready(core) {
        let _ = software_reset_data_line(core);
        return Err(WifiSdioCmd53ReadStatus::BufferReadTimeout);
    }

    let mut words = [0u32; SDIO_CMD53_BLOCK_WORDS];
    for word in words.iter_mut() {
        if !wait_buffer_read_enable(core) {
            let _ = software_reset_data_line(core);
            return Err(WifiSdioCmd53ReadStatus::BufferEnableTimeout);
        }

        *word = core.buf_data_r.read().bits();
    }

    if !wait_transfer_complete(core) {
        let _ = software_reset_data_line(core);
        return Err(WifiSdioCmd53ReadStatus::TransferTimeout);
    }

    Ok(words)
}

fn wait_cmd53_adma_write_complete(
    core: &sdhc0::CORE,
) -> Result<(), WifiSdioBackplaneWrite32Status> {
    if !wait_transfer_complete(core) {
        let _ = software_reset_command_and_data_lines(core);
        return Err(WifiSdioBackplaneWrite32Status::TransferTimeout);
    }
    cortex_m::asm::dsb();

    if !wait_data_line_free(core) {
        let _ = software_reset_command_and_data_lines(core);
        return Err(WifiSdioBackplaneWrite32Status::DataLineBusy);
    }

    Ok(())
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

fn backplane_read_data_status(status: WifiSdioCmd53ReadStatus) -> WifiSdioBackplaneReadStatus {
    match status {
        WifiSdioCmd53ReadStatus::BufferReadTimeout => {
            WifiSdioBackplaneReadStatus::BufferReadTimeout
        }
        WifiSdioCmd53ReadStatus::BufferEnableTimeout => {
            WifiSdioBackplaneReadStatus::BufferEnableTimeout
        }
        WifiSdioCmd53ReadStatus::TransferTimeout => WifiSdioBackplaneReadStatus::TransferTimeout,
        _ => WifiSdioBackplaneReadStatus::Cmd53Failed,
    }
}

fn backplane_read_report(
    p: &Peripherals,
    status: WifiSdioBackplaneReadStatus,
    setup_status: WifiSdioBackplaneStatus,
    address: u32,
    count: u8,
    window_base: u32,
    window_address: u32,
    response: u32,
    bytes: [u8; SDIO_CMD53_PREVIEW_BYTES],
    last_error: Option<CommandError>,
) -> WifiSdioBackplaneReadReport {
    WifiSdioBackplaneReadReport {
        status,
        setup_status,
        address,
        count,
        window_base,
        window_address,
        response,
        bytes,
        last_error,
        host: host_snapshot(p),
    }
}

fn backplane_write8_report(
    p: &Peripherals,
    status: WifiSdioBackplaneWrite8Status,
    setup_status: WifiSdioBackplaneStatus,
    address: u32,
    value: u8,
    window_base: u32,
    window_address: u32,
    response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioBackplaneWrite8Report {
    WifiSdioBackplaneWrite8Report {
        status,
        setup_status,
        address,
        value,
        window_base,
        window_address,
        response,
        last_error,
        host: host_snapshot(p),
    }
}

fn backplane_write32_report(
    p: &Peripherals,
    status: WifiSdioBackplaneWrite32Status,
    setup_status: WifiSdioBackplaneStatus,
    address: u32,
    value: u32,
    window_base: u32,
    window_address: u32,
    response: u32,
    readback: u32,
    last_error: Option<CommandError>,
) -> WifiSdioBackplaneWrite32Report {
    WifiSdioBackplaneWrite32Report {
        status,
        setup_status,
        address,
        value,
        window_base,
        window_address,
        response,
        readback,
        last_error,
        host: host_snapshot(p),
    }
}

fn socram_probe_report(
    p: &Peripherals,
    status: WifiSdioSocramProbeStatus,
    setup_status: WifiSdioBackplaneStatus,
    write_status: WifiSdioBackplaneWrite32Status,
    address: u32,
    pattern: u32,
    original: u32,
    readback: u32,
    restored: u32,
    last_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioSocramProbeReport {
    WifiSdioSocramProbeReport {
        status,
        setup_status,
        write_status,
        address,
        pattern,
        original,
        readback,
        restored,
        last_response,
        last_error,
        host: host_snapshot(p),
    }
}

fn socram_block_probe_report(
    p: &Peripherals,
    status: WifiSdioSocramBlockProbeStatus,
    setup_status: WifiSdioBackplaneStatus,
    read_status: WifiSdioBackplaneReadStatus,
    write_status: WifiSdioBackplaneWrite32Status,
    address: u32,
    seed: u32,
    original_checksum: u32,
    readback_checksum: u32,
    restored_checksum: u32,
    mismatch_index: u32,
    mismatch_expected: u32,
    mismatch_actual: u32,
    last_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioSocramBlockProbeReport {
    WifiSdioSocramBlockProbeReport {
        status,
        setup_status,
        read_status,
        write_status,
        address,
        seed,
        original_checksum,
        readback_checksum,
        restored_checksum,
        mismatch_index,
        mismatch_expected,
        mismatch_actual,
        last_response,
        last_error,
        host: host_snapshot(p),
    }
}

fn firmware_load_report(
    p: &Peripherals,
    status: WifiSdioFirmwareLoadStatus,
    setup_status: WifiSdioBackplaneStatus,
    read_status: WifiSdioBackplaneReadStatus,
    write_status: WifiSdioBackplaneWrite32Status,
    firmware_bytes: u32,
    processed_bytes: u32,
    chunk_count: u32,
    firmware_checksum: u32,
    verify_checksum: u32,
    mismatch_offset: u32,
    mismatch_expected: u32,
    mismatch_actual: u32,
    last_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioFirmwareLoadReport {
    WifiSdioFirmwareLoadReport {
        status,
        setup_status,
        read_status,
        write_status,
        firmware_bytes,
        processed_bytes,
        chunk_count,
        firmware_checksum,
        verify_checksum,
        mismatch_offset,
        mismatch_expected,
        mismatch_actual,
        last_response,
        last_error,
        host: host_snapshot(p),
    }
}

fn firmware_start_report(
    p: &Peripherals,
    status: WifiSdioFirmwareStartStatus,
    firmware_status: WifiSdioFirmwareLoadStatus,
    setup_status: WifiSdioBackplaneStatus,
    read_status: WifiSdioBackplaneReadStatus,
    write_status: WifiSdioBackplaneWrite32Status,
    firmware_bytes: u32,
    nvram_bytes: u32,
    nvram_rounded_bytes: u32,
    nvram_address: u32,
    nvram_size_word: u32,
    firmware_checksum: u32,
    nvram_checksum: u32,
    nvram_verify_checksum: u32,
    mismatch_offset: u32,
    mismatch_expected: u32,
    mismatch_actual: u32,
    arm_before: WifiSdioCoreSnapshot,
    arm_after: WifiSdioCoreSnapshot,
    ht_clock_csr: u8,
    ht_attempts: u16,
    io_enable: u8,
    io_ready: u8,
    f2_attempts: u16,
    last_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioFirmwareStartReport {
    WifiSdioFirmwareStartReport {
        status,
        firmware_status,
        setup_status,
        read_status,
        write_status,
        firmware_bytes,
        nvram_bytes,
        nvram_rounded_bytes,
        nvram_address,
        nvram_size_word,
        firmware_checksum,
        nvram_checksum,
        nvram_verify_checksum,
        mismatch_offset,
        mismatch_expected,
        mismatch_actual,
        arm_before,
        arm_after,
        ht_clock_csr,
        ht_attempts,
        io_enable,
        io_ready,
        f2_attempts,
        last_response,
        last_error,
        host: host_snapshot(p),
    }
}

fn firmware_start_command_error_report(
    p: &Peripherals,
    status: WifiSdioFirmwareStartStatus,
    firmware_report: &WifiSdioFirmwareLoadReport,
    nvram_write: &NvramWrite,
    nvram_verify: &NvramVerify,
    nvram_bytes: u32,
    last_response: u32,
    error: CommandError,
) -> WifiSdioFirmwareStartReport {
    firmware_start_report(
        p,
        status,
        firmware_report.status,
        firmware_report.setup_status,
        WifiSdioBackplaneReadStatus::Ready,
        WifiSdioBackplaneWrite32Status::Ready,
        firmware_report.firmware_bytes,
        nvram_bytes,
        nvram_write.rounded_bytes,
        nvram_write.address,
        nvram_write.size_word,
        firmware_report.firmware_checksum,
        nvram_write.checksum,
        nvram_verify.checksum,
        SOCSRAM_BLOCK_PROBE_NO_MISMATCH,
        0,
        0,
        empty_core_snapshot(),
        empty_core_snapshot(),
        0,
        0,
        0,
        0,
        0,
        last_response,
        Some(error),
    )
}

#[derive(Clone, Copy)]
struct F2Transfer {
    status: WifiSdioCmd53ReadStatus,
    response: u32,
    bytes: [u8; SDIO_CMD53_PREVIEW_BYTES],
    last_error: Option<CommandError>,
}

fn f2_read_bytes(p: &Peripherals, count: u8) -> F2Transfer {
    let core = &p.SDHC0.core;
    if !configure_cmd53_read(core, count, false) {
        return f2_transfer(WifiSdioCmd53ReadStatus::DataSetupBusy, 0, None);
    }

    let response = match send_command(
        core,
        Command {
            index: 53,
            argument: cmd53_argument(false, 2, false, true, 0, count as u16),
            response: ResponseType::Len48,
            command_type: CommandType::Normal,
            data_present: true,
            crc_check: true,
            index_check: true,
        },
    ) {
        Ok(response) => response,
        Err(error) => {
            return f2_transfer(WifiSdioCmd53ReadStatus::Cmd53Failed, 0, Some(error));
        }
    };

    let bytes = match read_cmd53_bytes(core, count) {
        Ok(bytes) => bytes,
        Err(status) => return f2_transfer(status, response, None),
    };

    F2Transfer {
        status: WifiSdioCmd53ReadStatus::Ready,
        response,
        bytes,
        last_error: None,
    }
}

fn f2_transfer(
    status: WifiSdioCmd53ReadStatus,
    response: u32,
    last_error: Option<CommandError>,
) -> F2Transfer {
    F2Transfer {
        status,
        response,
        bytes: [0; SDIO_CMD53_PREVIEW_BYTES],
        last_error,
    }
}

fn copy_bytes(
    destination: &mut [u8; SDIO_CMD53_PREVIEW_BYTES],
    destination_offset: usize,
    source: &[u8; SDIO_CMD53_PREVIEW_BYTES],
    count: u8,
) {
    let mut index = 0usize;
    while index < count as usize {
        destination[destination_offset + index] = source[index];
        index += 1;
    }
}

fn f2_header_report(
    p: &Peripherals,
    status: WifiSdioCmd53ReadStatus,
    response: u32,
    bytes: [u8; 4],
    last_error: Option<CommandError>,
) -> WifiSdioF2HeaderReport {
    let length = u16::from_le_bytes([bytes[0], bytes[1]]);
    let checksum = u16::from_le_bytes([bytes[2], bytes[3]]);
    WifiSdioF2HeaderReport {
        status,
        response,
        bytes,
        length,
        checksum,
        valid: length != 0 && (length ^ checksum) == 0xffff,
        last_error,
        host: host_snapshot(p),
    }
}

fn f2_frame_report(
    p: &Peripherals,
    status: WifiSdioF2FrameStatus,
    header_status: WifiSdioCmd53ReadStatus,
    body_status: WifiSdioCmd53ReadStatus,
    header_response: u32,
    body_response: u32,
    bytes: [u8; SDIO_CMD53_PREVIEW_BYTES],
    byte_count: u8,
    last_error: Option<CommandError>,
) -> WifiSdioF2FrameReport {
    let length = u16::from_le_bytes([bytes[0], bytes[1]]);
    let checksum = u16::from_le_bytes([bytes[2], bytes[3]]);
    let channel_and_flags = bytes[5];
    WifiSdioF2FrameReport {
        status,
        header_status,
        body_status,
        header_response,
        body_response,
        bytes,
        byte_count,
        length,
        checksum,
        valid: length != 0 && (length ^ checksum) == 0xffff,
        sequence: bytes[4],
        channel_and_flags,
        channel: channel_and_flags & 0x0f,
        flags: channel_and_flags >> 4,
        next_length: bytes[6],
        header_length: bytes[7],
        wireless_flow_control: bytes[8],
        bus_data_credit: bytes[9],
        reserved0: bytes[10],
        reserved1: bytes[11],
        last_error,
        host: host_snapshot(p),
    }
}

fn core_state_report(
    p: &Peripherals,
    status: WifiSdioCoreStateStatus,
    setup_status: WifiSdioBackplaneStatus,
    base: u32,
    ioctrl: u8,
    resetctrl: u8,
    resetstatus: u8,
    last_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioCoreStateReport {
    let snapshot = core_snapshot_from_registers(ioctrl, resetctrl, resetstatus);
    WifiSdioCoreStateReport {
        status,
        setup_status,
        base,
        ioctrl,
        resetctrl,
        resetstatus,
        clock_enabled: snapshot.clock_enabled,
        force_gated: snapshot.force_gated,
        in_reset: snapshot.in_reset,
        reset_busy: snapshot.reset_busy,
        core_up: snapshot.core_up,
        last_response,
        last_error,
        host: host_snapshot(p),
    }
}

fn empty_core_snapshot() -> WifiSdioCoreSnapshot {
    core_snapshot_from_registers(0, 0, 0)
}

fn core_snapshot_from_registers(
    ioctrl: u8,
    resetctrl: u8,
    resetstatus: u8,
) -> WifiSdioCoreSnapshot {
    let clock_enabled = ioctrl & SICF_CLOCK_EN != 0;
    let force_gated = ioctrl & SICF_FGC != 0;
    let in_reset = resetctrl & AIRC_RESET != 0;
    let reset_busy = resetstatus != 0;
    WifiSdioCoreSnapshot {
        ioctrl,
        resetctrl,
        resetstatus,
        clock_enabled,
        force_gated,
        in_reset,
        reset_busy,
        core_up: clock_enabled && !force_gated && !in_reset && !reset_busy,
    }
}

fn core_reset_report(
    p: &Peripherals,
    status: WifiSdioCoreResetStatus,
    setup_status: WifiSdioBackplaneStatus,
    base: u32,
    before: WifiSdioCoreSnapshot,
    after: WifiSdioCoreSnapshot,
    last_response: u32,
    last_error: Option<CommandError>,
) -> WifiSdioCoreResetReport {
    WifiSdioCoreResetReport {
        status,
        setup_status,
        base,
        before,
        after,
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
    bytes: [u8; SDIO_CMD53_PREVIEW_BYTES],
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
    let wrap_ctl = p.SDHC0.wrap.ctl.read().bits();
    if wrap_ctl & SDHC_WRAP_CTL_ENABLE == 0 {
        return disabled_host_snapshot(wrap_ctl);
    }

    let core = &p.SDHC0.core;
    WifiSdioHostSnapshot {
        wrap_ctl,
        gp_out: core.gp_out_r.read().bits(),
        gp_in: core.gp_in_r.read().bits(),
        xfer_mode: core.xfer_mode_r.read().bits(),
        block_size: core.blocksize_r.read().bits(),
        block_count: core.blockcount_r.read().bits(),
        sdmasa: core.sdmasa_r.read().bits(),
        adma_sa_low: core.adma_sa_low_r.read().bits(),
        adma_id_low: core.adma_id_low_r.read().bits(),
        adma_err_stat: core.adma_err_stat_r.read().bits(),
        bgap_ctrl: core.bgap_ctrl_r.read().bits(),
        host_ctrl1: core.host_ctrl1_r.read().bits(),
        host_ctrl2: core.host_ctrl2_r.read().bits(),
        capabilities1: core.capabilities1_r.read().bits(),
        capabilities2: core.capabilities2_r.read().bits(),
        mbiu_ctrl: core.mbiu_ctrl_r.read().bits(),
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

fn disabled_host_snapshot(wrap_ctl: u32) -> WifiSdioHostSnapshot {
    WifiSdioHostSnapshot {
        wrap_ctl,
        gp_out: 0,
        gp_in: 0,
        xfer_mode: 0,
        block_size: 0,
        block_count: 0,
        sdmasa: 0,
        adma_sa_low: 0,
        adma_id_low: 0,
        adma_err_stat: 0,
        bgap_ctrl: 0,
        host_ctrl1: 0,
        host_ctrl2: 0,
        capabilities1: 0,
        capabilities2: 0,
        mbiu_ctrl: 0,
        tout_ctrl: 0,
        clk_ctrl: 0,
        pwr_ctrl: 0,
        sw_rst: 0,
        normal_int: 0,
        error_int: 0,
        normal_int_stat_en: 0,
        error_int_stat_en: 0,
        normal_int_signal_en: 0,
        error_int_signal_en: 0,
        pstate: 0,
        cmd: 0,
        argument: 0,
        response01: 0,
        response23: 0,
        response45: 0,
        response67: 0,
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

fn software_reset_command_and_data_lines(core: &sdhc0::CORE) -> bool {
    let command_reset = software_reset_command_line(core);
    let data_reset = software_reset_data_line(core);
    command_reset && data_reset
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

fn wait_data_line_free(core: &sdhc0::CORE) -> bool {
    for _ in 0..1000 {
        if core.pstate_reg.read().bits() & PSTATE_DAT_LINE_ACTIVE == 0 {
            return true;
        }
        delay_us(3);
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
