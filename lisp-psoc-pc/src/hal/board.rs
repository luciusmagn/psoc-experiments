use core::ptr::{read_volatile, write_volatile};

use psoc6_pac::Peripherals;

use crate::hal::{micro_sd, wifi_sdio};
use crate::{lisp, lisp_store};

const BUTTON0_MASK: u32 = 1 << 4;
const PERIPHERAL_REGISTER_START: u32 = 0x4000_0000;
const PERIPHERAL_REGISTER_END: u32 = 0x40ff_fffc;

pub struct State {
    led_state: bool,
    heartbeat_enabled: bool,
    heartbeat_ms: u16,
    uptime_ms: u32,
}

impl State {
    pub const fn new() -> Self {
        Self {
            led_state: false,
            heartbeat_enabled: false,
            heartbeat_ms: 0,
            uptime_ms: 0,
        }
    }

    pub fn configure_hardware(p: &Peripherals) {
        configure_led(p);
        configure_button(p);
        micro_sd::configure_card_detect(p);
        micro_sd::configure_sdhc1_pins(p);
    }

    pub fn led_off(&mut self, p: &Peripherals) {
        self.heartbeat_enabled = false;
        self.led_state = false;
        led_set(p, false);
    }

    pub fn tick_ms(&mut self, p: &Peripherals) {
        self.uptime_ms = self.uptime_ms.wrapping_add(1);

        if self.heartbeat_enabled {
            self.heartbeat_ms += 1;
            if self.heartbeat_ms >= 500 {
                self.heartbeat_ms = 0;
                self.led_state = !self.led_state;
                led_set(p, self.led_state);
            }
        } else {
            self.heartbeat_ms = 0;
        }
    }

    pub fn lisp_board<'a>(&'a mut self, p: &'a Peripherals) -> PsocBoard<'a> {
        PsocBoard { p, state: self }
    }
}

pub struct PsocBoard<'a> {
    p: &'a Peripherals,
    state: &'a mut State,
}

impl lisp::Board for PsocBoard<'_> {
    fn led(&mut self, action: lisp::LedAction) -> bool {
        match action {
            lisp::LedAction::On => {
                self.state.heartbeat_enabled = false;
                self.state.led_state = true;
                led_set(self.p, true);
            }
            lisp::LedAction::Off => {
                self.state.heartbeat_enabled = false;
                self.state.led_state = false;
                led_set(self.p, false);
            }
            lisp::LedAction::Toggle => {
                self.state.heartbeat_enabled = false;
                self.state.led_state = !self.state.led_state;
                led_set(self.p, self.state.led_state);
            }
            lisp::LedAction::Status => {}
        }

        self.state.led_state
    }

    fn heartbeat(&mut self, enabled: bool) -> bool {
        self.state.heartbeat_enabled = enabled;
        self.state.heartbeat_enabled
    }

    fn button_pressed(&mut self, index: i32) -> Result<bool, lisp::Error> {
        if index != 0 {
            return Err(lisp::Error::new("unknown button"));
        }

        Ok(self.p.GPIO.prt0.in_.read().bits() & BUTTON0_MASK == 0)
    }

    fn millis(&mut self) -> u32 {
        self.state.uptime_ms
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
            peri_clock5: self.p.PERI.clock_ctl[5].read().bits(),
            peri_div8_0: self.p.PERI.div_8_ctl[0].read().bits(),
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

    fn sd_read(&mut self, sector: u32) -> lisp::SdReadReport {
        let report = micro_sd::read_sector(self.p, sector);
        lisp::SdReadReport {
            status: sd_read_status(report.status),
            init_status: sd_init_status(report.init_status),
            sector: report.sector,
            rca: report.rca,
            ocr: report.ocr,
            acmd41_attempts: report.acmd41_attempts,
            command_response: report.command_response,
            last_error: report.last_error.map(sd_command_error_report),
            first_words: report.first_words,
            mbr_signature: report.mbr_signature,
            partition_status: report.partition_status,
            partition_type: report.partition_type,
            partition_lba_start: report.partition_lba_start,
            partition_sector_count: report.partition_sector_count,
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

    fn sd_write_fill(&mut self, sector: u32, fill_word: u32) -> lisp::SdWriteReport {
        let report = micro_sd::write_sector_fill(self.p, sector, fill_word);
        lisp::SdWriteReport {
            status: sd_write_status(report.status),
            init_status: sd_init_status(report.init_status),
            sector: report.sector,
            fill_word: report.fill_word,
            rca: report.rca,
            ocr: report.ocr,
            acmd41_attempts: report.acmd41_attempts,
            command_response: report.command_response,
            last_error: report.last_error.map(sd_command_error_report),
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

    fn save_file(
        &mut self,
        path: lisp::StringBytes,
        content: lisp::StringBytes,
    ) -> lisp::StoreWriteReport {
        let report = lisp_store::write_file(self.p, path, content);
        lisp::StoreWriteReport {
            ready: matches!(report.status, lisp_store::StoreStatus::Ready),
            status: store_status(report.status),
            path_len: report.path_len,
            content_len: report.content_len,
            directory_sector: report.directory_sector,
            data_sector: report.data_sector,
        }
    }

    fn read_file(&mut self, path: lisp::StringBytes) -> lisp::StoreReadReport {
        let report = lisp_store::read_file(self.p, path);
        lisp::StoreReadReport {
            ready: matches!(report.status, lisp_store::StoreStatus::Ready),
            status: store_status(report.status),
            path_len: report.path_len,
            content_len: report.content_len,
            directory_sector: report.directory_sector,
            data_sector: report.data_sector,
            content: report.content,
        }
    }

    fn wifi_sdio_init(&mut self) -> lisp::WifiSdioReport {
        let report = wifi_sdio::initialize(self.p);
        lisp::WifiSdioReport {
            status: wifi_sdio_status(report.status),
            cmd5_response: report.cmd5_response,
            cmd5_attempts: report.cmd5_attempts,
            rca: report.rca,
            function_count: report.function_count,
            memory_present: report.memory_present,
            last_error: report.last_error.map(wifi_sdio_command_error_report),
            host: wifi_sdio_host_report(report.host),
            pins: wifi_sdio_pins_report(report.pins),
            clock: wifi_sdio_clock_report(report.clock),
        }
    }

    fn wifi_cmd52_read(&mut self, function: u8, address: u32) -> lisp::WifiSdioDirectReport {
        wifi_sdio_direct_report(wifi_sdio::cmd52_read(self.p, function, address))
    }

    fn wifi_cmd52_write(
        &mut self,
        function: u8,
        address: u32,
        data: u8,
    ) -> lisp::WifiSdioDirectReport {
        wifi_sdio_direct_report(wifi_sdio::cmd52_write(self.p, function, address, data))
    }

    fn wifi_enable_functions(&mut self, requested: u8) -> lisp::WifiSdioEnableReport {
        wifi_sdio_enable_report(wifi_sdio::enable_functions(self.p, requested))
    }

    fn wifi_setup_backplane(&mut self) -> lisp::WifiSdioBackplaneReport {
        wifi_sdio_backplane_report(wifi_sdio::setup_backplane(self.p))
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

fn configure_led(p: &Peripherals) {
    p.GPIO.prt13.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !(0x0f << 28);
        bits |= 0x06 << 28;
        w.bits(bits)
    });
    led_set(p, false);
}

fn configure_button(p: &Peripherals) {
    p.GPIO.prt0.cfg.modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !(0x0f << 16);
        bits |= 0x08 << 16;
        w.bits(bits)
    });
}

fn led_set(p: &Peripherals, on: bool) {
    if on {
        p.GPIO.prt13.out_clr.write(|w| w.out7().set_bit());
    } else {
        p.GPIO.prt13.out_set.write(|w| w.out7().set_bit());
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
        micro_sd::ReadStatus::AddressOverflow => b"address-overflow",
        micro_sd::ReadStatus::DataSetupBusy => b"data-setup-busy",
        micro_sd::ReadStatus::Cmd17Failed => b"cmd17-failed",
        micro_sd::ReadStatus::BufferReadTimeout => b"buffer-read-timeout",
        micro_sd::ReadStatus::BufferEnableTimeout => b"buffer-enable-timeout",
        micro_sd::ReadStatus::TransferTimeout => b"transfer-timeout",
    }
}

fn sd_write_status(status: micro_sd::WriteStatus) -> &'static [u8] {
    match status {
        micro_sd::WriteStatus::Ready => b"ready",
        micro_sd::WriteStatus::InitFailed => b"init-failed",
        micro_sd::WriteStatus::Cmd2Failed => b"cmd2-failed",
        micro_sd::WriteStatus::Cmd3Failed => b"cmd3-failed",
        micro_sd::WriteStatus::Cmd7Failed => b"cmd7-failed",
        micro_sd::WriteStatus::Cmd16Failed => b"cmd16-failed",
        micro_sd::WriteStatus::AddressOverflow => b"address-overflow",
        micro_sd::WriteStatus::DataSetupBusy => b"data-setup-busy",
        micro_sd::WriteStatus::Cmd24Failed => b"cmd24-failed",
        micro_sd::WriteStatus::BufferWriteTimeout => b"buffer-write-timeout",
        micro_sd::WriteStatus::BufferEnableTimeout => b"buffer-enable-timeout",
        micro_sd::WriteStatus::TransferTimeout => b"transfer-timeout",
        micro_sd::WriteStatus::DataLineBusy => b"data-line-busy",
    }
}

fn store_status(status: lisp_store::StoreStatus) -> &'static [u8] {
    match status {
        lisp_store::StoreStatus::Ready => b"ready",
        lisp_store::StoreStatus::EmptyPath => b"empty-path",
        lisp_store::StoreStatus::PathTooLong => b"path-too-long",
        lisp_store::StoreStatus::ContentTooLong => b"content-too-long",
        lisp_store::StoreStatus::NotFound => b"not-found",
        lisp_store::StoreStatus::DirectoryFull => b"directory-full",
        lisp_store::StoreStatus::DirectoryReadFailed => b"directory-read-failed",
        lisp_store::StoreStatus::DirectoryWriteFailed => b"directory-write-failed",
        lisp_store::StoreStatus::DataReadFailed => b"data-read-failed",
        lisp_store::StoreStatus::DataWriteFailed => b"data-write-failed",
        lisp_store::StoreStatus::CorruptData => b"corrupt-data",
        lisp_store::StoreStatus::ChecksumMismatch => b"checksum-mismatch",
    }
}

fn wifi_sdio_status(status: wifi_sdio::WifiSdioStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioStatus::Ready => b"ready",
        wifi_sdio::WifiSdioStatus::ClockNotStable => b"clock-not-stable",
        wifi_sdio::WifiSdioStatus::ResetTimeout => b"reset-timeout",
        wifi_sdio::WifiSdioStatus::Cmd0Failed => b"cmd0-failed",
        wifi_sdio::WifiSdioStatus::Cmd5Failed => b"cmd5-failed",
        wifi_sdio::WifiSdioStatus::Cmd5Busy => b"cmd5-busy",
        wifi_sdio::WifiSdioStatus::Cmd3Failed => b"cmd3-failed",
        wifi_sdio::WifiSdioStatus::Cmd7Failed => b"cmd7-failed",
        wifi_sdio::WifiSdioStatus::SelectBusy => b"select-busy",
    }
}

fn wifi_sdio_command_error(code: wifi_sdio::CommandErrorCode) -> &'static [u8] {
    match code {
        wifi_sdio::CommandErrorCode::CommandLineBusy => b"command-line-busy",
        wifi_sdio::CommandErrorCode::CommandTimeout => b"command-timeout",
        wifi_sdio::CommandErrorCode::CommandStatusError => b"command-status-error",
    }
}

fn wifi_sdio_direct_status(status: wifi_sdio::WifiSdioDirectStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioDirectStatus::Ready => b"ready",
        wifi_sdio::WifiSdioDirectStatus::InitFailed => b"init-failed",
        wifi_sdio::WifiSdioDirectStatus::InvalidFunction => b"invalid-function",
        wifi_sdio::WifiSdioDirectStatus::InvalidAddress => b"invalid-address",
        wifi_sdio::WifiSdioDirectStatus::Cmd52Failed => b"cmd52-failed",
    }
}

fn wifi_sdio_enable_status(status: wifi_sdio::WifiSdioEnableStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioEnableStatus::Ready => b"ready",
        wifi_sdio::WifiSdioEnableStatus::InitFailed => b"init-failed",
        wifi_sdio::WifiSdioEnableStatus::WriteFailed => b"write-failed",
        wifi_sdio::WifiSdioEnableStatus::ReadyReadFailed => b"ready-read-failed",
        wifi_sdio::WifiSdioEnableStatus::ReadyTimeout => b"ready-timeout",
    }
}

fn wifi_sdio_backplane_status(status: wifi_sdio::WifiSdioBackplaneStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioBackplaneStatus::Ready => b"ready",
        wifi_sdio::WifiSdioBackplaneStatus::InitFailed => b"init-failed",
        wifi_sdio::WifiSdioBackplaneStatus::IoEnableWriteFailed => b"io-enable-write-failed",
        wifi_sdio::WifiSdioBackplaneStatus::IoEnableReadFailed => b"io-enable-read-failed",
        wifi_sdio::WifiSdioBackplaneStatus::IoEnableTimeout => b"io-enable-timeout",
        wifi_sdio::WifiSdioBackplaneStatus::BusControlReadFailed => b"bus-control-read-failed",
        wifi_sdio::WifiSdioBackplaneStatus::BusControlWriteFailed => b"bus-control-write-failed",
        wifi_sdio::WifiSdioBackplaneStatus::BlockSizeWriteFailed => b"block-size-write-failed",
        wifi_sdio::WifiSdioBackplaneStatus::BlockSizeReadFailed => b"block-size-read-failed",
        wifi_sdio::WifiSdioBackplaneStatus::BlockSizeTimeout => b"block-size-timeout",
        wifi_sdio::WifiSdioBackplaneStatus::InterruptEnableWriteFailed => {
            b"interrupt-enable-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneStatus::InterruptEnableReadFailed => {
            b"interrupt-enable-read-failed"
        }
        wifi_sdio::WifiSdioBackplaneStatus::ReadyReadFailed => b"ready-read-failed",
        wifi_sdio::WifiSdioBackplaneStatus::ReadyTimeout => b"ready-timeout",
    }
}

fn wifi_sdio_command_error_report(
    error: wifi_sdio::CommandError,
) -> lisp::WifiSdioCommandErrorReport {
    lisp::WifiSdioCommandErrorReport {
        code: wifi_sdio_command_error(error.code),
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

fn wifi_sdio_direct_report(report: wifi_sdio::WifiSdioDirectReport) -> lisp::WifiSdioDirectReport {
    lisp::WifiSdioDirectReport {
        status: wifi_sdio_direct_status(report.status),
        init_status: wifi_sdio_status(report.init_status),
        function: report.function,
        address: report.address,
        write: report.write,
        data: report.data,
        response: report.response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_enable_report(report: wifi_sdio::WifiSdioEnableReport) -> lisp::WifiSdioEnableReport {
    lisp::WifiSdioEnableReport {
        status: wifi_sdio_enable_status(report.status),
        init_status: wifi_sdio_status(report.init_status),
        requested: report.requested,
        ready: report.ready,
        attempts: report.attempts,
        write_response: report.write_response,
        ready_response: report.ready_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_backplane_report(
    report: wifi_sdio::WifiSdioBackplaneReport,
) -> lisp::WifiSdioBackplaneReport {
    lisp::WifiSdioBackplaneReport {
        status: wifi_sdio_backplane_status(report.status),
        init_status: wifi_sdio_status(report.init_status),
        io_enable: report.io_enable,
        io_ready: report.io_ready,
        bus_control_before: report.bus_control_before,
        bus_control_after: report.bus_control_after,
        f0_block_size: report.f0_block_size,
        f1_block_size: report.f1_block_size,
        f2_block_size: report.f2_block_size,
        interrupt_enable: report.interrupt_enable,
        attempts: report.attempts,
        last_response: report.last_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_host_report(snapshot: wifi_sdio::WifiSdioHostSnapshot) -> lisp::WifiSdioHostReport {
    lisp::WifiSdioHostReport {
        wrap_ctl: snapshot.wrap_ctl,
        gp_out: snapshot.gp_out,
        gp_in: snapshot.gp_in,
        xfer_mode: snapshot.xfer_mode,
        host_ctrl1: snapshot.host_ctrl1,
        host_ctrl2: snapshot.host_ctrl2,
        tout_ctrl: snapshot.tout_ctrl,
        clk_ctrl: snapshot.clk_ctrl,
        pwr_ctrl: snapshot.pwr_ctrl,
        sw_rst: snapshot.sw_rst,
        normal_int: snapshot.normal_int,
        error_int: snapshot.error_int,
        normal_int_stat_en: snapshot.normal_int_stat_en,
        error_int_stat_en: snapshot.error_int_stat_en,
        normal_int_signal_en: snapshot.normal_int_signal_en,
        error_int_signal_en: snapshot.error_int_signal_en,
        pstate: snapshot.pstate,
        cmd: snapshot.cmd,
        argument: snapshot.argument,
        response01: snapshot.response01,
        response23: snapshot.response23,
        response45: snapshot.response45,
        response67: snapshot.response67,
    }
}

fn wifi_sdio_pins_report(snapshot: wifi_sdio::WifiSdioPinSnapshot) -> lisp::WifiSdioPinsReport {
    lisp::WifiSdioPinsReport {
        p2_sel0: snapshot.p2_sel0,
        p2_sel1: snapshot.p2_sel1,
        p2_cfg: snapshot.p2_cfg,
        p2_out: snapshot.p2_out,
        p2_in: snapshot.p2_in,
    }
}

fn wifi_sdio_clock_report(snapshot: wifi_sdio::WifiSdioClockSnapshot) -> lisp::WifiSdioClockReport {
    lisp::WifiSdioClockReport {
        path0: snapshot.path0,
        root0: snapshot.root0,
        root1: snapshot.root1,
        root2: snapshot.root2,
        root3: snapshot.root3,
        root4: snapshot.root4,
        fll_config: snapshot.fll_config,
        fll_config2: snapshot.fll_config2,
        fll_status: snapshot.fll_status,
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
