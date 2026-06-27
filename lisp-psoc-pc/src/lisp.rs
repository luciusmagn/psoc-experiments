use core::fmt::{self, Write};

const MAX_OBJECTS: usize = 448;
const MAX_SYMBOLS: usize = 512;
const MAX_GLOBALS: usize = 112;
const MAX_SYMBOL_BYTES: usize = 32;
pub const MAX_STRING_BYTES: usize = 96;
pub const MAX_STORE_FILES: usize = 5;
const MAX_CALL_ARGS: usize = 16;
const MAX_EVAL_DEPTH: u8 = 128;
const PRETTY_INDENT: usize = 4;
const PRETTY_INLINE_LIST_LIMIT: u8 = 6;

type ObjectId = u16;
type SymbolId = u16;

#[derive(Clone, Copy)]
pub struct StringBytes {
    pub len: u8,
    pub bytes: [u8; MAX_STRING_BYTES],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LedAction {
    On,
    Off,
    Toggle,
    Status,
}

#[derive(Clone, Copy)]
pub struct Error {
    message: &'static str,
}

impl Error {
    pub const fn new(message: &'static str) -> Self {
        Self { message }
    }

    pub fn message(self) -> &'static str {
        self.message
    }
}

pub trait FatFormatProgress {
    fn report(&mut self, phase: &'static [u8], written_sector_count: u32, total_sectors: u32);
}

pub trait Board {
    fn led(&mut self, action: LedAction) -> bool;
    fn heartbeat(&mut self, enabled: bool) -> bool;
    fn button_pressed(&mut self, index: i32) -> Result<bool, Error>;
    fn millis(&mut self) -> u32;
    fn read32(&mut self, address: u32) -> Result<u32, Error>;
    fn write32(&mut self, address: u32, value: u32) -> Result<(), Error>;
    fn registers(&mut self) -> RegisterReport;
    fn sd_status(&mut self) -> SdStatusReport;
    fn sd_pins(&mut self) -> SdPinsReport;
    fn sd_pinmux(&mut self) -> SdPinsReport;
    fn sd_clock(&mut self) -> SdClockReport;
    fn sd_init(&mut self) -> SdInitReport;
    fn sd_read(&mut self, sector: u32) -> SdReadReport;
    fn sd_write_fill(&mut self, sector: u32, fill_word: u32) -> SdWriteReport;
    fn format_store(&mut self) -> StoreFormatReport;
    fn save_file(&mut self, path: StringBytes, content: StringBytes) -> StoreWriteReport;
    fn read_file(&mut self, path: StringBytes) -> StoreReadReport;
    fn list_files(&mut self) -> StoreListReport;
    fn fat_info(&mut self) -> FatInfoReport;
    fn fat_format(&mut self, progress: &mut dyn FatFormatProgress) -> FatFormatReport;
    fn wifi_sdio_init(&mut self) -> WifiSdioReport;
    fn wifi_cmd52_read(&mut self, function: u8, address: u32) -> WifiSdioDirectReport;
    fn wifi_cmd52_write(&mut self, function: u8, address: u32, data: u8) -> WifiSdioDirectReport;
    fn wifi_enable_functions(&mut self, requested: u8) -> WifiSdioEnableReport;
    fn wifi_setup_backplane(&mut self) -> WifiSdioBackplaneReport;
    fn wifi_cmd53_read(&mut self, function: u8, address: u32, count: u8)
        -> WifiSdioCmd53ReadReport;
    fn wifi_backplane_read(&mut self, address: u32, count: u8) -> WifiSdioBackplaneReadReport;
    fn wifi_backplane_write8(&mut self, address: u32, value: u8) -> WifiSdioBackplaneWrite8Report;
    fn wifi_backplane_write32(
        &mut self,
        address: u32,
        value: u32,
    ) -> WifiSdioBackplaneWrite32Report;
    fn wifi_backplane_write32_bytes(
        &mut self,
        address: u32,
        value: u32,
    ) -> WifiSdioBackplaneWrite32Report;
    fn wifi_socram_probe(&mut self, address: u32, pattern: u32) -> WifiSdioSocramProbeReport;
    fn wifi_socram_block_probe(
        &mut self,
        address: u32,
        seed: u32,
    ) -> WifiSdioSocramBlockProbeReport;
    fn wifi_load_firmware(&mut self) -> WifiSdioFirmwareLoadReport;
    fn wifi_start_firmware(&mut self) -> WifiSdioFirmwareStartReport;
    fn wifi_f2_read_header(&mut self) -> WifiSdioF2HeaderReport;
    fn wifi_f2_read_frame(&mut self) -> WifiSdioF2FrameReport;
    fn wifi_f2_read_frame_single(&mut self) -> WifiSdioF2FrameReport;
    fn wifi_f2_read_frame_exact(&mut self, count: u8) -> WifiSdioF2FrameReport;
    fn wifi_f2_read_frame_block(&mut self) -> WifiSdioF2FrameReport;
    fn wifi_send_wlc_up(&mut self) -> WifiSdioF2ControlReport;
    fn wifi_wlc_up(&mut self) -> WifiSdioWlcUpReport;
    fn wifi_get_version(&mut self) -> WifiSdioGetVersionReport;
    fn wifi_get_mpc(&mut self) -> WifiSdioGetMpcReport;
    fn wifi_load_clm(&mut self) -> WifiSdioClmLoadReport;
    fn wifi_get_clm_version(&mut self) -> WifiSdioGetClmVersionReport;
    fn wifi_f2_read_frame_abort(&mut self) -> WifiSdioF2AbortProbeReport;
    fn wifi_poll_read_frame(&mut self) -> WifiSdioPollReadFrameReport;
    fn wifi_ack_interrupts(&mut self) -> WifiSdioInterruptAckReport;
    fn wifi_interrupt_state(&mut self) -> WifiSdioInterruptStateReport;
    fn wifi_keep_awake(&mut self) -> WifiSdioKeepAwakeReport;
    fn wifi_request_ht(&mut self) -> WifiSdioHtRequestReport;
    fn wifi_host_reset_lines(&mut self) -> WifiSdioHostResetReport;
    fn wifi_abort_read(&mut self) -> WifiSdioAbortReadReport;
    fn wifi_core_state(&mut self, base: u32) -> WifiSdioCoreStateReport;
    fn wifi_reset_core(&mut self, base: u32) -> WifiSdioCoreResetReport;
    fn sdhc_registers(&mut self) -> SdhcReport;
    fn reboot(&mut self) -> !;
}

#[derive(Clone, Copy)]
pub struct RegisterReport {
    pub scb5_ctrl: u32,
    pub scb5_uart_ctrl: u32,
    pub scb5_rx_status: u32,
    pub scb5_tx_status: u32,
    pub peri_clock5: u32,
    pub peri_div8_0: u32,
    pub hsiom_prt5_sel0: u32,
    pub gpio_prt5_cfg: u32,
    pub gpio_prt13_out: u32,
    pub gpio_prt13_cfg: u32,
}

#[derive(Clone, Copy)]
pub struct SdStatusReport {
    pub cd_low: bool,
    pub prt13_in: u32,
    pub prt13_cfg: u32,
}

#[derive(Clone, Copy)]
pub struct SdPinsReport {
    pub p12_sel1: u32,
    pub p13_sel0: u32,
    pub p12_cfg: u32,
    pub p13_cfg: u32,
}

#[derive(Clone, Copy)]
pub struct SdhcCoreReport {
    pub wrap_ctl: u32,
    pub host_version: u16,
    pub cap1: u32,
    pub cap2: u32,
    pub pstate: u32,
}

#[derive(Clone, Copy)]
pub struct SdhcReport {
    pub sdhc0: SdhcCoreReport,
    pub sdhc1: SdhcCoreReport,
    pub pins: SdPinsReport,
}

#[derive(Clone, Copy)]
pub struct SdClockReport {
    pub path0: u32,
    pub root0: u32,
    pub root2: u32,
    pub fll_config: u32,
    pub fll_config2: u32,
    pub fll_status: u32,
    pub selected_hf_hz: u32,
}

#[derive(Clone, Copy)]
pub struct SdCommandErrorReport {
    pub code: &'static [u8],
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
pub struct SdInitReport {
    pub status: &'static [u8],
    pub cmd8_response: u32,
    pub cmd8_error: Option<SdCommandErrorReport>,
    pub acmd41_ocr: u32,
    pub acmd41_attempts: u16,
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
    pub last_error: Option<SdCommandErrorReport>,
}

#[derive(Clone, Copy)]
pub struct SdReadReport {
    pub status: &'static [u8],
    pub init_status: &'static [u8],
    pub sector: u32,
    pub rca: u16,
    pub ocr: u32,
    pub acmd41_attempts: u16,
    pub command_response: u32,
    pub last_error: Option<SdCommandErrorReport>,
    pub first_words: [u32; 8],
    pub mbr_signature: u16,
    pub partition_status: u8,
    pub partition_type: u8,
    pub partition_lba_start: u32,
    pub partition_sector_count: u32,
    pub normal_int: u16,
    pub error_int: u16,
    pub pstate: u32,
    pub block_size: u16,
    pub block_count: u16,
    pub xfer_mode: u16,
    pub cmd: u16,
    pub argument: u32,
}

#[derive(Clone, Copy)]
pub struct SdWriteReport {
    pub status: &'static [u8],
    pub init_status: &'static [u8],
    pub sector: u32,
    pub fill_word: u32,
    pub rca: u16,
    pub ocr: u32,
    pub acmd41_attempts: u16,
    pub command_response: u32,
    pub last_error: Option<SdCommandErrorReport>,
    pub normal_int: u16,
    pub error_int: u16,
    pub pstate: u32,
    pub block_size: u16,
    pub block_count: u16,
    pub xfer_mode: u16,
    pub cmd: u16,
    pub argument: u32,
}

#[derive(Clone, Copy)]
pub struct StoreWriteReport {
    pub ready: bool,
    pub status: &'static [u8],
    pub path_len: u8,
    pub content_len: u8,
    pub directory_sector: u32,
    pub data_sector: u32,
}

#[derive(Clone, Copy)]
pub struct StoreFormatReport {
    pub ready: bool,
    pub status: &'static [u8],
    pub directory_sector: u32,
    pub data_start_sector: u32,
    pub data_sector_count: u8,
    pub failed_sector: u32,
}

#[derive(Clone, Copy)]
pub struct StoreReadReport {
    pub ready: bool,
    pub status: &'static [u8],
    pub path_len: u8,
    pub content_len: u8,
    pub directory_sector: u32,
    pub data_sector: u32,
    pub content: StringBytes,
}

#[derive(Clone, Copy)]
pub struct StoreListReport {
    pub ready: bool,
    pub status: &'static [u8],
    pub file_count: u8,
    pub directory_sector: u32,
    pub files: [StringBytes; MAX_STORE_FILES],
}

#[derive(Clone, Copy)]
pub struct FatInfoReport {
    pub ready: bool,
    pub status: &'static [u8],
    pub mbr_signature: u16,
    pub partition_status: u8,
    pub partition_type: u8,
    pub partition_lba_start: u32,
    pub partition_sector_count: u32,
    pub root_entry_count: u8,
    pub sample_count: u8,
    pub entries: [StringBytes; MAX_STORE_FILES],
}

#[derive(Clone, Copy)]
pub struct FatFormatReport {
    pub ready: bool,
    pub status: &'static [u8],
    pub mbr_signature: u16,
    pub partition_status: u8,
    pub partition_type_before: u8,
    pub partition_type_after: u8,
    pub partition_lba_start: u32,
    pub partition_sector_count: u32,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_count: u8,
    pub fat_size_sectors: u32,
    pub data_cluster_count: u32,
    pub root_cluster: u32,
    pub written_sector_count: u32,
    pub failed_sector: u32,
}

#[derive(Clone, Copy)]
pub struct WifiSdioCommandErrorReport {
    pub code: &'static [u8],
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
pub struct WifiSdioHostReport {
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

#[derive(Clone, Copy)]
pub struct WifiSdioPinsReport {
    pub p2_sel0: u32,
    pub p2_sel1: u32,
    pub p2_cfg: u32,
    pub p2_out: u32,
    pub p2_in: u32,
}

#[derive(Clone, Copy)]
pub struct WifiSdioClockReport {
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

#[derive(Clone, Copy)]
pub struct WifiSdioReport {
    pub status: &'static [u8],
    pub cmd5_response: u32,
    pub cmd5_attempts: u16,
    pub rca: u16,
    pub function_count: u8,
    pub memory_present: bool,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
    pub pins: WifiSdioPinsReport,
    pub clock: WifiSdioClockReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioDirectReport {
    pub status: &'static [u8],
    pub init_status: &'static [u8],
    pub function: u8,
    pub address: u32,
    pub write: bool,
    pub data: u8,
    pub response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioEnableReport {
    pub status: &'static [u8],
    pub init_status: &'static [u8],
    pub requested: u8,
    pub ready: u8,
    pub attempts: u16,
    pub write_response: u32,
    pub ready_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioBackplaneReport {
    pub status: &'static [u8],
    pub init_status: &'static [u8],
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
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioCmd53ReadReport {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub function: u8,
    pub address: u32,
    pub count: u8,
    pub response: u32,
    pub bytes: [u8; 64],
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioBackplaneReadReport {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub address: u32,
    pub count: u8,
    pub window_base: u32,
    pub window_address: u32,
    pub response: u32,
    pub bytes: [u8; 64],
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioBackplaneWrite8Report {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub address: u32,
    pub value: u8,
    pub window_base: u32,
    pub window_address: u32,
    pub response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioBackplaneWrite32Report {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub address: u32,
    pub value: u32,
    pub window_base: u32,
    pub window_address: u32,
    pub response: u32,
    pub readback: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioSocramProbeReport {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub write_status: &'static [u8],
    pub address: u32,
    pub pattern: u32,
    pub original: u32,
    pub readback: u32,
    pub restored: u32,
    pub last_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioSocramBlockProbeReport {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub read_status: &'static [u8],
    pub write_status: &'static [u8],
    pub address: u32,
    pub seed: u32,
    pub original_checksum: u32,
    pub readback_checksum: u32,
    pub restored_checksum: u32,
    pub mismatch_index: u32,
    pub mismatch_expected: u32,
    pub mismatch_actual: u32,
    pub last_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioFirmwareLoadReport {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub read_status: &'static [u8],
    pub write_status: &'static [u8],
    pub firmware_bytes: u32,
    pub processed_bytes: u32,
    pub chunk_count: u32,
    pub firmware_checksum: u32,
    pub verify_checksum: u32,
    pub mismatch_offset: u32,
    pub mismatch_expected: u32,
    pub mismatch_actual: u32,
    pub last_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioFirmwareStartReport {
    pub status: &'static [u8],
    pub firmware_status: &'static [u8],
    pub setup_status: &'static [u8],
    pub read_status: &'static [u8],
    pub write_status: &'static [u8],
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
    pub arm_before: WifiSdioCoreSnapshotReport,
    pub arm_after: WifiSdioCoreSnapshotReport,
    pub ht_clock_csr: u8,
    pub ht_attempts: u16,
    pub io_enable: u8,
    pub io_ready: u8,
    pub f2_attempts: u16,
    pub last_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioF2HeaderReport {
    pub status: &'static [u8],
    pub response: u32,
    pub bytes: [u8; 4],
    pub length: u16,
    pub checksum: u16,
    pub valid: bool,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioF2FrameReport {
    pub status: &'static [u8],
    pub header_status: &'static [u8],
    pub body_status: &'static [u8],
    pub header_response: u32,
    pub body_response: u32,
    pub bytes: [u8; 64],
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
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioF2ControlReport {
    pub status: &'static [u8],
    pub initial_tx_credit: u8,
    pub packet_length: u16,
    pub write_response: u32,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub write_last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioWlcUpReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub send_status: &'static [u8],
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub startup_status: &'static [u8],
    pub startup_length: u16,
    pub startup_channel: u8,
    pub startup_bus_data_credit: u8,
    pub response_status: &'static [u8],
    pub response_length: u16,
    pub response_sequence: u8,
    pub response_channel: u8,
    pub response_bus_data_credit: u8,
    pub cdc_command: u32,
    pub cdc_length: u32,
    pub cdc_flags: u32,
    pub cdc_id: u16,
    pub cdc_status: u32,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub startup_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioGetVersionReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub send_status: &'static [u8],
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub response_status: &'static [u8],
    pub response_length: u16,
    pub response_sequence: u8,
    pub response_channel: u8,
    pub response_bus_data_credit: u8,
    pub cdc_command: u32,
    pub cdc_length: u32,
    pub cdc_flags: u32,
    pub cdc_id: u16,
    pub cdc_status: u32,
    pub version: u32,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioGetMpcReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub send_status: &'static [u8],
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub response_status: &'static [u8],
    pub response_length: u16,
    pub response_sequence: u8,
    pub response_channel: u8,
    pub response_bus_data_credit: u8,
    pub cdc_command: u32,
    pub cdc_length: u32,
    pub cdc_flags: u32,
    pub cdc_id: u16,
    pub cdc_status: u32,
    pub value: u32,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioClmLoadReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub clm_bytes: u32,
    pub processed_bytes: u32,
    pub chunk_count: u32,
    pub chunk_index: u32,
    pub chunk_bytes: u16,
    pub chunk_flags: u16,
    pub payload_bytes: u16,
    pub send_status: &'static [u8],
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub response_attempts: u16,
    pub response_status: &'static [u8],
    pub response_length: u16,
    pub response_sequence: u8,
    pub response_channel: u8,
    pub response_bus_data_credit: u8,
    pub cdc_command: u32,
    pub cdc_length: u32,
    pub cdc_flags: u32,
    pub cdc_id: u16,
    pub cdc_status: u32,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioGetClmVersionReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub send_status: &'static [u8],
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub response_status: &'static [u8],
    pub response_length: u16,
    pub response_sequence: u8,
    pub response_channel: u8,
    pub response_bus_data_credit: u8,
    pub cdc_command: u32,
    pub cdc_length: u32,
    pub cdc_flags: u32,
    pub cdc_id: u16,
    pub cdc_status: u32,
    pub copied_bytes: u8,
    pub version_len: u8,
    pub version_truncated: bool,
    pub version: StringBytes,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioF2AbortProbeReport {
    pub frame_status: &'static [u8],
    pub frame_valid: bool,
    pub frame_length: u16,
    pub frame_channel: u8,
    pub frame_bus_data_credit: u8,
    pub frame_header_response: u32,
    pub frame_body_response: u32,
    pub abort_io_abort_response: u32,
    pub abort_frame_control_response: u32,
    pub post_io_enable: u8,
    pub post_io_ready: u8,
    pub post_interrupt_pending: u8,
    pub post_io_enable_response: u32,
    pub post_io_ready_response: u32,
    pub post_interrupt_pending_response: u32,
    pub post_host_normal_int: u16,
    pub post_host_error_int: u16,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub abort_last_error: Option<WifiSdioCommandErrorReport>,
    pub post_last_error: Option<WifiSdioCommandErrorReport>,
}

#[derive(Clone, Copy)]
pub struct WifiSdioPollReadFrameReport {
    pub ack_status: &'static [u8],
    pub ack_int_status_before: u32,
    pub ack_clear_value: u32,
    pub ack_int_status_after: u32,
    pub ack_final_response: u32,
    pub frame_status: &'static [u8],
    pub frame_valid: bool,
    pub frame_length: u16,
    pub frame_channel: u8,
    pub frame_bus_data_credit: u8,
    pub frame_header_response: u32,
    pub frame_body_response: u32,
    pub post_status: &'static [u8],
    pub post_io_enable: u8,
    pub post_io_ready: u8,
    pub post_interrupt_pending: u8,
    pub post_io_enable_response: u32,
    pub post_io_ready_response: u32,
    pub post_interrupt_pending_response: u32,
    pub post_host_normal_int: u16,
    pub post_host_error_int: u16,
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub post_last_error: Option<WifiSdioCommandErrorReport>,
}

#[derive(Clone, Copy)]
pub struct WifiSdioInterruptAckReport {
    pub status: &'static [u8],
    pub int_status_before: u32,
    pub mailbox_data: u32,
    pub mailbox_ack_value: u32,
    pub clear_value: u32,
    pub int_status_after: u32,
    pub host_normal_int_before: u16,
    pub host_error_int_before: u16,
    pub host_normal_int_after: u16,
    pub host_error_int_after: u16,
    pub int_status_response: u32,
    pub mailbox_response: u32,
    pub mailbox_ack_response: u32,
    pub mailbox_ack_readback: u32,
    pub clear_response: u32,
    pub clear_readback: u32,
    pub final_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioInterruptStateReport {
    pub status: &'static [u8],
    pub io_enable: u8,
    pub io_ready: u8,
    pub interrupt_enable: u8,
    pub interrupt_pending: u8,
    pub bus_control: u8,
    pub master_enabled: bool,
    pub function1_enabled: bool,
    pub function2_enabled: bool,
    pub function1_ready: bool,
    pub function2_ready: bool,
    pub function1_pending: bool,
    pub function2_pending: bool,
    pub host_card_interrupt: bool,
    pub io_enable_response: u32,
    pub io_ready_response: u32,
    pub interrupt_enable_response: u32,
    pub interrupt_pending_response: u32,
    pub bus_control_response: u32,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioKeepAwakeReport {
    pub status: &'static [u8],
    pub attempts: u16,
    pub write_value: u8,
    pub first_write_response: u32,
    pub second_write_response: u32,
    pub retry_write_response: u32,
    pub read_value: u8,
    pub read_response: u32,
    pub keep_wl_kso: bool,
    pub wl_devon: bool,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioHtRequestReport {
    pub status: &'static [u8],
    pub attempts: u16,
    pub write_value: u8,
    pub write_response: u32,
    pub read_value: u8,
    pub read_response: u32,
    pub ht_available: bool,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioHostResetReport {
    pub command_reset: bool,
    pub data_reset: bool,
    pub before: WifiSdioHostReport,
    pub after: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioAbortReadReport {
    pub io_abort_response: u32,
    pub frame_control_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioCoreStateReport {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
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
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioCoreSnapshotReport {
    pub ioctrl: u8,
    pub resetctrl: u8,
    pub resetstatus: u8,
    pub clock_enabled: bool,
    pub force_gated: bool,
    pub in_reset: bool,
    pub reset_busy: bool,
    pub core_up: bool,
}

#[derive(Clone, Copy)]
pub struct WifiSdioCoreResetReport {
    pub status: &'static [u8],
    pub setup_status: &'static [u8],
    pub base: u32,
    pub before: WifiSdioCoreSnapshotReport,
    pub after: WifiSdioCoreSnapshotReport,
    pub last_response: u32,
    pub last_error: Option<WifiSdioCommandErrorReport>,
    pub host: WifiSdioHostReport,
}

type LispResult<T> = Result<T, Error>;

#[derive(Clone, Copy)]
pub enum LoadFileOutcome {
    Loaded(Value),
    NotReady(StoreReadReport),
}

struct OutputFatFormatProgress<'a, W: Write> {
    output: &'a mut W,
}

impl<W: Write> FatFormatProgress for OutputFatFormatProgress<'_, W> {
    fn report(&mut self, phase: &'static [u8], written_sector_count: u32, total_sectors: u32) {
        write!(self.output, "fat-format ").ok();
        write_ascii_bytes(self.output, phase).ok();
        writeln!(self.output, " {}/{}", written_sector_count, total_sectors).ok();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i32),
    Word(u32),
    Symbol(SymbolId),
    Object(ObjectId),
    Primitive(Primitive),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Help,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    NumberEquals,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Not,
    EqPredicate,
    NilPredicate,
    AtomPredicate,
    PairPredicate,
    NumberPredicate,
    SymbolPredicate,
    BoolPredicate,
    StringPredicate,
    Cons,
    Car,
    Cdr,
    List,
    Led,
    Heartbeat,
    Button,
    Millis,
    Reg32,
    Poke32,
    Regs,
    SdStatus,
    SdPins,
    SdPinmux,
    SdClock,
    SdInit,
    SdRead,
    SdRead0,
    SdWriteFill,
    FormatStore,
    SaveFile,
    ReadFile,
    Load,
    Ls,
    Cat,
    FatInfo,
    FatFormat,
    WifiSdioInit,
    WifiCmd52Read,
    WifiCmd52Write,
    WifiEnableFunctions,
    WifiSetupBackplane,
    WifiCmd53Read,
    WifiBackplaneRead,
    WifiBackplaneWrite8,
    WifiBackplaneWrite32,
    WifiBackplaneWrite32Bytes,
    WifiSocramProbe,
    WifiSocramBlockProbe,
    WifiLoadFirmware,
    WifiStartFirmware,
    WifiF2ReadHeader,
    WifiF2ReadFrame,
    WifiF2ReadFrameSingle,
    WifiF2ReadFrameExact,
    WifiF2ReadFrameBlock,
    WifiSendWlcUp,
    WifiWlcUp,
    WifiGetVersion,
    WifiGetMpc,
    WifiLoadClm,
    WifiGetClmVersion,
    WifiF2ReadFrameAbort,
    WifiPollReadFrame,
    WifiAckInterrupts,
    WifiInterruptState,
    WifiKeepAwake,
    WifiRequestHt,
    WifiHostResetLines,
    WifiAbortRead,
    WifiCoreState,
    WifiResetCore,
    SdhcRegs,
    Heap,
    Gc,
    Reboot,
}

impl Primitive {
    fn name(self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "mod",
            Self::NumberEquals => "=",
            Self::LessThan => "<",
            Self::LessOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterOrEqual => ">=",
            Self::Not => "not",
            Self::EqPredicate => "eq?",
            Self::NilPredicate => "nil?",
            Self::AtomPredicate => "atom?",
            Self::PairPredicate => "pair?",
            Self::NumberPredicate => "number?",
            Self::SymbolPredicate => "symbol?",
            Self::BoolPredicate => "bool?",
            Self::StringPredicate => "string?",
            Self::Cons => "cons",
            Self::Car => "car",
            Self::Cdr => "cdr",
            Self::List => "list",
            Self::Led => "led",
            Self::Heartbeat => "heartbeat",
            Self::Button => "button",
            Self::Millis => "millis",
            Self::Reg32 => "reg32",
            Self::Poke32 => "poke32",
            Self::Regs => "regs",
            Self::SdStatus => "sd-status",
            Self::SdPins => "sd-pins",
            Self::SdPinmux => "sd-pinmux",
            Self::SdClock => "sd-clock",
            Self::SdInit => "sd-init",
            Self::SdRead => "sd-read",
            Self::SdRead0 => "sd-read0",
            Self::SdWriteFill => "sd-write-fill",
            Self::FormatStore => "format-store",
            Self::SaveFile => "save-file",
            Self::ReadFile => "read-file",
            Self::Load => "load",
            Self::Ls => "ls",
            Self::Cat => "cat",
            Self::FatInfo => "fat-info",
            Self::FatFormat => "fat-format",
            Self::WifiSdioInit => "wifi-sdio-init",
            Self::WifiCmd52Read => "wifi-cmd52-read",
            Self::WifiCmd52Write => "wifi-cmd52-write",
            Self::WifiEnableFunctions => "wifi-enable-functions",
            Self::WifiSetupBackplane => "wifi-setup-backplane",
            Self::WifiCmd53Read => "wifi-cmd53-read",
            Self::WifiBackplaneRead => "wifi-backplane-read",
            Self::WifiBackplaneWrite8 => "wifi-backplane-write8",
            Self::WifiBackplaneWrite32 => "wifi-backplane-write32",
            Self::WifiBackplaneWrite32Bytes => "wifi-backplane-write32-bytes",
            Self::WifiSocramProbe => "wifi-socram-probe",
            Self::WifiSocramBlockProbe => "wifi-socram-block-probe",
            Self::WifiLoadFirmware => "wifi-load-firmware",
            Self::WifiStartFirmware => "wifi-start-firmware",
            Self::WifiF2ReadHeader => "wifi-f2-read-header",
            Self::WifiF2ReadFrame => "wifi-f2-read-frame",
            Self::WifiF2ReadFrameSingle => "wifi-f2-read-frame-single",
            Self::WifiF2ReadFrameExact => "wifi-f2-read-frame-exact",
            Self::WifiF2ReadFrameBlock => "wifi-f2-read-frame-block",
            Self::WifiSendWlcUp => "wifi-send-wlc-up",
            Self::WifiWlcUp => "wifi-wlc-up",
            Self::WifiGetVersion => "wifi-get-version",
            Self::WifiGetMpc => "wifi-get-mpc",
            Self::WifiLoadClm => "wifi-load-clm",
            Self::WifiGetClmVersion => "wifi-get-clm-version",
            Self::WifiF2ReadFrameAbort => "wifi-f2-read-frame-abort",
            Self::WifiPollReadFrame => "wifi-poll-read-frame",
            Self::WifiAckInterrupts => "wifi-ack-interrupts",
            Self::WifiInterruptState => "wifi-interrupt-state",
            Self::WifiKeepAwake => "wifi-keep-awake",
            Self::WifiRequestHt => "wifi-request-ht",
            Self::WifiHostResetLines => "wifi-host-reset-lines",
            Self::WifiAbortRead => "wifi-abort-read",
            Self::WifiCoreState => "wifi-core-state",
            Self::WifiResetCore => "wifi-reset-core",
            Self::SdhcRegs => "sdhc-regs",
            Self::Heap => "heap",
            Self::Gc => "gc",
            Self::Reboot => "reboot",
        }
    }
}

#[derive(Clone, Copy)]
enum ObjectKind {
    Free,
    Pair {
        car: Value,
        cdr: Value,
    },
    Closure {
        params: Value,
        body: Value,
        env: Value,
    },
    Env {
        symbol: SymbolId,
        value: Value,
        next: Value,
    },
    String {
        len: u8,
        bytes: [u8; MAX_STRING_BYTES],
    },
}

#[derive(Clone, Copy)]
struct Object {
    marked: bool,
    next_free: Option<ObjectId>,
    kind: ObjectKind,
}

const FREE_OBJECT: Object = Object {
    marked: false,
    next_free: None,
    kind: ObjectKind::Free,
};

#[derive(Clone, Copy)]
struct SymbolEntry {
    occupied: bool,
    len: u8,
    bytes: [u8; MAX_SYMBOL_BYTES],
}

const EMPTY_SYMBOL: SymbolEntry = SymbolEntry {
    occupied: false,
    len: 0,
    bytes: [0; MAX_SYMBOL_BYTES],
};

#[derive(Clone, Copy)]
struct GlobalBinding {
    occupied: bool,
    symbol: SymbolId,
    value: Value,
}

const EMPTY_BINDING: GlobalBinding = GlobalBinding {
    occupied: false,
    symbol: 0,
    value: Value::Nil,
};

#[derive(Clone, Copy)]
struct SpecialSymbols {
    quote: SymbolId,
    if_: SymbolId,
    define: SymbolId,
    lambda: SymbolId,
    begin: SymbolId,
    let_: SymbolId,
    on: SymbolId,
    off: SymbolId,
    toggle: SymbolId,
    status: SymbolId,
}

const EMPTY_SPECIALS: SpecialSymbols = SpecialSymbols {
    quote: 0,
    if_: 0,
    define: 0,
    lambda: 0,
    begin: 0,
    let_: 0,
    on: 0,
    off: 0,
    toggle: 0,
    status: 0,
};

pub struct Machine {
    initialized: bool,
    symbols: [SymbolEntry; MAX_SYMBOLS],
    symbol_count: usize,
    globals: [GlobalBinding; MAX_GLOBALS],
    global_count: usize,
    objects: [Object; MAX_OBJECTS],
    free_head: Option<ObjectId>,
    specials: SpecialSymbols,
    active_expression: Value,
    collections: u32,
}

#[derive(Clone, Copy)]
struct HeapCounts {
    used: usize,
    free: usize,
    total: usize,
}

impl Machine {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            symbols: [EMPTY_SYMBOL; MAX_SYMBOLS],
            symbol_count: 0,
            globals: [EMPTY_BINDING; MAX_GLOBALS],
            global_count: 0,
            objects: [FREE_OBJECT; MAX_OBJECTS],
            free_head: None,
            specials: EMPTY_SPECIALS,
            active_expression: Value::Nil,
            collections: 0,
        }
    }

    pub fn bootstrap(&mut self) -> LispResult<()> {
        if self.initialized {
            return Ok(());
        }

        self.reset_heap();

        self.specials.quote = self.intern(b"quote")?;
        self.specials.if_ = self.intern(b"if")?;
        self.specials.define = self.intern(b"define")?;
        self.specials.lambda = self.intern(b"lambda")?;
        self.specials.begin = self.intern(b"begin")?;
        self.specials.let_ = self.intern(b"let")?;
        self.specials.on = self.intern(b"on")?;
        self.specials.off = self.intern(b"off")?;
        self.specials.toggle = self.intern(b"toggle")?;
        self.specials.status = self.intern(b"status")?;

        self.bind_self_evaluating_symbol(self.specials.on)?;
        self.bind_self_evaluating_symbol(self.specials.off)?;
        self.bind_self_evaluating_symbol(self.specials.toggle)?;
        self.bind_self_evaluating_symbol(self.specials.status)?;

        let true_symbol = self.intern(b"true")?;
        self.bind_global(true_symbol, Value::Bool(true))?;
        let false_symbol = self.intern(b"false")?;
        self.bind_global(false_symbol, Value::Bool(false))?;

        self.install_primitive(b"help", Primitive::Help)?;
        self.install_primitive(b"+", Primitive::Add)?;
        self.install_primitive(b"-", Primitive::Subtract)?;
        self.install_primitive(b"*", Primitive::Multiply)?;
        self.install_primitive(b"/", Primitive::Divide)?;
        self.install_primitive(b"mod", Primitive::Modulo)?;
        self.install_primitive(b"=", Primitive::NumberEquals)?;
        self.install_primitive(b"<", Primitive::LessThan)?;
        self.install_primitive(b"<=", Primitive::LessOrEqual)?;
        self.install_primitive(b">", Primitive::GreaterThan)?;
        self.install_primitive(b">=", Primitive::GreaterOrEqual)?;
        self.install_primitive(b"not", Primitive::Not)?;
        self.install_primitive(b"eq?", Primitive::EqPredicate)?;
        self.install_primitive(b"nil?", Primitive::NilPredicate)?;
        self.install_primitive(b"atom?", Primitive::AtomPredicate)?;
        self.install_primitive(b"pair?", Primitive::PairPredicate)?;
        self.install_primitive(b"number?", Primitive::NumberPredicate)?;
        self.install_primitive(b"symbol?", Primitive::SymbolPredicate)?;
        self.install_primitive(b"bool?", Primitive::BoolPredicate)?;
        self.install_primitive(b"string?", Primitive::StringPredicate)?;
        self.install_primitive(b"cons", Primitive::Cons)?;
        self.install_primitive(b"car", Primitive::Car)?;
        self.install_primitive(b"cdr", Primitive::Cdr)?;
        self.install_primitive(b"list", Primitive::List)?;
        self.install_primitive(b"led", Primitive::Led)?;
        self.install_primitive(b"heartbeat", Primitive::Heartbeat)?;
        self.install_primitive(b"button", Primitive::Button)?;
        self.install_primitive(b"millis", Primitive::Millis)?;
        self.install_primitive(b"reg32", Primitive::Reg32)?;
        self.install_primitive(b"poke32", Primitive::Poke32)?;
        self.install_primitive(b"regs", Primitive::Regs)?;
        self.install_primitive(b"sd-status", Primitive::SdStatus)?;
        self.install_primitive(b"sd-pins", Primitive::SdPins)?;
        self.install_primitive(b"sd-pinmux", Primitive::SdPinmux)?;
        self.install_primitive(b"sd-clock", Primitive::SdClock)?;
        self.install_primitive(b"sd-init", Primitive::SdInit)?;
        self.install_primitive(b"sd-read", Primitive::SdRead)?;
        self.install_primitive(b"sd-read0", Primitive::SdRead0)?;
        self.install_primitive(b"sd-write-fill", Primitive::SdWriteFill)?;
        self.install_primitive(b"format-store", Primitive::FormatStore)?;
        self.install_primitive(b"save-file", Primitive::SaveFile)?;
        self.install_primitive(b"read-file", Primitive::ReadFile)?;
        self.install_primitive(b"load", Primitive::Load)?;
        self.install_primitive(b"ls", Primitive::Ls)?;
        self.install_primitive(b"cat", Primitive::Cat)?;
        self.install_primitive(b"fat-info", Primitive::FatInfo)?;
        self.install_primitive(b"fat-format", Primitive::FatFormat)?;
        self.install_primitive(b"wifi-sdio-init", Primitive::WifiSdioInit)?;
        self.install_primitive(b"wifi-cmd52-read", Primitive::WifiCmd52Read)?;
        self.install_primitive(b"wifi-cmd52-write", Primitive::WifiCmd52Write)?;
        self.install_primitive(b"wifi-enable-functions", Primitive::WifiEnableFunctions)?;
        self.install_primitive(b"wifi-setup-backplane", Primitive::WifiSetupBackplane)?;
        self.install_primitive(b"wifi-cmd53-read", Primitive::WifiCmd53Read)?;
        self.install_primitive(b"wifi-backplane-read", Primitive::WifiBackplaneRead)?;
        self.install_primitive(b"wifi-backplane-write8", Primitive::WifiBackplaneWrite8)?;
        self.install_primitive(b"wifi-backplane-write32", Primitive::WifiBackplaneWrite32)?;
        self.install_primitive(
            b"wifi-backplane-write32-bytes",
            Primitive::WifiBackplaneWrite32Bytes,
        )?;
        self.install_primitive(b"wifi-socram-probe", Primitive::WifiSocramProbe)?;
        self.install_primitive(b"wifi-socram-block-probe", Primitive::WifiSocramBlockProbe)?;
        self.install_primitive(b"wifi-load-firmware", Primitive::WifiLoadFirmware)?;
        self.install_primitive(b"wifi-start-firmware", Primitive::WifiStartFirmware)?;
        self.install_primitive(b"wifi-f2-read-header", Primitive::WifiF2ReadHeader)?;
        self.install_primitive(b"wifi-f2-read-frame", Primitive::WifiF2ReadFrame)?;
        self.install_primitive(
            b"wifi-f2-read-frame-single",
            Primitive::WifiF2ReadFrameSingle,
        )?;
        self.install_primitive(b"wifi-f2-read-frame-exact", Primitive::WifiF2ReadFrameExact)?;
        self.install_primitive(b"wifi-f2-read-frame-block", Primitive::WifiF2ReadFrameBlock)?;
        self.install_primitive(b"wifi-send-wlc-up", Primitive::WifiSendWlcUp)?;
        self.install_primitive(b"wifi-wlc-up", Primitive::WifiWlcUp)?;
        self.install_primitive(b"wifi-get-version", Primitive::WifiGetVersion)?;
        self.install_primitive(b"wifi-get-mpc", Primitive::WifiGetMpc)?;
        self.install_primitive(b"wifi-load-clm", Primitive::WifiLoadClm)?;
        self.install_primitive(b"wifi-get-clm-version", Primitive::WifiGetClmVersion)?;
        self.install_primitive(b"wifi-f2-read-frame-abort", Primitive::WifiF2ReadFrameAbort)?;
        self.install_primitive(b"wifi-poll-read-frame", Primitive::WifiPollReadFrame)?;
        self.install_primitive(b"wifi-ack-interrupts", Primitive::WifiAckInterrupts)?;
        self.install_primitive(b"wifi-interrupt-state", Primitive::WifiInterruptState)?;
        self.install_primitive(b"wifi-keep-awake", Primitive::WifiKeepAwake)?;
        self.install_primitive(b"wifi-request-ht", Primitive::WifiRequestHt)?;
        self.install_primitive(b"wifi-host-reset-lines", Primitive::WifiHostResetLines)?;
        self.install_primitive(b"wifi-abort-read", Primitive::WifiAbortRead)?;
        self.install_primitive(b"wifi-core-state", Primitive::WifiCoreState)?;
        self.install_primitive(b"wifi-reset-core", Primitive::WifiResetCore)?;
        self.install_primitive(b"sdhc-regs", Primitive::SdhcRegs)?;
        self.install_primitive(b"heap", Primitive::Heap)?;
        self.install_primitive(b"gc", Primitive::Gc)?;
        self.install_primitive(b"reboot", Primitive::Reboot)?;

        self.initialized = true;
        Ok(())
    }

    pub fn eval_line<B: Board, W: Write>(
        &mut self,
        input: &[u8],
        board: &mut B,
        output: &mut W,
    ) -> fmt::Result {
        self.collect_garbage();

        let expression = match self.read(input) {
            Ok(expression) => expression,
            Err(error) => {
                writeln!(output, "error: {}", error.message())?;
                return Ok(());
            }
        };

        self.active_expression = expression;
        let result = self.eval(expression, Value::Nil, board, output, 0);
        self.active_expression = Value::Nil;

        match result {
            Ok(value) => {
                write!(output, "=> ")?;
                self.write_repl_value(value, output)?;
                writeln!(output)?;
            }
            Err(error) => {
                writeln!(output, "error: {}", error.message())?;
            }
        }

        self.collect_garbage();
        Ok(())
    }

    pub fn load_file<B: Board, W: Write>(
        &mut self,
        path: StringBytes,
        board: &mut B,
        output: &mut W,
    ) -> LispResult<LoadFileOutcome> {
        self.collect_garbage();
        let result = self.load_file_in_env(path, Value::Nil, board, output, 0);
        self.collect_garbage();
        result
    }

    pub fn write_value_to<W: Write>(&self, value: Value, output: &mut W) -> fmt::Result {
        self.write_value(value, output)
    }

    fn reset_heap(&mut self) {
        let mut index = 0usize;
        while index < MAX_OBJECTS {
            let next = if index + 1 < MAX_OBJECTS {
                Some((index + 1) as ObjectId)
            } else {
                None
            };
            self.objects[index] = Object {
                marked: false,
                next_free: next,
                kind: ObjectKind::Free,
            };
            index += 1;
        }
        self.free_head = Some(0);
        self.collections = 0;
    }

    fn install_primitive(&mut self, name: &[u8], primitive: Primitive) -> LispResult<()> {
        let symbol = self.intern(name)?;
        self.bind_global(symbol, Value::Primitive(primitive))
    }

    fn bind_self_evaluating_symbol(&mut self, symbol: SymbolId) -> LispResult<()> {
        self.bind_global(symbol, Value::Symbol(symbol))
    }

    fn read(&mut self, input: &[u8]) -> LispResult<Value> {
        let mut reader = Reader { input, position: 0 };
        let expression = reader.read_expression(self)?;
        reader.skip_ws();
        if reader.is_done() {
            Ok(expression)
        } else {
            Err(Error::new("trailing input"))
        }
    }

    fn intern(&mut self, name: &[u8]) -> LispResult<SymbolId> {
        if name.is_empty() {
            return Err(Error::new("empty symbol"));
        }
        if name.len() > MAX_SYMBOL_BYTES {
            return Err(Error::new("symbol too long"));
        }

        let mut index = 0usize;
        while index < MAX_SYMBOLS {
            let entry = self.symbols[index];
            if entry.occupied && entry.len as usize == name.len() {
                let mut equal = true;
                let mut byte_index = 0usize;
                while byte_index < name.len() {
                    if entry.bytes[byte_index] != name[byte_index] {
                        equal = false;
                        break;
                    }
                    byte_index += 1;
                }

                if equal {
                    return Ok(index as SymbolId);
                }
            }
            index += 1;
        }

        if self.symbol_count >= MAX_SYMBOLS {
            return Err(Error::new("symbol table full"));
        }

        let id = self.symbol_count;
        let mut bytes = [0u8; MAX_SYMBOL_BYTES];
        let mut byte_index = 0usize;
        while byte_index < name.len() {
            bytes[byte_index] = name[byte_index];
            byte_index += 1;
        }

        self.symbols[id] = SymbolEntry {
            occupied: true,
            len: name.len() as u8,
            bytes,
        };
        self.symbol_count += 1;
        Ok(id as SymbolId)
    }

    fn bind_global(&mut self, symbol: SymbolId, value: Value) -> LispResult<()> {
        let mut index = 0usize;
        while index < MAX_GLOBALS {
            if self.globals[index].occupied && self.globals[index].symbol == symbol {
                self.globals[index].value = value;
                return Ok(());
            }
            index += 1;
        }

        index = 0;
        while index < MAX_GLOBALS {
            if !self.globals[index].occupied {
                self.globals[index] = GlobalBinding {
                    occupied: true,
                    symbol,
                    value,
                };
                self.global_count += 1;
                return Ok(());
            }
            index += 1;
        }

        Err(Error::new("global environment full"))
    }

    fn lookup(&self, symbol: SymbolId, env: Value) -> Option<Value> {
        let mut cursor = env;
        while let Value::Object(id) = cursor {
            let kind = self.object_kind_by_id(id).ok()?;
            match kind {
                ObjectKind::Env {
                    symbol: entry_symbol,
                    value,
                    next,
                } => {
                    if entry_symbol == symbol {
                        return Some(value);
                    }
                    cursor = next;
                }
                _ => return None,
            }
        }

        let mut index = 0usize;
        while index < MAX_GLOBALS {
            let binding = self.globals[index];
            if binding.occupied && binding.symbol == symbol {
                return Some(binding.value);
            }
            index += 1;
        }

        None
    }

    fn eval<B: Board, W: Write>(
        &mut self,
        expression: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        if depth > MAX_EVAL_DEPTH {
            return Err(Error::new("evaluation depth limit"));
        }

        match expression {
            Value::Nil | Value::Bool(_) | Value::Int(_) | Value::Word(_) | Value::Primitive(_) => {
                Ok(expression)
            }
            Value::Symbol(symbol) => self.lookup(symbol, env).ok_or(Error::new("unbound symbol")),
            Value::Object(id) => match self.object_kind_by_id(id)? {
                ObjectKind::Pair { .. } => {
                    self.eval_call(expression, env, board, output, depth + 1)
                }
                ObjectKind::Closure { .. } => Ok(expression),
                ObjectKind::String { .. } => Ok(expression),
                ObjectKind::Env { .. } => Err(Error::new("environment object is not a value")),
                ObjectKind::Free => Err(Error::new("stale object")),
            },
        }
    }

    fn eval_call<B: Board, W: Write>(
        &mut self,
        expression: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        let operator = self.car(expression)?;
        let args = self.cdr(expression)?;

        if let Value::Symbol(symbol) = operator {
            if symbol == self.specials.quote {
                return self.form_quote(args);
            }
            if symbol == self.specials.if_ {
                return self.form_if(args, env, board, output, depth + 1);
            }
            if symbol == self.specials.define {
                return self.form_define(args, env, board, output, depth + 1);
            }
            if symbol == self.specials.lambda {
                return self.form_lambda(args, env);
            }
            if symbol == self.specials.begin {
                return self.eval_sequence(args, env, board, output, depth + 1);
            }
            if symbol == self.specials.let_ {
                return self.form_let(args, env, board, output, depth + 1);
            }
        }

        let function = self.eval(operator, env, board, output, depth + 1)?;
        self.apply(function, args, env, board, output, depth + 1)
    }

    fn form_quote(&self, args: Value) -> LispResult<Value> {
        let (value, rest) = self.require_pair(args)?;
        if rest != Value::Nil {
            return Err(Error::new("quote expects one argument"));
        }
        Ok(value)
    }

    fn form_if<B: Board, W: Write>(
        &mut self,
        args: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        let (test, rest) = self.require_pair(args)?;
        let (consequent, rest) = self.require_pair(rest)?;
        let alternate = if rest == Value::Nil {
            Value::Nil
        } else {
            let (alternate, rest) = self.require_pair(rest)?;
            if rest != Value::Nil {
                return Err(Error::new("if expects two or three arguments"));
            }
            alternate
        };

        let test_value = self.eval(test, env, board, output, depth + 1)?;
        if self.truthy(test_value) {
            self.eval(consequent, env, board, output, depth + 1)
        } else {
            self.eval(alternate, env, board, output, depth + 1)
        }
    }

    fn form_define<B: Board, W: Write>(
        &mut self,
        args: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        let (target, rest) = self.require_pair(args)?;

        if let Value::Symbol(symbol) = target {
            let (expression, rest) = self.require_pair(rest)?;
            if rest != Value::Nil {
                return Err(Error::new("define expects a name and one expression"));
            }

            let value = self.eval(expression, env, board, output, depth + 1)?;
            self.bind_global(symbol, value)?;
            return Ok(Value::Symbol(symbol));
        }

        if self.is_pair(target) {
            let name = self.car(target)?;
            let params = self.cdr(target)?;
            let symbol = match name {
                Value::Symbol(symbol) => symbol,
                _ => return Err(Error::new("function define needs a symbol name")),
            };

            if rest == Value::Nil {
                return Err(Error::new("function define needs a body"));
            }
            self.validate_params(params)?;
            let closure = self.alloc_object(ObjectKind::Closure {
                params,
                body: rest,
                env,
            })?;
            self.bind_global(symbol, closure)?;
            return Ok(Value::Symbol(symbol));
        }

        Err(Error::new(
            "define target must be a symbol or function form",
        ))
    }

    fn form_lambda(&mut self, args: Value, env: Value) -> LispResult<Value> {
        let (params, body) = self.require_pair(args)?;
        if body == Value::Nil {
            return Err(Error::new("lambda needs a body"));
        }
        self.validate_params(params)?;
        self.alloc_object(ObjectKind::Closure { params, body, env })
    }

    fn form_let<B: Board, W: Write>(
        &mut self,
        args: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        let (bindings, body) = self.require_pair(args)?;
        if body == Value::Nil {
            return Err(Error::new("let needs a body"));
        }

        let mut new_env = env;
        let mut cursor = bindings;
        while let Some((binding, rest)) = self.list_next(cursor)? {
            let (name, value_tail) = self.require_pair(binding)?;
            let (value_expr, value_rest) = self.require_pair(value_tail)?;
            if value_rest != Value::Nil {
                return Err(Error::new("let binding expects a name and one value"));
            }
            let symbol = match name {
                Value::Symbol(symbol) => symbol,
                _ => return Err(Error::new("let binding name must be a symbol")),
            };
            let value = self.eval(value_expr, env, board, output, depth + 1)?;
            new_env = self.push_env(symbol, value, new_env)?;
            cursor = rest;
        }

        self.eval_sequence(body, new_env, board, output, depth + 1)
    }

    fn eval_sequence<B: Board, W: Write>(
        &mut self,
        body: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        let mut result = Value::Nil;
        let mut cursor = body;
        while let Some((expression, rest)) = self.list_next(cursor)? {
            result = self.eval(expression, env, board, output, depth + 1)?;
            cursor = rest;
        }
        Ok(result)
    }

    fn apply<B: Board, W: Write>(
        &mut self,
        function: Value,
        arg_expressions: Value,
        caller_env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        let mut args = [Value::Nil; MAX_CALL_ARGS];
        let arg_count = self.eval_arguments(
            arg_expressions,
            caller_env,
            board,
            output,
            depth + 1,
            &mut args,
        )?;

        match function {
            Value::Primitive(primitive) => self.apply_primitive(
                primitive,
                &args[..arg_count],
                caller_env,
                board,
                output,
                depth + 1,
            ),
            Value::Object(id) => match self.object_kind_by_id(id)? {
                ObjectKind::Closure { params, body, env } => self.apply_closure(
                    params,
                    body,
                    env,
                    &args[..arg_count],
                    board,
                    output,
                    depth + 1,
                ),
                _ => Err(Error::new("value is not callable")),
            },
            _ => Err(Error::new("value is not callable")),
        }
    }

    fn eval_arguments<B: Board, W: Write>(
        &mut self,
        expressions: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
        args: &mut [Value; MAX_CALL_ARGS],
    ) -> LispResult<usize> {
        let mut count = 0usize;
        let mut cursor = expressions;

        while let Some((expression, rest)) = self.list_next(cursor)? {
            if count >= MAX_CALL_ARGS {
                return Err(Error::new("too many call arguments"));
            }
            args[count] = self.eval(expression, env, board, output, depth + 1)?;
            count += 1;
            cursor = rest;
        }

        Ok(count)
    }

    fn apply_closure<B: Board, W: Write>(
        &mut self,
        params: Value,
        body: Value,
        closure_env: Value,
        args: &[Value],
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        let mut param_cursor = params;
        let mut arg_index = 0usize;
        let mut call_env = closure_env;

        while let Some((param, rest)) = self.list_next(param_cursor)? {
            if arg_index >= args.len() {
                return Err(Error::new("not enough arguments"));
            }
            let symbol = match param {
                Value::Symbol(symbol) => symbol,
                _ => return Err(Error::new("lambda parameter must be a symbol")),
            };
            call_env = self.push_env(symbol, args[arg_index], call_env)?;
            arg_index += 1;
            param_cursor = rest;
        }

        if arg_index != args.len() {
            return Err(Error::new("too many arguments"));
        }

        self.eval_sequence(body, call_env, board, output, depth + 1)
    }

    fn apply_primitive<B: Board, W: Write>(
        &mut self,
        primitive: Primitive,
        args: &[Value],
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<Value> {
        match primitive {
            Primitive::Help => {
                self.expect_count(args, 0)?;
                self.help()
            }
            Primitive::Add => self.primitive_add(args),
            Primitive::Subtract => self.primitive_subtract(args),
            Primitive::Multiply => self.primitive_multiply(args),
            Primitive::Divide => self.primitive_divide(args),
            Primitive::Modulo => {
                self.expect_count(args, 2)?;
                let left = self.expect_int(args[0])?;
                let right = self.expect_int(args[1])?;
                if right == 0 {
                    return Err(Error::new("division by zero"));
                }
                Ok(Value::Int(left % right))
            }
            Primitive::NumberEquals => self.compare_numbers(args, |left, right| left == right),
            Primitive::LessThan => self.compare_numbers(args, |left, right| left < right),
            Primitive::LessOrEqual => self.compare_numbers(args, |left, right| left <= right),
            Primitive::GreaterThan => self.compare_numbers(args, |left, right| left > right),
            Primitive::GreaterOrEqual => self.compare_numbers(args, |left, right| left >= right),
            Primitive::Not => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(!self.truthy(args[0])))
            }
            Primitive::EqPredicate => {
                self.expect_count(args, 2)?;
                Ok(Value::Bool(args[0] == args[1]))
            }
            Primitive::NilPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(args[0] == Value::Nil))
            }
            Primitive::AtomPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(!self.is_pair(args[0])))
            }
            Primitive::PairPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(self.is_pair(args[0])))
            }
            Primitive::NumberPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(matches!(
                    args[0],
                    Value::Int(_) | Value::Word(_)
                )))
            }
            Primitive::SymbolPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
            }
            Primitive::BoolPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
            }
            Primitive::StringPredicate => {
                self.expect_count(args, 1)?;
                Ok(Value::Bool(matches!(
                    args[0],
                    Value::Object(id)
                        if matches!(self.object_kind_by_id(id), Ok(ObjectKind::String { .. }))
                )))
            }
            Primitive::Cons => {
                self.expect_count(args, 2)?;
                self.alloc_pair(args[0], args[1])
            }
            Primitive::Car => {
                self.expect_count(args, 1)?;
                self.car(args[0])
            }
            Primitive::Cdr => {
                self.expect_count(args, 1)?;
                self.cdr(args[0])
            }
            Primitive::List => self.make_list_from_values(args),
            Primitive::Led => {
                self.expect_count(args, 1)?;
                let action = self.led_action(args[0])?;
                Ok(Value::Bool(board.led(action)))
            }
            Primitive::Heartbeat => {
                self.expect_count(args, 1)?;
                let enabled = self.on_off(args[0])?;
                Ok(Value::Bool(board.heartbeat(enabled)))
            }
            Primitive::Button => {
                self.expect_count(args, 1)?;
                let index = self.expect_int(args[0])?;
                Ok(Value::Bool(board.button_pressed(index)?))
            }
            Primitive::Millis => {
                self.expect_count(args, 0)?;
                Ok(Value::Int(board.millis() as i32))
            }
            Primitive::Reg32 => {
                self.expect_count(args, 1)?;
                let address = self.expect_word_address(args[0])?;
                let value = board.read32(address)?;
                Ok(Value::Word(value))
            }
            Primitive::Poke32 => {
                self.expect_count(args, 2)?;
                let address = self.expect_word_address(args[0])?;
                let value = self.expect_u32(args[1])?;
                board.write32(address, value)?;
                Ok(Value::Word(value))
            }
            Primitive::Regs => {
                self.expect_count(args, 0)?;
                self.register_report(board.registers())
            }
            Primitive::SdStatus => {
                self.expect_count(args, 0)?;
                self.sd_status_report(board.sd_status())
            }
            Primitive::SdPins => {
                self.expect_count(args, 0)?;
                self.sd_pins_report(board.sd_pins())
            }
            Primitive::SdPinmux => {
                self.expect_count(args, 0)?;
                self.sd_pins_report(board.sd_pinmux())
            }
            Primitive::SdClock => {
                self.expect_count(args, 0)?;
                self.sd_clock_report(board.sd_clock())
            }
            Primitive::SdInit => {
                self.expect_count(args, 0)?;
                self.sd_init_report(board.sd_init())
            }
            Primitive::SdRead => {
                self.expect_count(args, 1)?;
                let sector = self.expect_u32(args[0])?;
                self.sd_read_report(board.sd_read(sector))
            }
            Primitive::SdRead0 => {
                self.expect_count(args, 0)?;
                self.sd_read_report(board.sd_read(0))
            }
            Primitive::SdWriteFill => {
                self.expect_count(args, 2)?;
                let sector = self.expect_u32(args[0])?;
                let fill_word = self.expect_u32(args[1])?;
                self.sd_write_report(board.sd_write_fill(sector, fill_word))
            }
            Primitive::FormatStore => {
                self.expect_count(args, 0)?;
                self.store_format_report(board.format_store())
            }
            Primitive::SaveFile => {
                self.expect_count(args, 2)?;
                let path = self.expect_string(args[0])?;
                let content = self.expect_string(args[1])?;
                self.store_write_report(board.save_file(path, content))
            }
            Primitive::ReadFile => {
                self.expect_count(args, 1)?;
                let path = self.expect_string(args[0])?;
                self.store_read_report(board.read_file(path))
            }
            Primitive::Load => {
                self.expect_count(args, 1)?;
                let path = self.expect_string(args[0])?;
                match self.load_file_in_env(path, env, board, output, depth + 1)? {
                    LoadFileOutcome::Loaded(value) => Ok(value),
                    LoadFileOutcome::NotReady(report) => self.store_read_report(report),
                }
            }
            Primitive::Ls => {
                self.expect_count(args, 0)?;
                self.store_list_report(board.list_files())
            }
            Primitive::Cat => {
                self.expect_count(args, 1)?;
                let path = self.expect_string(args[0])?;
                let report = board.read_file(path);
                if report.ready {
                    self.string_value(report.content)
                } else {
                    self.store_read_report(report)
                }
            }
            Primitive::FatInfo => {
                self.expect_count(args, 0)?;
                self.fat_info_report(board.fat_info())
            }
            Primitive::FatFormat => {
                self.expect_count(args, 0)?;
                let mut progress = OutputFatFormatProgress { output };
                self.fat_format_report(board.fat_format(&mut progress))
            }
            Primitive::WifiSdioInit => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_report(board.wifi_sdio_init())
            }
            Primitive::WifiCmd52Read => {
                self.expect_count(args, 2)?;
                let function = self.expect_u8(args[0])?;
                let address = self.expect_u32(args[1])?;
                self.wifi_sdio_direct_report(board.wifi_cmd52_read(function, address))
            }
            Primitive::WifiCmd52Write => {
                self.expect_count(args, 3)?;
                let function = self.expect_u8(args[0])?;
                let address = self.expect_u32(args[1])?;
                let data = self.expect_u8(args[2])?;
                self.wifi_sdio_direct_report(board.wifi_cmd52_write(function, address, data))
            }
            Primitive::WifiEnableFunctions => {
                self.expect_count(args, 1)?;
                let requested = self.expect_u8(args[0])?;
                self.wifi_sdio_enable_report(board.wifi_enable_functions(requested))
            }
            Primitive::WifiSetupBackplane => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_backplane_report(board.wifi_setup_backplane())
            }
            Primitive::WifiCmd53Read => {
                self.expect_count(args, 3)?;
                let function = self.expect_u8(args[0])?;
                let address = self.expect_u32(args[1])?;
                let count = self.expect_u8(args[2])?;
                self.wifi_sdio_cmd53_read_report(board.wifi_cmd53_read(function, address, count))
            }
            Primitive::WifiBackplaneRead => {
                self.expect_count(args, 2)?;
                let address = self.expect_u32(args[0])?;
                let count = self.expect_u8(args[1])?;
                self.wifi_sdio_backplane_read_report(board.wifi_backplane_read(address, count))
            }
            Primitive::WifiBackplaneWrite8 => {
                self.expect_count(args, 2)?;
                let address = self.expect_u32(args[0])?;
                let value = self.expect_u8(args[1])?;
                self.wifi_sdio_backplane_write8_report(board.wifi_backplane_write8(address, value))
            }
            Primitive::WifiBackplaneWrite32 => {
                self.expect_count(args, 2)?;
                let address = self.expect_u32(args[0])?;
                let value = self.expect_u32(args[1])?;
                self.wifi_sdio_backplane_write32_report(
                    board.wifi_backplane_write32(address, value),
                )
            }
            Primitive::WifiBackplaneWrite32Bytes => {
                self.expect_count(args, 2)?;
                let address = self.expect_u32(args[0])?;
                let value = self.expect_u32(args[1])?;
                self.wifi_sdio_backplane_write32_report(
                    board.wifi_backplane_write32_bytes(address, value),
                )
            }
            Primitive::WifiSocramProbe => {
                self.expect_count(args, 2)?;
                let address = self.expect_u32(args[0])?;
                let pattern = self.expect_u32(args[1])?;
                self.wifi_sdio_socram_probe_report(board.wifi_socram_probe(address, pattern))
            }
            Primitive::WifiSocramBlockProbe => {
                self.expect_count(args, 2)?;
                let address = self.expect_u32(args[0])?;
                let seed = self.expect_u32(args[1])?;
                self.wifi_sdio_socram_block_probe_report(
                    board.wifi_socram_block_probe(address, seed),
                )
            }
            Primitive::WifiLoadFirmware => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_firmware_load_report(board.wifi_load_firmware())
            }
            Primitive::WifiStartFirmware => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_firmware_start_report(board.wifi_start_firmware())
            }
            Primitive::WifiF2ReadHeader => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_f2_header_report(board.wifi_f2_read_header())
            }
            Primitive::WifiF2ReadFrame => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_f2_frame_report(board.wifi_f2_read_frame())
            }
            Primitive::WifiF2ReadFrameSingle => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_f2_frame_report(board.wifi_f2_read_frame_single())
            }
            Primitive::WifiF2ReadFrameExact => {
                self.expect_count(args, 1)?;
                let count = self.expect_u8(args[0])?;
                self.wifi_sdio_f2_frame_report(board.wifi_f2_read_frame_exact(count))
            }
            Primitive::WifiF2ReadFrameBlock => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_f2_frame_report(board.wifi_f2_read_frame_block())
            }
            Primitive::WifiSendWlcUp => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_f2_control_report(board.wifi_send_wlc_up())
            }
            Primitive::WifiWlcUp => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_wlc_up_report(board.wifi_wlc_up())
            }
            Primitive::WifiGetVersion => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_get_version_report(board.wifi_get_version())
            }
            Primitive::WifiGetMpc => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_get_mpc_report(board.wifi_get_mpc())
            }
            Primitive::WifiLoadClm => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_clm_load_report(board.wifi_load_clm())
            }
            Primitive::WifiGetClmVersion => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_get_clm_version_report(board.wifi_get_clm_version())
            }
            Primitive::WifiF2ReadFrameAbort => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_f2_abort_probe_report(board.wifi_f2_read_frame_abort())
            }
            Primitive::WifiPollReadFrame => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_poll_read_frame_report(board.wifi_poll_read_frame())
            }
            Primitive::WifiAckInterrupts => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_interrupt_ack_report(board.wifi_ack_interrupts())
            }
            Primitive::WifiInterruptState => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_interrupt_state_report(board.wifi_interrupt_state())
            }
            Primitive::WifiKeepAwake => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_keep_awake_report(board.wifi_keep_awake())
            }
            Primitive::WifiRequestHt => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_ht_request_report(board.wifi_request_ht())
            }
            Primitive::WifiHostResetLines => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_host_reset_report(board.wifi_host_reset_lines())
            }
            Primitive::WifiAbortRead => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_abort_read_report(board.wifi_abort_read())
            }
            Primitive::WifiCoreState => {
                self.expect_count(args, 1)?;
                let base = self.expect_u32(args[0])?;
                self.wifi_sdio_core_state_report(board.wifi_core_state(base))
            }
            Primitive::WifiResetCore => {
                self.expect_count(args, 1)?;
                let base = self.expect_u32(args[0])?;
                self.wifi_sdio_core_reset_report(board.wifi_reset_core(base))
            }
            Primitive::SdhcRegs => {
                self.expect_count(args, 0)?;
                self.sdhc_report(board.sdhc_registers())
            }
            Primitive::Heap => {
                self.expect_count(args, 0)?;
                let counts = self.heap_counts();
                let values = [
                    Value::Int(counts.used as i32),
                    Value::Int(counts.free as i32),
                    Value::Int(counts.total as i32),
                    Value::Int(self.collections as i32),
                ];
                self.make_list_from_values(&values)
            }
            Primitive::Gc => {
                self.expect_count(args, 0)?;
                let freed = self.collect_garbage_from(env);
                Ok(Value::Int(freed as i32))
            }
            Primitive::Reboot => {
                self.expect_count(args, 0)?;
                board.reboot()
            }
        }
    }

    fn primitive_add(&self, args: &[Value]) -> LispResult<Value> {
        let mut result = 0i32;
        let mut index = 0usize;
        while index < args.len() {
            result = result
                .checked_add(self.expect_int(args[index])?)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn load_file_in_env<B: Board, W: Write>(
        &mut self,
        path: StringBytes,
        env: Value,
        board: &mut B,
        output: &mut W,
        depth: u8,
    ) -> LispResult<LoadFileOutcome> {
        let report = board.read_file(path);
        if !report.ready {
            return Ok(LoadFileOutcome::NotReady(report));
        }

        let input = &report.content.bytes[..report.content.len as usize];
        let expression = self.read(input)?;
        let previous_expression = self.active_expression;
        self.active_expression = expression;
        let result = self.eval(expression, env, board, output, depth);
        self.active_expression = previous_expression;
        result.map(LoadFileOutcome::Loaded)
    }

    fn primitive_subtract(&self, args: &[Value]) -> LispResult<Value> {
        if args.is_empty() {
            return Err(Error::new("- expects at least one argument"));
        }

        let mut result = self.expect_int(args[0])?;
        if args.len() == 1 {
            result = result.checked_neg().ok_or(Error::new("integer overflow"))?;
            return Ok(Value::Int(result));
        }

        let mut index = 1usize;
        while index < args.len() {
            result = result
                .checked_sub(self.expect_int(args[index])?)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn primitive_multiply(&self, args: &[Value]) -> LispResult<Value> {
        let mut result = 1i32;
        let mut index = 0usize;
        while index < args.len() {
            result = result
                .checked_mul(self.expect_int(args[index])?)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn primitive_divide(&self, args: &[Value]) -> LispResult<Value> {
        if args.len() < 2 {
            return Err(Error::new("/ expects at least two arguments"));
        }

        let mut result = self.expect_int(args[0])?;
        let mut index = 1usize;
        while index < args.len() {
            let divisor = self.expect_int(args[index])?;
            if divisor == 0 {
                return Err(Error::new("division by zero"));
            }
            result = result
                .checked_div(divisor)
                .ok_or(Error::new("integer overflow"))?;
            index += 1;
        }
        Ok(Value::Int(result))
    }

    fn compare_numbers<F>(&self, args: &[Value], compare: F) -> LispResult<Value>
    where
        F: Fn(i32, i32) -> bool,
    {
        if args.len() < 2 {
            return Err(Error::new("comparison expects at least two arguments"));
        }

        let mut previous = self.expect_int(args[0])?;
        let mut index = 1usize;
        while index < args.len() {
            let current = self.expect_int(args[index])?;
            if !compare(previous, current) {
                return Ok(Value::Bool(false));
            }
            previous = current;
            index += 1;
        }
        Ok(Value::Bool(true))
    }

    fn expect_count(&self, args: &[Value], expected: usize) -> LispResult<()> {
        if args.len() == expected {
            Ok(())
        } else {
            Err(Error::new("wrong argument count"))
        }
    }

    fn expect_int(&self, value: Value) -> LispResult<i32> {
        match value {
            Value::Int(value) => Ok(value),
            _ => Err(Error::new("expected integer")),
        }
    }

    fn expect_u32(&self, value: Value) -> LispResult<u32> {
        match value {
            Value::Word(value) => Ok(value),
            Value::Int(value) if value >= 0 => Ok(value as u32),
            _ => Err(Error::new("expected non-negative integer or word")),
        }
    }

    fn expect_u8(&self, value: Value) -> LispResult<u8> {
        let value = self.expect_u32(value)?;
        if value <= u8::MAX as u32 {
            Ok(value as u8)
        } else {
            Err(Error::new("expected byte"))
        }
    }

    fn expect_string(&self, value: Value) -> LispResult<StringBytes> {
        match value {
            Value::Object(id) => match self.object_kind_by_id(id)? {
                ObjectKind::String { len, bytes } => Ok(StringBytes { len, bytes }),
                _ => Err(Error::new("expected string")),
            },
            _ => Err(Error::new("expected string")),
        }
    }

    fn expect_word_address(&self, value: Value) -> LispResult<u32> {
        let address = self.expect_u32(value)?;
        if address & 0x03 != 0 {
            return Err(Error::new("register address must be word aligned"));
        }
        Ok(address)
    }

    fn led_action(&self, value: Value) -> LispResult<LedAction> {
        match value {
            Value::Symbol(symbol) if symbol == self.specials.on => Ok(LedAction::On),
            Value::Symbol(symbol) if symbol == self.specials.off => Ok(LedAction::Off),
            Value::Symbol(symbol) if symbol == self.specials.toggle => Ok(LedAction::Toggle),
            Value::Symbol(symbol) if symbol == self.specials.status => Ok(LedAction::Status),
            _ => Err(Error::new("led expects on, off, toggle, or status")),
        }
    }

    fn on_off(&self, value: Value) -> LispResult<bool> {
        match value {
            Value::Symbol(symbol) if symbol == self.specials.on => Ok(true),
            Value::Symbol(symbol) if symbol == self.specials.off => Ok(false),
            _ => Err(Error::new("expected on or off")),
        }
    }

    fn truthy(&self, value: Value) -> bool {
        !matches!(value, Value::Nil | Value::Bool(false))
    }

    fn validate_params(&self, params: Value) -> LispResult<()> {
        let mut cursor = params;
        while let Some((param, rest)) = self.list_next(cursor)? {
            if !matches!(param, Value::Symbol(_)) {
                return Err(Error::new("lambda parameters must be symbols"));
            }
            cursor = rest;
        }
        Ok(())
    }

    fn push_env(&mut self, symbol: SymbolId, value: Value, next: Value) -> LispResult<Value> {
        self.alloc_object(ObjectKind::Env {
            symbol,
            value,
            next,
        })
    }

    fn alloc_pair(&mut self, car: Value, cdr: Value) -> LispResult<Value> {
        self.alloc_object(ObjectKind::Pair { car, cdr })
    }

    fn alloc_string(&mut self, value: &[u8]) -> LispResult<Value> {
        if value.len() > MAX_STRING_BYTES {
            return Err(Error::new("string too long"));
        }

        let mut bytes = [0u8; MAX_STRING_BYTES];
        let mut index = 0usize;
        while index < value.len() {
            bytes[index] = value[index];
            index += 1;
        }

        self.alloc_object(ObjectKind::String {
            len: value.len() as u8,
            bytes,
        })
    }

    fn alloc_object(&mut self, kind: ObjectKind) -> LispResult<Value> {
        let id = self.free_head.ok_or(Error::new("heap full"))?;
        let index = id as usize;
        self.free_head = self.objects[index].next_free;
        self.objects[index] = Object {
            marked: false,
            next_free: None,
            kind,
        };
        Ok(Value::Object(id))
    }

    fn make_list_from_values(&mut self, values: &[Value]) -> LispResult<Value> {
        let mut list = Value::Nil;
        let mut index = values.len();
        while index > 0 {
            index -= 1;
            list = self.alloc_pair(values[index], list)?;
        }
        Ok(list)
    }

    fn make_symbol_list(&mut self, names: &[&[u8]]) -> LispResult<Value> {
        let mut list = Value::Nil;
        let mut index = names.len();
        while index > 0 {
            index -= 1;
            let symbol = self.intern(names[index])?;
            list = self.alloc_pair(Value::Symbol(symbol), list)?;
        }
        Ok(list)
    }

    fn entry(&mut self, name: &[u8], value: Value) -> LispResult<Value> {
        let symbol = self.intern(name)?;
        self.alloc_pair(Value::Symbol(symbol), value)
    }

    fn symbol_entry(&mut self, name: &[u8], value: &[u8]) -> LispResult<Value> {
        let value = Value::Symbol(self.intern(value)?);
        self.entry(name, value)
    }

    fn bool_entry(&mut self, name: &[u8], value: bool) -> LispResult<Value> {
        self.entry(name, Value::Bool(value))
    }

    fn int_entry(&mut self, name: &[u8], value: i32) -> LispResult<Value> {
        self.entry(name, Value::Int(value))
    }

    fn word_entry(&mut self, name: &[u8], value: u32) -> LispResult<Value> {
        self.entry(name, Value::Word(value))
    }

    fn help(&mut self) -> LispResult<Value> {
        self.make_symbol_list(&[
            b"help",
            b"quote",
            b"if",
            b"define",
            b"lambda",
            b"begin",
            b"let",
            b"+",
            b"-",
            b"*",
            b"/",
            b"mod",
            b"=",
            b"<",
            b"<=",
            b">",
            b">=",
            b"not",
            b"eq?",
            b"nil?",
            b"atom?",
            b"pair?",
            b"number?",
            b"symbol?",
            b"bool?",
            b"string?",
            b"cons",
            b"car",
            b"cdr",
            b"list",
            b"led",
            b"heartbeat",
            b"button",
            b"millis",
            b"reg32",
            b"poke32",
            b"regs",
            b"sd-status",
            b"sd-pins",
            b"sd-pinmux",
            b"sd-clock",
            b"sd-init",
            b"sd-read",
            b"sd-read0",
            b"sd-write-fill",
            b"format-store",
            b"save-file",
            b"read-file",
            b"load",
            b"ls",
            b"cat",
            b"fat-info",
            b"fat-format",
            b"wifi-sdio-init",
            b"wifi-cmd52-read",
            b"wifi-cmd52-write",
            b"wifi-enable-functions",
            b"wifi-setup-backplane",
            b"wifi-cmd53-read",
            b"wifi-backplane-read",
            b"wifi-backplane-write8",
            b"wifi-backplane-write32",
            b"wifi-backplane-write32-bytes",
            b"wifi-socram-probe",
            b"wifi-socram-block-probe",
            b"wifi-load-firmware",
            b"wifi-start-firmware",
            b"wifi-f2-read-header",
            b"wifi-f2-read-frame",
            b"wifi-f2-read-frame-single",
            b"wifi-f2-read-frame-exact",
            b"wifi-f2-read-frame-block",
            b"wifi-send-wlc-up",
            b"wifi-wlc-up",
            b"wifi-get-version",
            b"wifi-get-mpc",
            b"wifi-load-clm",
            b"wifi-get-clm-version",
            b"wifi-f2-read-frame-abort",
            b"wifi-poll-read-frame",
            b"wifi-ack-interrupts",
            b"wifi-interrupt-state",
            b"wifi-keep-awake",
            b"wifi-request-ht",
            b"wifi-host-reset-lines",
            b"wifi-abort-read",
            b"wifi-core-state",
            b"wifi-reset-core",
            b"sdhc-regs",
            b"heap",
            b"gc",
            b"reboot",
        ])
    }

    fn register_report(&mut self, report: RegisterReport) -> LispResult<Value> {
        let scb5_ctrl = self.word_entry(b"SCB5.CTRL", report.scb5_ctrl)?;
        let scb5_uart_ctrl = self.word_entry(b"SCB5.UART_CTRL", report.scb5_uart_ctrl)?;
        let scb5_rx_status = self.word_entry(b"SCB5.RX_STATUS", report.scb5_rx_status)?;
        let scb5_tx_status = self.word_entry(b"SCB5.TX_STATUS", report.scb5_tx_status)?;
        let peri_clock5 = self.word_entry(b"PERI.CLOCK5", report.peri_clock5)?;
        let peri_div8_0 = self.word_entry(b"PERI.DIV8.0", report.peri_div8_0)?;
        let hsiom_prt5_sel0 = self.word_entry(b"HSIOM.PRT5.SEL0", report.hsiom_prt5_sel0)?;
        let gpio_prt5_cfg = self.word_entry(b"GPIO.PRT5.CFG", report.gpio_prt5_cfg)?;
        let gpio_prt13_out = self.word_entry(b"GPIO.PRT13.OUT", report.gpio_prt13_out)?;
        let gpio_prt13_cfg = self.word_entry(b"GPIO.PRT13.CFG", report.gpio_prt13_cfg)?;
        let entries = [
            scb5_ctrl,
            scb5_uart_ctrl,
            scb5_rx_status,
            scb5_tx_status,
            peri_clock5,
            peri_div8_0,
            hsiom_prt5_sel0,
            gpio_prt5_cfg,
            gpio_prt13_out,
            gpio_prt13_cfg,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_status_report(&mut self, report: SdStatusReport) -> LispResult<Value> {
        let cd_state: &[u8] = if report.cd_low { b"low" } else { b"high" };
        let cd_l = self.symbol_entry(b"CD_L", cd_state)?;
        let inserted = self.bool_entry(b"inserted", report.cd_low)?;
        let prt13_in = self.word_entry(b"GPIO.PRT13.IN", report.prt13_in)?;
        let prt13_cfg = self.word_entry(b"GPIO.PRT13.CFG", report.prt13_cfg)?;
        let entries = [cd_l, inserted, prt13_in, prt13_cfg];
        self.make_list_from_values(&entries)
    }

    fn sd_pins_report(&mut self, report: SdPinsReport) -> LispResult<Value> {
        let p12_sel1 = self.word_entry(b"P12.SEL1", report.p12_sel1)?;
        let p13_sel0 = self.word_entry(b"P13.SEL0", report.p13_sel0)?;
        let p12_cfg = self.word_entry(b"P12.CFG", report.p12_cfg)?;
        let p13_cfg = self.word_entry(b"P13.CFG", report.p13_cfg)?;
        let entries = [p12_sel1, p13_sel0, p12_cfg, p13_cfg];
        self.make_list_from_values(&entries)
    }

    fn sdhc_core_report(&mut self, report: SdhcCoreReport) -> LispResult<Value> {
        let wrap_ctl = self.word_entry(b"WRAP.CTL", report.wrap_ctl)?;
        let host_version = self.word_entry(b"HOST.VERSION", report.host_version as u32)?;
        let cap1 = self.word_entry(b"CAP1", report.cap1)?;
        let cap2 = self.word_entry(b"CAP2", report.cap2)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let entries = [wrap_ctl, host_version, cap1, cap2, pstate];
        self.make_list_from_values(&entries)
    }

    fn sdhc_report(&mut self, report: SdhcReport) -> LispResult<Value> {
        let sdhc0 = self.sdhc_core_report(report.sdhc0)?;
        let sdhc1 = self.sdhc_core_report(report.sdhc1)?;
        let pins = self.sd_pins_report(report.pins)?;
        let sdhc0_section = self.entry(b"SDHC0", sdhc0)?;
        let sdhc1_section = self.entry(b"SDHC1", sdhc1)?;
        let pins_section = self.entry(b"microSD-pins", pins)?;
        let entries = [sdhc0_section, sdhc1_section, pins_section];
        self.make_list_from_values(&entries)
    }

    fn sd_clock_report(&mut self, report: SdClockReport) -> LispResult<Value> {
        let path0 = self.word_entry(b"CLK_PATH0", report.path0)?;
        let root0 = self.word_entry(b"CLK_HF0", report.root0)?;
        let root2 = self.word_entry(b"CLK_HF2", report.root2)?;
        let fll_config = self.word_entry(b"FLL_CONFIG", report.fll_config)?;
        let fll_config2 = self.word_entry(b"FLL_CONFIG2", report.fll_config2)?;
        let fll_status = self.word_entry(b"FLL_STATUS", report.fll_status)?;
        let selected_hf_hz = self.word_entry(b"selected-HF-Hz", report.selected_hf_hz)?;
        let entries = [
            path0,
            root0,
            root2,
            fll_config,
            fll_config2,
            fll_status,
            selected_hf_hz,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_init_report(&mut self, report: SdInitReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let cmd8 = self.word_entry(b"CMD8", report.cmd8_response)?;
        let cmd8_error = self.sd_error_code_entry(b"CMD8.error", report.cmd8_error)?;
        let acmd41_ocr = self.word_entry(b"ACMD41.OCR", report.acmd41_ocr)?;
        let attempts = self.int_entry(b"attempts", report.acmd41_attempts as i32)?;
        let clk_ctrl = self.word_entry(b"CLK_CTRL", report.clk_ctrl as u32)?;
        let normal_int = self.word_entry(b"NORM_INT", report.normal_int as u32)?;
        let error_int = self.word_entry(b"ERR_INT", report.error_int as u32)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let cmd = self.word_entry(b"CMD_R", report.cmd as u32)?;
        let argument = self.word_entry(b"ARGUMENT", report.argument)?;
        let last_error = self.sd_error_code_entry(b"last-error", report.last_error)?;
        let pstate_error =
            self.sd_error_word_entry(b"PSTATE.error", report.last_error.map(|error| error.pstate))?;
        let pstate_after_write = self.sd_error_word_entry(
            b"PSTATE.after-write",
            report.last_error.map(|error| error.pstate_after_write),
        )?;
        let normal_int_after_write = self.sd_error_word_entry(
            b"NORM_INT.after-write",
            report
                .last_error
                .map(|error| error.normal_int_after_write as u32),
        )?;
        let error_int_after_write = self.sd_error_word_entry(
            b"ERR_INT.after-write",
            report
                .last_error
                .map(|error| error.error_int_after_write as u32),
        )?;
        let entries = [
            status,
            cmd8,
            cmd8_error,
            acmd41_ocr,
            attempts,
            clk_ctrl,
            normal_int,
            error_int,
            pstate,
            cmd,
            argument,
            last_error,
            pstate_error,
            pstate_after_write,
            normal_int_after_write,
            error_int_after_write,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_read_report(&mut self, report: SdReadReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let init_status = self.symbol_entry(b"init-status", report.init_status)?;
        let sector = self.word_entry(b"sector", report.sector)?;
        let rca = self.word_entry(b"RCA", report.rca as u32)?;
        let ocr = self.word_entry(b"OCR", report.ocr)?;
        let attempts = self.int_entry(b"attempts", report.acmd41_attempts as i32)?;
        let response = self.word_entry(b"CMD17.response", report.command_response)?;
        let last_error = self.sd_error_code_entry(b"last-error", report.last_error)?;
        let first_words = self.sd_word_list_entry(b"first-words", &report.first_words)?;
        let mbr_signature = self.word_entry(b"MBR.sig", report.mbr_signature as u32)?;
        let partition_status =
            self.word_entry(b"partition0.status", report.partition_status as u32)?;
        let partition_type = self.word_entry(b"partition0.type", report.partition_type as u32)?;
        let partition_lba_start = self.word_entry(b"partition0.lba", report.partition_lba_start)?;
        let partition_sector_count =
            self.word_entry(b"partition0.sectors", report.partition_sector_count)?;
        let normal_int = self.word_entry(b"NORM_INT", report.normal_int as u32)?;
        let error_int = self.word_entry(b"ERR_INT", report.error_int as u32)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let block_size = self.word_entry(b"BLOCK_SIZE", report.block_size as u32)?;
        let block_count = self.word_entry(b"BLOCK_COUNT", report.block_count as u32)?;
        let xfer_mode = self.word_entry(b"XFER_MODE", report.xfer_mode as u32)?;
        let cmd = self.word_entry(b"CMD_R", report.cmd as u32)?;
        let argument = self.word_entry(b"ARGUMENT", report.argument)?;
        let entries = [
            status,
            init_status,
            sector,
            rca,
            ocr,
            attempts,
            response,
            last_error,
            first_words,
            mbr_signature,
            partition_status,
            partition_type,
            partition_lba_start,
            partition_sector_count,
            normal_int,
            error_int,
            pstate,
            block_size,
            block_count,
            xfer_mode,
            cmd,
            argument,
        ];
        self.make_list_from_values(&entries)
    }

    fn sd_write_report(&mut self, report: SdWriteReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let init_status = self.symbol_entry(b"init-status", report.init_status)?;
        let sector = self.word_entry(b"sector", report.sector)?;
        let fill_word = self.word_entry(b"fill-word", report.fill_word)?;
        let rca = self.word_entry(b"RCA", report.rca as u32)?;
        let ocr = self.word_entry(b"OCR", report.ocr)?;
        let attempts = self.int_entry(b"attempts", report.acmd41_attempts as i32)?;
        let response = self.word_entry(b"CMD24.response", report.command_response)?;
        let last_error = self.sd_error_code_entry(b"last-error", report.last_error)?;
        let normal_int = self.word_entry(b"NORM_INT", report.normal_int as u32)?;
        let error_int = self.word_entry(b"ERR_INT", report.error_int as u32)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let block_size = self.word_entry(b"BLOCK_SIZE", report.block_size as u32)?;
        let block_count = self.word_entry(b"BLOCK_COUNT", report.block_count as u32)?;
        let xfer_mode = self.word_entry(b"XFER_MODE", report.xfer_mode as u32)?;
        let cmd = self.word_entry(b"CMD_R", report.cmd as u32)?;
        let argument = self.word_entry(b"ARGUMENT", report.argument)?;
        let entries = [
            status,
            init_status,
            sector,
            fill_word,
            rca,
            ocr,
            attempts,
            response,
            last_error,
            normal_int,
            error_int,
            pstate,
            block_size,
            block_count,
            xfer_mode,
            cmd,
            argument,
        ];
        self.make_list_from_values(&entries)
    }

    fn store_write_report(&mut self, report: StoreWriteReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ready = self.bool_entry(b"ready", report.ready)?;
        let path_len = self.int_entry(b"path-len", report.path_len as i32)?;
        let content_len = self.int_entry(b"content-len", report.content_len as i32)?;
        let directory_sector = self.word_entry(b"directory-sector", report.directory_sector)?;
        let data_sector = self.word_entry(b"data-sector", report.data_sector)?;
        let entries = [
            status,
            ready,
            path_len,
            content_len,
            directory_sector,
            data_sector,
        ];
        self.make_list_from_values(&entries)
    }

    fn store_format_report(&mut self, report: StoreFormatReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ready = self.bool_entry(b"ready", report.ready)?;
        let directory_sector = self.word_entry(b"directory-sector", report.directory_sector)?;
        let data_start_sector = self.word_entry(b"data-start-sector", report.data_start_sector)?;
        let data_sector_count =
            self.int_entry(b"data-sector-count", report.data_sector_count as i32)?;
        let failed_sector = self.word_entry(b"failed-sector", report.failed_sector)?;
        let entries = [
            status,
            ready,
            directory_sector,
            data_start_sector,
            data_sector_count,
            failed_sector,
        ];
        self.make_list_from_values(&entries)
    }

    fn store_read_report(&mut self, report: StoreReadReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ready = self.bool_entry(b"ready", report.ready)?;
        let path_len = self.int_entry(b"path-len", report.path_len as i32)?;
        let content_len = self.int_entry(b"content-len", report.content_len as i32)?;
        let directory_sector = self.word_entry(b"directory-sector", report.directory_sector)?;
        let data_sector = self.word_entry(b"data-sector", report.data_sector)?;
        let content = if report.ready {
            self.string_value(report.content)?
        } else {
            Value::Nil
        };
        let content = self.entry(b"content", content)?;
        let entries = [
            status,
            ready,
            path_len,
            content_len,
            directory_sector,
            data_sector,
            content,
        ];
        self.make_list_from_values(&entries)
    }

    fn store_list_report(&mut self, report: StoreListReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ready = self.bool_entry(b"ready", report.ready)?;
        let count = self.int_entry(b"count", report.file_count as i32)?;
        let directory_sector = self.word_entry(b"directory-sector", report.directory_sector)?;
        let files = if report.ready {
            self.string_list(&report.files, report.file_count)?
        } else {
            Value::Nil
        };
        let files = self.entry(b"files", files)?;
        let entries = [status, ready, count, directory_sector, files];
        self.make_list_from_values(&entries)
    }

    fn fat_info_report(&mut self, report: FatInfoReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ready = self.bool_entry(b"ready", report.ready)?;
        let mbr_signature = self.word_entry(b"MBR.sig", report.mbr_signature as u32)?;
        let partition_status =
            self.word_entry(b"partition0.status", report.partition_status as u32)?;
        let partition_type = self.word_entry(b"partition0.type", report.partition_type as u32)?;
        let partition_lba = self.word_entry(b"partition0.lba", report.partition_lba_start)?;
        let partition_sectors =
            self.word_entry(b"partition0.sectors", report.partition_sector_count)?;
        let root_entry_count =
            self.int_entry(b"root-entry-count", report.root_entry_count as i32)?;
        let entries = if report.ready {
            self.string_list(&report.entries, report.sample_count)?
        } else {
            Value::Nil
        };
        let entries = self.entry(b"entries", entries)?;
        let values = [
            status,
            ready,
            mbr_signature,
            partition_status,
            partition_type,
            partition_lba,
            partition_sectors,
            root_entry_count,
            entries,
        ];
        self.make_list_from_values(&values)
    }

    fn fat_format_report(&mut self, report: FatFormatReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ready = self.bool_entry(b"ready", report.ready)?;
        let mbr_signature = self.word_entry(b"MBR.sig", report.mbr_signature as u32)?;
        let partition_status =
            self.word_entry(b"partition0.status", report.partition_status as u32)?;
        let partition_type_before = self.word_entry(
            b"partition0.type.before",
            report.partition_type_before as u32,
        )?;
        let partition_type_after =
            self.word_entry(b"partition0.type.after", report.partition_type_after as u32)?;
        let partition_lba = self.word_entry(b"partition0.lba", report.partition_lba_start)?;
        let partition_sectors =
            self.word_entry(b"partition0.sectors", report.partition_sector_count)?;
        let sectors_per_cluster =
            self.int_entry(b"sectors-per-cluster", report.sectors_per_cluster as i32)?;
        let reserved_sectors =
            self.int_entry(b"reserved-sectors", report.reserved_sectors as i32)?;
        let fat_count = self.int_entry(b"fat-count", report.fat_count as i32)?;
        let fat_size_sectors = self.word_entry(b"fat-size-sectors", report.fat_size_sectors)?;
        let data_cluster_count =
            self.word_entry(b"data-cluster-count", report.data_cluster_count)?;
        let root_cluster = self.word_entry(b"root-cluster", report.root_cluster)?;
        let written_sector_count =
            self.word_entry(b"written-sector-count", report.written_sector_count)?;
        let failed_sector = self.word_entry(b"failed-sector", report.failed_sector)?;
        let values = [
            status,
            ready,
            mbr_signature,
            partition_status,
            partition_type_before,
            partition_type_after,
            partition_lba,
            partition_sectors,
            sectors_per_cluster,
            reserved_sectors,
            fat_count,
            fat_size_sectors,
            data_cluster_count,
            root_cluster,
            written_sector_count,
            failed_sector,
        ];
        self.make_list_from_values(&values)
    }

    fn string_value(&mut self, value: StringBytes) -> LispResult<Value> {
        self.alloc_string(&value.bytes[..value.len as usize])
    }

    fn string_list(
        &mut self,
        values: &[StringBytes; MAX_STORE_FILES],
        count: u8,
    ) -> LispResult<Value> {
        let mut list = Value::Nil;
        let mut index = usize::from(count);
        if index > MAX_STORE_FILES {
            return Err(Error::new("too many store files"));
        }
        while index > 0 {
            index -= 1;
            let value = self.string_value(values[index])?;
            list = self.alloc_pair(value, list)?;
        }
        Ok(list)
    }

    fn wifi_sdio_report(&mut self, report: WifiSdioReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let cmd5 = self.word_entry(b"CMD5.OCR", report.cmd5_response)?;
        let attempts = self.int_entry(b"attempts", report.cmd5_attempts as i32)?;
        let rca = self.word_entry(b"RCA", report.rca as u32)?;
        let function_count = self.int_entry(b"functions", report.function_count as i32)?;
        let memory_present = self.bool_entry(b"memory-present", report.memory_present)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let pins = self.wifi_sdio_pins_report(report.pins)?;
        let clock = self.wifi_sdio_clock_report(report.clock)?;
        let host = self.entry(b"SDHC0", host)?;
        let pins = self.entry(b"P2", pins)?;
        let clock = self.entry(b"clock", clock)?;
        let entries = [
            status,
            cmd5,
            attempts,
            rca,
            function_count,
            memory_present,
            last_error,
            host,
            pins,
            clock,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_direct_report(&mut self, report: WifiSdioDirectReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let init_status = self.symbol_entry(b"init-status", report.init_status)?;
        let function = self.int_entry(b"function", report.function as i32)?;
        let address = self.word_entry(b"address", report.address)?;
        let write = self.bool_entry(b"write", report.write)?;
        let data = self.word_entry(b"data", report.data as u32)?;
        let response = self.word_entry(b"response", report.response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            init_status,
            function,
            address,
            write,
            data,
            response,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_enable_report(&mut self, report: WifiSdioEnableReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let init_status = self.symbol_entry(b"init-status", report.init_status)?;
        let requested = self.word_entry(b"requested", report.requested as u32)?;
        let ready = self.word_entry(b"ready", report.ready as u32)?;
        let attempts = self.int_entry(b"attempts", report.attempts as i32)?;
        let write_response = self.word_entry(b"write-response", report.write_response)?;
        let ready_response = self.word_entry(b"ready-response", report.ready_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            init_status,
            requested,
            ready,
            attempts,
            write_response,
            ready_response,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_backplane_report(&mut self, report: WifiSdioBackplaneReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let init_status = self.symbol_entry(b"init-status", report.init_status)?;
        let io_enable = self.word_entry(b"IOEN", report.io_enable as u32)?;
        let io_ready = self.word_entry(b"IORDY", report.io_ready as u32)?;
        let bus_control_before =
            self.word_entry(b"BICTRL.before", report.bus_control_before as u32)?;
        let bus_control_after =
            self.word_entry(b"BICTRL.after", report.bus_control_after as u32)?;
        let f0_block_size = self.word_entry(b"F0.block-size", report.f0_block_size as u32)?;
        let f1_block_size = self.word_entry(b"F1.block-size", report.f1_block_size as u32)?;
        let f2_block_size = self.word_entry(b"F2.block-size", report.f2_block_size as u32)?;
        let interrupt_enable = self.word_entry(b"INTEN", report.interrupt_enable as u32)?;
        let attempts = self.int_entry(b"attempts", report.attempts as i32)?;
        let last_response = self.word_entry(b"last-response", report.last_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
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
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_cmd53_read_report(
        &mut self,
        report: WifiSdioCmd53ReadReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let function = self.int_entry(b"function", report.function as i32)?;
        let address = self.word_entry(b"address", report.address)?;
        let count = self.word_entry(b"count", report.count as u32)?;
        let response = self.word_entry(b"response", report.response)?;
        let bytes = self.wifi_byte_list_entry(b"bytes", &report.bytes, report.count)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            setup_status,
            function,
            address,
            count,
            response,
            bytes,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_backplane_read_report(
        &mut self,
        report: WifiSdioBackplaneReadReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let address = self.word_entry(b"address", report.address)?;
        let count = self.word_entry(b"count", report.count as u32)?;
        let window_base = self.word_entry(b"window-base", report.window_base)?;
        let window_address = self.word_entry(b"window-address", report.window_address)?;
        let response = self.word_entry(b"response", report.response)?;
        let bytes = self.wifi_byte_list_entry(b"bytes", &report.bytes, report.count)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            setup_status,
            address,
            count,
            window_base,
            window_address,
            response,
            bytes,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_backplane_write8_report(
        &mut self,
        report: WifiSdioBackplaneWrite8Report,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let address = self.word_entry(b"address", report.address)?;
        let value = self.word_entry(b"value", report.value as u32)?;
        let window_base = self.word_entry(b"window-base", report.window_base)?;
        let window_address = self.word_entry(b"window-address", report.window_address)?;
        let response = self.word_entry(b"response", report.response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            setup_status,
            address,
            value,
            window_base,
            window_address,
            response,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_backplane_write32_report(
        &mut self,
        report: WifiSdioBackplaneWrite32Report,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let address = self.word_entry(b"address", report.address)?;
        let value = self.word_entry(b"value", report.value)?;
        let window_base = self.word_entry(b"window-base", report.window_base)?;
        let window_address = self.word_entry(b"window-address", report.window_address)?;
        let response = self.word_entry(b"response", report.response)?;
        let readback = self.word_entry(b"readback", report.readback)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            setup_status,
            address,
            value,
            window_base,
            window_address,
            response,
            readback,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_socram_probe_report(
        &mut self,
        report: WifiSdioSocramProbeReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let write_status = self.symbol_entry(b"write-status", report.write_status)?;
        let address = self.word_entry(b"address", report.address)?;
        let pattern = self.word_entry(b"pattern", report.pattern)?;
        let original = self.word_entry(b"original", report.original)?;
        let readback = self.word_entry(b"readback", report.readback)?;
        let restored = self.word_entry(b"restored", report.restored)?;
        let last_response = self.word_entry(b"last-response", report.last_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
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
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_socram_block_probe_report(
        &mut self,
        report: WifiSdioSocramBlockProbeReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let read_status = self.symbol_entry(b"read-status", report.read_status)?;
        let write_status = self.symbol_entry(b"write-status", report.write_status)?;
        let address = self.word_entry(b"address", report.address)?;
        let seed = self.word_entry(b"seed", report.seed)?;
        let original_checksum = self.word_entry(b"original-checksum", report.original_checksum)?;
        let readback_checksum = self.word_entry(b"readback-checksum", report.readback_checksum)?;
        let restored_checksum = self.word_entry(b"restored-checksum", report.restored_checksum)?;
        let mismatch_index = self.word_entry(b"mismatch-index", report.mismatch_index)?;
        let mismatch_expected = self.word_entry(b"mismatch-expected", report.mismatch_expected)?;
        let mismatch_actual = self.word_entry(b"mismatch-actual", report.mismatch_actual)?;
        let last_response = self.word_entry(b"last-response", report.last_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
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
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_firmware_load_report(
        &mut self,
        report: WifiSdioFirmwareLoadReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let read_status = self.symbol_entry(b"read-status", report.read_status)?;
        let write_status = self.symbol_entry(b"write-status", report.write_status)?;
        let firmware_bytes = self.word_entry(b"firmware-bytes", report.firmware_bytes)?;
        let processed_bytes = self.word_entry(b"processed-bytes", report.processed_bytes)?;
        let chunk_count = self.word_entry(b"chunk-count", report.chunk_count)?;
        let firmware_checksum = self.word_entry(b"firmware-checksum", report.firmware_checksum)?;
        let verify_checksum = self.word_entry(b"verify-checksum", report.verify_checksum)?;
        let mismatch_offset = self.word_entry(b"mismatch-offset", report.mismatch_offset)?;
        let mismatch_expected = self.word_entry(b"mismatch-expected", report.mismatch_expected)?;
        let mismatch_actual = self.word_entry(b"mismatch-actual", report.mismatch_actual)?;
        let last_response = self.word_entry(b"last-response", report.last_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
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
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_firmware_start_report(
        &mut self,
        report: WifiSdioFirmwareStartReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let firmware_status = self.symbol_entry(b"firmware-status", report.firmware_status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let read_status = self.symbol_entry(b"read-status", report.read_status)?;
        let write_status = self.symbol_entry(b"write-status", report.write_status)?;
        let firmware_bytes = self.word_entry(b"firmware-bytes", report.firmware_bytes)?;
        let nvram_bytes = self.word_entry(b"nvram-bytes", report.nvram_bytes)?;
        let nvram_rounded_bytes =
            self.word_entry(b"nvram-rounded-bytes", report.nvram_rounded_bytes)?;
        let nvram_address = self.word_entry(b"nvram-address", report.nvram_address)?;
        let nvram_size_word = self.word_entry(b"nvram-size-word", report.nvram_size_word)?;
        let firmware_checksum = self.word_entry(b"firmware-checksum", report.firmware_checksum)?;
        let nvram_checksum = self.word_entry(b"nvram-checksum", report.nvram_checksum)?;
        let nvram_verify_checksum =
            self.word_entry(b"nvram-verify-checksum", report.nvram_verify_checksum)?;
        let mismatch_offset = self.word_entry(b"mismatch-offset", report.mismatch_offset)?;
        let mismatch_expected = self.word_entry(b"mismatch-expected", report.mismatch_expected)?;
        let mismatch_actual = self.word_entry(b"mismatch-actual", report.mismatch_actual)?;
        let arm_before = self.wifi_sdio_core_snapshot_report(report.arm_before)?;
        let arm_before = self.entry(b"arm-before", arm_before)?;
        let arm_after = self.wifi_sdio_core_snapshot_report(report.arm_after)?;
        let arm_after = self.entry(b"arm-after", arm_after)?;
        let ht_clock_csr = self.word_entry(b"HT-CLOCK-CSR", report.ht_clock_csr as u32)?;
        let ht_attempts = self.word_entry(b"HT-attempts", report.ht_attempts as u32)?;
        let io_enable = self.word_entry(b"IOEN", report.io_enable as u32)?;
        let io_ready = self.word_entry(b"IORDY", report.io_ready as u32)?;
        let f2_attempts = self.word_entry(b"F2-attempts", report.f2_attempts as u32)?;
        let last_response = self.word_entry(b"last-response", report.last_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
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
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_f2_header_report(&mut self, report: WifiSdioF2HeaderReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let response = self.word_entry(b"response", report.response)?;
        let length = self.word_entry(b"length", report.length as u32)?;
        let checksum = self.word_entry(b"checksum", report.checksum as u32)?;
        let valid = self.bool_entry(b"valid", report.valid)?;
        let byte0 = self.word_entry(b"byte0", report.bytes[0] as u32)?;
        let byte1 = self.word_entry(b"byte1", report.bytes[1] as u32)?;
        let byte2 = self.word_entry(b"byte2", report.bytes[2] as u32)?;
        let byte3 = self.word_entry(b"byte3", report.bytes[3] as u32)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status, response, length, checksum, valid, byte0, byte1, byte2, byte3, last_error, host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_f2_frame_report(&mut self, report: WifiSdioF2FrameReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let header_status = self.symbol_entry(b"header-status", report.header_status)?;
        let body_status = self.symbol_entry(b"body-status", report.body_status)?;
        let length = self.word_entry(b"length", report.length as u32)?;
        let checksum = self.word_entry(b"checksum", report.checksum as u32)?;
        let valid = self.bool_entry(b"valid", report.valid)?;
        let byte_count = self.word_entry(b"byte-count", report.byte_count as u32)?;
        let bytes = self.wifi_byte_list_entry(b"bytes", &report.bytes, report.byte_count)?;
        let sequence = self.word_entry(b"sequence", report.sequence as u32)?;
        let channel_and_flags =
            self.word_entry(b"channel-and-flags", report.channel_and_flags as u32)?;
        let channel = self.word_entry(b"channel", report.channel as u32)?;
        let flags = self.word_entry(b"flags", report.flags as u32)?;
        let next_length = self.word_entry(b"next-length", report.next_length as u32)?;
        let header_length = self.word_entry(b"header-length", report.header_length as u32)?;
        let wireless_flow_control = self.word_entry(
            b"wireless-flow-control",
            report.wireless_flow_control as u32,
        )?;
        let bus_data_credit = self.word_entry(b"bus-data-credit", report.bus_data_credit as u32)?;
        let reserved0 = self.word_entry(b"reserved0", report.reserved0 as u32)?;
        let reserved1 = self.word_entry(b"reserved1", report.reserved1 as u32)?;
        let header_response = self.word_entry(b"header-response", report.header_response)?;
        let body_response = self.word_entry(b"body-response", report.body_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            header_status,
            body_status,
            length,
            checksum,
            valid,
            byte_count,
            bytes,
            sequence,
            channel_and_flags,
            channel,
            flags,
            next_length,
            header_length,
            wireless_flow_control,
            bus_data_credit,
            reserved0,
            reserved1,
            header_response,
            body_response,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_f2_control_report(
        &mut self,
        report: WifiSdioF2ControlReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let initial_tx_credit =
            self.word_entry(b"initial-tx-credit", report.initial_tx_credit as u32)?;
        let packet_length = self.word_entry(b"packet.length", report.packet_length as u32)?;
        let write_response = self.word_entry(b"write.response", report.write_response)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let write_last_error =
            self.wifi_sdio_error_entry(b"write.last-error", report.write_last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            initial_tx_credit,
            packet_length,
            write_response,
            host_normal_int,
            host_error_int,
            write_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_wlc_up_report(&mut self, report: WifiSdioWlcUpReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let send_status = self.symbol_entry(b"send.status", report.send_status)?;
        let send_packet_length =
            self.word_entry(b"send.packet-length", report.send_packet_length as u32)?;
        let send_write_response =
            self.word_entry(b"send.write-response", report.send_write_response)?;
        let startup_status = self.symbol_entry(b"startup.status", report.startup_status)?;
        let startup_length = self.word_entry(b"startup.length", report.startup_length as u32)?;
        let startup_channel = self.word_entry(b"startup.channel", report.startup_channel as u32)?;
        let startup_bus_data_credit = self.word_entry(
            b"startup.bus-data-credit",
            report.startup_bus_data_credit as u32,
        )?;
        let response_status = self.symbol_entry(b"response.status", report.response_status)?;
        let response_length = self.word_entry(b"response.length", report.response_length as u32)?;
        let response_sequence =
            self.word_entry(b"response.sequence", report.response_sequence as u32)?;
        let response_channel =
            self.word_entry(b"response.channel", report.response_channel as u32)?;
        let response_bus_data_credit = self.word_entry(
            b"response.bus-data-credit",
            report.response_bus_data_credit as u32,
        )?;
        let cdc_command = self.word_entry(b"cdc.command", report.cdc_command)?;
        let cdc_length = self.word_entry(b"cdc.length", report.cdc_length)?;
        let cdc_flags = self.word_entry(b"cdc.flags", report.cdc_flags)?;
        let cdc_id = self.word_entry(b"cdc.id", report.cdc_id as u32)?;
        let cdc_status = self.word_entry(b"cdc.status", report.cdc_status)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let send_last_error =
            self.wifi_sdio_error_entry(b"send.last-error", report.send_last_error)?;
        let startup_last_error =
            self.wifi_sdio_error_entry(b"startup.last-error", report.startup_last_error)?;
        let response_last_error =
            self.wifi_sdio_error_entry(b"response.last-error", report.response_last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            send_status,
            send_packet_length,
            send_write_response,
            startup_status,
            startup_length,
            startup_channel,
            startup_bus_data_credit,
            response_status,
            response_length,
            response_sequence,
            response_channel,
            response_bus_data_credit,
            cdc_command,
            cdc_length,
            cdc_flags,
            cdc_id,
            cdc_status,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            startup_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_get_version_report(
        &mut self,
        report: WifiSdioGetVersionReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let send_status = self.symbol_entry(b"send.status", report.send_status)?;
        let send_packet_length =
            self.word_entry(b"send.packet-length", report.send_packet_length as u32)?;
        let send_write_response =
            self.word_entry(b"send.write-response", report.send_write_response)?;
        let response_status = self.symbol_entry(b"response.status", report.response_status)?;
        let response_length = self.word_entry(b"response.length", report.response_length as u32)?;
        let response_sequence =
            self.word_entry(b"response.sequence", report.response_sequence as u32)?;
        let response_channel =
            self.word_entry(b"response.channel", report.response_channel as u32)?;
        let response_bus_data_credit = self.word_entry(
            b"response.bus-data-credit",
            report.response_bus_data_credit as u32,
        )?;
        let cdc_command = self.word_entry(b"cdc.command", report.cdc_command)?;
        let cdc_length = self.word_entry(b"cdc.length", report.cdc_length)?;
        let cdc_flags = self.word_entry(b"cdc.flags", report.cdc_flags)?;
        let cdc_id = self.word_entry(b"cdc.id", report.cdc_id as u32)?;
        let cdc_status = self.word_entry(b"cdc.status", report.cdc_status)?;
        let version = self.word_entry(b"version", report.version)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let send_last_error =
            self.wifi_sdio_error_entry(b"send.last-error", report.send_last_error)?;
        let response_last_error =
            self.wifi_sdio_error_entry(b"response.last-error", report.response_last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            send_status,
            send_packet_length,
            send_write_response,
            response_status,
            response_length,
            response_sequence,
            response_channel,
            response_bus_data_credit,
            cdc_command,
            cdc_length,
            cdc_flags,
            cdc_id,
            cdc_status,
            version,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_get_mpc_report(&mut self, report: WifiSdioGetMpcReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let send_status = self.symbol_entry(b"send.status", report.send_status)?;
        let send_packet_length =
            self.word_entry(b"send.packet-length", report.send_packet_length as u32)?;
        let send_write_response =
            self.word_entry(b"send.write-response", report.send_write_response)?;
        let response_status = self.symbol_entry(b"response.status", report.response_status)?;
        let response_length = self.word_entry(b"response.length", report.response_length as u32)?;
        let response_sequence =
            self.word_entry(b"response.sequence", report.response_sequence as u32)?;
        let response_channel =
            self.word_entry(b"response.channel", report.response_channel as u32)?;
        let response_bus_data_credit = self.word_entry(
            b"response.bus-data-credit",
            report.response_bus_data_credit as u32,
        )?;
        let cdc_command = self.word_entry(b"cdc.command", report.cdc_command)?;
        let cdc_length = self.word_entry(b"cdc.length", report.cdc_length)?;
        let cdc_flags = self.word_entry(b"cdc.flags", report.cdc_flags)?;
        let cdc_id = self.word_entry(b"cdc.id", report.cdc_id as u32)?;
        let cdc_status = self.word_entry(b"cdc.status", report.cdc_status)?;
        let value = self.word_entry(b"value", report.value)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let send_last_error =
            self.wifi_sdio_error_entry(b"send.last-error", report.send_last_error)?;
        let response_last_error =
            self.wifi_sdio_error_entry(b"response.last-error", report.response_last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            send_status,
            send_packet_length,
            send_write_response,
            response_status,
            response_length,
            response_sequence,
            response_channel,
            response_bus_data_credit,
            cdc_command,
            cdc_length,
            cdc_flags,
            cdc_id,
            cdc_status,
            value,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_clm_load_report(&mut self, report: WifiSdioClmLoadReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let clm_bytes = self.word_entry(b"clm.bytes", report.clm_bytes)?;
        let processed_bytes = self.word_entry(b"processed.bytes", report.processed_bytes)?;
        let chunk_count = self.word_entry(b"chunk.count", report.chunk_count)?;
        let chunk_index = self.word_entry(b"chunk.index", report.chunk_index)?;
        let chunk_bytes = self.word_entry(b"chunk.bytes", report.chunk_bytes as u32)?;
        let chunk_flags = self.word_entry(b"chunk.flags", report.chunk_flags as u32)?;
        let payload_bytes = self.word_entry(b"payload.bytes", report.payload_bytes as u32)?;
        let send_status = self.symbol_entry(b"send.status", report.send_status)?;
        let send_packet_length =
            self.word_entry(b"send.packet-length", report.send_packet_length as u32)?;
        let send_write_response =
            self.word_entry(b"send.write-response", report.send_write_response)?;
        let response_attempts =
            self.word_entry(b"response.attempts", report.response_attempts as u32)?;
        let response_status = self.symbol_entry(b"response.status", report.response_status)?;
        let response_length = self.word_entry(b"response.length", report.response_length as u32)?;
        let response_sequence =
            self.word_entry(b"response.sequence", report.response_sequence as u32)?;
        let response_channel =
            self.word_entry(b"response.channel", report.response_channel as u32)?;
        let response_bus_data_credit = self.word_entry(
            b"response.bus-data-credit",
            report.response_bus_data_credit as u32,
        )?;
        let cdc_command = self.word_entry(b"cdc.command", report.cdc_command)?;
        let cdc_length = self.word_entry(b"cdc.length", report.cdc_length)?;
        let cdc_flags = self.word_entry(b"cdc.flags", report.cdc_flags)?;
        let cdc_id = self.word_entry(b"cdc.id", report.cdc_id as u32)?;
        let cdc_status = self.word_entry(b"cdc.status", report.cdc_status)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let send_last_error =
            self.wifi_sdio_error_entry(b"send.last-error", report.send_last_error)?;
        let response_last_error =
            self.wifi_sdio_error_entry(b"response.last-error", report.response_last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            clm_bytes,
            processed_bytes,
            chunk_count,
            chunk_index,
            chunk_bytes,
            chunk_flags,
            payload_bytes,
            send_status,
            send_packet_length,
            send_write_response,
            response_attempts,
            response_status,
            response_length,
            response_sequence,
            response_channel,
            response_bus_data_credit,
            cdc_command,
            cdc_length,
            cdc_flags,
            cdc_id,
            cdc_status,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_get_clm_version_report(
        &mut self,
        report: WifiSdioGetClmVersionReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let send_status = self.symbol_entry(b"send.status", report.send_status)?;
        let send_packet_length =
            self.word_entry(b"send.packet-length", report.send_packet_length as u32)?;
        let send_write_response =
            self.word_entry(b"send.write-response", report.send_write_response)?;
        let response_status = self.symbol_entry(b"response.status", report.response_status)?;
        let response_length = self.word_entry(b"response.length", report.response_length as u32)?;
        let response_sequence =
            self.word_entry(b"response.sequence", report.response_sequence as u32)?;
        let response_channel =
            self.word_entry(b"response.channel", report.response_channel as u32)?;
        let response_bus_data_credit = self.word_entry(
            b"response.bus-data-credit",
            report.response_bus_data_credit as u32,
        )?;
        let cdc_command = self.word_entry(b"cdc.command", report.cdc_command)?;
        let cdc_length = self.word_entry(b"cdc.length", report.cdc_length)?;
        let cdc_flags = self.word_entry(b"cdc.flags", report.cdc_flags)?;
        let cdc_id = self.word_entry(b"cdc.id", report.cdc_id as u32)?;
        let cdc_status = self.word_entry(b"cdc.status", report.cdc_status)?;
        let copied_bytes = self.word_entry(b"copied.bytes", report.copied_bytes as u32)?;
        let version_len = self.word_entry(b"version.length", report.version_len as u32)?;
        let version_truncated = self.bool_entry(b"version.truncated", report.version_truncated)?;
        let version = if report.version_len > 0 {
            self.string_value(report.version)?
        } else {
            Value::Nil
        };
        let version = self.entry(b"version", version)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let send_last_error =
            self.wifi_sdio_error_entry(b"send.last-error", report.send_last_error)?;
        let response_last_error =
            self.wifi_sdio_error_entry(b"response.last-error", report.response_last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            send_status,
            send_packet_length,
            send_write_response,
            response_status,
            response_length,
            response_sequence,
            response_channel,
            response_bus_data_credit,
            cdc_command,
            cdc_length,
            cdc_flags,
            cdc_id,
            cdc_status,
            copied_bytes,
            version_len,
            version_truncated,
            version,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_f2_abort_probe_report(
        &mut self,
        report: WifiSdioF2AbortProbeReport,
    ) -> LispResult<Value> {
        let frame_status = self.symbol_entry(b"frame.status", report.frame_status)?;
        let frame_valid = self.bool_entry(b"frame.valid", report.frame_valid)?;
        let frame_length = self.word_entry(b"frame.length", report.frame_length as u32)?;
        let frame_channel = self.word_entry(b"frame.channel", report.frame_channel as u32)?;
        let frame_bus_data_credit = self.word_entry(
            b"frame.bus-data-credit",
            report.frame_bus_data_credit as u32,
        )?;
        let frame_header_response =
            self.word_entry(b"frame.header-response", report.frame_header_response)?;
        let frame_body_response =
            self.word_entry(b"frame.body-response", report.frame_body_response)?;
        let abort_io_abort_response =
            self.word_entry(b"abort.io-abort-response", report.abort_io_abort_response)?;
        let abort_frame_control_response = self.word_entry(
            b"abort.frame-control-response",
            report.abort_frame_control_response,
        )?;
        let post_io_enable =
            self.word_entry(b"post.CCCR.IO_ENABLE", report.post_io_enable as u32)?;
        let post_io_ready = self.word_entry(b"post.CCCR.IO_READY", report.post_io_ready as u32)?;
        let post_interrupt_pending =
            self.word_entry(b"post.CCCR.INTPEND", report.post_interrupt_pending as u32)?;
        let post_io_enable_response = self.word_entry(
            b"post.CCCR.IO_ENABLE.response",
            report.post_io_enable_response,
        )?;
        let post_io_ready_response = self.word_entry(
            b"post.CCCR.IO_READY.response",
            report.post_io_ready_response,
        )?;
        let post_interrupt_pending_response = self.word_entry(
            b"post.CCCR.INTPEND.response",
            report.post_interrupt_pending_response,
        )?;
        let post_host_normal_int =
            self.word_entry(b"post.HOST.NORM_INT", report.post_host_normal_int as u32)?;
        let post_host_error_int =
            self.word_entry(b"post.HOST.ERR_INT", report.post_host_error_int as u32)?;
        let frame_last_error =
            self.wifi_sdio_error_entry(b"frame.last-error", report.frame_last_error)?;
        let abort_last_error =
            self.wifi_sdio_error_entry(b"abort.last-error", report.abort_last_error)?;
        let post_last_error =
            self.wifi_sdio_error_entry(b"post.last-error", report.post_last_error)?;
        let entries = [
            frame_status,
            frame_valid,
            frame_length,
            frame_channel,
            frame_bus_data_credit,
            frame_header_response,
            frame_body_response,
            abort_io_abort_response,
            abort_frame_control_response,
            post_io_enable,
            post_io_ready,
            post_interrupt_pending,
            post_io_enable_response,
            post_io_ready_response,
            post_interrupt_pending_response,
            post_host_normal_int,
            post_host_error_int,
            frame_last_error,
            abort_last_error,
            post_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_poll_read_frame_report(
        &mut self,
        report: WifiSdioPollReadFrameReport,
    ) -> LispResult<Value> {
        let ack_status = self.symbol_entry(b"ack.status", report.ack_status)?;
        let ack_int_status_before =
            self.word_entry(b"ack.INT_STATUS.before", report.ack_int_status_before)?;
        let ack_clear_value = self.word_entry(b"ack.INT_STATUS.clear", report.ack_clear_value)?;
        let ack_int_status_after =
            self.word_entry(b"ack.INT_STATUS.after", report.ack_int_status_after)?;
        let ack_final_response =
            self.word_entry(b"ack.INT_STATUS.final-response", report.ack_final_response)?;
        let frame_status = self.symbol_entry(b"frame.status", report.frame_status)?;
        let frame_valid = self.bool_entry(b"frame.valid", report.frame_valid)?;
        let frame_length = self.word_entry(b"frame.length", report.frame_length as u32)?;
        let frame_channel = self.word_entry(b"frame.channel", report.frame_channel as u32)?;
        let frame_bus_data_credit = self.word_entry(
            b"frame.bus-data-credit",
            report.frame_bus_data_credit as u32,
        )?;
        let frame_header_response =
            self.word_entry(b"frame.header-response", report.frame_header_response)?;
        let frame_body_response =
            self.word_entry(b"frame.body-response", report.frame_body_response)?;
        let post_status = self.symbol_entry(b"post.status", report.post_status)?;
        let post_io_enable =
            self.word_entry(b"post.CCCR.IO_ENABLE", report.post_io_enable as u32)?;
        let post_io_ready = self.word_entry(b"post.CCCR.IO_READY", report.post_io_ready as u32)?;
        let post_interrupt_pending =
            self.word_entry(b"post.CCCR.INTPEND", report.post_interrupt_pending as u32)?;
        let post_io_enable_response = self.word_entry(
            b"post.CCCR.IO_ENABLE.response",
            report.post_io_enable_response,
        )?;
        let post_io_ready_response = self.word_entry(
            b"post.CCCR.IO_READY.response",
            report.post_io_ready_response,
        )?;
        let post_interrupt_pending_response = self.word_entry(
            b"post.CCCR.INTPEND.response",
            report.post_interrupt_pending_response,
        )?;
        let post_host_normal_int =
            self.word_entry(b"post.HOST.NORM_INT", report.post_host_normal_int as u32)?;
        let post_host_error_int =
            self.word_entry(b"post.HOST.ERR_INT", report.post_host_error_int as u32)?;
        let ack_last_error =
            self.wifi_sdio_error_entry(b"ack.last-error", report.ack_last_error)?;
        let frame_last_error =
            self.wifi_sdio_error_entry(b"frame.last-error", report.frame_last_error)?;
        let post_last_error =
            self.wifi_sdio_error_entry(b"post.last-error", report.post_last_error)?;
        let entries = [
            ack_status,
            ack_int_status_before,
            ack_clear_value,
            ack_int_status_after,
            ack_final_response,
            frame_status,
            frame_valid,
            frame_length,
            frame_channel,
            frame_bus_data_credit,
            frame_header_response,
            frame_body_response,
            post_status,
            post_io_enable,
            post_io_ready,
            post_interrupt_pending,
            post_io_enable_response,
            post_io_ready_response,
            post_interrupt_pending_response,
            post_host_normal_int,
            post_host_error_int,
            ack_last_error,
            frame_last_error,
            post_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_interrupt_ack_report(
        &mut self,
        report: WifiSdioInterruptAckReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let int_status_before = self.word_entry(b"INT_STATUS.before", report.int_status_before)?;
        let mailbox_data = self.word_entry(b"TO_HOST_MAILBOX_DATA", report.mailbox_data)?;
        let mailbox_ack_value = self.word_entry(b"TO_SB_MAILBOX.ack", report.mailbox_ack_value)?;
        let clear_value = self.word_entry(b"INT_STATUS.clear", report.clear_value)?;
        let int_status_after = self.word_entry(b"INT_STATUS.after", report.int_status_after)?;
        let host_normal_int_before = self.word_entry(
            b"HOST.NORM_INT.before",
            report.host_normal_int_before as u32,
        )?;
        let host_error_int_before =
            self.word_entry(b"HOST.ERR_INT.before", report.host_error_int_before as u32)?;
        let host_normal_int_after =
            self.word_entry(b"HOST.NORM_INT.after", report.host_normal_int_after as u32)?;
        let host_error_int_after =
            self.word_entry(b"HOST.ERR_INT.after", report.host_error_int_after as u32)?;
        let int_status_response =
            self.word_entry(b"INT_STATUS.response", report.int_status_response)?;
        let mailbox_response =
            self.word_entry(b"TO_HOST_MAILBOX_DATA.response", report.mailbox_response)?;
        let mailbox_ack_response =
            self.word_entry(b"TO_SB_MAILBOX.response", report.mailbox_ack_response)?;
        let mailbox_ack_readback =
            self.word_entry(b"TO_SB_MAILBOX.readback", report.mailbox_ack_readback)?;
        let clear_response =
            self.word_entry(b"INT_STATUS.clear-response", report.clear_response)?;
        let clear_readback =
            self.word_entry(b"INT_STATUS.clear-readback", report.clear_readback)?;
        let final_response =
            self.word_entry(b"INT_STATUS.final-response", report.final_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            int_status_before,
            mailbox_data,
            mailbox_ack_value,
            clear_value,
            int_status_after,
            host_normal_int_before,
            host_error_int_before,
            host_normal_int_after,
            host_error_int_after,
            int_status_response,
            mailbox_response,
            mailbox_ack_response,
            mailbox_ack_readback,
            clear_response,
            clear_readback,
            final_response,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_interrupt_state_report(
        &mut self,
        report: WifiSdioInterruptStateReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let io_enable = self.word_entry(b"CCCR.IO_ENABLE", report.io_enable as u32)?;
        let io_ready = self.word_entry(b"CCCR.IO_READY", report.io_ready as u32)?;
        let interrupt_enable =
            self.word_entry(b"CCCR.INTERRUPT_ENABLE", report.interrupt_enable as u32)?;
        let interrupt_pending =
            self.word_entry(b"CCCR.INTERRUPT_PENDING", report.interrupt_pending as u32)?;
        let bus_control = self.word_entry(b"CCCR.BUS_CONTROL", report.bus_control as u32)?;
        let master_enabled = self.bool_entry(b"master-enabled", report.master_enabled)?;
        let function1_enabled = self.bool_entry(b"function1-enabled", report.function1_enabled)?;
        let function2_enabled = self.bool_entry(b"function2-enabled", report.function2_enabled)?;
        let function1_ready = self.bool_entry(b"function1-ready", report.function1_ready)?;
        let function2_ready = self.bool_entry(b"function2-ready", report.function2_ready)?;
        let function1_pending = self.bool_entry(b"function1-pending", report.function1_pending)?;
        let function2_pending = self.bool_entry(b"function2-pending", report.function2_pending)?;
        let host_card_interrupt =
            self.bool_entry(b"host-card-interrupt", report.host_card_interrupt)?;
        let io_enable_response =
            self.word_entry(b"CCCR.IO_ENABLE.response", report.io_enable_response)?;
        let io_ready_response =
            self.word_entry(b"CCCR.IO_READY.response", report.io_ready_response)?;
        let interrupt_enable_response = self.word_entry(
            b"CCCR.INTERRUPT_ENABLE.response",
            report.interrupt_enable_response,
        )?;
        let interrupt_pending_response = self.word_entry(
            b"CCCR.INTERRUPT_PENDING.response",
            report.interrupt_pending_response,
        )?;
        let bus_control_response =
            self.word_entry(b"CCCR.BUS_CONTROL.response", report.bus_control_response)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            io_enable,
            io_ready,
            interrupt_enable,
            interrupt_pending,
            bus_control,
            master_enabled,
            function1_enabled,
            function2_enabled,
            function1_ready,
            function2_ready,
            function1_pending,
            function2_pending,
            host_card_interrupt,
            io_enable_response,
            io_ready_response,
            interrupt_enable_response,
            interrupt_pending_response,
            bus_control_response,
            host_normal_int,
            host_error_int,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_keep_awake_report(
        &mut self,
        report: WifiSdioKeepAwakeReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let attempts = self.word_entry(b"attempts", report.attempts as u32)?;
        let write_value = self.word_entry(b"write-value", report.write_value as u32)?;
        let first_write_response =
            self.word_entry(b"first-write-response", report.first_write_response)?;
        let second_write_response =
            self.word_entry(b"second-write-response", report.second_write_response)?;
        let retry_write_response =
            self.word_entry(b"retry-write-response", report.retry_write_response)?;
        let read_value = self.word_entry(b"read-value", report.read_value as u32)?;
        let read_response = self.word_entry(b"read-response", report.read_response)?;
        let keep_wl_kso = self.bool_entry(b"keep-wl-kso", report.keep_wl_kso)?;
        let wl_devon = self.bool_entry(b"wl-devon", report.wl_devon)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            attempts,
            write_value,
            first_write_response,
            second_write_response,
            retry_write_response,
            read_value,
            read_response,
            keep_wl_kso,
            wl_devon,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_ht_request_report(
        &mut self,
        report: WifiSdioHtRequestReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let attempts = self.word_entry(b"attempts", report.attempts as u32)?;
        let write_value = self.word_entry(b"write-value", report.write_value as u32)?;
        let write_response = self.word_entry(b"write-response", report.write_response)?;
        let read_value = self.word_entry(b"read-value", report.read_value as u32)?;
        let read_response = self.word_entry(b"read-response", report.read_response)?;
        let ht_available = self.bool_entry(b"ht-available", report.ht_available)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            attempts,
            write_value,
            write_response,
            read_value,
            read_response,
            ht_available,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_host_reset_report(
        &mut self,
        report: WifiSdioHostResetReport,
    ) -> LispResult<Value> {
        let command_reset = self.bool_entry(b"command-reset", report.command_reset)?;
        let data_reset = self.bool_entry(b"data-reset", report.data_reset)?;
        let before = self.wifi_sdio_host_report(report.before)?;
        let before = self.entry(b"before", before)?;
        let after = self.wifi_sdio_host_report(report.after)?;
        let after = self.entry(b"after", after)?;
        let entries = [command_reset, data_reset, before, after];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_abort_read_report(
        &mut self,
        report: WifiSdioAbortReadReport,
    ) -> LispResult<Value> {
        let io_abort_response = self.word_entry(b"io-abort-response", report.io_abort_response)?;
        let frame_control_response =
            self.word_entry(b"frame-control-response", report.frame_control_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [io_abort_response, frame_control_response, last_error, host];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_core_state_report(
        &mut self,
        report: WifiSdioCoreStateReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let base = self.word_entry(b"base", report.base)?;
        let ioctrl = self.word_entry(b"IOCTRL", report.ioctrl as u32)?;
        let resetctrl = self.word_entry(b"RESETCTRL", report.resetctrl as u32)?;
        let resetstatus = self.word_entry(b"RESETSTATUS", report.resetstatus as u32)?;
        let clock_enabled = self.bool_entry(b"clock-enabled", report.clock_enabled)?;
        let force_gated = self.bool_entry(b"force-gated", report.force_gated)?;
        let in_reset = self.bool_entry(b"in-reset", report.in_reset)?;
        let reset_busy = self.bool_entry(b"reset-busy", report.reset_busy)?;
        let core_up = self.bool_entry(b"core-up", report.core_up)?;
        let last_response = self.word_entry(b"last-response", report.last_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            setup_status,
            base,
            ioctrl,
            resetctrl,
            resetstatus,
            clock_enabled,
            force_gated,
            in_reset,
            reset_busy,
            core_up,
            last_response,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_core_snapshot_report(
        &mut self,
        report: WifiSdioCoreSnapshotReport,
    ) -> LispResult<Value> {
        let ioctrl = self.word_entry(b"IOCTRL", report.ioctrl as u32)?;
        let resetctrl = self.word_entry(b"RESETCTRL", report.resetctrl as u32)?;
        let resetstatus = self.word_entry(b"RESETSTATUS", report.resetstatus as u32)?;
        let clock_enabled = self.bool_entry(b"clock-enabled", report.clock_enabled)?;
        let force_gated = self.bool_entry(b"force-gated", report.force_gated)?;
        let in_reset = self.bool_entry(b"in-reset", report.in_reset)?;
        let reset_busy = self.bool_entry(b"reset-busy", report.reset_busy)?;
        let core_up = self.bool_entry(b"core-up", report.core_up)?;
        let entries = [
            ioctrl,
            resetctrl,
            resetstatus,
            clock_enabled,
            force_gated,
            in_reset,
            reset_busy,
            core_up,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_core_reset_report(
        &mut self,
        report: WifiSdioCoreResetReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let setup_status = self.symbol_entry(b"setup-status", report.setup_status)?;
        let base = self.word_entry(b"base", report.base)?;
        let before = self.wifi_sdio_core_snapshot_report(report.before)?;
        let before = self.entry(b"before", before)?;
        let after = self.wifi_sdio_core_snapshot_report(report.after)?;
        let after = self.entry(b"after", after)?;
        let last_response = self.word_entry(b"last-response", report.last_response)?;
        let last_error = self.wifi_sdio_error_entry(b"last-error", report.last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            setup_status,
            base,
            before,
            after,
            last_response,
            last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_host_report(&mut self, report: WifiSdioHostReport) -> LispResult<Value> {
        let wrap_ctl = self.word_entry(b"WRAP.CTL", report.wrap_ctl)?;
        let gp_out = self.word_entry(b"GP_OUT", report.gp_out)?;
        let gp_in = self.word_entry(b"GP_IN", report.gp_in)?;
        let xfer_mode = self.word_entry(b"XFER_MODE", report.xfer_mode as u32)?;
        let block_size = self.word_entry(b"BLOCKSIZE", report.block_size as u32)?;
        let block_count = self.word_entry(b"BLOCKCOUNT", report.block_count as u32)?;
        let sdmasa = self.word_entry(b"SDMASA", report.sdmasa)?;
        let adma_sa_low = self.word_entry(b"ADMA_SA_LOW", report.adma_sa_low)?;
        let adma_id_low = self.word_entry(b"ADMA_ID_LOW", report.adma_id_low)?;
        let adma_err_stat = self.word_entry(b"ADMA_ERR_STAT", report.adma_err_stat as u32)?;
        let bgap_ctrl = self.word_entry(b"BGAP_CTRL", report.bgap_ctrl as u32)?;
        let host_ctrl1 = self.word_entry(b"HOST_CTRL1", report.host_ctrl1 as u32)?;
        let host_ctrl2 = self.word_entry(b"HOST_CTRL2", report.host_ctrl2 as u32)?;
        let capabilities1 = self.word_entry(b"CAPABILITIES1", report.capabilities1)?;
        let capabilities2 = self.word_entry(b"CAPABILITIES2", report.capabilities2)?;
        let mbiu_ctrl = self.word_entry(b"MBIU_CTRL", report.mbiu_ctrl as u32)?;
        let tout_ctrl = self.word_entry(b"TOUT_CTRL", report.tout_ctrl as u32)?;
        let clk_ctrl = self.word_entry(b"CLK_CTRL", report.clk_ctrl as u32)?;
        let pwr_ctrl = self.word_entry(b"PWR_CTRL", report.pwr_ctrl as u32)?;
        let sw_rst = self.word_entry(b"SW_RST", report.sw_rst as u32)?;
        let normal_int = self.word_entry(b"NORM_INT", report.normal_int as u32)?;
        let error_int = self.word_entry(b"ERR_INT", report.error_int as u32)?;
        let normal_int_stat_en =
            self.word_entry(b"NORM_INT_STAT_EN", report.normal_int_stat_en as u32)?;
        let error_int_stat_en =
            self.word_entry(b"ERR_INT_STAT_EN", report.error_int_stat_en as u32)?;
        let normal_int_signal_en =
            self.word_entry(b"NORM_INT_SIGNAL_EN", report.normal_int_signal_en as u32)?;
        let error_int_signal_en =
            self.word_entry(b"ERR_INT_SIGNAL_EN", report.error_int_signal_en as u32)?;
        let pstate = self.word_entry(b"PSTATE", report.pstate)?;
        let cmd = self.word_entry(b"CMD_R", report.cmd as u32)?;
        let argument = self.word_entry(b"ARGUMENT", report.argument)?;
        let response01 = self.word_entry(b"RESP01", report.response01)?;
        let response23 = self.word_entry(b"RESP23", report.response23)?;
        let response45 = self.word_entry(b"RESP45", report.response45)?;
        let response67 = self.word_entry(b"RESP67", report.response67)?;
        let entries = [
            wrap_ctl,
            gp_out,
            gp_in,
            xfer_mode,
            block_size,
            block_count,
            sdmasa,
            adma_sa_low,
            adma_id_low,
            adma_err_stat,
            bgap_ctrl,
            host_ctrl1,
            host_ctrl2,
            capabilities1,
            capabilities2,
            mbiu_ctrl,
            tout_ctrl,
            clk_ctrl,
            pwr_ctrl,
            sw_rst,
            normal_int,
            error_int,
            normal_int_stat_en,
            error_int_stat_en,
            normal_int_signal_en,
            error_int_signal_en,
            pstate,
            cmd,
            argument,
            response01,
            response23,
            response45,
            response67,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_pins_report(&mut self, report: WifiSdioPinsReport) -> LispResult<Value> {
        let p2_sel0 = self.word_entry(b"P2.SEL0", report.p2_sel0)?;
        let p2_sel1 = self.word_entry(b"P2.SEL1", report.p2_sel1)?;
        let p2_cfg = self.word_entry(b"P2.CFG", report.p2_cfg)?;
        let p2_out = self.word_entry(b"P2.OUT", report.p2_out)?;
        let p2_in = self.word_entry(b"P2.IN", report.p2_in)?;
        let entries = [p2_sel0, p2_sel1, p2_cfg, p2_out, p2_in];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_clock_report(&mut self, report: WifiSdioClockReport) -> LispResult<Value> {
        let path0 = self.word_entry(b"CLK_PATH0", report.path0)?;
        let root0 = self.word_entry(b"CLK_HF0", report.root0)?;
        let root1 = self.word_entry(b"CLK_HF1", report.root1)?;
        let root2 = self.word_entry(b"CLK_HF2", report.root2)?;
        let root3 = self.word_entry(b"CLK_HF3", report.root3)?;
        let root4 = self.word_entry(b"CLK_HF4", report.root4)?;
        let fll_config = self.word_entry(b"FLL_CONFIG", report.fll_config)?;
        let fll_config2 = self.word_entry(b"FLL_CONFIG2", report.fll_config2)?;
        let fll_status = self.word_entry(b"FLL_STATUS", report.fll_status)?;
        let entries = [
            path0,
            root0,
            root1,
            root2,
            root3,
            root4,
            fll_config,
            fll_config2,
            fll_status,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_error_entry(
        &mut self,
        name: &[u8],
        report: Option<WifiSdioCommandErrorReport>,
    ) -> LispResult<Value> {
        match report {
            Some(error) => {
                let code = self.symbol_entry(b"code", error.code)?;
                let normal_int = self.word_entry(b"NORM_INT", error.normal_int as u32)?;
                let error_int = self.word_entry(b"ERR_INT", error.error_int as u32)?;
                let pstate = self.word_entry(b"PSTATE", error.pstate)?;
                let command = self.word_entry(b"CMD_R", error.command as u32)?;
                let argument = self.word_entry(b"ARGUMENT", error.argument)?;
                let pstate_after_write =
                    self.word_entry(b"PSTATE.after-write", error.pstate_after_write)?;
                let normal_int_after_write =
                    self.word_entry(b"NORM_INT.after-write", error.normal_int_after_write as u32)?;
                let error_int_after_write =
                    self.word_entry(b"ERR_INT.after-write", error.error_int_after_write as u32)?;
                let entries = [
                    code,
                    normal_int,
                    error_int,
                    pstate,
                    command,
                    argument,
                    pstate_after_write,
                    normal_int_after_write,
                    error_int_after_write,
                ];
                let details = self.make_list_from_values(&entries)?;
                self.entry(name, details)
            }
            None => self.entry(name, Value::Nil),
        }
    }

    fn sd_word_list_entry(&mut self, name: &[u8], words: &[u32]) -> LispResult<Value> {
        let mut values = [Value::Nil; 8];
        for (index, word) in words.iter().enumerate() {
            values[index] = Value::Word(*word);
        }
        let list = self.make_list_from_values(&values)?;
        self.entry(name, list)
    }

    fn wifi_byte_list_entry(
        &mut self,
        name: &[u8],
        bytes: &[u8; 64],
        count: u8,
    ) -> LispResult<Value> {
        let mut values = [Value::Nil; 64];
        let mut index = 0usize;
        while index < count as usize && index < values.len() {
            values[index] = Value::Word(bytes[index] as u32);
            index += 1;
        }
        let list = self.make_list_from_values(&values[..index])?;
        self.entry(name, list)
    }

    fn sd_error_code_entry(
        &mut self,
        name: &[u8],
        report: Option<SdCommandErrorReport>,
    ) -> LispResult<Value> {
        match report {
            Some(error) => self.symbol_entry(name, error.code),
            None => self.entry(name, Value::Nil),
        }
    }

    fn sd_error_word_entry(&mut self, name: &[u8], value: Option<u32>) -> LispResult<Value> {
        match value {
            Some(value) => self.word_entry(name, value),
            None => self.entry(name, Value::Nil),
        }
    }

    fn is_pair(&self, value: Value) -> bool {
        match value {
            Value::Object(id) => matches!(self.object_kind_by_id(id), Ok(ObjectKind::Pair { .. })),
            _ => false,
        }
    }

    fn car(&self, value: Value) -> LispResult<Value> {
        match self.object_kind(value)? {
            ObjectKind::Pair { car, .. } => Ok(car),
            _ => Err(Error::new("expected pair")),
        }
    }

    fn cdr(&self, value: Value) -> LispResult<Value> {
        match self.object_kind(value)? {
            ObjectKind::Pair { cdr, .. } => Ok(cdr),
            _ => Err(Error::new("expected pair")),
        }
    }

    fn set_cdr(&mut self, pair: Value, cdr: Value) -> LispResult<()> {
        let id = match pair {
            Value::Object(id) => id,
            _ => return Err(Error::new("expected pair")),
        };
        let index = id as usize;
        let car = match self.object_kind_by_id(id)? {
            ObjectKind::Pair { car, .. } => car,
            _ => return Err(Error::new("expected pair")),
        };
        self.objects[index].kind = ObjectKind::Pair { car, cdr };
        Ok(())
    }

    fn object_kind(&self, value: Value) -> LispResult<ObjectKind> {
        match value {
            Value::Object(id) => self.object_kind_by_id(id),
            _ => Err(Error::new("expected heap object")),
        }
    }

    fn object_kind_by_id(&self, id: ObjectId) -> LispResult<ObjectKind> {
        let index = id as usize;
        if index >= MAX_OBJECTS {
            return Err(Error::new("invalid object"));
        }
        match self.objects[index].kind {
            ObjectKind::Free => Err(Error::new("stale object")),
            kind => Ok(kind),
        }
    }

    fn require_pair(&self, value: Value) -> LispResult<(Value, Value)> {
        match self.object_kind(value)? {
            ObjectKind::Pair { car, cdr } => Ok((car, cdr)),
            _ => Err(Error::new("expected proper list")),
        }
    }

    fn list_next(&self, cursor: Value) -> LispResult<Option<(Value, Value)>> {
        if cursor == Value::Nil {
            return Ok(None);
        }
        match self.object_kind(cursor)? {
            ObjectKind::Pair { car, cdr } => Ok(Some((car, cdr))),
            _ => Err(Error::new("expected proper list")),
        }
    }

    fn heap_counts(&self) -> HeapCounts {
        let mut free = 0usize;
        let mut index = 0usize;
        while index < MAX_OBJECTS {
            if matches!(self.objects[index].kind, ObjectKind::Free) {
                free += 1;
            }
            index += 1;
        }
        HeapCounts {
            used: MAX_OBJECTS - free,
            free,
            total: MAX_OBJECTS,
        }
    }

    fn collect_garbage(&mut self) -> usize {
        self.collect_garbage_from(Value::Nil)
    }

    fn collect_garbage_from(&mut self, env: Value) -> usize {
        let mut index = 0usize;
        while index < MAX_GLOBALS {
            let binding = self.globals[index];
            if binding.occupied {
                self.mark_value(binding.value);
            }
            index += 1;
        }

        self.mark_value(self.active_expression);
        self.mark_value(env);

        let mut freed = 0usize;
        index = 0;
        while index < MAX_OBJECTS {
            if matches!(self.objects[index].kind, ObjectKind::Free) {
                index += 1;
                continue;
            }

            if self.objects[index].marked {
                self.objects[index].marked = false;
            } else {
                self.objects[index] = Object {
                    marked: false,
                    next_free: self.free_head,
                    kind: ObjectKind::Free,
                };
                self.free_head = Some(index as ObjectId);
                freed += 1;
            }

            index += 1;
        }

        self.collections = self.collections.wrapping_add(1);
        freed
    }

    fn mark_value(&mut self, value: Value) {
        let id = match value {
            Value::Object(id) => id,
            _ => return,
        };

        let index = id as usize;
        if index >= MAX_OBJECTS || self.objects[index].marked {
            return;
        }

        self.objects[index].marked = true;
        match self.objects[index].kind {
            ObjectKind::Pair { car, cdr } => {
                self.mark_value(car);
                self.mark_value(cdr);
            }
            ObjectKind::Closure { params, body, env } => {
                self.mark_value(params);
                self.mark_value(body);
                self.mark_value(env);
            }
            ObjectKind::Env { value, next, .. } => {
                self.mark_value(value);
                self.mark_value(next);
            }
            ObjectKind::String { .. } => {}
            ObjectKind::Free => {}
        }
    }

    fn write_value<W: Write>(&self, value: Value, output: &mut W) -> fmt::Result {
        match value {
            Value::Nil => output.write_str("nil"),
            Value::Bool(true) => output.write_str("#t"),
            Value::Bool(false) => output.write_str("#f"),
            Value::Int(value) => write!(output, "{}", value),
            Value::Word(value) => write!(output, "#x{:08x}", value),
            Value::Symbol(symbol) => self.write_symbol(symbol, output),
            Value::Primitive(primitive) => write!(output, "<primitive {}>", primitive.name()),
            Value::Object(id) => match self.object_kind_by_id(id) {
                Ok(ObjectKind::Pair { .. }) => self.write_pair(id, output),
                Ok(ObjectKind::Closure { .. }) => output.write_str("<lambda>"),
                Ok(ObjectKind::Env { .. }) => output.write_str("<env>"),
                Ok(ObjectKind::String { len, bytes }) => self.write_string(len, &bytes, output),
                _ => output.write_str("<stale>"),
            },
        }
    }

    fn write_repl_value<W: Write>(&self, value: Value, output: &mut W) -> fmt::Result {
        if self.should_pretty_print(value) {
            self.write_pretty_value(value, output, 0)
        } else {
            self.write_value(value, output)
        }
    }

    fn should_pretty_print(&self, value: Value) -> bool {
        match value {
            Value::Object(id) if self.is_pair(Value::Object(id)) => self.list_needs_pretty(value),
            _ => false,
        }
    }

    fn list_needs_pretty(&self, mut cursor: Value) -> bool {
        let mut count = 0u8;
        loop {
            match cursor {
                Value::Nil => return false,
                Value::Object(id) => match self.object_kind_by_id(id) {
                    Ok(ObjectKind::Pair { car, cdr }) => {
                        count = count.saturating_add(1);
                        if count > PRETTY_INLINE_LIST_LIMIT || self.is_pair(car) {
                            return true;
                        }
                        cursor = cdr;
                    }
                    _ => return false,
                },
                _ => return count > 0,
            }
        }
    }

    fn write_pretty_value<W: Write>(
        &self,
        value: Value,
        output: &mut W,
        indent: usize,
    ) -> fmt::Result {
        match value {
            Value::Object(id) => match self.object_kind_by_id(id) {
                Ok(ObjectKind::Pair { .. }) => self.write_pretty_pair(id, output, indent),
                _ => self.write_value(value, output),
            },
            _ => self.write_value(value, output),
        }
    }

    fn write_pretty_pair<W: Write>(
        &self,
        id: ObjectId,
        output: &mut W,
        indent: usize,
    ) -> fmt::Result {
        output.write_char('(')?;
        let mut cursor = Value::Object(id);
        let mut first = true;

        loop {
            match self.object_kind(cursor) {
                Ok(ObjectKind::Pair { car, cdr }) => {
                    if !first {
                        output.write_char('\n')?;
                        write_indent(output, indent + PRETTY_INDENT)?;
                    }
                    self.write_pretty_value(car, output, indent + PRETTY_INDENT)?;
                    first = false;

                    match cdr {
                        Value::Nil => {
                            output.write_char(')')?;
                            return Ok(());
                        }
                        Value::Object(next_id)
                            if matches!(
                                self.object_kind_by_id(next_id),
                                Ok(ObjectKind::Pair { .. })
                            ) =>
                        {
                            cursor = cdr;
                        }
                        value => {
                            output.write_str(" . ")?;
                            self.write_pretty_value(value, output, indent + PRETTY_INDENT)?;
                            output.write_char(')')?;
                            return Ok(());
                        }
                    }
                }
                _ => {
                    output.write_str("<bad-list>)")?;
                    return Ok(());
                }
            }
        }
    }

    fn write_pair<W: Write>(&self, id: ObjectId, output: &mut W) -> fmt::Result {
        output.write_char('(')?;
        let mut cursor = Value::Object(id);
        let mut first = true;

        loop {
            match self.object_kind(cursor) {
                Ok(ObjectKind::Pair { car, cdr }) => {
                    if !first {
                        output.write_char(' ')?;
                    }
                    self.write_value(car, output)?;
                    first = false;

                    match cdr {
                        Value::Nil => {
                            output.write_char(')')?;
                            return Ok(());
                        }
                        Value::Object(next_id)
                            if matches!(
                                self.object_kind_by_id(next_id),
                                Ok(ObjectKind::Pair { .. })
                            ) =>
                        {
                            cursor = cdr;
                        }
                        value => {
                            output.write_str(" . ")?;
                            self.write_value(value, output)?;
                            output.write_char(')')?;
                            return Ok(());
                        }
                    }
                }
                _ => {
                    output.write_str("<bad-list>)")?;
                    return Ok(());
                }
            }
        }
    }

    fn write_symbol<W: Write>(&self, symbol: SymbolId, output: &mut W) -> fmt::Result {
        let index = symbol as usize;
        if index >= MAX_SYMBOLS || !self.symbols[index].occupied {
            return output.write_str("<bad-symbol>");
        }

        let entry = self.symbols[index];
        let mut byte_index = 0usize;
        while byte_index < entry.len as usize {
            output.write_char(entry.bytes[byte_index] as char)?;
            byte_index += 1;
        }
        Ok(())
    }

    fn write_string<W: Write>(
        &self,
        len: u8,
        bytes: &[u8; MAX_STRING_BYTES],
        output: &mut W,
    ) -> fmt::Result {
        output.write_char('"')?;
        let mut index = 0usize;
        while index < len as usize {
            match bytes[index] {
                b'"' => output.write_str("\\\"")?,
                b'\\' => output.write_str("\\\\")?,
                b'\n' => output.write_str("\\n")?,
                b'\r' => output.write_str("\\r")?,
                b'\t' => output.write_str("\\t")?,
                byte if (0x20..=0x7e).contains(&byte) => output.write_char(byte as char)?,
                byte => write!(output, "\\x{:02x}", byte)?,
            }
            index += 1;
        }
        output.write_char('"')
    }
}

fn write_ascii_bytes<W: Write>(output: &mut W, bytes: &[u8]) -> fmt::Result {
    let mut index = 0usize;
    while index < bytes.len() {
        output.write_char(bytes[index] as char)?;
        index += 1;
    }
    Ok(())
}

fn write_indent<W: Write>(output: &mut W, count: usize) -> fmt::Result {
    let mut index = 0usize;
    while index < count {
        output.write_char(' ')?;
        index += 1;
    }
    Ok(())
}

struct Reader<'a> {
    input: &'a [u8],
    position: usize,
}

impl Reader<'_> {
    fn read_expression(&mut self, machine: &mut Machine) -> LispResult<Value> {
        self.skip_ws();
        match self.peek() {
            Some(b'(') => {
                self.position += 1;
                self.read_list(machine)
            }
            Some(b')') => Err(Error::new("unexpected ')'")),
            Some(b'\'') => {
                self.position += 1;
                let quoted = self.read_expression(machine)?;
                let quote_tail = machine.alloc_pair(quoted, Value::Nil)?;
                machine.alloc_pair(Value::Symbol(machine.specials.quote), quote_tail)
            }
            Some(b'"') => self.read_string(machine),
            Some(_) => self.read_atom(machine),
            None => Err(Error::new("unexpected end of input")),
        }
    }

    fn read_list(&mut self, machine: &mut Machine) -> LispResult<Value> {
        let mut head = Value::Nil;
        let mut tail = Value::Nil;

        loop {
            self.skip_ws();
            match self.peek() {
                Some(b')') => {
                    self.position += 1;
                    return Ok(head);
                }
                Some(b'.') => {
                    self.position += 1;
                    if tail == Value::Nil {
                        return Err(Error::new("dot needs a list head"));
                    }
                    let cdr = self.read_expression(machine)?;
                    self.skip_ws();
                    self.expect(b')')?;
                    machine.set_cdr(tail, cdr)?;
                    return Ok(head);
                }
                Some(_) => {
                    let value = self.read_expression(machine)?;
                    let cell = machine.alloc_pair(value, Value::Nil)?;
                    if head == Value::Nil {
                        head = cell;
                    } else {
                        machine.set_cdr(tail, cell)?;
                    }
                    tail = cell;
                }
                None => return Err(Error::new("unterminated list")),
            }
        }
    }

    fn read_string(&mut self, machine: &mut Machine) -> LispResult<Value> {
        self.expect(b'"')?;

        let mut bytes = [0u8; MAX_STRING_BYTES];
        let mut len = 0usize;

        loop {
            let byte = match self.peek() {
                Some(byte) => byte,
                None => return Err(Error::new("unterminated string")),
            };
            self.position += 1;

            match byte {
                b'"' => return machine.alloc_string(&bytes[..len]),
                b'\\' => {
                    let escaped = match self.peek() {
                        Some(b'"') => b'"',
                        Some(b'\\') => b'\\',
                        Some(b'n') => b'\n',
                        Some(b'r') => b'\r',
                        Some(b't') => b'\t',
                        Some(_) => return Err(Error::new("invalid string escape")),
                        None => return Err(Error::new("unterminated string")),
                    };
                    self.position += 1;
                    push_string_byte(&mut bytes, &mut len, escaped)?;
                }
                b'\r' | b'\n' => return Err(Error::new("unterminated string")),
                byte => push_string_byte(&mut bytes, &mut len, byte)?,
            }
        }
    }

    fn read_atom(&mut self, machine: &mut Machine) -> LispResult<Value> {
        let start = self.position;
        while let Some(byte) = self.peek() {
            if is_delimiter(byte) {
                break;
            }
            self.position += 1;
        }

        let token = &self.input[start..self.position];
        if token == b"nil" {
            return Ok(Value::Nil);
        }
        if token == b"#t" {
            return Ok(Value::Bool(true));
        }
        if token == b"#f" {
            return Ok(Value::Bool(false));
        }
        if let Some(value) = parse_decimal(token)? {
            return Ok(Value::Int(value));
        }
        if let Some(value) = parse_hex(token)? {
            return Ok(Value::Word(value));
        }

        Ok(Value::Symbol(machine.intern(token)?))
    }

    fn skip_ws(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.position += 1;
            }

            if self.peek() == Some(b';') {
                while let Some(byte) = self.peek() {
                    self.position += 1;
                    if byte == b'\n' {
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn expect(&mut self, byte: u8) -> LispResult<()> {
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(Error::new("unexpected syntax"))
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    fn is_done(&self) -> bool {
        self.position >= self.input.len()
    }
}

fn push_string_byte(
    bytes: &mut [u8; MAX_STRING_BYTES],
    len: &mut usize,
    byte: u8,
) -> LispResult<()> {
    if *len >= MAX_STRING_BYTES {
        return Err(Error::new("string too long"));
    }

    bytes[*len] = byte;
    *len += 1;
    Ok(())
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t' | b'\r' | b'\n' | b'(' | b')' | b'\'' | b';'
    )
}

fn parse_decimal(token: &[u8]) -> LispResult<Option<i32>> {
    if token.is_empty() {
        return Ok(None);
    }

    let mut index = 0usize;
    let mut negative = false;
    if token[0] == b'-' {
        negative = true;
        index = 1;
        if index == token.len() {
            return Ok(None);
        }
    }

    let mut value = 0i64;
    while index < token.len() {
        let byte = token[index];
        if !byte.is_ascii_digit() {
            return Ok(None);
        }
        value = value * 10 + (byte - b'0') as i64;
        index += 1;
    }

    if negative {
        value = -value;
    }

    if value < i32::MIN as i64 || value > i32::MAX as i64 {
        return Err(Error::new("integer overflow"));
    }

    Ok(Some(value as i32))
}

fn parse_hex(token: &[u8]) -> LispResult<Option<u32>> {
    if token.len() < 3 || token[0] != b'#' || !matches!(token[1], b'x' | b'X') {
        return Ok(None);
    }

    let mut value = 0u32;
    let mut index = 2usize;
    while index < token.len() {
        let digit = hex_digit(token[index]).ok_or(Error::new("invalid hex integer"))?;
        value = value
            .checked_mul(16)
            .and_then(|value| value.checked_add(digit as u32))
            .ok_or(Error::new("integer overflow"))?;
        index += 1;
    }

    Ok(Some(value))
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + byte - b'a'),
        b'A'..=b'F' => Some(10 + byte - b'A'),
        _ => None,
    }
}
