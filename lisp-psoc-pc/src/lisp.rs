use core::fmt::{self, Write};

use crate::wifi_credentials;

const MAX_OBJECTS: usize = 448;
const MAX_SYMBOLS: usize = 512;
const MAX_GLOBALS: usize = 144;
const MAX_SYMBOL_BYTES: usize = 32;
pub const MAX_STRING_BYTES: usize = 96;
pub const MAX_FILE_BYTES: usize = 512;
pub const NET_REPL_REQUEST_PAYLOAD_BYTES: usize = MAX_STRING_BYTES;
pub const NET_REPL_RESPONSE_BYTES: usize = 1400;
pub const MAX_STORE_FILES: usize = 5;
const MAX_CALL_ARGS: usize = 16;
const MAX_EVAL_DEPTH: u8 = 128;
const PRETTY_INDENT: usize = 4;
const PRETTY_INLINE_LIST_LIMIT: u8 = 6;
const MAX_PROCESSES: usize = 6;

type ObjectId = u16;
type SymbolId = u16;

#[derive(Clone, Copy)]
pub struct StringBytes {
    pub len: u8,
    pub bytes: [u8; MAX_STRING_BYTES],
}

#[derive(Clone, Copy)]
pub struct FileBytes {
    pub len: u16,
    pub bytes: [u8; MAX_FILE_BYTES],
}

const EMPTY_STRING_BYTES: StringBytes = StringBytes {
    len: 0,
    bytes: [0; MAX_STRING_BYTES],
};

#[derive(Clone, Copy)]
pub struct NetReplResponseBytes {
    pub len: u16,
    pub truncated: bool,
    pub bytes: [u8; NET_REPL_RESPONSE_BYTES],
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

const STATUS_READY: &[u8] = b"ready";
const STATUS_NOT_RUN: &[u8] = b"not-run";
const STATUS_STEP_FAILED: &[u8] = b"step-failed";
const STATUS_TIMEOUT: &[u8] = b"timeout";
const STATUS_LISTEN_TIMEOUT: &[u8] = b"listen-timeout";
const STATUS_CONNECTED: &[u8] = b"connected";
const STATUS_ACK_ONLY: &[u8] = b"ack-only";
const STATUS_PEER_RESET: &[u8] = b"peer-reset";
const STATUS_PEER_CLOSED: &[u8] = b"peer-closed";
const STATUS_DUPLICATE: &[u8] = b"duplicate";
const STATUS_ACK: &[u8] = b"ack";
const STATUS_READ_ONLY_DENIED: &[u8] = b"read-only-denied";
const STATUS_LINE_PENDING: &[u8] = b"line-pending";
const STATUS_PROTOCOL: &[u8] = b"protocol";
const STEP_DONE: &[u8] = b"done";
const WIFI_NET_REPL_SERVICE_DEFAULT_POLL_FRAMES: u8 = 1;
const WIFI_TCP_REPL_SERVICE_DEFAULT_PORT: u16 = 2323;
const WIFI_TCP_REPL_SERVICE_DEFAULT_POLL_FRAMES: u8 = 1;
const TELNET_SE: u8 = 240;
const TELNET_SB: u8 = 250;
const TELNET_WILL: u8 = 251;
const TELNET_WONT: u8 = 252;
const TELNET_DO: u8 = 253;
const TELNET_DONT: u8 = 254;
const TELNET_IAC: u8 = 255;
const TELNET_AYT: u8 = 246;
const TELNET_EC: u8 = 247;
const TELNET_EL: u8 = 248;
const TELNET_STATE_DATA: u8 = 0;
const TELNET_STATE_IAC: u8 = 1;
const TELNET_STATE_OPTION: u8 = 2;
const TELNET_STATE_SUBNEGOTIATION: u8 = 3;
const TELNET_STATE_SUBNEGOTIATION_IAC: u8 = 4;
const TELNET_STATE_CR: u8 = 5;
const TELNET_PROMPT: &[u8] = b"lisp> ";
const TELNET_AYT_RESPONSE: &[u8] = b"[yes]\n";
const TCP_FLAG_FIN: u8 = 0x01;

#[derive(Clone, Copy)]
struct WifiNetReplCycleReport {
    status: &'static [u8],
    request: WifiNetReplRequestReport,
    reply: Option<WifiNetReplReplyReport>,
    eval_status: &'static [u8],
    response_length: u16,
    response_hash: u32,
    response_truncated: bool,
}

#[derive(Clone, Copy)]
struct WifiTcpReplCycleReport {
    status: &'static [u8],
    request: WifiSdioTcpReceiveReport,
    reply: Option<WifiSdioTcpReplReplyReport>,
    eval_status: &'static [u8],
    response_length: u16,
    response_hash: u32,
    response_truncated: bool,
}

#[derive(Clone, Copy)]
struct WifiNetReplServiceState {
    enabled: bool,
    poll_frames: u8,
    polls: u32,
    requests_handled: u32,
    acks_received: u32,
    last_status: &'static [u8],
    last_request_status: &'static [u8],
    last_reply_status: &'static [u8],
    last_eval_status: &'static [u8],
    last_sequence: u32,
    last_request_read_only: bool,
    last_response_length: u16,
    last_response_hash: u32,
    last_response_truncated: bool,
    last_ack_sequence: u32,
    last_ack_response_hash: u32,
}

#[derive(Clone, Copy)]
struct WifiTcpReplServiceState {
    enabled: bool,
    listen_port: u16,
    poll_frames: u8,
    polls: u32,
    requests_handled: u32,
    last_status: &'static [u8],
    last_request_status: &'static [u8],
    last_reply_status: &'static [u8],
    last_eval_status: &'static [u8],
    last_peer_ip_address: u32,
    last_peer_port: u16,
    last_payload_length: u16,
    last_payload_hash: u32,
    last_response_length: u16,
    last_response_hash: u32,
    last_response_truncated: bool,
    processing_telnet_request: bool,
    reset_peer_after_reply: bool,
    telnet_state: u8,
    telnet_command: u8,
    telnet_line: [u8; NET_REPL_REQUEST_PAYLOAD_BYTES],
    telnet_line_len: u8,
}

const EMPTY_WIFI_NET_REPL_SERVICE_STATE: WifiNetReplServiceState = WifiNetReplServiceState {
    enabled: false,
    poll_frames: WIFI_NET_REPL_SERVICE_DEFAULT_POLL_FRAMES,
    polls: 0,
    requests_handled: 0,
    acks_received: 0,
    last_status: STATUS_NOT_RUN,
    last_request_status: STATUS_NOT_RUN,
    last_reply_status: STATUS_NOT_RUN,
    last_eval_status: STATUS_NOT_RUN,
    last_sequence: 0,
    last_request_read_only: false,
    last_response_length: 0,
    last_response_hash: 0,
    last_response_truncated: false,
    last_ack_sequence: 0,
    last_ack_response_hash: 0,
};

const EMPTY_WIFI_TCP_REPL_SERVICE_STATE: WifiTcpReplServiceState = WifiTcpReplServiceState {
    enabled: false,
    listen_port: WIFI_TCP_REPL_SERVICE_DEFAULT_PORT,
    poll_frames: WIFI_TCP_REPL_SERVICE_DEFAULT_POLL_FRAMES,
    polls: 0,
    requests_handled: 0,
    last_status: STATUS_NOT_RUN,
    last_request_status: STATUS_NOT_RUN,
    last_reply_status: STATUS_NOT_RUN,
    last_eval_status: STATUS_NOT_RUN,
    last_peer_ip_address: 0,
    last_peer_port: 0,
    last_payload_length: 0,
    last_payload_hash: 0,
    last_response_length: 0,
    last_response_hash: 0,
    last_response_truncated: false,
    processing_telnet_request: false,
    reset_peer_after_reply: false,
    telnet_state: TELNET_STATE_DATA,
    telnet_command: 0,
    telnet_line: [0; NET_REPL_REQUEST_PAYLOAD_BYTES],
    telnet_line_len: 0,
};

#[derive(Clone, Copy)]
struct WifiNetReplResponseCache {
    valid: bool,
    source_ip_address: u32,
    source_mac_hash: u32,
    sequence: u32,
    read_only: bool,
    payload_hash: u32,
    response: NetReplResponseBytes,
}

const EMPTY_NET_REPL_RESPONSE_BYTES: NetReplResponseBytes = NetReplResponseBytes {
    len: 0,
    truncated: false,
    bytes: [0; NET_REPL_RESPONSE_BYTES],
};

const EMPTY_WIFI_NET_REPL_RESPONSE_CACHE: WifiNetReplResponseCache = WifiNetReplResponseCache {
    valid: false,
    source_ip_address: 0,
    source_mac_hash: 0,
    sequence: 0,
    read_only: false,
    payload_hash: 0,
    response: EMPTY_NET_REPL_RESPONSE_BYTES,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProcessState {
    Free,
    Ready,
    Sleeping,
    Done,
    Error,
    Killed,
}

#[derive(Clone, Copy)]
struct Process {
    pid: u32,
    state: ProcessState,
    body: Value,
    cursor: Value,
    env: Value,
    wake_ms: u32,
    last_value: Value,
    error: &'static str,
    steps: u32,
}

const EMPTY_PROCESS: Process = Process {
    pid: 0,
    state: ProcessState::Free,
    body: Value::Nil,
    cursor: Value::Nil,
    env: Value::Nil,
    wake_ms: 0,
    last_value: Value::Nil,
    error: "",
    steps: 0,
};

#[derive(Clone, Copy)]
struct ProcessPollReport {
    ran: u32,
    ready: u32,
    sleeping: u32,
    done: u32,
    error: u32,
    killed: u32,
}

enum ProcessControl {
    Yield,
    Sleep { duration_ms: u32 },
}

pub trait Board {
    fn led(&mut self, action: LedAction) -> bool;
    fn heartbeat(&mut self, enabled: bool) -> bool;
    fn button_pressed(&mut self, index: i32) -> Result<bool, Error>;
    fn millis(&mut self) -> u32;
    fn read32(&mut self, address: u32) -> Result<u32, Error>;
    fn write32(&mut self, address: u32, value: u32) -> Result<(), Error>;
    fn registers(&mut self) -> RegisterReport;
    fn pdm_status(&mut self) -> PdmStatusReport;
    fn thermistor_status(&mut self) -> ThermistorStatusReport;
    fn thermistor_read(&mut self) -> ThermistorReadReport;
    fn capsense_status(&mut self) -> CapsenseStatusReport;
    fn sd_status(&mut self) -> SdStatusReport;
    fn sd_pins(&mut self) -> SdPinsReport;
    fn sd_pinmux(&mut self) -> SdPinsReport;
    fn sd_clock(&mut self) -> SdClockReport;
    fn sd_init(&mut self) -> SdInitReport;
    fn sd_read(&mut self, sector: u32) -> SdReadReport;
    fn sd_write_fill(&mut self, sector: u32, fill_word: u32) -> SdWriteReport;
    fn format_store(&mut self) -> StoreFormatReport;
    fn save_file(&mut self, path: StringBytes, content: StringBytes) -> StoreWriteReport;
    fn save_file_bytes(&mut self, path: StringBytes, content: FileBytes) -> StoreWriteReport;
    fn append_file(&mut self, path: StringBytes, content: StringBytes) -> StoreWriteReport;
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
    fn wifi_link_status(&mut self) -> WifiSdioLinkStatusReport;
    fn wifi_dhcp_discover(&mut self) -> WifiSdioDhcpDiscoverReport;
    fn wifi_dhcp_acquire(&mut self) -> WifiSdioDhcpAcquireReport;
    fn wifi_lease_status(&mut self) -> WifiSdioLeaseStatusReport;
    fn wifi_arp_router(&mut self) -> WifiSdioArpRouterReport;
    fn wifi_dns_query(&mut self, name: StringBytes) -> WifiSdioDnsQueryReport;
    fn wifi_tcp_syn(&mut self, name: StringBytes, port: u16) -> WifiSdioTcpSynReport;
    fn wifi_tcp_syn_ip(&mut self, remote_ip_address: u32, port: u16) -> WifiSdioTcpSynReport;
    fn wifi_tcp_listen_once(&mut self, port: u16, poll_frames: u8) -> WifiSdioTcpListenReport;
    fn wifi_tcp_receive_once(&mut self, port: u16, poll_frames: u8) -> WifiSdioTcpReceiveReport;
    fn wifi_tcp_repl_poll(&mut self, port: u16, poll_frames: u8) -> WifiSdioTcpReceiveReport;
    fn wifi_tcp_repl_reply(&mut self, payload: NetReplResponseBytes) -> WifiSdioTcpReplReplyReport;
    fn wifi_tcp_repl_service_poll(
        &mut self,
        port: u16,
        poll_frames: u8,
    ) -> WifiSdioTcpReceiveReport;
    fn wifi_tcp_repl_service_send(
        &mut self,
        payload: NetReplResponseBytes,
    ) -> WifiSdioTcpReplReplyReport;
    fn wifi_tcp_repl_service_reset(&mut self);
    fn http_get(&mut self, url: StringBytes) -> WifiSdioHttpGetReport;
    fn wifi_net_repl_poll(&mut self, poll_frames: u8) -> WifiNetReplRequestReport;
    fn wifi_net_repl_reply(
        &mut self,
        sequence: u32,
        payload: NetReplResponseBytes,
    ) -> WifiNetReplReplyReport;
    fn wifi_load_clm(&mut self) -> WifiSdioClmLoadReport;
    fn wifi_get_country(&mut self) -> WifiSdioCountryReport;
    fn wifi_set_country(&mut self, country_code: [u8; 2], revision: i32) -> WifiSdioCountryReport;
    fn wifi_disable_tx_glomming(&mut self) -> WifiSdioTxGlommingReport;
    fn wifi_enable_network_events(&mut self) -> WifiSdioEventMaskReport;
    fn wifi_start_scan(&mut self) -> WifiSdioScanStartReport;
    fn wifi_prepare_join(&mut self) -> WifiPrepareJoinReport {
        let mut report = WifiPrepareJoinReport::new();

        let sdio = self.wifi_sdio_init();
        report.sdio_status = sdio.status;
        if !status_ready(sdio.status) {
            report.mark_failed(b"wifi-sdio-init");
            return report;
        }

        let firmware = self.wifi_start_firmware();
        report.firmware_status = firmware.status;
        if !status_ready(firmware.status) {
            report.mark_failed(b"wifi-start-firmware");
            return report;
        }

        let wlc_up = self.wifi_wlc_up();
        report.wlc_up_status = wlc_up.status;
        if !status_ready(wlc_up.status) {
            report.mark_failed(b"wifi-wlc-up");
            return report;
        }

        let ack = self.wifi_ack_interrupts();
        report.ack_status = ack.status;
        if !status_ready(ack.status) {
            report.mark_failed(b"wifi-ack-interrupts");
            return report;
        }

        let clm = self.wifi_load_clm();
        report.clm_status = clm.status;
        if !status_ready(clm.status) {
            report.mark_failed(b"wifi-load-clm");
            return report;
        }

        let tx_glomming = self.wifi_disable_tx_glomming();
        report.tx_glomming_status = tx_glomming.status;
        if !status_ready(tx_glomming.status) {
            report.mark_failed(b"wifi-disable-tx-glomming");
            return report;
        }

        let country = self.wifi_set_country(*b"XX", -1);
        report.country_status = country.status;
        if !status_ready(country.status) {
            report.mark_failed(b"wifi-set-country");
            return report;
        }

        report.status = STATUS_READY;
        report.step = STEP_DONE;
        report
    }
    fn wifi_join_wpa2(&mut self, ssid: StringBytes, passphrase: StringBytes) -> WifiSdioJoinReport;
    fn wifi_drain_scan_events(&mut self) -> WifiSdioScanEventDrainReport;
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

fn status_ready(status: &[u8]) -> bool {
    status == STATUS_READY
}

fn net_repl_read_only_path_safe(path: StringBytes) -> bool {
    if path.len == 0 {
        return false;
    }

    let mut index = 0usize;
    while index < path.len as usize {
        let byte = path.bytes[index];
        let safe = byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'/');
        if !safe {
            return false;
        }
        index += 1;
    }

    true
}

fn append_string_byte(target: &mut StringBytes, byte: u8) -> LispResult<()> {
    let index = target.len as usize;
    if index >= MAX_STRING_BYTES {
        return Err(Error::new("string too long"));
    }
    target.bytes[index] = byte;
    target.len += 1;
    Ok(())
}

fn string_bytes_from_slice(bytes: &[u8], empty_message: &'static str) -> LispResult<StringBytes> {
    if bytes.is_empty() {
        return Err(Error::new(empty_message));
    }
    if bytes.len() > MAX_STRING_BYTES {
        return Err(Error::new("string too long"));
    }

    let mut value = EMPTY_STRING_BYTES;
    let mut index = 0usize;
    while index < bytes.len() {
        value.bytes[index] = bytes[index];
        index += 1;
    }
    value.len = bytes.len() as u8;
    Ok(value)
}

#[derive(Clone, Copy)]
pub struct RegisterReport {
    pub scb5_ctrl: u32,
    pub scb5_uart_ctrl: u32,
    pub scb5_rx_status: u32,
    pub scb5_tx_status: u32,
    pub peri_clock5: u32,
    pub peri_div8_0: u32,
    pub peri_div16_5_0: u32,
    pub hsiom_prt5_sel0: u32,
    pub gpio_prt5_cfg: u32,
    pub gpio_prt13_out: u32,
    pub gpio_prt13_cfg: u32,
}

#[derive(Clone, Copy)]
pub struct PdmStatusReport {
    pub gpio_prt10_cfg: u32,
    pub gpio_prt10_in: u32,
    pub hsiom_prt10_sel0: u32,
    pub hsiom_prt10_sel1: u32,
    pub pdm_ctl: u32,
    pub pdm_clock_ctl: u32,
    pub pdm_mode_ctl: u32,
    pub pdm_data_ctl: u32,
    pub pdm_rx_fifo_ctl: u32,
    pub pdm_rx_fifo_status: u32,
}

#[derive(Clone, Copy)]
pub struct ThermistorStatusReport {
    pub gpio_prt10_cfg: u32,
    pub gpio_prt10_in: u32,
    pub hsiom_prt10_sel0: u32,
    pub hsiom_prt10_sel1: u32,
}

#[derive(Clone, Copy)]
pub enum ThermistorReadStatus {
    Ready,
    Timeout,
}

#[derive(Clone, Copy)]
pub struct ThermistorReadReport {
    pub status: ThermistorReadStatus,
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
pub struct CapsenseStatusReport {
    pub gpio_prt1_cfg: u32,
    pub gpio_prt7_cfg: u32,
    pub gpio_prt8_cfg: u32,
    pub gpio_prt8_in: u32,
    pub hsiom_prt1_sel0: u32,
    pub hsiom_prt7_sel1: u32,
    pub hsiom_prt8_sel0: u32,
    pub hsiom_prt8_sel1: u32,
    pub csd_config: u32,
    pub csd_status: u32,
    pub csd_stat_seq: u32,
    pub csd_intr_masked: u32,
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
    pub content_len: u16,
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
    pub content_len: u16,
    pub directory_sector: u32,
    pub data_sector: u32,
    pub content: FileBytes,
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
pub struct WifiSdioLinkStatusReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub mac_status: &'static [u8],
    pub bssid_status: &'static [u8],
    pub rssi_status: &'static [u8],
    pub mac_hash: u32,
    pub mac_present: bool,
    pub bssid_hash: u32,
    pub bssid_present: bool,
    pub rssi: i32,
    pub mac_cdc_status: u32,
    pub bssid_cdc_status: u32,
    pub rssi_cdc_status: u32,
    pub mac_cdc_length: u32,
    pub bssid_cdc_length: u32,
    pub rssi_cdc_length: u32,
    pub skipped_frames: u8,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioDhcpDiscoverReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub mac_status: &'static [u8],
    pub mac_hash: u32,
    pub mac_present: bool,
    pub mac_cdc_status: u32,
    pub mac_cdc_length: u32,
    pub skipped_frames: u8,
    pub transaction_id: u32,
    pub ethernet_length: u16,
    pub ethernet_hash: u32,
    pub ip_total_length: u16,
    pub udp_length: u16,
    pub dhcp_payload_length: u16,
    pub send_status: &'static [u8],
    pub send_initial_tx_credit: u8,
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioDhcpAcquireReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub mac_status: &'static [u8],
    pub mac_hash: u32,
    pub mac_present: bool,
    pub mac_cdc_status: u32,
    pub mac_cdc_length: u32,
    pub transaction_id: u32,
    pub discover_status: &'static [u8],
    pub discover_packet_length: u16,
    pub discover_write_response: u32,
    pub offer_poll_status: &'static [u8],
    pub offer_parse_status: &'static [u8],
    pub offer_polls: u8,
    pub offer_frames_read: u8,
    pub offer_non_data_frames: u8,
    pub offer_non_dhcp_frames: u8,
    pub offer_message_type: u8,
    pub offered_ip_address: u32,
    pub server_identifier: u32,
    pub request_status: &'static [u8],
    pub request_packet_length: u16,
    pub request_write_response: u32,
    pub ack_poll_status: &'static [u8],
    pub ack_parse_status: &'static [u8],
    pub ack_polls: u8,
    pub ack_frames_read: u8,
    pub ack_non_data_frames: u8,
    pub ack_non_dhcp_frames: u8,
    pub ack_message_type: u8,
    pub leased_ip_address: u32,
    pub subnet_mask: u32,
    pub router: u32,
    pub dns_server: u32,
    pub lease_seconds: u32,
    pub lease_valid: bool,
    pub ack_status: &'static [u8],
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub discover_last_error: Option<WifiSdioCommandErrorReport>,
    pub request_last_error: Option<WifiSdioCommandErrorReport>,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioLeaseStatusReport {
    pub status: &'static [u8],
    pub lease_valid: bool,
    pub transaction_id: u32,
    pub ip_address: u32,
    pub subnet_mask: u32,
    pub router: u32,
    pub dns_server: u32,
    pub server_identifier: u32,
    pub lease_seconds: u32,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioArpRouterReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub lease_valid: bool,
    pub local_ip_address: u32,
    pub router_ip_address: u32,
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub mac_status: &'static [u8],
    pub mac_hash: u32,
    pub mac_present: bool,
    pub mac_cdc_status: u32,
    pub mac_cdc_length: u32,
    pub request_status: &'static [u8],
    pub request_ethernet_length: u16,
    pub request_ethernet_hash: u32,
    pub request_packet_length: u16,
    pub request_write_response: u32,
    pub reply_poll_status: &'static [u8],
    pub reply_parse_status: &'static [u8],
    pub reply_polls: u8,
    pub reply_frames_read: u8,
    pub reply_non_data_frames: u8,
    pub reply_non_arp_frames: u8,
    pub router_mac_hash: u32,
    pub router_mac_present: bool,
    pub router_mac_stored: bool,
    pub ack_status: &'static [u8],
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub request_last_error: Option<WifiSdioCommandErrorReport>,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioDnsQueryReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub lease_valid: bool,
    pub local_ip_address: u32,
    pub dns_server_ip_address: u32,
    pub router_ip_address: u32,
    pub router_mac_present: bool,
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub mac_status: &'static [u8],
    pub mac_hash: u32,
    pub mac_present: bool,
    pub mac_cdc_status: u32,
    pub mac_cdc_length: u32,
    pub transaction_id: u16,
    pub query_name_length: u8,
    pub query_payload_length: u16,
    pub query_payload_hash: u32,
    pub request_status: &'static [u8],
    pub request_ethernet_length: u16,
    pub request_ethernet_hash: u32,
    pub request_packet_length: u16,
    pub request_write_response: u32,
    pub response_poll_status: &'static [u8],
    pub response_parse_status: &'static [u8],
    pub response_polls: u8,
    pub response_frames_read: u8,
    pub response_non_data_frames: u8,
    pub response_non_dns_frames: u8,
    pub response_answer_count: u16,
    pub answer_ip_address: u32,
    pub answer_ttl_seconds: u32,
    pub answer_valid: bool,
    pub ack_status: &'static [u8],
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub request_last_error: Option<WifiSdioCommandErrorReport>,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioTcpSynReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub lease_valid: bool,
    pub local_ip_address: u32,
    pub remote_ip_address: u32,
    pub remote_port: u16,
    pub source_port: u16,
    pub local_sequence: u32,
    pub remote_sequence: u32,
    pub ack_number: u32,
    pub response_flags: u8,
    pub router_mac_present: bool,
    pub local_mac_present: bool,
    pub dns_status: &'static [u8],
    pub dns_parse_status: &'static [u8],
    pub request_status: &'static [u8],
    pub request_ethernet_length: u16,
    pub request_ethernet_hash: u32,
    pub request_packet_length: u16,
    pub request_write_response: u32,
    pub response_poll_status: &'static [u8],
    pub response_parse_status: &'static [u8],
    pub response_polls: u8,
    pub response_frames_read: u8,
    pub response_non_data_frames: u8,
    pub response_non_tcp_frames: u8,
    pub reset_status: &'static [u8],
    pub reset_packet_length: u16,
    pub reset_write_response: u32,
    pub ack_status: &'static [u8],
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub request_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub reset_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioTcpListenReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub local_ip_address: u32,
    pub listen_port: u16,
    pub peer_ip_address: u32,
    pub peer_port: u16,
    pub peer_sequence: u32,
    pub ack_number: u32,
    pub flags: u8,
    pub listen_poll_status: &'static [u8],
    pub listen_parse_status: &'static [u8],
    pub listen_polls: u8,
    pub listen_frames_read: u8,
    pub syn_ack_status: &'static [u8],
    pub ack_poll_status: &'static [u8],
    pub ack_parse_status: &'static [u8],
    pub ack_polls: u8,
    pub ack_frames_read: u8,
    pub reset_status: &'static [u8],
    pub interrupt_ack_status: &'static [u8],
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioTcpReceiveReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub local_ip_address: u32,
    pub listen_port: u16,
    pub peer_ip_address: u32,
    pub peer_port: u16,
    pub peer_sequence: u32,
    pub ack_number: u32,
    pub flags: u8,
    pub listen_poll_status: &'static [u8],
    pub listen_parse_status: &'static [u8],
    pub listen_polls: u8,
    pub listen_frames_read: u8,
    pub syn_ack_status: &'static [u8],
    pub ack_poll_status: &'static [u8],
    pub ack_parse_status: &'static [u8],
    pub ack_polls: u8,
    pub ack_frames_read: u8,
    pub payload_poll_status: &'static [u8],
    pub payload_parse_status: &'static [u8],
    pub payload_polls: u8,
    pub payload_frames_read: u8,
    pub payload_bytes: u16,
    pub payload_hash: u32,
    pub payload_preview: StringBytes,
    pub payload: [u8; NET_REPL_REQUEST_PAYLOAD_BYTES],
    pub reset_status: &'static [u8],
    pub interrupt_ack_status: &'static [u8],
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub payload_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioTcpReplReplyReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub lease_valid: bool,
    pub local_ip_address: u32,
    pub peer_valid: bool,
    pub peer_ip_address: u32,
    pub peer_port: u16,
    pub peer_mac_hash: u32,
    pub listen_port: u16,
    pub local_sequence: u32,
    pub ack_number: u32,
    pub local_mac_present: bool,
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub payload_length: u16,
    pub payload_hash: u32,
    pub ethernet_length: u16,
    pub ethernet_hash: u32,
    pub send_status: &'static [u8],
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiSdioHttpGetReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub remote_ip_address: u32,
    pub source_port: u16,
    pub dns_status: &'static [u8],
    pub syn_status: &'static [u8],
    pub get_status: &'static [u8],
    pub response_status: &'static [u8],
    pub response_parse_status: &'static [u8],
    pub response_polls: u8,
    pub response_payload_bytes: u16,
    pub http_status_code: u16,
    pub response_preview: StringBytes,
    pub reset_status: &'static [u8],
}

#[derive(Clone, Copy)]
pub struct WifiNetReplRequestReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub lease_valid: bool,
    pub local_ip_address: u32,
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub parse_status: &'static [u8],
    pub poll_limit: u8,
    pub polls: u8,
    pub frames_read: u8,
    pub non_data_frames: u8,
    pub non_repl_frames: u8,
    pub source_ip_address: u32,
    pub source_port: u16,
    pub source_mac_hash: u32,
    pub sequence: u32,
    pub read_only: bool,
    pub payload_length: u8,
    pub payload_hash: u32,
    pub ack_response_hash: u32,
    pub payload: [u8; NET_REPL_REQUEST_PAYLOAD_BYTES],
    pub ack_status: &'static [u8],
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiNetReplReplyReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub lease_valid: bool,
    pub local_ip_address: u32,
    pub peer_valid: bool,
    pub peer_ip_address: u32,
    pub peer_port: u16,
    pub peer_mac_hash: u32,
    pub sequence: u32,
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub mac_status: &'static [u8],
    pub mac_hash: u32,
    pub mac_present: bool,
    pub mac_cdc_status: u32,
    pub mac_cdc_length: u32,
    pub payload_length: u16,
    pub payload_hash: u32,
    pub ethernet_length: u16,
    pub ethernet_hash: u32,
    pub send_status: &'static [u8],
    pub send_packet_length: u16,
    pub send_write_response: u32,
    pub ack_status: &'static [u8],
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub mac_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
}

#[derive(Clone, Copy)]
pub struct WifiNetworkBootstrapReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub prepare_status: &'static [u8],
    pub join_status: &'static [u8],
    pub join_flags: u32,
    pub dhcp_status: &'static [u8],
    pub lease_status: &'static [u8],
    pub lease_valid: bool,
    pub local_ip_address: u32,
    pub router_ip_address: u32,
    pub arp_status: &'static [u8],
    pub arp_reply_poll_status: &'static [u8],
    pub arp_reply_parse_status: &'static [u8],
    pub router_mac_hash: u32,
    pub router_mac_present: bool,
    pub router_mac_stored: bool,
}

impl WifiNetworkBootstrapReport {
    fn new() -> Self {
        Self {
            status: STATUS_NOT_RUN,
            step: b"start",
            prepare_status: STATUS_NOT_RUN,
            join_status: STATUS_NOT_RUN,
            join_flags: 0,
            dhcp_status: STATUS_NOT_RUN,
            lease_status: STATUS_NOT_RUN,
            lease_valid: false,
            local_ip_address: 0,
            router_ip_address: 0,
            arp_status: STATUS_NOT_RUN,
            arp_reply_poll_status: STATUS_NOT_RUN,
            arp_reply_parse_status: STATUS_NOT_RUN,
            router_mac_hash: 0,
            router_mac_present: false,
            router_mac_stored: false,
        }
    }

    fn mark_failed(&mut self, step: &'static [u8]) {
        self.status = STATUS_STEP_FAILED;
        self.step = step;
    }
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
pub struct WifiSdioCountryReport {
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
    pub country_abbrev: StringBytes,
    pub revision: i32,
    pub country_code: StringBytes,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioEventMaskReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub enabled_events: u8,
    pub mask_words: [u32; 4],
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
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioTxGlommingReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub enabled: bool,
    pub value: u32,
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
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioScanStartReport {
    pub status: &'static [u8],
    pub ht_status: &'static [u8],
    pub ht_attempts: u16,
    pub ht_write_response: u32,
    pub ht_read_value: u8,
    pub ht_read_response: u32,
    pub ht_available: bool,
    pub scan_payload_bytes: u16,
    pub scan_version: u32,
    pub scan_action: u16,
    pub scan_sync_id: u16,
    pub scan_type: u8,
    pub bss_type: u8,
    pub bssid_filter_broadcast: bool,
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
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiPrepareJoinReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub sdio_status: &'static [u8],
    pub firmware_status: &'static [u8],
    pub wlc_up_status: &'static [u8],
    pub ack_status: &'static [u8],
    pub clm_status: &'static [u8],
    pub tx_glomming_status: &'static [u8],
    pub country_status: &'static [u8],
}

impl WifiPrepareJoinReport {
    fn new() -> Self {
        Self {
            status: STATUS_NOT_RUN,
            step: b"start",
            sdio_status: STATUS_NOT_RUN,
            firmware_status: STATUS_NOT_RUN,
            wlc_up_status: STATUS_NOT_RUN,
            ack_status: STATUS_NOT_RUN,
            clm_status: STATUS_NOT_RUN,
            tx_glomming_status: STATUS_NOT_RUN,
            country_status: STATUS_NOT_RUN,
        }
    }

    fn mark_failed(&mut self, step: &'static [u8]) {
        self.status = STATUS_STEP_FAILED;
        self.step = step;
    }
}

#[derive(Clone, Copy)]
pub struct WifiSdioJoinReport {
    pub status: &'static [u8],
    pub step: &'static [u8],
    pub ssid_len: u8,
    pub passphrase_len: u8,
    pub optional_cdc_errors: u8,
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
    pub requested_polls: u8,
    pub polls: u8,
    pub frames_read: u8,
    pub non_event_frames: u8,
    pub events_seen: u8,
    pub join_flags: u32,
    pub last_frame_status: &'static [u8],
    pub last_frame_length: u16,
    pub last_frame_channel: u8,
    pub last_frame_bus_data_credit: u8,
    pub last_event_type: u32,
    pub last_event_status: u32,
    pub last_event_reason: u32,
    pub last_event_flags: u32,
    pub last_event_datalen: u32,
    pub last_event_ifidx: u8,
    pub last_event_bsscfgidx: u8,
    pub ack_status: &'static [u8],
    pub ack_int_status_before: u32,
    pub ack_clear_value: u32,
    pub ack_int_status_after: u32,
    pub ack_final_response: u32,
    pub ht_last_error: Option<WifiSdioCommandErrorReport>,
    pub send_last_error: Option<WifiSdioCommandErrorReport>,
    pub response_last_error: Option<WifiSdioCommandErrorReport>,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
    pub host_normal_int: u16,
    pub host_error_int: u16,
    pub host: WifiSdioHostReport,
}

#[derive(Clone, Copy)]
pub struct WifiSdioScanEventDrainReport {
    pub status: &'static [u8],
    pub stop_reason: &'static [u8],
    pub requested_frames: u8,
    pub frames_read: u8,
    pub non_event_frames: u8,
    pub events_seen: u8,
    pub other_events: u8,
    pub scan_events: u8,
    pub scan_partial: u8,
    pub scan_complete: u8,
    pub scan_abort: u8,
    pub scan_other_status: u8,
    pub last_frame_status: &'static [u8],
    pub last_frame_length: u16,
    pub last_frame_channel: u8,
    pub last_frame_bus_data_credit: u8,
    pub last_event_type: u32,
    pub last_event_status: u32,
    pub last_event_reason: u32,
    pub last_event_datalen: u32,
    pub last_event_ifidx: u8,
    pub last_event_bsscfgidx: u8,
    pub last_escan_buflen: u32,
    pub last_escan_version: u32,
    pub last_escan_sync_id: u16,
    pub last_escan_bss_count: u16,
    pub ack_status: &'static [u8],
    pub ack_int_status_before: u32,
    pub ack_clear_value: u32,
    pub ack_int_status_after: u32,
    pub ack_final_response: u32,
    pub frame_last_error: Option<WifiSdioCommandErrorReport>,
    pub ack_last_error: Option<WifiSdioCommandErrorReport>,
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

struct NetReplResponseWriter {
    len: usize,
    truncated: bool,
    bytes: [u8; NET_REPL_RESPONSE_BYTES],
}

impl NetReplResponseWriter {
    fn new() -> Self {
        Self {
            len: 0,
            truncated: false,
            bytes: [0; NET_REPL_RESPONSE_BYTES],
        }
    }

    fn response(self) -> NetReplResponseBytes {
        NetReplResponseBytes {
            len: self.len as u16,
            truncated: self.truncated,
            bytes: self.bytes,
        }
    }
}

impl Write for NetReplResponseWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let bytes = value.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            if self.len >= NET_REPL_RESPONSE_BYTES {
                self.truncated = true;
                return Ok(());
            }
            self.bytes[self.len] = bytes[index];
            self.len += 1;
            index += 1;
        }
        Ok(())
    }
}

struct TelnetResponseBuilder {
    len: usize,
    truncated: bool,
    bytes: [u8; NET_REPL_RESPONSE_BYTES],
}

impl TelnetResponseBuilder {
    fn new() -> Self {
        Self {
            len: 0,
            truncated: false,
            bytes: [0; NET_REPL_RESPONSE_BYTES],
        }
    }

    fn response(self) -> NetReplResponseBytes {
        NetReplResponseBytes {
            len: self.len as u16,
            truncated: self.truncated,
            bytes: self.bytes,
        }
    }

    fn append_control(&mut self, byte: u8) {
        self.append_raw(byte);
    }

    fn append_nvt_text(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.append_nvt_byte(*byte);
        }
    }

    fn append_lisp_response(&mut self, response: &NetReplResponseBytes) {
        self.truncated |= response.truncated;
        self.append_nvt_text(&response.bytes[..response.len as usize]);
    }

    fn append_nvt_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.append_raw(b'\r');
                self.append_raw(b'\n');
            }
            b'\r' => {
                self.append_raw(b'\r');
                self.append_raw(0);
            }
            TELNET_IAC => {
                self.append_raw(TELNET_IAC);
                self.append_raw(TELNET_IAC);
            }
            byte => self.append_raw(byte),
        }
    }

    fn append_raw(&mut self, byte: u8) {
        if self.len < self.bytes.len() {
            self.bytes[self.len] = byte;
            self.len += 1;
        } else {
            self.truncated = true;
        }
    }
}

struct FileBytesWriter {
    len: usize,
    truncated: bool,
    bytes: [u8; MAX_FILE_BYTES],
}

impl FileBytesWriter {
    fn new() -> Self {
        Self {
            len: 0,
            truncated: false,
            bytes: [0; MAX_FILE_BYTES],
        }
    }
}

impl Write for FileBytesWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let bytes = value.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            if self.len >= MAX_FILE_BYTES {
                self.truncated = true;
                return Ok(());
            }
            self.bytes[self.len] = bytes[index];
            self.len += 1;
            index += 1;
        }
        Ok(())
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
    Print,
    Cons,
    Car,
    Cdr,
    List,
    Led,
    Heartbeat,
    ConsoleEcho,
    Button,
    Millis,
    Spawn,
    Processes,
    ProcessPoll,
    Kill,
    Yield,
    SleepMs,
    Reg32,
    Poke32,
    Regs,
    PdmStatus,
    ThermistorStatus,
    ThermistorRead,
    CapsenseStatus,
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
    AppendFile,
    SaveDefs,
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
    WifiLinkStatus,
    WifiDhcpDiscover,
    WifiDhcpAcquire,
    WifiLeaseStatus,
    WifiArpRouter,
    WifiDnsQuery,
    WifiTcpSyn,
    WifiTcpSynIp,
    WifiTcpListenOnce,
    WifiTcpReceiveOnce,
    WifiTcpReplOnce,
    HttpGet,
    WifiNetReplOnce,
    WifiNetReplService,
    WifiTcpReplService,
    WifiNetworkBootstrap,
    WifiLoadClm,
    WifiGetCountry,
    WifiSetCountry,
    WifiDisableTxGlomming,
    WifiEnableNetworkEvents,
    WifiStartScan,
    WifiPrepareJoin,
    WifiJoinWpa2,
    WifiConnectWpa2,
    WifiConnectLocal,
    WifiSsid,
    WifiPassphrase,
    WifiSsidClear,
    WifiSsidByte,
    WifiPassphraseClear,
    WifiPassphraseByte,
    WifiConnect,
    WifiDrainScanEvents,
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
            Self::Print => "print",
            Self::Cons => "cons",
            Self::Car => "car",
            Self::Cdr => "cdr",
            Self::List => "list",
            Self::Led => "led",
            Self::Heartbeat => "heartbeat",
            Self::ConsoleEcho => "console-echo",
            Self::Button => "button",
            Self::Millis => "millis",
            Self::Spawn => "spawn",
            Self::Processes => "processes",
            Self::ProcessPoll => "process-poll",
            Self::Kill => "kill",
            Self::Yield => "yield",
            Self::SleepMs => "sleep-ms",
            Self::Reg32 => "reg32",
            Self::Poke32 => "poke32",
            Self::Regs => "regs",
            Self::PdmStatus => "pdm-status",
            Self::ThermistorStatus => "thermistor-status",
            Self::ThermistorRead => "thermistor-read",
            Self::CapsenseStatus => "capsense-status",
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
            Self::AppendFile => "append-file",
            Self::SaveDefs => "save-defs",
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
            Self::WifiLinkStatus => "wifi-link-status",
            Self::WifiDhcpDiscover => "wifi-dhcp-discover",
            Self::WifiDhcpAcquire => "wifi-dhcp-acquire",
            Self::WifiLeaseStatus => "wifi-lease-status",
            Self::WifiArpRouter => "wifi-arp-router",
            Self::WifiDnsQuery => "wifi-dns-query",
            Self::WifiTcpSyn => "wifi-tcp-syn",
            Self::WifiTcpSynIp => "wifi-tcp-syn-ip",
            Self::WifiTcpListenOnce => "wifi-tcp-listen-once",
            Self::WifiTcpReceiveOnce => "wifi-tcp-receive-once",
            Self::WifiTcpReplOnce => "wifi-tcp-repl-once",
            Self::HttpGet => "http-get",
            Self::WifiNetReplOnce => "wifi-net-repl-once",
            Self::WifiNetReplService => "wifi-net-repl-service",
            Self::WifiTcpReplService => "wifi-tcp-repl-service",
            Self::WifiNetworkBootstrap => "wifi-network-bootstrap",
            Self::WifiLoadClm => "wifi-load-clm",
            Self::WifiGetCountry => "wifi-get-country",
            Self::WifiSetCountry => "wifi-set-country",
            Self::WifiDisableTxGlomming => "wifi-disable-tx-glomming",
            Self::WifiEnableNetworkEvents => "wifi-enable-network-events",
            Self::WifiStartScan => "wifi-start-scan",
            Self::WifiPrepareJoin => "wifi-prepare-join",
            Self::WifiJoinWpa2 => "wifi-join-wpa2",
            Self::WifiConnectWpa2 => "wifi-connect-wpa2",
            Self::WifiConnectLocal => "wifi-connect-local",
            Self::WifiSsid => "wifi-ssid",
            Self::WifiPassphrase => "wifi-passphrase",
            Self::WifiSsidClear => "wifi-ssid-clear",
            Self::WifiSsidByte => "wifi-ssid-byte",
            Self::WifiPassphraseClear => "wifi-pass-clear",
            Self::WifiPassphraseByte => "wifi-pass-byte",
            Self::WifiConnect => "wifi-connect",
            Self::WifiDrainScanEvents => "wifi-drain-scan-events",
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
    yield_: SymbolId,
    sleep_ms: SymbolId,
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
    yield_: 0,
    sleep_ms: 0,
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
    processes: [Process; MAX_PROCESSES],
    next_process_pid: u32,
    next_process_slot: usize,
    console_echo: bool,
    wifi_ssid: StringBytes,
    wifi_ssid_set: bool,
    wifi_passphrase: StringBytes,
    wifi_passphrase_set: bool,
    wifi_net_repl_service: WifiNetReplServiceState,
    wifi_tcp_repl_service: WifiTcpReplServiceState,
    wifi_net_repl_response_cache: WifiNetReplResponseCache,
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
            processes: [EMPTY_PROCESS; MAX_PROCESSES],
            next_process_pid: 1,
            next_process_slot: 0,
            console_echo: true,
            wifi_ssid: EMPTY_STRING_BYTES,
            wifi_ssid_set: false,
            wifi_passphrase: EMPTY_STRING_BYTES,
            wifi_passphrase_set: false,
            wifi_net_repl_service: EMPTY_WIFI_NET_REPL_SERVICE_STATE,
            wifi_tcp_repl_service: EMPTY_WIFI_TCP_REPL_SERVICE_STATE,
            wifi_net_repl_response_cache: EMPTY_WIFI_NET_REPL_RESPONSE_CACHE,
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
        self.specials.yield_ = self.intern(b"yield")?;
        self.specials.sleep_ms = self.intern(b"sleep-ms")?;

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
        self.install_primitive(b"print", Primitive::Print)?;
        self.install_primitive(b"cons", Primitive::Cons)?;
        self.install_primitive(b"car", Primitive::Car)?;
        self.install_primitive(b"cdr", Primitive::Cdr)?;
        self.install_primitive(b"list", Primitive::List)?;
        self.install_primitive(b"led", Primitive::Led)?;
        self.install_primitive(b"heartbeat", Primitive::Heartbeat)?;
        self.install_primitive(b"console-echo", Primitive::ConsoleEcho)?;
        self.install_primitive(b"button", Primitive::Button)?;
        self.install_primitive(b"millis", Primitive::Millis)?;
        self.install_primitive(b"spawn", Primitive::Spawn)?;
        self.install_primitive(b"processes", Primitive::Processes)?;
        self.install_primitive(b"process-poll", Primitive::ProcessPoll)?;
        self.install_primitive(b"kill", Primitive::Kill)?;
        self.install_primitive(b"yield", Primitive::Yield)?;
        self.install_primitive(b"sleep-ms", Primitive::SleepMs)?;
        self.install_primitive(b"reg32", Primitive::Reg32)?;
        self.install_primitive(b"poke32", Primitive::Poke32)?;
        self.install_primitive(b"regs", Primitive::Regs)?;
        self.install_primitive(b"pdm-status", Primitive::PdmStatus)?;
        self.install_primitive(b"thermistor-status", Primitive::ThermistorStatus)?;
        self.install_primitive(b"thermistor-read", Primitive::ThermistorRead)?;
        self.install_primitive(b"capsense-status", Primitive::CapsenseStatus)?;
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
        self.install_primitive(b"append-file", Primitive::AppendFile)?;
        self.install_primitive(b"save-defs", Primitive::SaveDefs)?;
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
        self.install_primitive(b"wifi-link-status", Primitive::WifiLinkStatus)?;
        self.install_primitive(b"wifi-dhcp-discover", Primitive::WifiDhcpDiscover)?;
        self.install_primitive(b"wifi-dhcp-acquire", Primitive::WifiDhcpAcquire)?;
        self.install_primitive(b"wifi-lease-status", Primitive::WifiLeaseStatus)?;
        self.install_primitive(b"wifi-arp-router", Primitive::WifiArpRouter)?;
        self.install_primitive(b"wifi-dns-query", Primitive::WifiDnsQuery)?;
        self.install_primitive(b"wifi-tcp-syn", Primitive::WifiTcpSyn)?;
        self.install_primitive(b"wifi-tcp-syn-ip", Primitive::WifiTcpSynIp)?;
        self.install_primitive(b"wifi-tcp-listen-once", Primitive::WifiTcpListenOnce)?;
        self.install_primitive(b"wifi-tcp-receive-once", Primitive::WifiTcpReceiveOnce)?;
        self.install_primitive(b"wifi-tcp-repl-once", Primitive::WifiTcpReplOnce)?;
        self.install_primitive(b"http-get", Primitive::HttpGet)?;
        self.install_primitive(b"wifi-net-repl-once", Primitive::WifiNetReplOnce)?;
        self.install_primitive(b"wifi-net-repl-service", Primitive::WifiNetReplService)?;
        self.install_primitive(b"wifi-tcp-repl-service", Primitive::WifiTcpReplService)?;
        self.install_primitive(b"wifi-network-bootstrap", Primitive::WifiNetworkBootstrap)?;
        self.install_primitive(b"wifi-load-clm", Primitive::WifiLoadClm)?;
        self.install_primitive(b"wifi-get-country", Primitive::WifiGetCountry)?;
        self.install_primitive(b"wifi-set-country", Primitive::WifiSetCountry)?;
        self.install_primitive(
            b"wifi-disable-tx-glomming",
            Primitive::WifiDisableTxGlomming,
        )?;
        self.install_primitive(
            b"wifi-enable-network-events",
            Primitive::WifiEnableNetworkEvents,
        )?;
        self.install_primitive(b"wifi-start-scan", Primitive::WifiStartScan)?;
        self.install_primitive(b"wifi-prepare-join", Primitive::WifiPrepareJoin)?;
        self.install_primitive(b"wifi-join-wpa2", Primitive::WifiJoinWpa2)?;
        self.install_primitive(b"wifi-connect-wpa2", Primitive::WifiConnectWpa2)?;
        self.install_primitive(b"wifi-connect-local", Primitive::WifiConnectLocal)?;
        self.install_primitive(b"wifi-ssid", Primitive::WifiSsid)?;
        self.install_primitive(b"wifi-passphrase", Primitive::WifiPassphrase)?;
        self.install_primitive(b"wifi-ssid-clear", Primitive::WifiSsidClear)?;
        self.install_primitive(b"wifi-ssid-byte", Primitive::WifiSsidByte)?;
        self.install_primitive(b"wifi-pass-clear", Primitive::WifiPassphraseClear)?;
        self.install_primitive(b"wifi-pass-byte", Primitive::WifiPassphraseByte)?;
        self.install_primitive(b"wifi-connect", Primitive::WifiConnect)?;
        self.install_primitive(b"wifi-drain-scan-events", Primitive::WifiDrainScanEvents)?;
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
        let previous_expression = self.active_expression;
        if let Ok(LoadFileOutcome::Loaded(value)) = result {
            self.active_expression = value;
        }
        self.collect_garbage();
        self.active_expression = previous_expression;
        result
    }

    pub fn write_value_to<W: Write>(&self, value: Value, output: &mut W) -> fmt::Result {
        self.write_value(value, output)
    }

    pub fn console_echo_enabled(&self) -> bool {
        self.console_echo
    }

    pub fn wifi_net_repl_service_enabled(&self) -> bool {
        self.wifi_net_repl_service.enabled
    }

    pub fn enable_wifi_net_repl_service(&mut self, poll_frames: u8) {
        self.wifi_net_repl_service.enabled = true;
        if poll_frames != 0 {
            self.wifi_net_repl_service.poll_frames = poll_frames;
        }
    }

    pub fn poll_wifi_net_repl_service<B: Board>(&mut self, board: &mut B) {
        if !self.wifi_net_repl_service.enabled {
            return;
        }

        let poll_frames = self.wifi_net_repl_service.poll_frames;
        let cycle = self.run_wifi_net_repl_cycle(board, poll_frames);
        self.record_wifi_net_repl_service_cycle(cycle);
    }

    pub fn wifi_net_repl_service_polls(&self) -> u32 {
        self.wifi_net_repl_service.polls
    }

    pub fn wifi_net_repl_service_requests_handled(&self) -> u32 {
        self.wifi_net_repl_service.requests_handled
    }

    pub fn wifi_net_repl_service_last_status(&self) -> &'static [u8] {
        self.wifi_net_repl_service.last_status
    }

    pub fn wifi_net_repl_service_last_reply_status(&self) -> &'static [u8] {
        self.wifi_net_repl_service.last_reply_status
    }

    pub fn wifi_net_repl_service_last_sequence(&self) -> u32 {
        self.wifi_net_repl_service.last_sequence
    }

    pub fn wifi_tcp_repl_service_enabled(&self) -> bool {
        self.wifi_tcp_repl_service.enabled
    }

    pub fn enable_wifi_tcp_repl_service(&mut self, listen_port: u16, poll_frames: u8) {
        self.wifi_tcp_repl_service.enabled = true;
        self.wifi_tcp_repl_service.listen_port = listen_port;
        self.wifi_tcp_repl_service.poll_frames = poll_frames;
    }

    pub fn poll_wifi_tcp_repl_service<B: Board>(&mut self, board: &mut B) {
        if !self.wifi_tcp_repl_service.enabled {
            return;
        }

        let listen_port = self.wifi_tcp_repl_service.listen_port;
        let poll_frames = self.wifi_tcp_repl_service.poll_frames;
        let cycle = self.run_wifi_telnet_repl_service_cycle(board, listen_port, poll_frames);
        self.record_wifi_tcp_repl_service_cycle(cycle);
    }

    fn clear_wifi_passphrase(&mut self) {
        self.wifi_passphrase = EMPTY_STRING_BYTES;
        self.wifi_passphrase_set = false;
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

    fn symbol_name_eq(&self, symbol: SymbolId, name: &[u8]) -> bool {
        let index = symbol as usize;
        if index >= MAX_SYMBOLS {
            return false;
        }
        let entry = self.symbols[index];
        if !entry.occupied || entry.len as usize != name.len() {
            return false;
        }

        let mut byte_index = 0usize;
        while byte_index < name.len() {
            if entry.bytes[byte_index] != name[byte_index] {
                return false;
            }
            byte_index += 1;
        }
        true
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
            Primitive::Print => self.primitive_print(args, output),
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
            Primitive::ConsoleEcho => {
                self.expect_count(args, 1)?;
                match args[0] {
                    Value::Symbol(symbol) if symbol == self.specials.status => {
                        Ok(Value::Bool(self.console_echo))
                    }
                    value => {
                        let enabled = self.on_off(value)?;
                        self.console_echo = enabled;
                        Ok(Value::Bool(self.console_echo))
                    }
                }
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
            Primitive::Spawn => {
                self.expect_count(args, 1)?;
                self.spawn_process(args[0], env)
            }
            Primitive::Processes => {
                self.expect_count(args, 0)?;
                self.processes_report()
            }
            Primitive::ProcessPoll => {
                let max_steps = match args.len() {
                    0 => 1,
                    1 => self.expect_u8(args[0])?,
                    _ => return Err(Error::new("process-poll expects zero or one argument")),
                };
                let now_ms = board.millis();
                let report = self.poll_processes(board, output, now_ms, max_steps);
                self.process_poll_report(report)
            }
            Primitive::Kill => {
                self.expect_count(args, 1)?;
                let pid = self.expect_u32(args[0])?;
                Ok(Value::Bool(self.kill_process(pid)))
            }
            Primitive::Yield => Err(Error::new("yield is only valid inside a process")),
            Primitive::SleepMs => Err(Error::new("sleep-ms is only valid inside a process")),
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
            Primitive::PdmStatus => {
                self.expect_count(args, 0)?;
                self.pdm_status_report(board.pdm_status())
            }
            Primitive::ThermistorStatus => {
                self.expect_count(args, 0)?;
                self.thermistor_status_report(board.thermistor_status())
            }
            Primitive::ThermistorRead => {
                self.expect_count(args, 0)?;
                self.thermistor_read_report(board.thermistor_read())
            }
            Primitive::CapsenseStatus => {
                self.expect_count(args, 0)?;
                self.capsense_status_report(board.capsense_status())
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
            Primitive::AppendFile => {
                self.expect_count(args, 2)?;
                let path = self.expect_string(args[0])?;
                let content = self.expect_string(args[1])?;
                self.store_write_report(board.append_file(path, content))
            }
            Primitive::SaveDefs => {
                self.expect_count(args, 1)?;
                let path = self.expect_string(args[0])?;
                self.save_defs(path, board)
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
                    self.file_bytes_string_value(report.content)
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
            Primitive::WifiLinkStatus => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_link_status_report(board.wifi_link_status())
            }
            Primitive::WifiDhcpDiscover => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_dhcp_discover_report(board.wifi_dhcp_discover())
            }
            Primitive::WifiDhcpAcquire => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_dhcp_acquire_report(board.wifi_dhcp_acquire())
            }
            Primitive::WifiLeaseStatus => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_lease_status_report(board.wifi_lease_status())
            }
            Primitive::WifiArpRouter => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_arp_router_report(board.wifi_arp_router())
            }
            Primitive::WifiDnsQuery => {
                self.expect_count(args, 1)?;
                let name = self.expect_string(args[0])?;
                self.wifi_sdio_dns_query_report(board.wifi_dns_query(name))
            }
            Primitive::WifiTcpSyn => {
                self.expect_count(args, 2)?;
                let name = self.expect_string(args[0])?;
                let port = self.expect_u16(args[1])?;
                self.wifi_sdio_tcp_syn_report(board.wifi_tcp_syn(name, port))
            }
            Primitive::WifiTcpSynIp => {
                self.expect_count(args, 2)?;
                let remote_ip_address = self.expect_u32(args[0])?;
                let port = self.expect_u16(args[1])?;
                self.wifi_sdio_tcp_syn_report(board.wifi_tcp_syn_ip(remote_ip_address, port))
            }
            Primitive::WifiTcpListenOnce => {
                let port = match args.len() {
                    1 | 2 => self.expect_u16(args[0])?,
                    _ => {
                        return Err(Error::new(
                            "wifi-tcp-listen-once expects port and optional poll count",
                        ))
                    }
                };
                let poll_frames = match args.len() {
                    1 => 32,
                    2 => self.expect_u8(args[1])?,
                    _ => unreachable!(),
                };
                self.wifi_sdio_tcp_listen_report(board.wifi_tcp_listen_once(port, poll_frames))
            }
            Primitive::WifiTcpReceiveOnce => {
                let port = match args.len() {
                    1 | 2 => self.expect_u16(args[0])?,
                    _ => {
                        return Err(Error::new(
                            "wifi-tcp-receive-once expects port and optional poll count",
                        ))
                    }
                };
                let poll_frames = match args.len() {
                    1 => 32,
                    2 => self.expect_u8(args[1])?,
                    _ => unreachable!(),
                };
                self.wifi_sdio_tcp_receive_report(board.wifi_tcp_receive_once(port, poll_frames))
            }
            Primitive::WifiTcpReplOnce => {
                let port = match args.len() {
                    1 | 2 => self.expect_u16(args[0])?,
                    _ => {
                        return Err(Error::new(
                            "wifi-tcp-repl-once expects port and optional poll count",
                        ))
                    }
                };
                let poll_frames = match args.len() {
                    1 => 32,
                    2 => self.expect_u8(args[1])?,
                    _ => unreachable!(),
                };
                self.wifi_tcp_repl_once(board, port, poll_frames)
            }
            Primitive::HttpGet => {
                self.expect_count(args, 1)?;
                let url = self.expect_string(args[0])?;
                self.wifi_sdio_http_get_report(board.http_get(url))
            }
            Primitive::WifiNetReplOnce => {
                let poll_frames = match args.len() {
                    0 => 1,
                    1 => self.expect_u8(args[0])?,
                    _ => {
                        return Err(Error::new(
                            "wifi-net-repl-once expects zero or one argument",
                        ))
                    }
                };
                self.wifi_net_repl_once(board, poll_frames)
            }
            Primitive::WifiNetReplService => self.wifi_net_repl_service(args),
            Primitive::WifiTcpReplService => self.wifi_tcp_repl_service(args, board),
            Primitive::WifiNetworkBootstrap => {
                self.expect_count(args, 0)?;
                self.wifi_network_bootstrap(board)
            }
            Primitive::WifiLoadClm => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_clm_load_report(board.wifi_load_clm())
            }
            Primitive::WifiGetCountry => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_country_report(board.wifi_get_country())
            }
            Primitive::WifiSetCountry => {
                self.expect_count(args, 2)?;
                let country_code = self.expect_country_code(args[0])?;
                let revision = self.expect_int(args[1])?;
                self.wifi_sdio_country_report(board.wifi_set_country(country_code, revision))
            }
            Primitive::WifiDisableTxGlomming => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_tx_glomming_report(board.wifi_disable_tx_glomming())
            }
            Primitive::WifiEnableNetworkEvents => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_event_mask_report(board.wifi_enable_network_events())
            }
            Primitive::WifiStartScan => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_scan_start_report(board.wifi_start_scan())
            }
            Primitive::WifiPrepareJoin => {
                self.expect_count(args, 0)?;
                self.wifi_prepare_join_report(board.wifi_prepare_join())
            }
            Primitive::WifiJoinWpa2 => {
                self.expect_count(args, 2)?;
                let ssid = self.expect_string(args[0])?;
                let passphrase = self.expect_string(args[1])?;
                self.wifi_sdio_join_report(board.wifi_join_wpa2(ssid, passphrase))
            }
            Primitive::WifiConnectWpa2 => {
                self.expect_count(args, 2)?;
                let ssid = self.expect_string(args[0])?;
                let passphrase = self.expect_string(args[1])?;
                let prepare = board.wifi_prepare_join();
                if !status_ready(prepare.status) {
                    self.wifi_prepare_join_report(prepare)
                } else {
                    self.wifi_sdio_join_report(board.wifi_join_wpa2(ssid, passphrase))
                }
            }
            Primitive::WifiConnectLocal => {
                self.expect_count(args, 0)?;
                let ssid = string_bytes_from_slice(
                    wifi_credentials::local_ssid(),
                    "local wifi ssid missing",
                )?;
                let passphrase = string_bytes_from_slice(
                    wifi_credentials::local_passphrase(),
                    "local wifi passphrase missing",
                )?;
                let prepare = board.wifi_prepare_join();
                if !status_ready(prepare.status) {
                    self.wifi_prepare_join_report(prepare)
                } else {
                    self.wifi_sdio_join_report(board.wifi_join_wpa2(ssid, passphrase))
                }
            }
            Primitive::WifiSsid => {
                self.expect_count(args, 1)?;
                let ssid = self.expect_string(args[0])?;
                self.wifi_ssid = ssid;
                self.wifi_ssid_set = true;
                self.wifi_credential_report(b"ssid", ssid.len)
            }
            Primitive::WifiPassphrase => {
                self.expect_count(args, 1)?;
                let passphrase = self.expect_string(args[0])?;
                self.wifi_passphrase = passphrase;
                self.wifi_passphrase_set = true;
                self.wifi_credential_report(b"passphrase", passphrase.len)
            }
            Primitive::WifiSsidClear => {
                self.expect_count(args, 0)?;
                self.wifi_ssid = EMPTY_STRING_BYTES;
                self.wifi_ssid_set = false;
                self.wifi_credential_report(b"ssid", 0)
            }
            Primitive::WifiSsidByte => {
                self.expect_count(args, 1)?;
                let byte = self.expect_u8(args[0])?;
                append_string_byte(&mut self.wifi_ssid, byte)?;
                self.wifi_ssid_set = true;
                self.wifi_credential_report(b"ssid", self.wifi_ssid.len)
            }
            Primitive::WifiPassphraseClear => {
                self.expect_count(args, 0)?;
                self.clear_wifi_passphrase();
                self.wifi_credential_report(b"passphrase", 0)
            }
            Primitive::WifiPassphraseByte => {
                self.expect_count(args, 1)?;
                let byte = self.expect_u8(args[0])?;
                append_string_byte(&mut self.wifi_passphrase, byte)?;
                self.wifi_passphrase_set = true;
                self.wifi_credential_report(b"passphrase", self.wifi_passphrase.len)
            }
            Primitive::WifiConnect => {
                self.expect_count(args, 0)?;
                if !self.wifi_ssid_set {
                    return Err(Error::new("wifi ssid not set"));
                }
                if !self.wifi_passphrase_set {
                    return Err(Error::new("wifi passphrase not set"));
                }
                let ssid = self.wifi_ssid;
                let passphrase = self.wifi_passphrase;
                let prepare = board.wifi_prepare_join();
                if !status_ready(prepare.status) {
                    self.clear_wifi_passphrase();
                    self.wifi_prepare_join_report(prepare)
                } else {
                    let report = board.wifi_join_wpa2(ssid, passphrase);
                    self.clear_wifi_passphrase();
                    self.wifi_sdio_join_report(report)
                }
            }
            Primitive::WifiDrainScanEvents => {
                self.expect_count(args, 0)?;
                self.wifi_sdio_scan_event_drain_report(board.wifi_drain_scan_events())
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

    fn primitive_print<W: Write>(&self, args: &[Value], output: &mut W) -> LispResult<Value> {
        let mut index = 0usize;
        while index < args.len() {
            if index > 0 {
                output
                    .write_char(' ')
                    .map_err(|_| Error::new("print output failed"))?;
            }
            self.write_repl_value(args[index], output)
                .map_err(|_| Error::new("print output failed"))?;
            index += 1;
        }

        output
            .write_char('\n')
            .map_err(|_| Error::new("print output failed"))?;

        if args.is_empty() {
            Ok(Value::Nil)
        } else {
            Ok(args[args.len() - 1])
        }
    }

    pub fn poll_background_processes<B: Board, W: Write>(
        &mut self,
        board: &mut B,
        output: &mut W,
        now_ms: u32,
        max_steps: u8,
    ) {
        self.poll_processes(board, output, now_ms, max_steps);
    }

    fn spawn_process(&mut self, program: Value, env: Value) -> LispResult<Value> {
        let body = self.process_body_from_program(program)?;
        let mut slot = None;
        let mut offset = 0usize;
        while offset < MAX_PROCESSES {
            let index = (self.next_process_slot + offset) % MAX_PROCESSES;
            if Self::process_slot_reusable(self.processes[index].state) {
                slot = Some(index);
                break;
            }
            offset += 1;
        }

        let index = slot.ok_or(Error::new("process table full"))?;
        let pid = self.next_process_pid;
        self.next_process_pid = self.next_process_pid.wrapping_add(1);
        if self.next_process_pid == 0 {
            self.next_process_pid = 1;
        }
        self.next_process_slot = (index + 1) % MAX_PROCESSES;
        self.processes[index] = Process {
            pid,
            state: ProcessState::Ready,
            body,
            cursor: body,
            env,
            wake_ms: 0,
            last_value: Value::Nil,
            error: "",
            steps: 0,
        };

        Ok(Value::Word(pid))
    }

    fn process_body_from_program(&mut self, program: Value) -> LispResult<Value> {
        if let Ok(ObjectKind::Pair { car, cdr }) = self.object_kind(program) {
            if matches!(car, Value::Symbol(symbol) if symbol == self.specials.begin) {
                if cdr == Value::Nil {
                    return Err(Error::new("spawn begin needs a body"));
                }
                return Ok(cdr);
            }
        }

        self.alloc_pair(program, Value::Nil)
    }

    fn kill_process(&mut self, pid: u32) -> bool {
        let mut index = 0usize;
        while index < MAX_PROCESSES {
            let mut process = self.processes[index];
            if process.pid == pid && !Self::process_slot_reusable(process.state) {
                process.state = ProcessState::Killed;
                process.cursor = Value::Nil;
                process.error = "killed";
                self.processes[index] = process;
                return true;
            }
            index += 1;
        }
        false
    }

    fn poll_processes<B: Board, W: Write>(
        &mut self,
        board: &mut B,
        output: &mut W,
        now_ms: u32,
        max_steps: u8,
    ) -> ProcessPollReport {
        let mut ran = 0u32;
        let mut remaining = max_steps;
        while remaining > 0 {
            let mut ran_one = false;
            let mut scanned = 0usize;
            while scanned < MAX_PROCESSES {
                let index = (self.next_process_slot + scanned) % MAX_PROCESSES;
                if self.run_process_step(index, board, output, now_ms) {
                    self.next_process_slot = (index + 1) % MAX_PROCESSES;
                    ran = ran.wrapping_add(1);
                    ran_one = true;
                    break;
                }
                scanned += 1;
            }

            if !ran_one {
                break;
            }
            remaining -= 1;
        }

        let mut report = self.process_counts();
        report.ran = ran;
        report
    }

    fn run_process_step<B: Board, W: Write>(
        &mut self,
        index: usize,
        board: &mut B,
        output: &mut W,
        now_ms: u32,
    ) -> bool {
        let mut process = self.processes[index];
        match process.state {
            ProcessState::Free
            | ProcessState::Done
            | ProcessState::Error
            | ProcessState::Killed => {
                return false;
            }
            ProcessState::Sleeping => {
                if !time_reached(now_ms, process.wake_ms) {
                    return false;
                }
                process.state = ProcessState::Ready;
            }
            ProcessState::Ready => {}
        }

        let (expression, rest) = match self.list_next(process.cursor) {
            Ok(Some(next)) => next,
            Ok(None) => {
                process.state = ProcessState::Done;
                process.cursor = Value::Nil;
                self.processes[index] = process;
                return false;
            }
            Err(error) => {
                process.state = ProcessState::Error;
                process.error = error.message();
                self.processes[index] = process;
                return true;
            }
        };

        let previous_active_expression = self.active_expression;
        self.active_expression = expression;
        let control = self.process_control(expression, process.env, board, output);
        self.active_expression = previous_active_expression;

        match control {
            Ok(Some(ProcessControl::Yield)) => {
                process.cursor = rest;
                process.last_value = Value::Symbol(self.specials.yield_);
                process.steps = process.steps.wrapping_add(1);
                process.error = "";
                process.state = if rest == Value::Nil {
                    ProcessState::Done
                } else {
                    ProcessState::Ready
                };
                self.processes[index] = process;
                true
            }
            Ok(Some(ProcessControl::Sleep { duration_ms })) => {
                process.cursor = rest;
                process.last_value = Value::Word(duration_ms);
                process.steps = process.steps.wrapping_add(1);
                process.error = "";
                process.wake_ms = now_ms.wrapping_add(duration_ms);
                process.state = ProcessState::Sleeping;
                self.processes[index] = process;
                true
            }
            Ok(None) => {
                let previous_active_expression = self.active_expression;
                self.active_expression = expression;
                let result = self.eval(expression, process.env, board, output, 0);
                self.active_expression = previous_active_expression;

                process.steps = process.steps.wrapping_add(1);
                match result {
                    Ok(value) => {
                        process.cursor = rest;
                        process.last_value = value;
                        process.error = "";
                        process.state = if rest == Value::Nil {
                            ProcessState::Done
                        } else {
                            ProcessState::Ready
                        };
                    }
                    Err(error) => {
                        process.state = ProcessState::Error;
                        process.error = error.message();
                    }
                }

                if self.processes[index].pid == process.pid
                    && self.processes[index].state != ProcessState::Killed
                {
                    self.processes[index] = process;
                }
                true
            }
            Err(error) => {
                process.state = ProcessState::Error;
                process.error = error.message();
                process.steps = process.steps.wrapping_add(1);
                self.processes[index] = process;
                true
            }
        }
    }

    fn process_control<B: Board, W: Write>(
        &mut self,
        expression: Value,
        env: Value,
        board: &mut B,
        output: &mut W,
    ) -> LispResult<Option<ProcessControl>> {
        if !matches!(expression, Value::Object(_)) {
            return Ok(None);
        }

        let (operator, args) = match self.object_kind(expression)? {
            ObjectKind::Pair { car, cdr } => (car, cdr),
            _ => return Ok(None),
        };

        let symbol = match operator {
            Value::Symbol(symbol) => symbol,
            _ => return Ok(None),
        };

        if symbol == self.specials.yield_ {
            if args != Value::Nil {
                return Err(Error::new("yield expects no arguments"));
            }
            return Ok(Some(ProcessControl::Yield));
        }

        if symbol == self.specials.sleep_ms {
            let (duration_expression, rest) = self.require_pair(args)?;
            if rest != Value::Nil {
                return Err(Error::new("sleep-ms expects one argument"));
            }
            let duration_value = self.eval(duration_expression, env, board, output, 0)?;
            let duration_ms = self.expect_u32(duration_value)?;
            return Ok(Some(ProcessControl::Sleep { duration_ms }));
        }

        Ok(None)
    }

    fn process_counts(&self) -> ProcessPollReport {
        let mut report = ProcessPollReport {
            ran: 0,
            ready: 0,
            sleeping: 0,
            done: 0,
            error: 0,
            killed: 0,
        };

        let mut index = 0usize;
        while index < MAX_PROCESSES {
            match self.processes[index].state {
                ProcessState::Free => {}
                ProcessState::Ready => report.ready = report.ready.wrapping_add(1),
                ProcessState::Sleeping => report.sleeping = report.sleeping.wrapping_add(1),
                ProcessState::Done => report.done = report.done.wrapping_add(1),
                ProcessState::Error => report.error = report.error.wrapping_add(1),
                ProcessState::Killed => report.killed = report.killed.wrapping_add(1),
            }
            index += 1;
        }

        report
    }

    fn processes_report(&mut self) -> LispResult<Value> {
        let mut list = Value::Nil;
        let mut index = MAX_PROCESSES;
        while index > 0 {
            index -= 1;
            let process = self.processes[index];
            if process.state != ProcessState::Free {
                let report = self.process_report(process)?;
                list = self.alloc_pair(report, list)?;
            }
        }
        Ok(list)
    }

    fn process_report(&mut self, process: Process) -> LispResult<Value> {
        let pid = self.word_entry(b"pid", process.pid)?;
        let state = self.symbol_entry(b"state", Self::process_state_name(process.state))?;
        let wake_ms = self.word_entry(b"wake-ms", process.wake_ms)?;
        let steps = self.word_entry(b"steps", process.steps)?;
        let error = self.string_entry(b"error", process.error.as_bytes())?;
        let entries = [pid, state, wake_ms, steps, error];
        self.make_list_from_values(&entries)
    }

    fn process_poll_report(&mut self, report: ProcessPollReport) -> LispResult<Value> {
        let ran = self.word_entry(b"ran", report.ran)?;
        let ready = self.word_entry(b"ready", report.ready)?;
        let sleeping = self.word_entry(b"sleeping", report.sleeping)?;
        let done = self.word_entry(b"done", report.done)?;
        let error = self.word_entry(b"error", report.error)?;
        let killed = self.word_entry(b"killed", report.killed)?;
        let entries = [ran, ready, sleeping, done, error, killed];
        self.make_list_from_values(&entries)
    }

    fn process_state_name(state: ProcessState) -> &'static [u8] {
        match state {
            ProcessState::Free => b"free",
            ProcessState::Ready => b"ready",
            ProcessState::Sleeping => b"sleeping",
            ProcessState::Done => b"done",
            ProcessState::Error => b"error",
            ProcessState::Killed => b"killed",
        }
    }

    fn process_slot_reusable(state: ProcessState) -> bool {
        matches!(
            state,
            ProcessState::Free | ProcessState::Done | ProcessState::Error | ProcessState::Killed
        )
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

        let input = &report.content.bytes[..usize::from(report.content.len)];
        let expression = self.read(input)?;
        let previous_expression = self.active_expression;
        self.active_expression = expression;
        let result = self.eval(expression, env, board, output, depth);
        self.active_expression = previous_expression;
        result.map(LoadFileOutcome::Loaded)
    }

    fn save_defs<B: Board>(&mut self, path: StringBytes, board: &mut B) -> LispResult<Value> {
        let mut writer = FileBytesWriter::new();
        writer
            .write_str("(begin\n")
            .map_err(|_| Error::new("save-defs format failed"))?;

        let mut saved = 0u16;
        let mut skipped = 0u16;
        let mut index = 0usize;
        while index < MAX_GLOBALS {
            let binding = self.globals[index];
            if binding.occupied && !self.is_intrinsic_global(binding) {
                if self.write_saved_definition(&mut writer, binding.symbol, binding.value)? {
                    saved = saved.saturating_add(1);
                } else {
                    skipped = skipped.saturating_add(1);
                }
            }
            index += 1;
        }

        writer
            .write_str(")\n")
            .map_err(|_| Error::new("save-defs format failed"))?;

        if writer.truncated {
            return self.save_defs_report(
                b"too-large",
                false,
                saved,
                skipped,
                writer.len as u16,
                0,
                true,
                STATUS_NOT_RUN,
            );
        }

        let content = FileBytes {
            len: writer.len as u16,
            bytes: writer.bytes,
        };
        let report = board.save_file_bytes(path, content);
        if !report.ready {
            return self.save_defs_report(
                b"write-failed",
                false,
                saved,
                skipped,
                writer.len as u16,
                1,
                false,
                report.status,
            );
        }

        self.save_defs_report(
            b"ready",
            true,
            saved,
            skipped,
            writer.len as u16,
            1,
            false,
            report.status,
        )
    }

    fn write_saved_definition<W: Write>(
        &self,
        output: &mut W,
        symbol: SymbolId,
        value: Value,
    ) -> LispResult<bool> {
        if let Value::Object(id) = value {
            if let ObjectKind::Closure { params, body, env } = self.object_kind_by_id(id)? {
                if env != Value::Nil || !self.value_is_saveable_data(params, 0)? {
                    return Ok(false);
                }
                if !self.value_is_saveable_data(body, 0)? {
                    return Ok(false);
                }

                output
                    .write_str("    (define ")
                    .and_then(|_| self.write_symbol(symbol, output))
                    .and_then(|_| output.write_str(" (lambda "))
                    .and_then(|_| self.write_value(params, output))
                    .map_err(|_| Error::new("save-defs format failed"))?;

                let mut cursor = body;
                while let Some((expression, rest)) = self.list_next(cursor)? {
                    output
                        .write_char(' ')
                        .and_then(|_| self.write_value(expression, output))
                        .map_err(|_| Error::new("save-defs format failed"))?;
                    cursor = rest;
                }

                output
                    .write_str("))\n")
                    .map_err(|_| Error::new("save-defs format failed"))?;
                return Ok(true);
            }
        }

        if !self.value_is_saveable_data(value, 0)? {
            return Ok(false);
        }

        output
            .write_str("    (define ")
            .and_then(|_| self.write_symbol(symbol, output))
            .and_then(|_| output.write_char(' '))
            .map_err(|_| Error::new("save-defs format failed"))?;

        if self.value_needs_quote(value) {
            output
                .write_str("(quote ")
                .and_then(|_| self.write_value(value, output))
                .and_then(|_| output.write_char(')'))
                .map_err(|_| Error::new("save-defs format failed"))?;
        } else {
            self.write_value(value, output)
                .map_err(|_| Error::new("save-defs format failed"))?;
        }

        output
            .write_str(")\n")
            .map_err(|_| Error::new("save-defs format failed"))?;
        Ok(true)
    }

    fn is_intrinsic_global(&self, binding: GlobalBinding) -> bool {
        if matches!(binding.value, Value::Primitive(_)) {
            return true;
        }

        self.is_reserved_symbol(binding.symbol)
    }

    fn is_reserved_symbol(&self, symbol: SymbolId) -> bool {
        symbol == self.specials.quote
            || symbol == self.specials.if_
            || symbol == self.specials.define
            || symbol == self.specials.lambda
            || symbol == self.specials.begin
            || symbol == self.specials.let_
            || symbol == self.specials.on
            || symbol == self.specials.off
            || symbol == self.specials.toggle
            || symbol == self.specials.status
            || self.symbol_name_eq(symbol, b"true")
            || self.symbol_name_eq(symbol, b"false")
    }

    fn value_is_saveable_data(&self, value: Value, depth: u8) -> LispResult<bool> {
        if depth > MAX_EVAL_DEPTH {
            return Ok(false);
        }

        match value {
            Value::Nil | Value::Bool(_) | Value::Int(_) | Value::Word(_) | Value::Symbol(_) => {
                Ok(true)
            }
            Value::Primitive(_) => Ok(false),
            Value::Object(id) => match self.object_kind_by_id(id)? {
                ObjectKind::Pair { car, cdr } => Ok(self.value_is_saveable_data(car, depth + 1)?
                    && self.value_is_saveable_data(cdr, depth + 1)?),
                ObjectKind::String { .. } => Ok(true),
                ObjectKind::Closure { .. } | ObjectKind::Env { .. } | ObjectKind::Free => Ok(false),
            },
        }
    }

    fn value_needs_quote(&self, value: Value) -> bool {
        match value {
            Value::Symbol(_) => true,
            Value::Object(id) => matches!(self.object_kind_by_id(id), Ok(ObjectKind::Pair { .. })),
            _ => false,
        }
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

    fn expect_u16(&self, value: Value) -> LispResult<u16> {
        let value = self.expect_u32(value)?;
        if value <= u16::MAX as u32 {
            Ok(value as u16)
        } else {
            Err(Error::new("expected 16-bit value"))
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

    fn expect_country_code(&self, value: Value) -> LispResult<[u8; 2]> {
        let value = self.expect_string(value)?;
        if value.len != 2 {
            return Err(Error::new("expected two-byte country code"));
        }
        let code = [value.bytes[0], value.bytes[1]];
        if is_country_code_byte(code[0]) && is_country_code_byte(code[1]) {
            Ok(code)
        } else {
            Err(Error::new("expected uppercase country code"))
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

    fn expect_net_repl_service_poll_frames(&self, value: Value) -> LispResult<u8> {
        let poll_frames = self.expect_u8(value)?;
        if poll_frames == 0 {
            Err(Error::new("expected non-zero poll frame count"))
        } else {
            Ok(poll_frames)
        }
    }

    fn expect_tcp_repl_service_port(&self, value: Value) -> LispResult<u16> {
        let port = self.expect_u16(value)?;
        if port == 0 {
            Err(Error::new("expected non-zero TCP port"))
        } else {
            Ok(port)
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

    fn string_entry(&mut self, name: &[u8], value: &[u8]) -> LispResult<Value> {
        let value = self.alloc_string(value)?;
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
            b"print",
            b"cons",
            b"car",
            b"cdr",
            b"list",
            b"led",
            b"heartbeat",
            b"console-echo",
            b"button",
            b"millis",
            b"spawn",
            b"processes",
            b"process-poll",
            b"kill",
            b"yield",
            b"sleep-ms",
            b"reg32",
            b"poke32",
            b"regs",
            b"pdm-status",
            b"thermistor-status",
            b"thermistor-read",
            b"capsense-status",
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
            b"append-file",
            b"save-defs",
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
            b"wifi-link-status",
            b"wifi-dhcp-discover",
            b"wifi-dhcp-acquire",
            b"wifi-lease-status",
            b"wifi-arp-router",
            b"wifi-dns-query",
            b"wifi-tcp-syn",
            b"wifi-tcp-syn-ip",
            b"wifi-tcp-listen-once",
            b"wifi-tcp-receive-once",
            b"wifi-tcp-repl-once",
            b"http-get",
            b"wifi-net-repl-once",
            b"wifi-net-repl-service",
            b"wifi-tcp-repl-service",
            b"wifi-network-bootstrap",
            b"wifi-load-clm",
            b"wifi-get-country",
            b"wifi-set-country",
            b"wifi-disable-tx-glomming",
            b"wifi-enable-network-events",
            b"wifi-start-scan",
            b"wifi-prepare-join",
            b"wifi-join-wpa2",
            b"wifi-connect-wpa2",
            b"wifi-connect-local",
            b"wifi-ssid",
            b"wifi-passphrase",
            b"wifi-ssid-clear",
            b"wifi-ssid-byte",
            b"wifi-pass-clear",
            b"wifi-pass-byte",
            b"wifi-connect",
            b"wifi-drain-scan-events",
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
        let peri_div16_5_0 = self.word_entry(b"PERI.DIV16_5.0", report.peri_div16_5_0)?;
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
            peri_div16_5_0,
            hsiom_prt5_sel0,
            gpio_prt5_cfg,
            gpio_prt13_out,
            gpio_prt13_cfg,
        ];
        self.make_list_from_values(&entries)
    }

    fn pdm_status_report(&mut self, report: PdmStatusReport) -> LispResult<Value> {
        let clock = self.symbol_entry(b"clock", b"P10.4")?;
        let data = self.symbol_entry(b"data", b"P10.5")?;
        let gpio_cfg = self.word_entry(b"GPIO.PRT10.CFG", report.gpio_prt10_cfg)?;
        let gpio_in = self.word_entry(b"GPIO.PRT10.IN", report.gpio_prt10_in)?;
        let hsiom_sel0 = self.word_entry(b"HSIOM.PRT10.SEL0", report.hsiom_prt10_sel0)?;
        let hsiom_sel1 = self.word_entry(b"HSIOM.PRT10.SEL1", report.hsiom_prt10_sel1)?;
        let pdm_ctl = self.word_entry(b"PDM.CTL", report.pdm_ctl)?;
        let pdm_clock_ctl = self.word_entry(b"PDM.CLOCK_CTL", report.pdm_clock_ctl)?;
        let pdm_mode_ctl = self.word_entry(b"PDM.MODE_CTL", report.pdm_mode_ctl)?;
        let pdm_data_ctl = self.word_entry(b"PDM.DATA_CTL", report.pdm_data_ctl)?;
        let pdm_rx_fifo_ctl = self.word_entry(b"PDM.RX_FIFO_CTL", report.pdm_rx_fifo_ctl)?;
        let pdm_rx_fifo_status =
            self.word_entry(b"PDM.RX_FIFO_STATUS", report.pdm_rx_fifo_status)?;
        let entries = [
            clock,
            data,
            gpio_cfg,
            gpio_in,
            hsiom_sel0,
            hsiom_sel1,
            pdm_ctl,
            pdm_clock_ctl,
            pdm_mode_ctl,
            pdm_data_ctl,
            pdm_rx_fifo_ctl,
            pdm_rx_fifo_status,
        ];
        self.make_list_from_values(&entries)
    }

    fn thermistor_status_report(&mut self, report: ThermistorStatusReport) -> LispResult<Value> {
        let out0 = self.symbol_entry(b"out0", b"P10.1")?;
        let out1 = self.symbol_entry(b"out1", b"P10.2")?;
        let vdd = self.symbol_entry(b"vdd", b"P10.3")?;
        let gnd = self.symbol_entry(b"gnd", b"P10.0")?;
        let part = self.symbol_entry(b"RT1", b"NCP18XH103F03RB")?;
        let r36 = self.symbol_entry(b"R36", b"populated")?;
        let r37 = self.symbol_entry(b"R37", b"populated")?;
        let gpio_cfg = self.word_entry(b"GPIO.PRT10.CFG", report.gpio_prt10_cfg)?;
        let gpio_in = self.word_entry(b"GPIO.PRT10.IN", report.gpio_prt10_in)?;
        let hsiom_sel0 = self.word_entry(b"HSIOM.PRT10.SEL0", report.hsiom_prt10_sel0)?;
        let hsiom_sel1 = self.word_entry(b"HSIOM.PRT10.SEL1", report.hsiom_prt10_sel1)?;
        let entries = [
            out0, out1, vdd, gnd, part, r36, r37, gpio_cfg, gpio_in, hsiom_sel0, hsiom_sel1,
        ];
        self.make_list_from_values(&entries)
    }

    fn thermistor_read_report(&mut self, report: ThermistorReadReport) -> LispResult<Value> {
        let status_value = match report.status {
            ThermistorReadStatus::Ready => STATUS_READY,
            ThermistorReadStatus::Timeout => STATUS_TIMEOUT,
        };
        let status = self.symbol_entry(b"status", status_value)?;
        let source = self.symbol_entry(b"source", b"RT1")?;
        let out0 = self.symbol_entry(b"out0", b"P10.1")?;
        let out1 = self.symbol_entry(b"out1", b"P10.2")?;
        let reference = self.symbol_entry(b"reference", b"VDDA")?;
        let reference_mv = self.word_entry(b"reference-mv", report.reference_mv as u32)?;
        let out0_counts = self.word_entry(b"out0.counts", report.out0_counts as u32)?;
        let out1_counts = self.word_entry(b"out1.counts", report.out1_counts as u32)?;
        let out0_mv = self.word_entry(b"out0.mv", report.out0_mv as u32)?;
        let out1_mv = self.word_entry(b"out1.mv", report.out1_mv as u32)?;
        let delta_counts = self.word_entry(b"delta.counts", report.delta_counts as u32)?;
        let powered_after = self.bool_entry(b"powered-after", false)?;
        let out0_poll_count = self.word_entry(b"out0.polls", report.out0_poll_count)?;
        let out1_poll_count = self.word_entry(b"out1.polls", report.out1_poll_count)?;
        let sar_ctrl = self.word_entry(b"SAR.CTRL", report.sar_ctrl)?;
        let sar_sample_ctrl = self.word_entry(b"SAR.SAMPLE_CTRL", report.sar_sample_ctrl)?;
        let sar_chan_config0 = self.word_entry(b"SAR.CHAN_CONFIG0", report.sar_chan_config0)?;
        let sar_chan_en = self.word_entry(b"SAR.CHAN_EN", report.sar_chan_en)?;
        let sar_intr = self.word_entry(b"SAR.INTR", report.sar_intr)?;
        let sar_status = self.word_entry(b"SAR.STATUS", report.sar_status)?;
        let sar_mux_switch0 = self.word_entry(b"SAR.MUX_SWITCH0", report.sar_mux_switch0)?;
        let sar_mux_switch_sq_ctrl =
            self.word_entry(b"SAR.MUX_SWITCH_SQ_CTRL", report.sar_mux_switch_sq_ctrl)?;
        let peri_clock_sar = self.word_entry(b"PERI.CLOCK_CTL.SAR", report.peri_clock_sar)?;
        let peri_div8_sar = self.word_entry(b"PERI.DIV_8_CTL.SAR", report.peri_div8_sar)?;
        let gpio_cfg = self.word_entry(b"GPIO.PRT10.CFG", report.gpio_prt10_cfg)?;
        let gpio_out = self.word_entry(b"GPIO.PRT10.OUT", report.gpio_prt10_out)?;
        let gpio_in = self.word_entry(b"GPIO.PRT10.IN", report.gpio_prt10_in)?;
        let hsiom_sel0 = self.word_entry(b"HSIOM.PRT10.SEL0", report.hsiom_prt10_sel0)?;
        let entries = [
            status,
            source,
            out0,
            out1,
            reference,
            reference_mv,
            out0_counts,
            out1_counts,
            out0_mv,
            out1_mv,
            delta_counts,
            powered_after,
            out0_poll_count,
            out1_poll_count,
            sar_ctrl,
            sar_sample_ctrl,
            sar_chan_config0,
            sar_chan_en,
            sar_intr,
            sar_status,
            sar_mux_switch0,
            sar_mux_switch_sq_ctrl,
            peri_clock_sar,
            peri_div8_sar,
            gpio_cfg,
            gpio_out,
            gpio_in,
            hsiom_sel0,
        ];
        self.make_list_from_values(&entries)
    }

    fn capsense_status_report(&mut self, report: CapsenseStatusReport) -> LispResult<Value> {
        let tx = self.symbol_entry(b"tx", b"P1.0")?;
        let cmod = self.symbol_entry(b"cmod", b"P7.7")?;
        let buttons = self.symbol_entry(b"buttons", b"P8.1-P8.2")?;
        let slider = self.symbol_entry(b"slider", b"P8.3-P8.7")?;
        let gpio_prt1_cfg = self.word_entry(b"GPIO.PRT1.CFG", report.gpio_prt1_cfg)?;
        let gpio_prt7_cfg = self.word_entry(b"GPIO.PRT7.CFG", report.gpio_prt7_cfg)?;
        let gpio_prt8_cfg = self.word_entry(b"GPIO.PRT8.CFG", report.gpio_prt8_cfg)?;
        let gpio_prt8_in = self.word_entry(b"GPIO.PRT8.IN", report.gpio_prt8_in)?;
        let hsiom_prt1_sel0 = self.word_entry(b"HSIOM.PRT1.SEL0", report.hsiom_prt1_sel0)?;
        let hsiom_prt7_sel1 = self.word_entry(b"HSIOM.PRT7.SEL1", report.hsiom_prt7_sel1)?;
        let hsiom_prt8_sel0 = self.word_entry(b"HSIOM.PRT8.SEL0", report.hsiom_prt8_sel0)?;
        let hsiom_prt8_sel1 = self.word_entry(b"HSIOM.PRT8.SEL1", report.hsiom_prt8_sel1)?;
        let csd_config = self.word_entry(b"CSD.CONFIG", report.csd_config)?;
        let csd_status = self.word_entry(b"CSD.STATUS", report.csd_status)?;
        let csd_stat_seq = self.word_entry(b"CSD.STAT_SEQ", report.csd_stat_seq)?;
        let csd_intr_masked = self.word_entry(b"CSD.INTR_MASKED", report.csd_intr_masked)?;
        let entries = [
            tx,
            cmod,
            buttons,
            slider,
            gpio_prt1_cfg,
            gpio_prt7_cfg,
            gpio_prt8_cfg,
            gpio_prt8_in,
            hsiom_prt1_sel0,
            hsiom_prt7_sel1,
            hsiom_prt8_sel0,
            hsiom_prt8_sel1,
            csd_config,
            csd_status,
            csd_stat_seq,
            csd_intr_masked,
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

    fn save_defs_report(
        &mut self,
        status: &'static [u8],
        ready: bool,
        saved: u16,
        skipped: u16,
        bytes: u16,
        chunks: u8,
        truncated: bool,
        write_status: &'static [u8],
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", status)?;
        let ready = self.bool_entry(b"ready", ready)?;
        let saved = self.int_entry(b"saved", saved as i32)?;
        let skipped = self.int_entry(b"skipped", skipped as i32)?;
        let bytes = self.int_entry(b"bytes", bytes as i32)?;
        let chunks = self.int_entry(b"chunks", chunks as i32)?;
        let truncated = self.bool_entry(b"truncated", truncated)?;
        let write_status = self.symbol_entry(b"write.status", write_status)?;
        let entries = [
            status,
            ready,
            saved,
            skipped,
            bytes,
            chunks,
            truncated,
            write_status,
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
        let content = if report.ready && usize::from(report.content.len) <= MAX_STRING_BYTES {
            self.file_bytes_string_value(report.content)?
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

    fn file_bytes_string_value(&mut self, value: FileBytes) -> LispResult<Value> {
        if usize::from(value.len) > MAX_STRING_BYTES {
            return Err(Error::new("file too long for string"));
        }
        self.alloc_string(&value.bytes[..usize::from(value.len)])
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

    fn wifi_sdio_link_status_report(
        &mut self,
        report: WifiSdioLinkStatusReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let mac_status = self.symbol_entry(b"mac.status", report.mac_status)?;
        let mac_present = self.bool_entry(b"mac.present", report.mac_present)?;
        let mac_hash = self.word_entry(b"mac.hash", report.mac_hash)?;
        let bssid_status = self.symbol_entry(b"bssid.status", report.bssid_status)?;
        let bssid_present = self.bool_entry(b"bssid.present", report.bssid_present)?;
        let bssid_hash = self.word_entry(b"bssid.hash", report.bssid_hash)?;
        let rssi_status = self.symbol_entry(b"rssi.status", report.rssi_status)?;
        let rssi = self.int_entry(b"rssi.dbm", report.rssi)?;
        let mac_cdc_status = self.word_entry(b"mac.cdc.status", report.mac_cdc_status)?;
        let bssid_cdc_status = self.word_entry(b"bssid.cdc.status", report.bssid_cdc_status)?;
        let rssi_cdc_status = self.word_entry(b"rssi.cdc.status", report.rssi_cdc_status)?;
        let mac_cdc_length = self.word_entry(b"mac.cdc.length", report.mac_cdc_length)?;
        let bssid_cdc_length = self.word_entry(b"bssid.cdc.length", report.bssid_cdc_length)?;
        let rssi_cdc_length = self.word_entry(b"rssi.cdc.length", report.rssi_cdc_length)?;
        let skipped_frames = self.word_entry(b"skipped.frames", report.skipped_frames as u32)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let entries = [
            status,
            step,
            mac_status,
            mac_present,
            mac_hash,
            bssid_status,
            bssid_present,
            bssid_hash,
            rssi_status,
            rssi,
            mac_cdc_status,
            bssid_cdc_status,
            rssi_cdc_status,
            mac_cdc_length,
            bssid_cdc_length,
            rssi_cdc_length,
            skipped_frames,
            host_normal_int,
            host_error_int,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_dhcp_discover_report(
        &mut self,
        report: WifiSdioDhcpDiscoverReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let mac_status = self.symbol_entry(b"mac.status", report.mac_status)?;
        let mac_present = self.bool_entry(b"mac.present", report.mac_present)?;
        let mac_hash = self.word_entry(b"mac.hash", report.mac_hash)?;
        let mac_cdc_status = self.word_entry(b"mac.cdc.status", report.mac_cdc_status)?;
        let mac_cdc_length = self.word_entry(b"mac.cdc.length", report.mac_cdc_length)?;
        let skipped_frames = self.word_entry(b"skipped.frames", report.skipped_frames as u32)?;
        let transaction_id = self.word_entry(b"dhcp.transaction-id", report.transaction_id)?;
        let ethernet_length = self.word_entry(b"ethernet.length", report.ethernet_length as u32)?;
        let ethernet_hash = self.word_entry(b"ethernet.hash", report.ethernet_hash)?;
        let ip_total_length = self.word_entry(b"ip.total-length", report.ip_total_length as u32)?;
        let udp_length = self.word_entry(b"udp.length", report.udp_length as u32)?;
        let dhcp_payload_length =
            self.word_entry(b"dhcp.payload-length", report.dhcp_payload_length as u32)?;
        let send_status = self.symbol_entry(b"send.status", report.send_status)?;
        let send_initial_tx_credit = self.word_entry(
            b"send.initial-tx-credit",
            report.send_initial_tx_credit as u32,
        )?;
        let send_packet_length =
            self.word_entry(b"send.packet-length", report.send_packet_length as u32)?;
        let send_write_response =
            self.word_entry(b"send.write-response", report.send_write_response)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let send_last_error =
            self.wifi_sdio_error_entry(b"send.last-error", report.send_last_error)?;
        let entries = [
            status,
            step,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            mac_status,
            mac_present,
            mac_hash,
            mac_cdc_status,
            mac_cdc_length,
            skipped_frames,
            transaction_id,
            ethernet_length,
            ethernet_hash,
            ip_total_length,
            udp_length,
            dhcp_payload_length,
            send_status,
            send_initial_tx_credit,
            send_packet_length,
            send_write_response,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_dhcp_acquire_report(
        &mut self,
        report: WifiSdioDhcpAcquireReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let mac_status = self.symbol_entry(b"mac.status", report.mac_status)?;
        let mac_present = self.bool_entry(b"mac.present", report.mac_present)?;
        let mac_hash = self.word_entry(b"mac.hash", report.mac_hash)?;
        let mac_cdc_status = self.word_entry(b"mac.cdc.status", report.mac_cdc_status)?;
        let mac_cdc_length = self.word_entry(b"mac.cdc.length", report.mac_cdc_length)?;
        let transaction_id = self.word_entry(b"dhcp.transaction-id", report.transaction_id)?;
        let discover_status = self.symbol_entry(b"discover.status", report.discover_status)?;
        let discover_packet_length = self.word_entry(
            b"discover.packet-length",
            report.discover_packet_length as u32,
        )?;
        let discover_write_response =
            self.word_entry(b"discover.write-response", report.discover_write_response)?;
        let offer_poll_status =
            self.symbol_entry(b"offer.poll.status", report.offer_poll_status)?;
        let offer_parse_status =
            self.symbol_entry(b"offer.parse.status", report.offer_parse_status)?;
        let offer_polls = self.word_entry(b"offer.polls", report.offer_polls as u32)?;
        let offer_frames_read =
            self.word_entry(b"offer.frames-read", report.offer_frames_read as u32)?;
        let offer_non_data_frames = self.word_entry(
            b"offer.non-data-frames",
            report.offer_non_data_frames as u32,
        )?;
        let offer_non_dhcp_frames = self.word_entry(
            b"offer.non-dhcp-frames",
            report.offer_non_dhcp_frames as u32,
        )?;
        let offer_message_type =
            self.word_entry(b"offer.message-type", report.offer_message_type as u32)?;
        let offered_ip_address = self.word_entry(b"offer.ip", report.offered_ip_address)?;
        let server_identifier = self.word_entry(b"dhcp.server", report.server_identifier)?;
        let request_status = self.symbol_entry(b"request.status", report.request_status)?;
        let request_packet_length = self.word_entry(
            b"request.packet-length",
            report.request_packet_length as u32,
        )?;
        let request_write_response =
            self.word_entry(b"request.write-response", report.request_write_response)?;
        let ack_poll_status = self.symbol_entry(b"ack.poll.status", report.ack_poll_status)?;
        let ack_parse_status = self.symbol_entry(b"ack.parse.status", report.ack_parse_status)?;
        let ack_polls = self.word_entry(b"ack.polls", report.ack_polls as u32)?;
        let ack_frames_read = self.word_entry(b"ack.frames-read", report.ack_frames_read as u32)?;
        let ack_non_data_frames =
            self.word_entry(b"ack.non-data-frames", report.ack_non_data_frames as u32)?;
        let ack_non_dhcp_frames =
            self.word_entry(b"ack.non-dhcp-frames", report.ack_non_dhcp_frames as u32)?;
        let ack_message_type =
            self.word_entry(b"ack.message-type", report.ack_message_type as u32)?;
        let leased_ip_address = self.word_entry(b"lease.ip", report.leased_ip_address)?;
        let subnet_mask = self.word_entry(b"lease.subnet-mask", report.subnet_mask)?;
        let router = self.word_entry(b"lease.router", report.router)?;
        let dns_server = self.word_entry(b"lease.dns", report.dns_server)?;
        let lease_seconds = self.word_entry(b"lease.seconds", report.lease_seconds)?;
        let lease_valid = self.bool_entry(b"lease.valid", report.lease_valid)?;
        let ack_status = self.symbol_entry(b"ack.status", report.ack_status)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let discover_last_error =
            self.wifi_sdio_error_entry(b"discover.last-error", report.discover_last_error)?;
        let request_last_error =
            self.wifi_sdio_error_entry(b"request.last-error", report.request_last_error)?;
        let frame_last_error =
            self.wifi_sdio_error_entry(b"frame.last-error", report.frame_last_error)?;
        let ack_last_error =
            self.wifi_sdio_error_entry(b"ack.last-error", report.ack_last_error)?;
        let entries = [
            status,
            step,
            ht_status,
            ht_attempts,
            ht_available,
            mac_status,
            mac_present,
            mac_hash,
            mac_cdc_status,
            mac_cdc_length,
            transaction_id,
            discover_status,
            discover_packet_length,
            discover_write_response,
            offer_poll_status,
            offer_parse_status,
            offer_polls,
            offer_frames_read,
            offer_non_data_frames,
            offer_non_dhcp_frames,
            offer_message_type,
            offered_ip_address,
            server_identifier,
            request_status,
            request_packet_length,
            request_write_response,
            ack_poll_status,
            ack_parse_status,
            ack_polls,
            ack_frames_read,
            ack_non_data_frames,
            ack_non_dhcp_frames,
            ack_message_type,
            leased_ip_address,
            subnet_mask,
            router,
            dns_server,
            lease_seconds,
            lease_valid,
            ack_status,
            host_normal_int,
            host_error_int,
            ht_last_error,
            discover_last_error,
            request_last_error,
            frame_last_error,
            ack_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_lease_status_report(
        &mut self,
        report: WifiSdioLeaseStatusReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let lease_valid = self.bool_entry(b"lease.valid", report.lease_valid)?;
        let transaction_id = self.word_entry(b"dhcp.transaction-id", report.transaction_id)?;
        let ip_address = self.word_entry(b"lease.ip", report.ip_address)?;
        let subnet_mask = self.word_entry(b"lease.subnet-mask", report.subnet_mask)?;
        let router = self.word_entry(b"lease.router", report.router)?;
        let dns_server = self.word_entry(b"lease.dns", report.dns_server)?;
        let server_identifier = self.word_entry(b"dhcp.server", report.server_identifier)?;
        let lease_seconds = self.word_entry(b"lease.seconds", report.lease_seconds)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let entries = [
            status,
            lease_valid,
            transaction_id,
            ip_address,
            subnet_mask,
            router,
            dns_server,
            server_identifier,
            lease_seconds,
            host_normal_int,
            host_error_int,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_arp_router_report(
        &mut self,
        report: WifiSdioArpRouterReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let lease_valid = self.bool_entry(b"lease.valid", report.lease_valid)?;
        let local_ip_address = self.word_entry(b"lease.ip", report.local_ip_address)?;
        let router_ip_address = self.word_entry(b"lease.router", report.router_ip_address)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let mac_status = self.symbol_entry(b"mac.status", report.mac_status)?;
        let mac_hash = self.word_entry(b"mac.hash", report.mac_hash)?;
        let mac_present = self.bool_entry(b"mac.present", report.mac_present)?;
        let mac_cdc_status = self.word_entry(b"mac.cdc.status", report.mac_cdc_status)?;
        let mac_cdc_length = self.word_entry(b"mac.cdc.length", report.mac_cdc_length)?;
        let request_status = self.symbol_entry(b"request.status", report.request_status)?;
        let request_ethernet_length = self.word_entry(
            b"request.ethernet-length",
            report.request_ethernet_length as u32,
        )?;
        let request_ethernet_hash =
            self.word_entry(b"request.ethernet-hash", report.request_ethernet_hash)?;
        let request_packet_length = self.word_entry(
            b"request.packet-length",
            report.request_packet_length as u32,
        )?;
        let request_write_response =
            self.word_entry(b"request.write-response", report.request_write_response)?;
        let reply_poll_status =
            self.symbol_entry(b"reply.poll.status", report.reply_poll_status)?;
        let reply_parse_status =
            self.symbol_entry(b"reply.parse.status", report.reply_parse_status)?;
        let reply_polls = self.word_entry(b"reply.polls", report.reply_polls as u32)?;
        let reply_frames_read =
            self.word_entry(b"reply.frames-read", report.reply_frames_read as u32)?;
        let reply_non_data_frames = self.word_entry(
            b"reply.non-data-frames",
            report.reply_non_data_frames as u32,
        )?;
        let reply_non_arp_frames =
            self.word_entry(b"reply.non-arp-frames", report.reply_non_arp_frames as u32)?;
        let router_mac_hash = self.word_entry(b"router.mac.hash", report.router_mac_hash)?;
        let router_mac_present =
            self.bool_entry(b"router.mac.present", report.router_mac_present)?;
        let router_mac_stored = self.bool_entry(b"router.mac.stored", report.router_mac_stored)?;
        let ack_status = self.symbol_entry(b"ack.status", report.ack_status)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ack_last_error =
            self.wifi_sdio_error_entry(b"ack.last-error", report.ack_last_error)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let request_last_error =
            self.wifi_sdio_error_entry(b"request.last-error", report.request_last_error)?;
        let frame_last_error =
            self.wifi_sdio_error_entry(b"frame.last-error", report.frame_last_error)?;
        let entries = [
            status,
            step,
            lease_valid,
            local_ip_address,
            router_ip_address,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            mac_status,
            mac_hash,
            mac_present,
            mac_cdc_status,
            mac_cdc_length,
            request_status,
            request_ethernet_length,
            request_ethernet_hash,
            request_packet_length,
            request_write_response,
            reply_poll_status,
            reply_parse_status,
            reply_polls,
            reply_frames_read,
            reply_non_data_frames,
            reply_non_arp_frames,
            router_mac_hash,
            router_mac_present,
            router_mac_stored,
            ack_status,
            host_normal_int,
            host_error_int,
            ack_last_error,
            ht_last_error,
            request_last_error,
            frame_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_dns_query_report(&mut self, report: WifiSdioDnsQueryReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let lease_valid = self.bool_entry(b"lease.valid", report.lease_valid)?;
        let local_ip_address = self.word_entry(b"lease.ip", report.local_ip_address)?;
        let dns_server_ip_address = self.word_entry(b"lease.dns", report.dns_server_ip_address)?;
        let router_ip_address = self.word_entry(b"lease.router", report.router_ip_address)?;
        let router_mac_present =
            self.bool_entry(b"router.mac.present", report.router_mac_present)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let mac_status = self.symbol_entry(b"mac.status", report.mac_status)?;
        let mac_hash = self.word_entry(b"mac.hash", report.mac_hash)?;
        let mac_present = self.bool_entry(b"mac.present", report.mac_present)?;
        let mac_cdc_status = self.word_entry(b"mac.cdc.status", report.mac_cdc_status)?;
        let mac_cdc_length = self.word_entry(b"mac.cdc.length", report.mac_cdc_length)?;
        let transaction_id =
            self.word_entry(b"dns.transaction-id", report.transaction_id as u32)?;
        let query_name_length =
            self.word_entry(b"query.name-length", report.query_name_length as u32)?;
        let query_payload_length =
            self.word_entry(b"query.payload-length", report.query_payload_length as u32)?;
        let query_payload_hash =
            self.word_entry(b"query.payload-hash", report.query_payload_hash)?;
        let request_status = self.symbol_entry(b"request.status", report.request_status)?;
        let request_ethernet_length = self.word_entry(
            b"request.ethernet-length",
            report.request_ethernet_length as u32,
        )?;
        let request_ethernet_hash =
            self.word_entry(b"request.ethernet-hash", report.request_ethernet_hash)?;
        let request_packet_length = self.word_entry(
            b"request.packet-length",
            report.request_packet_length as u32,
        )?;
        let request_write_response =
            self.word_entry(b"request.write-response", report.request_write_response)?;
        let response_poll_status =
            self.symbol_entry(b"response.poll.status", report.response_poll_status)?;
        let response_parse_status =
            self.symbol_entry(b"response.parse.status", report.response_parse_status)?;
        let response_polls = self.word_entry(b"response.polls", report.response_polls as u32)?;
        let response_frames_read =
            self.word_entry(b"response.frames-read", report.response_frames_read as u32)?;
        let response_non_data_frames = self.word_entry(
            b"response.non-data-frames",
            report.response_non_data_frames as u32,
        )?;
        let response_non_dns_frames = self.word_entry(
            b"response.non-dns-frames",
            report.response_non_dns_frames as u32,
        )?;
        let response_answer_count = self.word_entry(
            b"response.answer-count",
            report.response_answer_count as u32,
        )?;
        let answer_ip_address = self.word_entry(b"answer.ip", report.answer_ip_address)?;
        let answer_ttl_seconds = self.word_entry(b"answer.ttl", report.answer_ttl_seconds)?;
        let answer_valid = self.bool_entry(b"answer.valid", report.answer_valid)?;
        let ack_status = self.symbol_entry(b"ack.status", report.ack_status)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ack_last_error =
            self.wifi_sdio_error_entry(b"ack.last-error", report.ack_last_error)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let request_last_error =
            self.wifi_sdio_error_entry(b"request.last-error", report.request_last_error)?;
        let frame_last_error =
            self.wifi_sdio_error_entry(b"frame.last-error", report.frame_last_error)?;
        let entries = [
            status,
            step,
            lease_valid,
            local_ip_address,
            dns_server_ip_address,
            router_ip_address,
            router_mac_present,
            ht_status,
            ht_attempts,
            ht_write_response,
            ht_read_value,
            ht_read_response,
            ht_available,
            mac_status,
            mac_hash,
            mac_present,
            mac_cdc_status,
            mac_cdc_length,
            transaction_id,
            query_name_length,
            query_payload_length,
            query_payload_hash,
            request_status,
            request_ethernet_length,
            request_ethernet_hash,
            request_packet_length,
            request_write_response,
            response_poll_status,
            response_parse_status,
            response_polls,
            response_frames_read,
            response_non_data_frames,
            response_non_dns_frames,
            response_answer_count,
            answer_ip_address,
            answer_ttl_seconds,
            answer_valid,
            ack_status,
            host_normal_int,
            host_error_int,
            ack_last_error,
            ht_last_error,
            request_last_error,
            frame_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_tcp_syn_report(&mut self, report: WifiSdioTcpSynReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let local_ip_address = self.word_entry(b"local.ip", report.local_ip_address)?;
        let remote_ip_address = self.word_entry(b"remote.ip", report.remote_ip_address)?;
        let remote_port = self.word_entry(b"remote.port", report.remote_port as u32)?;
        let source_port = self.word_entry(b"source.port", report.source_port as u32)?;
        let local_sequence = self.word_entry(b"local.seq", report.local_sequence)?;
        let remote_sequence = self.word_entry(b"remote.seq", report.remote_sequence)?;
        let ack_number = self.word_entry(b"ack", report.ack_number)?;
        let response_flags = self.word_entry(b"flags", report.response_flags as u32)?;
        let dns_status = self.symbol_entry(b"dns.status", report.dns_status)?;
        let request_status = self.symbol_entry(b"request.status", report.request_status)?;
        let response_poll_status =
            self.symbol_entry(b"response.status", report.response_poll_status)?;
        let response_parse_status =
            self.symbol_entry(b"response.parse", report.response_parse_status)?;
        let response_polls = self.word_entry(b"response.polls", report.response_polls as u32)?;
        let reset_status = self.symbol_entry(b"reset.status", report.reset_status)?;
        let entries = [
            status,
            step,
            local_ip_address,
            remote_ip_address,
            remote_port,
            source_port,
            local_sequence,
            remote_sequence,
            ack_number,
            response_flags,
            dns_status,
            request_status,
            response_poll_status,
            response_parse_status,
            response_polls,
            reset_status,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_tcp_listen_report(
        &mut self,
        report: WifiSdioTcpListenReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let local_ip_address = self.word_entry(b"local.ip", report.local_ip_address)?;
        let listen_port = self.word_entry(b"listen.port", report.listen_port as u32)?;
        let peer_ip_address = self.word_entry(b"peer.ip", report.peer_ip_address)?;
        let peer_port = self.word_entry(b"peer.port", report.peer_port as u32)?;
        let peer_sequence = self.word_entry(b"peer.seq", report.peer_sequence)?;
        let ack_number = self.word_entry(b"ack", report.ack_number)?;
        let flags = self.word_entry(b"flags", report.flags as u32)?;
        let listen_poll_status = self.symbol_entry(b"listen.status", report.listen_poll_status)?;
        let listen_parse_status = self.symbol_entry(b"listen.parse", report.listen_parse_status)?;
        let listen_polls = self.word_entry(b"listen.polls", report.listen_polls as u32)?;
        let listen_frames_read =
            self.word_entry(b"listen.frames-read", report.listen_frames_read as u32)?;
        let syn_ack_status = self.symbol_entry(b"syn-ack.status", report.syn_ack_status)?;
        let ack_poll_status = self.symbol_entry(b"ack.status", report.ack_poll_status)?;
        let ack_parse_status = self.symbol_entry(b"ack.parse", report.ack_parse_status)?;
        let ack_polls = self.word_entry(b"ack.polls", report.ack_polls as u32)?;
        let ack_frames_read = self.word_entry(b"ack.frames-read", report.ack_frames_read as u32)?;
        let reset_status = self.symbol_entry(b"reset.status", report.reset_status)?;
        let interrupt_ack_status =
            self.symbol_entry(b"interrupt.ack.status", report.interrupt_ack_status)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;

        let entries = [
            status,
            step,
            local_ip_address,
            listen_port,
            peer_ip_address,
            peer_port,
            peer_sequence,
            ack_number,
            flags,
            listen_poll_status,
            listen_parse_status,
            listen_polls,
            listen_frames_read,
            syn_ack_status,
            ack_poll_status,
            ack_parse_status,
            ack_polls,
            ack_frames_read,
            reset_status,
            interrupt_ack_status,
            host_normal_int,
            host_error_int,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_tcp_receive_report(
        &mut self,
        report: WifiSdioTcpReceiveReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let local_ip_address = self.word_entry(b"local.ip", report.local_ip_address)?;
        let listen_port = self.word_entry(b"listen.port", report.listen_port as u32)?;
        let peer_ip_address = self.word_entry(b"peer.ip", report.peer_ip_address)?;
        let peer_port = self.word_entry(b"peer.port", report.peer_port as u32)?;
        let peer_sequence = self.word_entry(b"peer.seq", report.peer_sequence)?;
        let ack_number = self.word_entry(b"ack", report.ack_number)?;
        let flags = self.word_entry(b"flags", report.flags as u32)?;
        let listen_poll_status = self.symbol_entry(b"listen.status", report.listen_poll_status)?;
        let listen_parse_status = self.symbol_entry(b"listen.parse", report.listen_parse_status)?;
        let listen_polls = self.word_entry(b"listen.polls", report.listen_polls as u32)?;
        let listen_frames_read =
            self.word_entry(b"listen.frames-read", report.listen_frames_read as u32)?;
        let syn_ack_status = self.symbol_entry(b"syn-ack.status", report.syn_ack_status)?;
        let ack_poll_status = self.symbol_entry(b"ack.status", report.ack_poll_status)?;
        let ack_parse_status = self.symbol_entry(b"ack.parse", report.ack_parse_status)?;
        let ack_polls = self.word_entry(b"ack.polls", report.ack_polls as u32)?;
        let ack_frames_read = self.word_entry(b"ack.frames-read", report.ack_frames_read as u32)?;
        let payload_poll_status =
            self.symbol_entry(b"payload.status", report.payload_poll_status)?;
        let payload_parse_status =
            self.symbol_entry(b"payload.parse", report.payload_parse_status)?;
        let payload_polls = self.word_entry(b"payload.polls", report.payload_polls as u32)?;
        let payload_frames_read =
            self.word_entry(b"payload.frames-read", report.payload_frames_read as u32)?;
        let payload_bytes = self.word_entry(b"payload.bytes", report.payload_bytes as u32)?;
        let payload_hash = self.word_entry(b"payload.hash", report.payload_hash)?;
        let preview_value = self.string_value(report.payload_preview)?;
        let preview = self.entry(b"preview", preview_value)?;
        let reset_status = self.symbol_entry(b"reset.status", report.reset_status)?;
        let interrupt_ack_status =
            self.symbol_entry(b"interrupt.ack.status", report.interrupt_ack_status)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;

        let entries = [
            status,
            step,
            local_ip_address,
            listen_port,
            peer_ip_address,
            peer_port,
            peer_sequence,
            ack_number,
            flags,
            listen_poll_status,
            listen_parse_status,
            listen_polls,
            listen_frames_read,
            syn_ack_status,
            ack_poll_status,
            ack_parse_status,
            ack_polls,
            ack_frames_read,
            payload_poll_status,
            payload_parse_status,
            payload_polls,
            payload_frames_read,
            payload_bytes,
            payload_hash,
            preview,
            reset_status,
            interrupt_ack_status,
            host_normal_int,
            host_error_int,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_tcp_repl_once<B: Board>(
        &mut self,
        board: &mut B,
        port: u16,
        poll_frames: u8,
    ) -> LispResult<Value> {
        let cycle = self.run_wifi_tcp_repl_cycle(board, port, poll_frames);
        self.wifi_tcp_repl_once_report(
            cycle.status,
            cycle.request,
            cycle.reply,
            cycle.eval_status,
            cycle.response_length,
            cycle.response_hash,
            cycle.response_truncated,
        )
    }

    fn run_wifi_tcp_repl_cycle<B: Board>(
        &mut self,
        board: &mut B,
        port: u16,
        poll_frames: u8,
    ) -> WifiTcpReplCycleReport {
        let request = board.wifi_tcp_repl_poll(port, poll_frames);
        if !status_ready(request.status) {
            return WifiTcpReplCycleReport {
                status: request.status,
                request,
                reply: None,
                eval_status: STATUS_NOT_RUN,
                response_length: 0,
                response_hash: 0,
                response_truncated: false,
            };
        }

        let (eval_status, response) = self.eval_net_repl_payload(
            &request.payload[..request.payload_bytes as usize],
            false,
            board,
        );
        let response_length = response.len;
        let response_truncated = response.truncated;
        let response_hash = checksum_bytes(&response.bytes[..response.len as usize]);
        let reply = board.wifi_tcp_repl_reply(response);
        let status: &'static [u8] = if status_ready(reply.status) {
            b"ready"
        } else {
            b"reply-failed"
        };

        WifiTcpReplCycleReport {
            status,
            request,
            reply: Some(reply),
            eval_status,
            response_length,
            response_hash,
            response_truncated,
        }
    }

    fn run_wifi_telnet_repl_service_cycle<B: Board>(
        &mut self,
        board: &mut B,
        port: u16,
        poll_frames: u8,
    ) -> WifiTcpReplCycleReport {
        let request = board.wifi_tcp_repl_service_poll(port, poll_frames);
        if request.status == STATUS_LISTEN_TIMEOUT || request.status == STATUS_ACK_ONLY {
            return self.wifi_tcp_repl_cycle_without_reply(request.status, request);
        }
        if request.status == STATUS_PEER_RESET || request.status == STATUS_PEER_CLOSED {
            self.reset_telnet_repl_connection();
            return self.wifi_tcp_repl_cycle_without_reply(request.status, request);
        }

        if request.status == STATUS_CONNECTED {
            self.reset_telnet_repl_connection();
            let response = self.telnet_prompt_response();
            return self.send_wifi_telnet_repl_response(board, request, response, STATUS_NOT_RUN);
        }

        if !status_ready(request.status) {
            return self.wifi_tcp_repl_cycle_without_reply(request.status, request);
        }

        let payload_len = request.payload_bytes as usize;
        self.wifi_tcp_repl_service.processing_telnet_request = true;
        let (eval_status, response) =
            self.process_telnet_repl_payload(&request.payload[..payload_len], board);
        self.wifi_tcp_repl_service.processing_telnet_request = false;
        let should_ack = payload_len > 0 && response.len == 0;
        if response.len == 0 && !should_ack {
            return WifiTcpReplCycleReport {
                status: STATUS_LINE_PENDING,
                request,
                reply: None,
                eval_status,
                response_length: 0,
                response_hash: 0,
                response_truncated: false,
            };
        }

        self.send_wifi_telnet_repl_response(board, request, response, eval_status)
    }

    fn wifi_tcp_repl_cycle_without_reply(
        &self,
        status: &'static [u8],
        request: WifiSdioTcpReceiveReport,
    ) -> WifiTcpReplCycleReport {
        WifiTcpReplCycleReport {
            status,
            request,
            reply: None,
            eval_status: STATUS_NOT_RUN,
            response_length: 0,
            response_hash: 0,
            response_truncated: false,
        }
    }

    fn send_wifi_telnet_repl_response<B: Board>(
        &mut self,
        board: &mut B,
        request: WifiSdioTcpReceiveReport,
        response: NetReplResponseBytes,
        eval_status: &'static [u8],
    ) -> WifiTcpReplCycleReport {
        let response_length = response.len;
        let response_truncated = response.truncated;
        let response_hash = checksum_bytes(&response.bytes[..response.len as usize]);
        let reply = if request.flags & TCP_FLAG_FIN != 0 {
            board.wifi_tcp_repl_reply(response)
        } else {
            board.wifi_tcp_repl_service_send(response)
        };
        if self.wifi_tcp_repl_service.reset_peer_after_reply {
            self.wifi_tcp_repl_service.reset_peer_after_reply = false;
            board.wifi_tcp_repl_service_reset();
        }
        let status: &'static [u8] = if status_ready(reply.status) {
            if eval_status == STATUS_NOT_RUN {
                STATUS_PROTOCOL
            } else {
                STATUS_READY
            }
        } else {
            b"reply-failed"
        };

        WifiTcpReplCycleReport {
            status,
            request,
            reply: Some(reply),
            eval_status,
            response_length,
            response_hash,
            response_truncated,
        }
    }

    fn reset_telnet_repl_connection(&mut self) {
        self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
        self.wifi_tcp_repl_service.telnet_command = 0;
        self.wifi_tcp_repl_service.telnet_line_len = 0;
        self.wifi_tcp_repl_service.telnet_line = [0; NET_REPL_REQUEST_PAYLOAD_BYTES];
    }

    fn reset_wifi_tcp_repl_service_peer<B: Board>(&mut self, board: &mut B) {
        if self.wifi_tcp_repl_service.processing_telnet_request {
            self.wifi_tcp_repl_service.reset_peer_after_reply = true;
        } else {
            self.wifi_tcp_repl_service.reset_peer_after_reply = false;
            board.wifi_tcp_repl_service_reset();
        }
    }

    fn telnet_prompt_response(&self) -> NetReplResponseBytes {
        let mut builder = TelnetResponseBuilder::new();
        builder.append_nvt_text(TELNET_PROMPT);
        builder.response()
    }

    fn process_telnet_repl_payload<B: Board>(
        &mut self,
        payload: &[u8],
        board: &mut B,
    ) -> (&'static [u8], NetReplResponseBytes) {
        let mut builder = TelnetResponseBuilder::new();
        let mut eval_status = STATUS_NOT_RUN;

        for byte in payload {
            if self.telnet_consume_byte(*byte, &mut builder) {
                let line_len = self.wifi_tcp_repl_service.telnet_line_len as usize;
                if line_len == 0 {
                    builder.append_nvt_text(TELNET_PROMPT);
                    continue;
                }

                let line = self.wifi_tcp_repl_service.telnet_line;
                self.wifi_tcp_repl_service.telnet_line_len = 0;
                let (line_status, line_response) =
                    self.eval_net_repl_payload(&line[..line_len], false, board);
                eval_status = line_status;
                builder.append_lisp_response(&line_response);
                builder.append_nvt_text(TELNET_PROMPT);
            }
        }

        (eval_status, builder.response())
    }

    fn telnet_consume_byte(&mut self, byte: u8, builder: &mut TelnetResponseBuilder) -> bool {
        match self.wifi_tcp_repl_service.telnet_state {
            TELNET_STATE_DATA => self.telnet_consume_data_byte(byte, builder),
            TELNET_STATE_IAC => {
                self.telnet_consume_iac_command(byte, builder);
                false
            }
            TELNET_STATE_OPTION => {
                self.telnet_consume_option(byte, builder);
                false
            }
            TELNET_STATE_SUBNEGOTIATION => {
                if byte == TELNET_IAC {
                    self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_SUBNEGOTIATION_IAC;
                }
                false
            }
            TELNET_STATE_SUBNEGOTIATION_IAC => {
                self.wifi_tcp_repl_service.telnet_state = if byte == TELNET_SE {
                    TELNET_STATE_DATA
                } else {
                    TELNET_STATE_SUBNEGOTIATION
                };
                false
            }
            TELNET_STATE_CR => {
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
                if byte == b'\n' {
                    true
                } else if byte == 0 {
                    self.telnet_append_line_byte(b'\r', builder);
                    false
                } else {
                    self.telnet_append_line_byte(b'\r', builder);
                    self.telnet_consume_data_byte(byte, builder)
                }
            }
            _ => {
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
                false
            }
        }
    }

    fn telnet_consume_data_byte(&mut self, byte: u8, builder: &mut TelnetResponseBuilder) -> bool {
        match byte {
            TELNET_IAC => {
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_IAC;
                false
            }
            b'\r' => {
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_CR;
                false
            }
            b'\n' => true,
            0 => false,
            byte => {
                self.telnet_append_line_byte(byte, builder);
                false
            }
        }
    }

    fn telnet_append_line_byte(&mut self, byte: u8, builder: &mut TelnetResponseBuilder) {
        let len = self.wifi_tcp_repl_service.telnet_line_len as usize;
        if len < self.wifi_tcp_repl_service.telnet_line.len() {
            self.wifi_tcp_repl_service.telnet_line[len] = byte;
            self.wifi_tcp_repl_service.telnet_line_len =
                self.wifi_tcp_repl_service.telnet_line_len.saturating_add(1);
        } else {
            self.wifi_tcp_repl_service.telnet_line_len = 0;
            builder.append_nvt_text(b"error: telnet line too long\n");
            builder.append_nvt_text(TELNET_PROMPT);
        }
    }

    fn telnet_consume_iac_command(&mut self, command: u8, builder: &mut TelnetResponseBuilder) {
        match command {
            TELNET_IAC => {
                self.telnet_append_line_byte(TELNET_IAC, builder);
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
            }
            TELNET_DO | TELNET_DONT | TELNET_WILL | TELNET_WONT => {
                self.wifi_tcp_repl_service.telnet_command = command;
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_OPTION;
            }
            TELNET_SB => {
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_SUBNEGOTIATION;
            }
            TELNET_AYT => {
                builder.append_nvt_text(TELNET_AYT_RESPONSE);
                builder.append_nvt_text(TELNET_PROMPT);
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
            }
            TELNET_EC => {
                self.wifi_tcp_repl_service.telnet_line_len =
                    self.wifi_tcp_repl_service.telnet_line_len.saturating_sub(1);
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
            }
            TELNET_EL => {
                self.wifi_tcp_repl_service.telnet_line_len = 0;
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
            }
            _ => {
                self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
            }
        }
    }

    fn telnet_consume_option(&mut self, option: u8, builder: &mut TelnetResponseBuilder) {
        match self.wifi_tcp_repl_service.telnet_command {
            TELNET_DO => {
                builder.append_control(TELNET_IAC);
                builder.append_control(TELNET_WONT);
                builder.append_control(option);
            }
            TELNET_WILL => {
                builder.append_control(TELNET_IAC);
                builder.append_control(TELNET_DONT);
                builder.append_control(option);
            }
            _ => {}
        }
        self.wifi_tcp_repl_service.telnet_command = 0;
        self.wifi_tcp_repl_service.telnet_state = TELNET_STATE_DATA;
    }

    fn wifi_tcp_repl_once_report(
        &mut self,
        status_value: &'static [u8],
        request: WifiSdioTcpReceiveReport,
        reply: Option<WifiSdioTcpReplReplyReport>,
        eval_status_value: &'static [u8],
        response_length_value: u16,
        response_hash_value: u32,
        response_truncated_value: bool,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", status_value)?;
        let request_status = self.symbol_entry(b"request.status", request.status)?;
        let request_step = self.symbol_entry(b"request.step", request.step)?;
        let local_ip_address = self.word_entry(b"local.ip", request.local_ip_address)?;
        let listen_port = self.word_entry(b"listen.port", request.listen_port as u32)?;
        let peer_ip_address = self.word_entry(b"request.peer.ip", request.peer_ip_address)?;
        let peer_port = self.word_entry(b"request.peer.port", request.peer_port as u32)?;
        let listen_status =
            self.symbol_entry(b"request.listen.status", request.listen_poll_status)?;
        let ack_status = self.symbol_entry(b"request.ack.status", request.ack_poll_status)?;
        let payload_status =
            self.symbol_entry(b"request.payload.status", request.payload_poll_status)?;
        let payload_parse =
            self.symbol_entry(b"request.payload.parse", request.payload_parse_status)?;
        let payload_bytes =
            self.word_entry(b"request.payload.bytes", request.payload_bytes as u32)?;
        let payload_hash = self.word_entry(b"request.payload.hash", request.payload_hash)?;
        let eval_status = self.symbol_entry(b"eval.status", eval_status_value)?;
        let response_length = self.word_entry(b"response.length", response_length_value as u32)?;
        let response_hash = self.word_entry(b"response.hash", response_hash_value)?;
        let response_truncated =
            self.bool_entry(b"response.truncated", response_truncated_value)?;

        let (
            reply_status_value,
            reply_step_value,
            reply_peer_valid_value,
            reply_peer_ip_value,
            reply_peer_port_value,
            reply_peer_mac_hash_value,
            reply_payload_length_value,
            reply_payload_hash_value,
            reply_ethernet_length_value,
            reply_ethernet_hash_value,
            reply_send_status_value,
            reply_send_packet_length_value,
            reply_send_write_response_value,
            reply_host_normal_int_value,
            reply_host_error_int_value,
            reply_ht_last_error_value,
            reply_send_last_error_value,
        ) = match reply {
            Some(reply) => (
                reply.status,
                reply.step,
                reply.peer_valid,
                reply.peer_ip_address,
                reply.peer_port,
                reply.peer_mac_hash,
                reply.payload_length,
                reply.payload_hash,
                reply.ethernet_length,
                reply.ethernet_hash,
                reply.send_status,
                reply.send_packet_length,
                reply.send_write_response,
                reply.host_normal_int,
                reply.host_error_int,
                reply.ht_last_error,
                reply.send_last_error,
            ),
            None => (
                STATUS_NOT_RUN,
                STATUS_NOT_RUN,
                false,
                0u32,
                0u16,
                0u32,
                0u16,
                0u32,
                0u16,
                0u32,
                STATUS_NOT_RUN,
                0u16,
                0u32,
                0u16,
                0u16,
                None,
                None,
            ),
        };

        let reply_status = self.symbol_entry(b"reply.status", reply_status_value)?;
        let reply_step = self.symbol_entry(b"reply.step", reply_step_value)?;
        let reply_peer_valid = self.bool_entry(b"reply.peer.valid", reply_peer_valid_value)?;
        let reply_peer_ip = self.word_entry(b"reply.peer.ip", reply_peer_ip_value)?;
        let reply_peer_port = self.word_entry(b"reply.peer.port", reply_peer_port_value as u32)?;
        let reply_peer_mac_hash =
            self.word_entry(b"reply.peer.mac-hash", reply_peer_mac_hash_value)?;
        let reply_payload_length =
            self.word_entry(b"reply.payload.length", reply_payload_length_value as u32)?;
        let reply_payload_hash =
            self.word_entry(b"reply.payload.hash", reply_payload_hash_value)?;
        let reply_ethernet_length =
            self.word_entry(b"reply.ethernet.length", reply_ethernet_length_value as u32)?;
        let reply_ethernet_hash =
            self.word_entry(b"reply.ethernet.hash", reply_ethernet_hash_value)?;
        let reply_send_status = self.symbol_entry(b"reply.send.status", reply_send_status_value)?;
        let reply_send_packet_length = self.word_entry(
            b"reply.send.packet-length",
            reply_send_packet_length_value as u32,
        )?;
        let reply_send_write_response = self.word_entry(
            b"reply.send.write-response",
            reply_send_write_response_value,
        )?;
        let request_host_normal_int =
            self.word_entry(b"request.HOST.NORM_INT", request.host_normal_int as u32)?;
        let request_host_error_int =
            self.word_entry(b"request.HOST.ERR_INT", request.host_error_int as u32)?;
        let reply_host_normal_int =
            self.word_entry(b"reply.HOST.NORM_INT", reply_host_normal_int_value as u32)?;
        let reply_host_error_int =
            self.word_entry(b"reply.HOST.ERR_INT", reply_host_error_int_value as u32)?;
        let request_ack_last_error =
            self.wifi_sdio_error_entry(b"request.ack.last-error", request.ack_last_error)?;
        let request_payload_last_error =
            self.wifi_sdio_error_entry(b"request.payload.last-error", request.payload_last_error)?;
        let reply_ht_last_error =
            self.wifi_sdio_error_entry(b"reply.ht.last-error", reply_ht_last_error_value)?;
        let reply_send_last_error =
            self.wifi_sdio_error_entry(b"reply.send.last-error", reply_send_last_error_value)?;

        let entries = [
            status,
            request_status,
            request_step,
            local_ip_address,
            listen_port,
            peer_ip_address,
            peer_port,
            listen_status,
            ack_status,
            payload_status,
            payload_parse,
            payload_bytes,
            payload_hash,
            eval_status,
            response_length,
            response_hash,
            response_truncated,
            reply_status,
            reply_step,
            reply_peer_valid,
            reply_peer_ip,
            reply_peer_port,
            reply_peer_mac_hash,
            reply_payload_length,
            reply_payload_hash,
            reply_ethernet_length,
            reply_ethernet_hash,
            reply_send_status,
            reply_send_packet_length,
            reply_send_write_response,
            request_host_normal_int,
            request_host_error_int,
            reply_host_normal_int,
            reply_host_error_int,
            request_ack_last_error,
            request_payload_last_error,
            reply_ht_last_error,
            reply_send_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_http_get_report(&mut self, report: WifiSdioHttpGetReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let code = self.word_entry(b"code", report.http_status_code as u32)?;
        let remote_ip_address = self.word_entry(b"remote.ip", report.remote_ip_address)?;
        let source_port = self.word_entry(b"source.port", report.source_port as u32)?;
        let response_payload_bytes =
            self.word_entry(b"bytes", report.response_payload_bytes as u32)?;
        let response_polls = self.word_entry(b"polls", report.response_polls as u32)?;
        let dns_status = self.symbol_entry(b"dns", report.dns_status)?;
        let syn_status = self.symbol_entry(b"syn", report.syn_status)?;
        let get_status = self.symbol_entry(b"get", report.get_status)?;
        let response_status = self.symbol_entry(b"response", report.response_status)?;
        let response_parse_status = self.symbol_entry(b"parse", report.response_parse_status)?;
        let reset_status = self.symbol_entry(b"reset", report.reset_status)?;
        let preview_value = self.string_value(report.response_preview)?;
        let preview = self.entry(b"preview", preview_value)?;
        let entries = [
            status,
            step,
            code,
            remote_ip_address,
            source_port,
            response_payload_bytes,
            response_polls,
            dns_status,
            syn_status,
            get_status,
            response_status,
            response_parse_status,
            reset_status,
            preview,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_net_repl_once<B: Board>(
        &mut self,
        board: &mut B,
        poll_frames: u8,
    ) -> LispResult<Value> {
        let cycle = self.run_wifi_net_repl_cycle(board, poll_frames);
        self.wifi_net_repl_once_report(
            cycle.status,
            cycle.request,
            cycle.reply,
            cycle.eval_status,
            cycle.response_length,
            cycle.response_hash,
            cycle.response_truncated,
        )
    }

    fn run_wifi_net_repl_cycle<B: Board>(
        &mut self,
        board: &mut B,
        poll_frames: u8,
    ) -> WifiNetReplCycleReport {
        let request = board.wifi_net_repl_poll(poll_frames);
        if request.status == STATUS_ACK {
            return WifiNetReplCycleReport {
                status: STATUS_ACK,
                request,
                reply: None,
                eval_status: STATUS_NOT_RUN,
                response_length: 0,
                response_hash: 0,
                response_truncated: false,
            };
        }
        if !status_ready(request.status) {
            return WifiNetReplCycleReport {
                status: request.status,
                request,
                reply: None,
                eval_status: STATUS_NOT_RUN,
                response_length: 0,
                response_hash: 0,
                response_truncated: false,
            };
        }

        let (eval_status, response) = match self.cached_wifi_net_repl_response(&request) {
            Some(response) => (STATUS_DUPLICATE, response),
            None => {
                let (eval_status, response) = self.eval_net_repl_payload(
                    &request.payload[..request.payload_length as usize],
                    request.read_only,
                    board,
                );
                self.store_wifi_net_repl_response(&request, response);
                (eval_status, response)
            }
        };
        let response_length = response.len;
        let response_truncated = response.truncated;
        let response_hash = checksum_bytes(&response.bytes[..response.len as usize]);
        let reply = board.wifi_net_repl_reply(request.sequence, response);
        let status: &'static [u8] = if status_ready(reply.status) {
            b"ready"
        } else {
            b"reply-failed"
        };

        WifiNetReplCycleReport {
            status,
            request,
            reply: Some(reply),
            eval_status,
            response_length,
            response_hash,
            response_truncated,
        }
    }

    fn cached_wifi_net_repl_response(
        &self,
        request: &WifiNetReplRequestReport,
    ) -> Option<NetReplResponseBytes> {
        let cache = self.wifi_net_repl_response_cache;
        if cache.valid
            && cache.source_ip_address == request.source_ip_address
            && cache.source_mac_hash == request.source_mac_hash
            && cache.sequence == request.sequence
            && cache.read_only == request.read_only
            && cache.payload_hash == request.payload_hash
        {
            Some(cache.response)
        } else {
            None
        }
    }

    fn store_wifi_net_repl_response(
        &mut self,
        request: &WifiNetReplRequestReport,
        response: NetReplResponseBytes,
    ) {
        self.wifi_net_repl_response_cache = WifiNetReplResponseCache {
            valid: true,
            source_ip_address: request.source_ip_address,
            source_mac_hash: request.source_mac_hash,
            sequence: request.sequence,
            read_only: request.read_only,
            payload_hash: request.payload_hash,
            response,
        };
    }

    fn record_wifi_net_repl_service_cycle(&mut self, cycle: WifiNetReplCycleReport) {
        let reply_status = match cycle.reply {
            Some(reply) => reply.status,
            None => STATUS_NOT_RUN,
        };

        self.wifi_net_repl_service.polls = self.wifi_net_repl_service.polls.saturating_add(1);
        if cycle.reply.is_none() && cycle.request.status == STATUS_TIMEOUT {
            return;
        }

        if cycle.request.status == STATUS_ACK {
            self.wifi_net_repl_service.acks_received =
                self.wifi_net_repl_service.acks_received.saturating_add(1);
            self.wifi_net_repl_service.last_status = STATUS_ACK;
            self.wifi_net_repl_service.last_request_status = STATUS_ACK;
            self.wifi_net_repl_service.last_reply_status = STATUS_NOT_RUN;
            self.wifi_net_repl_service.last_eval_status = STATUS_NOT_RUN;
            self.wifi_net_repl_service.last_ack_sequence = cycle.request.sequence;
            self.wifi_net_repl_service.last_ack_response_hash = cycle.request.ack_response_hash;
            return;
        }

        if cycle.reply.is_some() {
            self.wifi_net_repl_service.requests_handled = self
                .wifi_net_repl_service
                .requests_handled
                .saturating_add(1);
        }

        self.wifi_net_repl_service.last_status = cycle.status;
        self.wifi_net_repl_service.last_request_status = cycle.request.status;
        self.wifi_net_repl_service.last_reply_status = reply_status;
        self.wifi_net_repl_service.last_eval_status = cycle.eval_status;
        self.wifi_net_repl_service.last_sequence = cycle.request.sequence;
        self.wifi_net_repl_service.last_request_read_only = cycle.request.read_only;
        self.wifi_net_repl_service.last_response_length = cycle.response_length;
        self.wifi_net_repl_service.last_response_hash = cycle.response_hash;
        self.wifi_net_repl_service.last_response_truncated = cycle.response_truncated;
    }

    fn wifi_net_repl_service(&mut self, args: &[Value]) -> LispResult<Value> {
        match args.len() {
            0 => {}
            1 => match args[0] {
                Value::Symbol(symbol) if symbol == self.specials.status => {}
                Value::Symbol(symbol) if symbol == self.specials.on => {
                    self.wifi_net_repl_service.enabled = true;
                }
                Value::Symbol(symbol) if symbol == self.specials.off => {
                    self.wifi_net_repl_service.enabled = false;
                }
                _ => {
                    return Err(Error::new(
                        "wifi-net-repl-service expects status, on, or off",
                    ))
                }
            },
            2 => match args[0] {
                Value::Symbol(symbol) if symbol == self.specials.on => {
                    let poll_frames = self.expect_net_repl_service_poll_frames(args[1])?;
                    self.enable_wifi_net_repl_service(poll_frames);
                }
                _ => return Err(Error::new("wifi-net-repl-service poll count requires on")),
            },
            _ => {
                return Err(Error::new(
                    "wifi-net-repl-service expects up to two arguments",
                ))
            }
        }

        self.wifi_net_repl_service_report()
    }

    fn wifi_net_repl_service_report(&mut self) -> LispResult<Value> {
        let service = self.wifi_net_repl_service;
        let enabled = self.bool_entry(b"enabled", service.enabled)?;
        let poll_frames = self.word_entry(b"poll-frames", service.poll_frames as u32)?;
        let polls = self.word_entry(b"polls", service.polls)?;
        let requests_handled = self.word_entry(b"requests-handled", service.requests_handled)?;
        let acks_received = self.word_entry(b"acks-received", service.acks_received)?;
        let last_status = self.symbol_entry(b"last.status", service.last_status)?;
        let last_request_status =
            self.symbol_entry(b"last.request.status", service.last_request_status)?;
        let last_reply_status =
            self.symbol_entry(b"last.reply.status", service.last_reply_status)?;
        let last_eval_status = self.symbol_entry(b"last.eval.status", service.last_eval_status)?;
        let last_sequence = self.word_entry(b"last.sequence", service.last_sequence)?;
        let last_request_read_only =
            self.bool_entry(b"last.request.read-only", service.last_request_read_only)?;
        let last_response_length =
            self.word_entry(b"last.response.length", service.last_response_length as u32)?;
        let last_response_hash =
            self.word_entry(b"last.response.hash", service.last_response_hash)?;
        let last_response_truncated =
            self.bool_entry(b"last.response.truncated", service.last_response_truncated)?;
        let last_ack_sequence = self.word_entry(b"last.ack.sequence", service.last_ack_sequence)?;
        let last_ack_response_hash =
            self.word_entry(b"last.ack.response-hash", service.last_ack_response_hash)?;

        let entries = [
            enabled,
            poll_frames,
            polls,
            requests_handled,
            acks_received,
            last_status,
            last_request_status,
            last_reply_status,
            last_eval_status,
            last_sequence,
            last_request_read_only,
            last_response_length,
            last_response_hash,
            last_response_truncated,
            last_ack_sequence,
            last_ack_response_hash,
        ];
        self.make_list_from_values(&entries)
    }

    fn record_wifi_tcp_repl_service_cycle(&mut self, cycle: WifiTcpReplCycleReport) {
        let reply_status = match cycle.reply {
            Some(reply) => reply.status,
            None => STATUS_NOT_RUN,
        };

        self.wifi_tcp_repl_service.polls = self.wifi_tcp_repl_service.polls.saturating_add(1);
        if cycle.reply.is_none() && cycle.request.status == STATUS_LISTEN_TIMEOUT {
            return;
        }

        if cycle.reply.is_some() && cycle.eval_status != STATUS_NOT_RUN {
            self.wifi_tcp_repl_service.requests_handled = self
                .wifi_tcp_repl_service
                .requests_handled
                .saturating_add(1);
        }

        self.wifi_tcp_repl_service.last_status = cycle.status;
        self.wifi_tcp_repl_service.last_request_status = cycle.request.status;
        self.wifi_tcp_repl_service.last_reply_status = reply_status;
        self.wifi_tcp_repl_service.last_eval_status = cycle.eval_status;
        self.wifi_tcp_repl_service.last_peer_ip_address = cycle.request.peer_ip_address;
        self.wifi_tcp_repl_service.last_peer_port = cycle.request.peer_port;
        self.wifi_tcp_repl_service.last_payload_length = cycle.request.payload_bytes;
        self.wifi_tcp_repl_service.last_payload_hash = cycle.request.payload_hash;
        self.wifi_tcp_repl_service.last_response_length = cycle.response_length;
        self.wifi_tcp_repl_service.last_response_hash = cycle.response_hash;
        self.wifi_tcp_repl_service.last_response_truncated = cycle.response_truncated;
    }

    fn wifi_tcp_repl_service<B: Board>(
        &mut self,
        args: &[Value],
        board: &mut B,
    ) -> LispResult<Value> {
        match args.len() {
            0 => {}
            1 => match args[0] {
                Value::Symbol(symbol) if symbol == self.specials.status => {}
                Value::Symbol(symbol) if symbol == self.specials.on => {
                    self.wifi_tcp_repl_service.enabled = true;
                    self.reset_telnet_repl_connection();
                    self.reset_wifi_tcp_repl_service_peer(board);
                }
                Value::Symbol(symbol) if symbol == self.specials.off => {
                    self.wifi_tcp_repl_service.enabled = false;
                    self.reset_telnet_repl_connection();
                    self.reset_wifi_tcp_repl_service_peer(board);
                }
                _ => {
                    return Err(Error::new(
                        "wifi-tcp-repl-service expects status, on, or off",
                    ))
                }
            },
            2 | 3 => match args[0] {
                Value::Symbol(symbol) if symbol == self.specials.on => {
                    let listen_port = self.expect_tcp_repl_service_port(args[1])?;
                    let poll_frames = if args.len() == 3 {
                        self.expect_net_repl_service_poll_frames(args[2])?
                    } else {
                        WIFI_TCP_REPL_SERVICE_DEFAULT_POLL_FRAMES
                    };
                    self.enable_wifi_tcp_repl_service(listen_port, poll_frames);
                    self.reset_telnet_repl_connection();
                    self.reset_wifi_tcp_repl_service_peer(board);
                }
                _ => return Err(Error::new("wifi-tcp-repl-service port requires on")),
            },
            _ => {
                return Err(Error::new(
                    "wifi-tcp-repl-service expects up to three arguments",
                ))
            }
        }

        self.wifi_tcp_repl_service_report()
    }

    fn wifi_tcp_repl_service_report(&mut self) -> LispResult<Value> {
        let service = self.wifi_tcp_repl_service;
        let enabled = self.bool_entry(b"enabled", service.enabled)?;
        let listen_port = self.word_entry(b"listen-port", service.listen_port as u32)?;
        let poll_frames = self.word_entry(b"poll-frames", service.poll_frames as u32)?;
        let polls = self.word_entry(b"polls", service.polls)?;
        let requests_handled = self.word_entry(b"requests-handled", service.requests_handled)?;
        let last_status = self.symbol_entry(b"last.status", service.last_status)?;
        let last_request_status =
            self.symbol_entry(b"last.request.status", service.last_request_status)?;
        let last_reply_status =
            self.symbol_entry(b"last.reply.status", service.last_reply_status)?;
        let last_eval_status = self.symbol_entry(b"last.eval.status", service.last_eval_status)?;
        let last_peer_ip = self.word_entry(b"last.peer.ip", service.last_peer_ip_address)?;
        let last_peer_port = self.word_entry(b"last.peer.port", service.last_peer_port as u32)?;
        let last_payload_length =
            self.word_entry(b"last.payload.length", service.last_payload_length as u32)?;
        let last_payload_hash = self.word_entry(b"last.payload.hash", service.last_payload_hash)?;
        let last_response_length =
            self.word_entry(b"last.response.length", service.last_response_length as u32)?;
        let last_response_hash =
            self.word_entry(b"last.response.hash", service.last_response_hash)?;
        let last_response_truncated =
            self.bool_entry(b"last.response.truncated", service.last_response_truncated)?;

        let entries = [
            enabled,
            listen_port,
            poll_frames,
            polls,
            requests_handled,
            last_status,
            last_request_status,
            last_reply_status,
            last_eval_status,
            last_peer_ip,
            last_peer_port,
            last_payload_length,
            last_payload_hash,
            last_response_length,
            last_response_hash,
            last_response_truncated,
        ];
        self.make_list_from_values(&entries)
    }

    fn eval_net_repl_payload<B: Board>(
        &mut self,
        input: &[u8],
        read_only: bool,
        board: &mut B,
    ) -> (&'static [u8], NetReplResponseBytes) {
        let mut writer = NetReplResponseWriter::new();
        self.collect_garbage();

        let expression = match self.read(input) {
            Ok(expression) => expression,
            Err(error) => {
                writeln!(writer, "error: {}", error.message()).ok();
                self.collect_garbage();
                return (b"read-error", writer.response());
            }
        };

        if read_only && !self.net_repl_read_only_expression_allowed(expression) {
            writeln!(writer, "error: read-only request denied").ok();
            self.collect_garbage();
            return (STATUS_READ_ONLY_DENIED, writer.response());
        }

        self.active_expression = expression;
        let result = self.eval(expression, Value::Nil, board, &mut writer, 0);
        self.active_expression = Value::Nil;

        let status: &'static [u8] = match result {
            Ok(value) => {
                let formatted = write!(writer, "=> ")
                    .and_then(|_| self.write_repl_value(value, &mut writer))
                    .and_then(|_| writeln!(writer));
                if formatted.is_ok() {
                    b"ready"
                } else {
                    b"format-error"
                }
            }
            Err(error) => {
                if writeln!(writer, "error: {}", error.message()).is_ok() {
                    b"eval-error"
                } else {
                    b"format-error"
                }
            }
        };

        self.collect_garbage();
        (status, writer.response())
    }

    fn net_repl_read_only_expression_allowed(&self, expression: Value) -> bool {
        let (operator, args) = match self.list_next(expression) {
            Ok(Some(pair)) => pair,
            _ => return false,
        };
        let symbol = match operator {
            Value::Symbol(symbol) => symbol,
            _ => return false,
        };

        if self.net_repl_read_only_no_arg_symbol(symbol) {
            return args == Value::Nil;
        }
        if self.symbol_name_eq(symbol, b"wifi-net-repl-service") {
            return self.net_repl_read_only_status_arg(args);
        }
        if self.symbol_name_eq(symbol, b"wifi-tcp-repl-service") {
            return self.net_repl_read_only_status_arg(args);
        }
        if self.symbol_name_eq(symbol, b"cat") || self.symbol_name_eq(symbol, b"read-file") {
            return self.net_repl_read_only_file_arg(args);
        }

        false
    }

    fn net_repl_read_only_no_arg_symbol(&self, symbol: SymbolId) -> bool {
        self.symbol_name_eq(symbol, b"help")
            || self.symbol_name_eq(symbol, b"millis")
            || self.symbol_name_eq(symbol, b"processes")
            || self.symbol_name_eq(symbol, b"regs")
            || self.symbol_name_eq(symbol, b"heap")
            || self.symbol_name_eq(symbol, b"ls")
            || self.symbol_name_eq(symbol, b"fat-info")
            || self.symbol_name_eq(symbol, b"sd-status")
            || self.symbol_name_eq(symbol, b"pdm-status")
            || self.symbol_name_eq(symbol, b"thermistor-status")
            || self.symbol_name_eq(symbol, b"capsense-status")
            || self.symbol_name_eq(symbol, b"wifi-link-status")
            || self.symbol_name_eq(symbol, b"wifi-lease-status")
    }

    fn net_repl_read_only_status_arg(&self, args: Value) -> bool {
        match self.single_list_arg(args) {
            Some(Value::Symbol(symbol)) => symbol == self.specials.status,
            _ => false,
        }
    }

    fn net_repl_read_only_file_arg(&self, args: Value) -> bool {
        match self.single_list_arg(args) {
            Some(value) => match self.expect_string(value) {
                Ok(path) => net_repl_read_only_path_safe(path),
                Err(_) => false,
            },
            None => false,
        }
    }

    fn single_list_arg(&self, args: Value) -> Option<Value> {
        match self.list_next(args) {
            Ok(Some((arg, tail))) if tail == Value::Nil => Some(arg),
            _ => None,
        }
    }

    fn wifi_net_repl_once_report(
        &mut self,
        status_value: &'static [u8],
        request: WifiNetReplRequestReport,
        reply: Option<WifiNetReplReplyReport>,
        eval_status_value: &'static [u8],
        response_length_value: u16,
        response_hash_value: u32,
        response_truncated_value: bool,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", status_value)?;
        let request_status = self.symbol_entry(b"request.status", request.status)?;
        let request_step = self.symbol_entry(b"request.step", request.step)?;
        let request_parse_status =
            self.symbol_entry(b"request.parse.status", request.parse_status)?;
        let lease_valid = self.bool_entry(b"lease.valid", request.lease_valid)?;
        let local_ip_address = self.word_entry(b"lease.ip", request.local_ip_address)?;
        let request_poll_limit =
            self.word_entry(b"request.poll-limit", request.poll_limit as u32)?;
        let request_polls = self.word_entry(b"request.polls", request.polls as u32)?;
        let request_frames_read =
            self.word_entry(b"request.frames-read", request.frames_read as u32)?;
        let request_non_data_frames =
            self.word_entry(b"request.non-data-frames", request.non_data_frames as u32)?;
        let request_non_repl_frames =
            self.word_entry(b"request.non-repl-frames", request.non_repl_frames as u32)?;
        let request_source_ip = self.word_entry(b"request.source-ip", request.source_ip_address)?;
        let request_source_port =
            self.word_entry(b"request.source-port", request.source_port as u32)?;
        let request_source_mac_hash =
            self.word_entry(b"request.source-mac-hash", request.source_mac_hash)?;
        let request_sequence = self.word_entry(b"request.sequence", request.sequence)?;
        let request_read_only = self.bool_entry(b"request.read-only", request.read_only)?;
        let request_payload_length =
            self.word_entry(b"request.payload-length", request.payload_length as u32)?;
        let request_payload_hash =
            self.word_entry(b"request.payload-hash", request.payload_hash)?;
        let request_ack_response_hash =
            self.word_entry(b"request.ack.response-hash", request.ack_response_hash)?;
        let request_ht_status = self.symbol_entry(b"request.ht.status", request.ht_status)?;
        let request_ack_status = self.symbol_entry(b"request.ack.status", request.ack_status)?;
        let request_host_normal_int =
            self.word_entry(b"request.HOST.NORM_INT", request.host_normal_int as u32)?;
        let request_host_error_int =
            self.word_entry(b"request.HOST.ERR_INT", request.host_error_int as u32)?;
        let eval_status = self.symbol_entry(b"eval.status", eval_status_value)?;
        let response_length = self.word_entry(b"response.length", response_length_value as u32)?;
        let response_hash = self.word_entry(b"response.hash", response_hash_value)?;
        let response_truncated =
            self.bool_entry(b"response.truncated", response_truncated_value)?;

        let (
            reply_status_value,
            reply_step_value,
            reply_peer_valid_value,
            reply_peer_ip_value,
            reply_peer_port_value,
            reply_peer_mac_hash_value,
            reply_sequence_value,
            reply_payload_length_value,
            reply_payload_hash_value,
            reply_ethernet_length_value,
            reply_ethernet_hash_value,
            reply_send_status_value,
            reply_send_packet_length_value,
            reply_send_write_response_value,
            reply_host_normal_int_value,
            reply_host_error_int_value,
            reply_ht_last_error_value,
            reply_mac_last_error_value,
            reply_send_last_error_value,
        ) = match reply {
            Some(reply) => (
                reply.status,
                reply.step,
                reply.peer_valid,
                reply.peer_ip_address,
                reply.peer_port,
                reply.peer_mac_hash,
                reply.sequence,
                reply.payload_length,
                reply.payload_hash,
                reply.ethernet_length,
                reply.ethernet_hash,
                reply.send_status,
                reply.send_packet_length,
                reply.send_write_response,
                reply.host_normal_int,
                reply.host_error_int,
                reply.ht_last_error,
                reply.mac_last_error,
                reply.send_last_error,
            ),
            None => (
                b"not-run" as &'static [u8],
                b"not-run" as &'static [u8],
                false,
                0u32,
                0u16,
                0u32,
                0u32,
                0u16,
                0u32,
                0u16,
                0u32,
                b"not-run" as &'static [u8],
                0u16,
                0u32,
                0u16,
                0u16,
                None,
                None,
                None,
            ),
        };

        let reply_status = self.symbol_entry(b"reply.status", reply_status_value)?;
        let reply_step = self.symbol_entry(b"reply.step", reply_step_value)?;
        let reply_peer_valid = self.bool_entry(b"reply.peer.valid", reply_peer_valid_value)?;
        let reply_peer_ip = self.word_entry(b"reply.peer-ip", reply_peer_ip_value)?;
        let reply_peer_port = self.word_entry(b"reply.peer-port", reply_peer_port_value as u32)?;
        let reply_peer_mac_hash =
            self.word_entry(b"reply.peer-mac-hash", reply_peer_mac_hash_value)?;
        let reply_sequence = self.word_entry(b"reply.sequence", reply_sequence_value)?;
        let reply_payload_length =
            self.word_entry(b"reply.payload-length", reply_payload_length_value as u32)?;
        let reply_payload_hash =
            self.word_entry(b"reply.payload-hash", reply_payload_hash_value)?;
        let reply_ethernet_length =
            self.word_entry(b"reply.ethernet-length", reply_ethernet_length_value as u32)?;
        let reply_ethernet_hash =
            self.word_entry(b"reply.ethernet-hash", reply_ethernet_hash_value)?;
        let reply_send_status = self.symbol_entry(b"reply.send.status", reply_send_status_value)?;
        let reply_send_packet_length = self.word_entry(
            b"reply.send.packet-length",
            reply_send_packet_length_value as u32,
        )?;
        let reply_send_write_response = self.word_entry(
            b"reply.send.write-response",
            reply_send_write_response_value,
        )?;
        let reply_host_normal_int =
            self.word_entry(b"reply.HOST.NORM_INT", reply_host_normal_int_value as u32)?;
        let reply_host_error_int =
            self.word_entry(b"reply.HOST.ERR_INT", reply_host_error_int_value as u32)?;
        let request_ack_last_error =
            self.wifi_sdio_error_entry(b"request.ack.last-error", request.ack_last_error)?;
        let request_ht_last_error =
            self.wifi_sdio_error_entry(b"request.ht.last-error", request.ht_last_error)?;
        let request_frame_last_error =
            self.wifi_sdio_error_entry(b"request.frame.last-error", request.frame_last_error)?;
        let reply_ht_last_error =
            self.wifi_sdio_error_entry(b"reply.ht.last-error", reply_ht_last_error_value)?;
        let reply_mac_last_error =
            self.wifi_sdio_error_entry(b"reply.mac.last-error", reply_mac_last_error_value)?;
        let reply_send_last_error =
            self.wifi_sdio_error_entry(b"reply.send.last-error", reply_send_last_error_value)?;

        let entries = [
            status,
            request_status,
            request_step,
            request_parse_status,
            lease_valid,
            local_ip_address,
            request_poll_limit,
            request_polls,
            request_frames_read,
            request_non_data_frames,
            request_non_repl_frames,
            request_source_ip,
            request_source_port,
            request_source_mac_hash,
            request_sequence,
            request_read_only,
            request_payload_length,
            request_payload_hash,
            request_ack_response_hash,
            request_ht_status,
            request_ack_status,
            request_host_normal_int,
            request_host_error_int,
            eval_status,
            response_length,
            response_hash,
            response_truncated,
            reply_status,
            reply_step,
            reply_peer_valid,
            reply_peer_ip,
            reply_peer_port,
            reply_peer_mac_hash,
            reply_sequence,
            reply_payload_length,
            reply_payload_hash,
            reply_ethernet_length,
            reply_ethernet_hash,
            reply_send_status,
            reply_send_packet_length,
            reply_send_write_response,
            reply_host_normal_int,
            reply_host_error_int,
            request_ack_last_error,
            request_ht_last_error,
            request_frame_last_error,
            reply_ht_last_error,
            reply_mac_last_error,
            reply_send_last_error,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_network_bootstrap<B: Board>(&mut self, board: &mut B) -> LispResult<Value> {
        let mut report = WifiNetworkBootstrapReport::new();
        let ssid =
            string_bytes_from_slice(wifi_credentials::local_ssid(), "local wifi ssid missing")?;
        let passphrase = string_bytes_from_slice(
            wifi_credentials::local_passphrase(),
            "local wifi passphrase missing",
        )?;

        report.step = b"wifi-prepare-join";
        let prepare = board.wifi_prepare_join();
        report.prepare_status = prepare.status;
        if !status_ready(prepare.status) {
            report.mark_failed(b"wifi-prepare-join");
            return self.wifi_network_bootstrap_report(report);
        }

        report.step = b"wifi-join-wpa2";
        let join = board.wifi_join_wpa2(ssid, passphrase);
        report.join_status = join.status;
        report.join_flags = join.join_flags;
        if !status_ready(join.status) {
            report.mark_failed(b"wifi-join-wpa2");
            return self.wifi_network_bootstrap_report(report);
        }

        report.step = b"wifi-dhcp-acquire";
        let dhcp = board.wifi_dhcp_acquire();
        report.dhcp_status = dhcp.status;
        report.lease_valid = dhcp.lease_valid;
        report.local_ip_address = dhcp.leased_ip_address;
        report.router_ip_address = dhcp.router;
        if !status_ready(dhcp.status) {
            report.mark_failed(b"wifi-dhcp-acquire");
            return self.wifi_network_bootstrap_report(report);
        }

        report.step = b"wifi-lease-status";
        let lease = board.wifi_lease_status();
        report.lease_status = lease.status;
        report.lease_valid = lease.lease_valid;
        report.local_ip_address = lease.ip_address;
        report.router_ip_address = lease.router;
        if !status_ready(lease.status) {
            report.mark_failed(b"wifi-lease-status");
            return self.wifi_network_bootstrap_report(report);
        }

        report.step = b"wifi-arp-router";
        let arp = board.wifi_arp_router();
        report.arp_status = arp.status;
        report.arp_reply_poll_status = arp.reply_poll_status;
        report.arp_reply_parse_status = arp.reply_parse_status;
        report.router_mac_hash = arp.router_mac_hash;
        report.router_mac_present = arp.router_mac_present;
        report.router_mac_stored = arp.router_mac_stored;
        if !status_ready(arp.status) {
            report.mark_failed(b"wifi-arp-router");
            return self.wifi_network_bootstrap_report(report);
        }

        report.status = STATUS_READY;
        report.step = STEP_DONE;
        self.wifi_network_bootstrap_report(report)
    }

    fn wifi_network_bootstrap_report(
        &mut self,
        report: WifiNetworkBootstrapReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let prepare_status = self.symbol_entry(b"prepare.status", report.prepare_status)?;
        let join_status = self.symbol_entry(b"join.status", report.join_status)?;
        let join_flags = self.word_entry(b"join.flags", report.join_flags)?;
        let dhcp_status = self.symbol_entry(b"dhcp.status", report.dhcp_status)?;
        let lease_status = self.symbol_entry(b"lease.status", report.lease_status)?;
        let lease_valid = self.bool_entry(b"lease.valid", report.lease_valid)?;
        let local_ip_address = self.word_entry(b"lease.ip", report.local_ip_address)?;
        let router_ip_address = self.word_entry(b"lease.router", report.router_ip_address)?;
        let arp_status = self.symbol_entry(b"arp.status", report.arp_status)?;
        let arp_reply_poll_status =
            self.symbol_entry(b"arp.reply.poll.status", report.arp_reply_poll_status)?;
        let arp_reply_parse_status =
            self.symbol_entry(b"arp.reply.parse.status", report.arp_reply_parse_status)?;
        let router_mac_hash = self.word_entry(b"router.mac.hash", report.router_mac_hash)?;
        let router_mac_present =
            self.bool_entry(b"router.mac.present", report.router_mac_present)?;
        let router_mac_stored = self.bool_entry(b"router.mac.stored", report.router_mac_stored)?;
        let entries = [
            status,
            step,
            prepare_status,
            join_status,
            join_flags,
            dhcp_status,
            lease_status,
            lease_valid,
            local_ip_address,
            router_ip_address,
            arp_status,
            arp_reply_poll_status,
            arp_reply_parse_status,
            router_mac_hash,
            router_mac_present,
            router_mac_stored,
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

    fn wifi_sdio_country_report(&mut self, report: WifiSdioCountryReport) -> LispResult<Value> {
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
        let country_abbrev = self.string_value(report.country_abbrev)?;
        let country_abbrev = self.entry(b"country.abbrev", country_abbrev)?;
        let revision = self.int_entry(b"country.revision", report.revision)?;
        let country_code = self.string_value(report.country_code)?;
        let country_code = self.entry(b"country.code", country_code)?;
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
            country_abbrev,
            revision,
            country_code,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_event_mask_report(
        &mut self,
        report: WifiSdioEventMaskReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let enabled_events = self.word_entry(b"events.enabled", report.enabled_events as u32)?;
        let mask_word0 = self.word_entry(b"event-mask.word0", report.mask_words[0])?;
        let mask_word1 = self.word_entry(b"event-mask.word1", report.mask_words[1])?;
        let mask_word2 = self.word_entry(b"event-mask.word2", report.mask_words[2])?;
        let mask_word3 = self.word_entry(b"event-mask.word3", report.mask_words[3])?;
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
            enabled_events,
            mask_word0,
            mask_word1,
            mask_word2,
            mask_word3,
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
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_tx_glomming_report(
        &mut self,
        report: WifiSdioTxGlommingReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let enabled = self.bool_entry(b"tx-glomming.enabled", report.enabled)?;
        let value = self.word_entry(b"tx-glomming.value", report.value)?;
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
            enabled,
            value,
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
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_scan_start_report(
        &mut self,
        report: WifiSdioScanStartReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let ht_status = self.symbol_entry(b"ht.status", report.ht_status)?;
        let ht_attempts = self.word_entry(b"ht.attempts", report.ht_attempts as u32)?;
        let ht_write_response = self.word_entry(b"ht.write-response", report.ht_write_response)?;
        let ht_read_value = self.word_entry(b"ht.read-value", report.ht_read_value as u32)?;
        let ht_read_response = self.word_entry(b"ht.read-response", report.ht_read_response)?;
        let ht_available = self.bool_entry(b"ht.available", report.ht_available)?;
        let scan_payload_bytes =
            self.word_entry(b"scan.payload-bytes", report.scan_payload_bytes as u32)?;
        let scan_version = self.word_entry(b"scan.version", report.scan_version)?;
        let scan_action = self.word_entry(b"scan.action", report.scan_action as u32)?;
        let scan_sync_id = self.word_entry(b"scan.sync-id", report.scan_sync_id as u32)?;
        let scan_type = self.word_entry(b"scan.type", report.scan_type as u32)?;
        let bss_type = self.word_entry(b"scan.bss-type", report.bss_type as u32)?;
        let bssid_filter_broadcast = self.bool_entry(
            b"scan.bssid-filter-broadcast",
            report.bssid_filter_broadcast,
        )?;
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
            scan_payload_bytes,
            scan_version,
            scan_action,
            scan_sync_id,
            scan_type,
            bss_type,
            bssid_filter_broadcast,
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
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_prepare_join_report(&mut self, report: WifiPrepareJoinReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let sdio_status = self.symbol_entry(b"sdio.status", report.sdio_status)?;
        let firmware_status = self.symbol_entry(b"firmware.status", report.firmware_status)?;
        let wlc_up_status = self.symbol_entry(b"wlc-up.status", report.wlc_up_status)?;
        let ack_status = self.symbol_entry(b"ack.status", report.ack_status)?;
        let clm_status = self.symbol_entry(b"clm.status", report.clm_status)?;
        let tx_glomming_status =
            self.symbol_entry(b"tx-glomming.status", report.tx_glomming_status)?;
        let country_status = self.symbol_entry(b"country.status", report.country_status)?;
        let entries = [
            status,
            step,
            sdio_status,
            firmware_status,
            wlc_up_status,
            ack_status,
            clm_status,
            tx_glomming_status,
            country_status,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_credential_report(&mut self, field: &'static [u8], length: u8) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", STATUS_READY)?;
        let field = self.symbol_entry(b"field", field)?;
        let length = self.word_entry(b"length", length as u32)?;
        let entries = [status, field, length];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_join_report(&mut self, report: WifiSdioJoinReport) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let step = self.symbol_entry(b"step", report.step)?;
        let ssid_len = self.word_entry(b"ssid.length", report.ssid_len as u32)?;
        let passphrase_len = self.word_entry(b"passphrase.length", report.passphrase_len as u32)?;
        let optional_cdc_errors =
            self.word_entry(b"optional-cdc.errors", report.optional_cdc_errors as u32)?;
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
        let requested_polls =
            self.word_entry(b"join.requested-polls", report.requested_polls as u32)?;
        let polls = self.word_entry(b"join.polls", report.polls as u32)?;
        let frames_read = self.word_entry(b"frames.read", report.frames_read as u32)?;
        let non_event_frames =
            self.word_entry(b"non-event.frames", report.non_event_frames as u32)?;
        let events_seen = self.word_entry(b"events.seen", report.events_seen as u32)?;
        let join_flags = self.word_entry(b"join.flags", report.join_flags)?;
        let last_frame_status =
            self.symbol_entry(b"last.frame.status", report.last_frame_status)?;
        let last_frame_length =
            self.word_entry(b"last.frame.length", report.last_frame_length as u32)?;
        let last_frame_channel =
            self.word_entry(b"last.frame.channel", report.last_frame_channel as u32)?;
        let last_frame_bus_data_credit = self.word_entry(
            b"last.frame.bus-data-credit",
            report.last_frame_bus_data_credit as u32,
        )?;
        let last_event_type = self.word_entry(b"last.event.type", report.last_event_type)?;
        let last_event_status = self.word_entry(b"last.event.status", report.last_event_status)?;
        let last_event_reason = self.word_entry(b"last.event.reason", report.last_event_reason)?;
        let last_event_flags = self.word_entry(b"last.event.flags", report.last_event_flags)?;
        let last_event_datalen =
            self.word_entry(b"last.event.datalen", report.last_event_datalen)?;
        let last_event_ifidx =
            self.word_entry(b"last.event.ifidx", report.last_event_ifidx as u32)?;
        let last_event_bsscfgidx =
            self.word_entry(b"last.event.bsscfgidx", report.last_event_bsscfgidx as u32)?;
        let ack_status = self.symbol_entry(b"ack.status", report.ack_status)?;
        let ack_int_status_before =
            self.word_entry(b"ack.INT_STATUS.before", report.ack_int_status_before)?;
        let ack_clear_value = self.word_entry(b"ack.INT_STATUS.clear", report.ack_clear_value)?;
        let ack_int_status_after =
            self.word_entry(b"ack.INT_STATUS.after", report.ack_int_status_after)?;
        let ack_final_response =
            self.word_entry(b"ack.INT_STATUS.final-response", report.ack_final_response)?;
        let host_normal_int = self.word_entry(b"HOST.NORM_INT", report.host_normal_int as u32)?;
        let host_error_int = self.word_entry(b"HOST.ERR_INT", report.host_error_int as u32)?;
        let ht_last_error = self.wifi_sdio_error_entry(b"ht.last-error", report.ht_last_error)?;
        let send_last_error =
            self.wifi_sdio_error_entry(b"send.last-error", report.send_last_error)?;
        let response_last_error =
            self.wifi_sdio_error_entry(b"response.last-error", report.response_last_error)?;
        let frame_last_error =
            self.wifi_sdio_error_entry(b"frame.last-error", report.frame_last_error)?;
        let ack_last_error =
            self.wifi_sdio_error_entry(b"ack.last-error", report.ack_last_error)?;
        let host = self.wifi_sdio_host_report(report.host)?;
        let host = self.entry(b"SDHC0", host)?;
        let entries = [
            status,
            step,
            ssid_len,
            passphrase_len,
            optional_cdc_errors,
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
            requested_polls,
            polls,
            frames_read,
            non_event_frames,
            events_seen,
            join_flags,
            last_frame_status,
            last_frame_length,
            last_frame_channel,
            last_frame_bus_data_credit,
            last_event_type,
            last_event_status,
            last_event_reason,
            last_event_flags,
            last_event_datalen,
            last_event_ifidx,
            last_event_bsscfgidx,
            ack_status,
            ack_int_status_before,
            ack_clear_value,
            ack_int_status_after,
            ack_final_response,
            host_normal_int,
            host_error_int,
            ht_last_error,
            send_last_error,
            response_last_error,
            frame_last_error,
            ack_last_error,
            host,
        ];
        self.make_list_from_values(&entries)
    }

    fn wifi_sdio_scan_event_drain_report(
        &mut self,
        report: WifiSdioScanEventDrainReport,
    ) -> LispResult<Value> {
        let status = self.symbol_entry(b"status", report.status)?;
        let stop_reason = self.symbol_entry(b"stop.reason", report.stop_reason)?;
        let requested_frames =
            self.word_entry(b"requested.frames", report.requested_frames as u32)?;
        let frames_read = self.word_entry(b"frames.read", report.frames_read as u32)?;
        let non_event_frames =
            self.word_entry(b"non-event.frames", report.non_event_frames as u32)?;
        let events_seen = self.word_entry(b"events.seen", report.events_seen as u32)?;
        let other_events = self.word_entry(b"events.other", report.other_events as u32)?;
        let scan_events = self.word_entry(b"scan.events", report.scan_events as u32)?;
        let scan_partial = self.word_entry(b"scan.partial", report.scan_partial as u32)?;
        let scan_complete = self.word_entry(b"scan.complete", report.scan_complete as u32)?;
        let scan_abort = self.word_entry(b"scan.abort", report.scan_abort as u32)?;
        let scan_other_status =
            self.word_entry(b"scan.other-status", report.scan_other_status as u32)?;
        let last_frame_status =
            self.symbol_entry(b"last.frame.status", report.last_frame_status)?;
        let last_frame_length =
            self.word_entry(b"last.frame.length", report.last_frame_length as u32)?;
        let last_frame_channel =
            self.word_entry(b"last.frame.channel", report.last_frame_channel as u32)?;
        let last_frame_bus_data_credit = self.word_entry(
            b"last.frame.bus-data-credit",
            report.last_frame_bus_data_credit as u32,
        )?;
        let last_event_type = self.word_entry(b"last.event.type", report.last_event_type)?;
        let last_event_status = self.word_entry(b"last.event.status", report.last_event_status)?;
        let last_event_reason = self.word_entry(b"last.event.reason", report.last_event_reason)?;
        let last_event_datalen =
            self.word_entry(b"last.event.datalen", report.last_event_datalen)?;
        let last_event_ifidx =
            self.word_entry(b"last.event.ifidx", report.last_event_ifidx as u32)?;
        let last_event_bsscfgidx =
            self.word_entry(b"last.event.bsscfgidx", report.last_event_bsscfgidx as u32)?;
        let last_escan_buflen = self.word_entry(b"last.escan.buflen", report.last_escan_buflen)?;
        let last_escan_version =
            self.word_entry(b"last.escan.version", report.last_escan_version)?;
        let last_escan_sync_id =
            self.word_entry(b"last.escan.sync-id", report.last_escan_sync_id as u32)?;
        let last_escan_bss_count =
            self.word_entry(b"last.escan.bss-count", report.last_escan_bss_count as u32)?;
        let ack_status = self.symbol_entry(b"ack.status", report.ack_status)?;
        let ack_int_status_before =
            self.word_entry(b"ack.INT_STATUS.before", report.ack_int_status_before)?;
        let ack_clear_value = self.word_entry(b"ack.INT_STATUS.clear", report.ack_clear_value)?;
        let ack_int_status_after =
            self.word_entry(b"ack.INT_STATUS.after", report.ack_int_status_after)?;
        let ack_final_response =
            self.word_entry(b"ack.INT_STATUS.final-response", report.ack_final_response)?;
        let frame_last_error =
            self.wifi_sdio_error_entry(b"frame.last-error", report.frame_last_error)?;
        let ack_last_error =
            self.wifi_sdio_error_entry(b"ack.last-error", report.ack_last_error)?;
        let entries = [
            status,
            stop_reason,
            requested_frames,
            frames_read,
            non_event_frames,
            events_seen,
            other_events,
            scan_events,
            scan_partial,
            scan_complete,
            scan_abort,
            scan_other_status,
            last_frame_status,
            last_frame_length,
            last_frame_channel,
            last_frame_bus_data_credit,
            last_event_type,
            last_event_status,
            last_event_reason,
            last_event_datalen,
            last_event_ifidx,
            last_event_bsscfgidx,
            last_escan_buflen,
            last_escan_version,
            last_escan_sync_id,
            last_escan_bss_count,
            ack_status,
            ack_int_status_before,
            ack_clear_value,
            ack_int_status_after,
            ack_final_response,
            frame_last_error,
            ack_last_error,
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
        self.mark_process_roots();

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

    fn mark_process_roots(&mut self) {
        let mut index = 0usize;
        while index < MAX_PROCESSES {
            let process = self.processes[index];
            if process.state != ProcessState::Free {
                self.mark_value(process.body);
                self.mark_value(process.cursor);
                self.mark_value(process.env);
                self.mark_value(process.last_value);
            }
            index += 1;
        }
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

fn checksum_bytes(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    let mut index = 0usize;
    while index < bytes.len() {
        hash ^= bytes[index] as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        index += 1;
    }
    hash
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

fn is_country_code_byte(byte: u8) -> bool {
    byte.is_ascii_uppercase() || byte.is_ascii_digit()
}

fn time_reached(now_ms: u32, target_ms: u32) -> bool {
    now_ms.wrapping_sub(target_ms) < 0x8000_0000
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
