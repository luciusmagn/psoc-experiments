use core::ptr::{read_volatile, write_volatile};

use psoc6_pac::Peripherals;

use crate::hal::{micro_sd, wifi_sdio};
use crate::{lisp, lisp_fat, lisp_store, wifi_resources};

const BUTTON0_MASK: u32 = 1 << 4;
const PERIPHERAL_REGISTER_START: u32 = 0x4000_0000;
const PERIPHERAL_REGISTER_END: u32 = 0x40ff_fffc;

pub struct State {
    led_state: bool,
    heartbeat_enabled: bool,
    heartbeat_ms: u16,
    uptime_ms: u32,
    wifi_control_state: wifi_sdio::WifiSdioControlState,
}

impl State {
    pub const fn new() -> Self {
        Self {
            led_state: false,
            heartbeat_enabled: false,
            heartbeat_ms: 0,
            uptime_ms: 0,
            wifi_control_state: wifi_sdio::WifiSdioControlState::new(),
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

    pub fn reboot_marker(&mut self, p: &Peripherals, delay: &mut cortex_m::delay::Delay) {
        const PATTERN: &[(bool, u32)] = &[
            (true, 80),
            (false, 80),
            (true, 80),
            (false, 180),
            (true, 320),
            (false, 180),
            (true, 80),
            (false, 80),
            (true, 320),
        ];

        self.heartbeat_enabled = false;
        for &(on, duration_ms) in PATTERN {
            self.led_state = on;
            led_set(p, on);
            delay.delay_ms(duration_ms);
        }
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

struct FatFormatProgressAdapter<'a> {
    progress: &'a mut dyn lisp::FatFormatProgress,
}

impl lisp_fat::FormatProgress for FatFormatProgressAdapter<'_> {
    fn report(&mut self, phase: &'static [u8], written_sector_count: u32, total_sectors: u32) {
        self.progress
            .report(phase, written_sector_count, total_sectors);
    }
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

    fn format_store(&mut self) -> lisp::StoreFormatReport {
        let report = lisp_store::format_store(self.p);
        lisp::StoreFormatReport {
            ready: matches!(report.status, lisp_store::StoreStatus::Ready),
            status: store_status(report.status),
            directory_sector: report.directory_sector,
            data_start_sector: report.data_start_sector,
            data_sector_count: report.data_sector_count,
            failed_sector: report.failed_sector,
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

    fn list_files(&mut self) -> lisp::StoreListReport {
        let report = lisp_store::list_files(self.p);
        lisp::StoreListReport {
            ready: matches!(report.status, lisp_store::StoreStatus::Ready),
            status: store_status(report.status),
            file_count: report.file_count,
            directory_sector: report.directory_sector,
            files: report.files,
        }
    }

    fn fat_info(&mut self) -> lisp::FatInfoReport {
        let report = lisp_fat::info(self.p);
        lisp::FatInfoReport {
            ready: matches!(report.status, lisp_fat::FatStatus::Ready),
            status: fat_status(report.status),
            mbr_signature: report.mbr_signature,
            partition_status: report.partition_status,
            partition_type: report.partition_type,
            partition_lba_start: report.partition_lba_start,
            partition_sector_count: report.partition_sector_count,
            root_entry_count: report.root_entry_count,
            sample_count: report.sample_count,
            entries: report.entries,
        }
    }

    fn fat_format(&mut self, progress: &mut dyn lisp::FatFormatProgress) -> lisp::FatFormatReport {
        let mut progress = FatFormatProgressAdapter { progress };
        let report = lisp_fat::format_fat32_with_progress(self.p, &mut progress);
        lisp::FatFormatReport {
            ready: matches!(report.status, lisp_fat::FatStatus::Ready),
            status: fat_status(report.status),
            mbr_signature: report.mbr_signature,
            partition_status: report.partition_status,
            partition_type_before: report.partition_type_before,
            partition_type_after: report.partition_type_after,
            partition_lba_start: report.partition_lba_start,
            partition_sector_count: report.partition_sector_count,
            sectors_per_cluster: report.sectors_per_cluster,
            reserved_sectors: report.reserved_sectors,
            fat_count: report.fat_count,
            fat_size_sectors: report.fat_size_sectors,
            data_cluster_count: report.data_cluster_count,
            root_cluster: report.root_cluster,
            written_sector_count: report.written_sector_count,
            failed_sector: report.failed_sector,
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

    fn wifi_cmd53_read(
        &mut self,
        function: u8,
        address: u32,
        count: u8,
    ) -> lisp::WifiSdioCmd53ReadReport {
        wifi_sdio_cmd53_read_report(wifi_sdio::cmd53_read(self.p, function, address, count))
    }

    fn wifi_backplane_read(
        &mut self,
        address: u32,
        count: u8,
    ) -> lisp::WifiSdioBackplaneReadReport {
        wifi_sdio_backplane_read_report(wifi_sdio::backplane_read(self.p, address, count))
    }

    fn wifi_backplane_write8(
        &mut self,
        address: u32,
        value: u8,
    ) -> lisp::WifiSdioBackplaneWrite8Report {
        wifi_sdio_backplane_write8_report(wifi_sdio::backplane_write8(self.p, address, value))
    }

    fn wifi_backplane_write32(
        &mut self,
        address: u32,
        value: u32,
    ) -> lisp::WifiSdioBackplaneWrite32Report {
        wifi_sdio_backplane_write32_report(wifi_sdio::backplane_write32(self.p, address, value))
    }

    fn wifi_backplane_write32_bytes(
        &mut self,
        address: u32,
        value: u32,
    ) -> lisp::WifiSdioBackplaneWrite32Report {
        wifi_sdio_backplane_write32_report(wifi_sdio::backplane_write32_bytes(
            self.p, address, value,
        ))
    }

    fn wifi_socram_probe(&mut self, address: u32, pattern: u32) -> lisp::WifiSdioSocramProbeReport {
        wifi_sdio_socram_probe_report(wifi_sdio::socram_probe(self.p, address, pattern))
    }

    fn wifi_socram_block_probe(
        &mut self,
        address: u32,
        seed: u32,
    ) -> lisp::WifiSdioSocramBlockProbeReport {
        wifi_sdio_socram_block_probe_report(wifi_sdio::socram_block_probe(self.p, address, seed))
    }

    fn wifi_load_firmware(&mut self) -> lisp::WifiSdioFirmwareLoadReport {
        wifi_sdio_firmware_load_report(wifi_sdio::load_firmware(
            self.p,
            wifi_resources::cyw4343w_firmware(),
        ))
    }

    fn wifi_start_firmware(&mut self) -> lisp::WifiSdioFirmwareStartReport {
        self.state.wifi_control_state.reset();
        wifi_sdio_firmware_start_report(wifi_sdio::start_firmware(
            self.p,
            wifi_resources::cyw4343w_firmware(),
            wifi_resources::cyw4343w_nvram(),
        ))
    }

    fn wifi_f2_read_header(&mut self) -> lisp::WifiSdioF2HeaderReport {
        wifi_sdio_f2_header_report(wifi_sdio::f2_read_header(self.p))
    }

    fn wifi_f2_read_frame(&mut self) -> lisp::WifiSdioF2FrameReport {
        wifi_sdio_f2_frame_report(wifi_sdio::f2_read_frame(self.p))
    }

    fn wifi_f2_read_frame_single(&mut self) -> lisp::WifiSdioF2FrameReport {
        wifi_sdio_f2_frame_report(wifi_sdio::f2_read_frame_single(self.p))
    }

    fn wifi_f2_read_frame_exact(&mut self, count: u8) -> lisp::WifiSdioF2FrameReport {
        wifi_sdio_f2_frame_report(wifi_sdio::f2_read_frame_exact(self.p, count))
    }

    fn wifi_f2_read_frame_block(&mut self) -> lisp::WifiSdioF2FrameReport {
        wifi_sdio_f2_frame_report(wifi_sdio::f2_read_frame_block(self.p))
    }

    fn wifi_send_wlc_up(&mut self) -> lisp::WifiSdioF2ControlReport {
        wifi_sdio_f2_control_report(wifi_sdio::send_wlc_up(self.p))
    }

    fn wifi_wlc_up(&mut self) -> lisp::WifiSdioWlcUpReport {
        wifi_sdio_wlc_up_report(wifi_sdio::wlc_up(
            self.p,
            &mut self.state.wifi_control_state,
        ))
    }

    fn wifi_get_version(&mut self) -> lisp::WifiSdioGetVersionReport {
        wifi_sdio_get_version_report(wifi_sdio::get_version(
            self.p,
            &mut self.state.wifi_control_state,
        ))
    }

    fn wifi_get_mpc(&mut self) -> lisp::WifiSdioGetMpcReport {
        wifi_sdio_get_mpc_report(wifi_sdio::get_mpc(
            self.p,
            &mut self.state.wifi_control_state,
        ))
    }

    fn wifi_f2_read_frame_abort(&mut self) -> lisp::WifiSdioF2AbortProbeReport {
        let frame = wifi_sdio::f2_read_frame(self.p);
        let abort = wifi_sdio::abort_read(self.p);
        let post = wifi_sdio::interrupt_state(self.p);
        lisp::WifiSdioF2AbortProbeReport {
            frame_status: wifi_sdio_f2_frame_status(frame.status),
            frame_valid: frame.valid,
            frame_length: frame.length,
            frame_channel: frame.channel,
            frame_bus_data_credit: frame.bus_data_credit,
            frame_header_response: frame.header_response,
            frame_body_response: frame.body_response,
            abort_io_abort_response: abort.io_abort_response,
            abort_frame_control_response: abort.frame_control_response,
            post_io_enable: post.io_enable,
            post_io_ready: post.io_ready,
            post_interrupt_pending: post.interrupt_pending,
            post_io_enable_response: post.io_enable_response,
            post_io_ready_response: post.io_ready_response,
            post_interrupt_pending_response: post.interrupt_pending_response,
            post_host_normal_int: post.host_normal_int,
            post_host_error_int: post.host_error_int,
            frame_last_error: frame.last_error.map(wifi_sdio_command_error_report),
            abort_last_error: abort.last_error.map(wifi_sdio_command_error_report),
            post_last_error: post.last_error.map(wifi_sdio_command_error_report),
        }
    }

    fn wifi_poll_read_frame(&mut self) -> lisp::WifiSdioPollReadFrameReport {
        let ack = wifi_sdio::ack_interrupts(self.p);
        let frame = wifi_sdio::f2_read_frame(self.p);
        let post = wifi_sdio::interrupt_state(self.p);
        lisp::WifiSdioPollReadFrameReport {
            ack_status: wifi_sdio_interrupt_ack_status(ack.status),
            ack_int_status_before: ack.int_status_before,
            ack_clear_value: ack.clear_value,
            ack_int_status_after: ack.int_status_after,
            ack_final_response: ack.final_response,
            frame_status: wifi_sdio_f2_frame_status(frame.status),
            frame_valid: frame.valid,
            frame_length: frame.length,
            frame_channel: frame.channel,
            frame_bus_data_credit: frame.bus_data_credit,
            frame_header_response: frame.header_response,
            frame_body_response: frame.body_response,
            post_status: wifi_sdio_interrupt_state_status(post.status),
            post_io_enable: post.io_enable,
            post_io_ready: post.io_ready,
            post_interrupt_pending: post.interrupt_pending,
            post_io_enable_response: post.io_enable_response,
            post_io_ready_response: post.io_ready_response,
            post_interrupt_pending_response: post.interrupt_pending_response,
            post_host_normal_int: post.host_normal_int,
            post_host_error_int: post.host_error_int,
            ack_last_error: ack.last_error.map(wifi_sdio_command_error_report),
            frame_last_error: frame.last_error.map(wifi_sdio_command_error_report),
            post_last_error: post.last_error.map(wifi_sdio_command_error_report),
        }
    }

    fn wifi_ack_interrupts(&mut self) -> lisp::WifiSdioInterruptAckReport {
        wifi_sdio_interrupt_ack_report(wifi_sdio::ack_interrupts(self.p))
    }

    fn wifi_interrupt_state(&mut self) -> lisp::WifiSdioInterruptStateReport {
        wifi_sdio_interrupt_state_report(wifi_sdio::interrupt_state(self.p))
    }

    fn wifi_keep_awake(&mut self) -> lisp::WifiSdioKeepAwakeReport {
        wifi_sdio_keep_awake_report(wifi_sdio::keep_awake(self.p))
    }

    fn wifi_request_ht(&mut self) -> lisp::WifiSdioHtRequestReport {
        wifi_sdio_ht_request_report(wifi_sdio::request_ht(self.p))
    }

    fn wifi_host_reset_lines(&mut self) -> lisp::WifiSdioHostResetReport {
        wifi_sdio_host_reset_report(wifi_sdio::host_reset_lines(self.p))
    }

    fn wifi_abort_read(&mut self) -> lisp::WifiSdioAbortReadReport {
        wifi_sdio_abort_read_report(wifi_sdio::abort_read(self.p))
    }

    fn wifi_core_state(&mut self, base: u32) -> lisp::WifiSdioCoreStateReport {
        wifi_sdio_core_state_report(wifi_sdio::core_state(self.p, base))
    }

    fn wifi_reset_core(&mut self, base: u32) -> lisp::WifiSdioCoreResetReport {
        wifi_sdio_core_reset_report(wifi_sdio::reset_core(self.p, base))
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

fn fat_status(status: lisp_fat::FatStatus) -> &'static [u8] {
    match status {
        lisp_fat::FatStatus::Ready => b"ready",
        lisp_fat::FatStatus::MbrReadFailed => b"mbr-read-failed",
        lisp_fat::FatStatus::MissingMbrSignature => b"missing-mbr-signature",
        lisp_fat::FatStatus::UnsupportedPartition => b"unsupported-partition",
        lisp_fat::FatStatus::BlockDeviceFailed => b"block-device-failed",
        lisp_fat::FatStatus::FormatGeometryInvalid => b"format-geometry-invalid",
        lisp_fat::FatStatus::FormatMbrWriteFailed => b"format-mbr-write-failed",
        lisp_fat::FatStatus::FormatBootWriteFailed => b"format-boot-write-failed",
        lisp_fat::FatStatus::FormatFsInfoWriteFailed => b"format-fsinfo-write-failed",
        lisp_fat::FatStatus::FormatFatClearFailed => b"format-fat-clear-failed",
        lisp_fat::FatStatus::FormatFatHeaderWriteFailed => b"format-fat-header-write-failed",
        lisp_fat::FatStatus::FormatRootClearFailed => b"format-root-clear-failed",
        lisp_fat::FatStatus::VolumeOpenFailed => b"volume-open-failed",
        lisp_fat::FatStatus::RootOpenFailed => b"root-open-failed",
        lisp_fat::FatStatus::RootIterateFailed => b"root-iterate-failed",
        lisp_fat::FatStatus::RootCloseFailed => b"root-close-failed",
        lisp_fat::FatStatus::VolumeCloseFailed => b"volume-close-failed",
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

fn wifi_sdio_cmd53_read_status(status: wifi_sdio::WifiSdioCmd53ReadStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioCmd53ReadStatus::Ready => b"ready",
        wifi_sdio::WifiSdioCmd53ReadStatus::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioCmd53ReadStatus::InvalidFunction => b"invalid-function",
        wifi_sdio::WifiSdioCmd53ReadStatus::InvalidAddress => b"invalid-address",
        wifi_sdio::WifiSdioCmd53ReadStatus::InvalidCount => b"invalid-count",
        wifi_sdio::WifiSdioCmd53ReadStatus::DataSetupBusy => b"data-setup-busy",
        wifi_sdio::WifiSdioCmd53ReadStatus::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioCmd53ReadStatus::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioCmd53ReadStatus::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioCmd53ReadStatus::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioCmd53ReadStatus::Cmd53Failed => b"cmd53-failed",
        wifi_sdio::WifiSdioCmd53ReadStatus::BufferReadTimeout => b"buffer-read-timeout",
        wifi_sdio::WifiSdioCmd53ReadStatus::BufferEnableTimeout => b"buffer-enable-timeout",
        wifi_sdio::WifiSdioCmd53ReadStatus::TransferTimeout => b"transfer-timeout",
    }
}

fn wifi_sdio_backplane_read_status(
    status: wifi_sdio::WifiSdioBackplaneReadStatus,
) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioBackplaneReadStatus::Ready => b"ready",
        wifi_sdio::WifiSdioBackplaneReadStatus::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::InvalidAddress => b"invalid-address",
        wifi_sdio::WifiSdioBackplaneReadStatus::InvalidCount => b"invalid-count",
        wifi_sdio::WifiSdioBackplaneReadStatus::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioBackplaneReadStatus::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::WindowHighWriteFailed => {
            b"window-high-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneReadStatus::WindowMidWriteFailed => b"window-mid-write-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::WindowLowWriteFailed => b"window-low-write-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::DataSetupBusy => b"data-setup-busy",
        wifi_sdio::WifiSdioBackplaneReadStatus::Cmd52Failed => b"cmd52-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::Cmd53Failed => b"cmd53-failed",
        wifi_sdio::WifiSdioBackplaneReadStatus::BufferReadTimeout => b"buffer-read-timeout",
        wifi_sdio::WifiSdioBackplaneReadStatus::BufferEnableTimeout => b"buffer-enable-timeout",
        wifi_sdio::WifiSdioBackplaneReadStatus::TransferTimeout => b"transfer-timeout",
    }
}

fn wifi_sdio_backplane_write8_status(
    status: wifi_sdio::WifiSdioBackplaneWrite8Status,
) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioBackplaneWrite8Status::Ready => b"ready",
        wifi_sdio::WifiSdioBackplaneWrite8Status::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioBackplaneWrite8Status::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioBackplaneWrite8Status::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioBackplaneWrite8Status::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioBackplaneWrite8Status::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioBackplaneWrite8Status::WindowHighWriteFailed => {
            b"window-high-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneWrite8Status::WindowMidWriteFailed => {
            b"window-mid-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneWrite8Status::WindowLowWriteFailed => {
            b"window-low-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneWrite8Status::Cmd52Failed => b"cmd52-failed",
    }
}

fn wifi_sdio_backplane_write32_status(
    status: wifi_sdio::WifiSdioBackplaneWrite32Status,
) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioBackplaneWrite32Status::Ready => b"ready",
        wifi_sdio::WifiSdioBackplaneWrite32Status::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioBackplaneWrite32Status::InvalidAddress => b"invalid-address",
        wifi_sdio::WifiSdioBackplaneWrite32Status::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioBackplaneWrite32Status::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioBackplaneWrite32Status::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioBackplaneWrite32Status::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioBackplaneWrite32Status::WindowHighWriteFailed => {
            b"window-high-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneWrite32Status::WindowMidWriteFailed => {
            b"window-mid-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneWrite32Status::WindowLowWriteFailed => {
            b"window-low-write-failed"
        }
        wifi_sdio::WifiSdioBackplaneWrite32Status::DataSetupBusy => b"data-setup-busy",
        wifi_sdio::WifiSdioBackplaneWrite32Status::Cmd52Failed => b"cmd52-failed",
        wifi_sdio::WifiSdioBackplaneWrite32Status::Cmd53Failed => b"cmd53-failed",
        wifi_sdio::WifiSdioBackplaneWrite32Status::TransferTimeout => b"transfer-timeout",
        wifi_sdio::WifiSdioBackplaneWrite32Status::DataLineBusy => b"data-line-busy",
        wifi_sdio::WifiSdioBackplaneWrite32Status::ReadbackCmd52Failed => b"readback-cmd52-failed",
    }
}

fn wifi_sdio_socram_probe_status(status: wifi_sdio::WifiSdioSocramProbeStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioSocramProbeStatus::Ready => b"ready",
        wifi_sdio::WifiSdioSocramProbeStatus::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::InvalidAddress => b"invalid-address",
        wifi_sdio::WifiSdioSocramProbeStatus::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioSocramProbeStatus::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::ArmDisableFailed => b"arm-disable-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::SocramDisableFailed => b"socram-disable-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::SocramResetFailed => b"socram-reset-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::BankIndexWriteFailed => b"bank-index-write-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::BankPdaWriteFailed => b"bank-pda-write-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::OriginalReadFailed => b"original-read-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::ProbeWriteFailed => b"probe-write-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::ProbeReadbackMismatch => b"probe-readback-mismatch",
        wifi_sdio::WifiSdioSocramProbeStatus::RestoreWriteFailed => b"restore-write-failed",
        wifi_sdio::WifiSdioSocramProbeStatus::RestoreReadbackMismatch => {
            b"restore-readback-mismatch"
        }
    }
}

fn wifi_sdio_socram_block_probe_status(
    status: wifi_sdio::WifiSdioSocramBlockProbeStatus,
) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioSocramBlockProbeStatus::Ready => b"ready",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::InvalidAddress => b"invalid-address",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::ArmDisableFailed => b"arm-disable-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::SocramDisableFailed => b"socram-disable-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::SocramResetFailed => b"socram-reset-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::BankIndexWriteFailed => {
            b"bank-index-write-failed"
        }
        wifi_sdio::WifiSdioSocramBlockProbeStatus::BankPdaWriteFailed => b"bank-pda-write-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::OriginalReadFailed => b"original-read-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::ProbeWriteFailed => b"probe-write-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::ProbeReadFailed => b"probe-read-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::ProbeReadbackMismatch => {
            b"probe-readback-mismatch"
        }
        wifi_sdio::WifiSdioSocramBlockProbeStatus::RestoreWriteFailed => b"restore-write-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::RestoreReadFailed => b"restore-read-failed",
        wifi_sdio::WifiSdioSocramBlockProbeStatus::RestoreReadbackMismatch => {
            b"restore-readback-mismatch"
        }
    }
}

fn wifi_sdio_firmware_load_status(status: wifi_sdio::WifiSdioFirmwareLoadStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioFirmwareLoadStatus::Ready => b"ready",
        wifi_sdio::WifiSdioFirmwareLoadStatus::BlobMissing => b"blob-missing",
        wifi_sdio::WifiSdioFirmwareLoadStatus::BlobTooLarge => b"blob-too-large",
        wifi_sdio::WifiSdioFirmwareLoadStatus::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioFirmwareLoadStatus::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::ArmDisableFailed => b"arm-disable-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::SocramDisableFailed => b"socram-disable-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::SocramResetFailed => b"socram-reset-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::BankIndexWriteFailed => b"bank-index-write-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::BankPdaWriteFailed => b"bank-pda-write-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::WriteFailed => b"write-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::VerifyReadFailed => b"verify-read-failed",
        wifi_sdio::WifiSdioFirmwareLoadStatus::VerifyMismatch => b"verify-mismatch",
    }
}

fn wifi_sdio_firmware_start_status(
    status: wifi_sdio::WifiSdioFirmwareStartStatus,
) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioFirmwareStartStatus::Ready => b"ready",
        wifi_sdio::WifiSdioFirmwareStartStatus::FirmwareFailed => b"firmware-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::NvramMissing => b"nvram-missing",
        wifi_sdio::WifiSdioFirmwareStartStatus::NvramTooLarge => b"nvram-too-large",
        wifi_sdio::WifiSdioFirmwareStartStatus::NvramWriteFailed => b"nvram-write-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::NvramVerifyReadFailed => {
            b"nvram-verify-read-failed"
        }
        wifi_sdio::WifiSdioFirmwareStartStatus::NvramVerifyMismatch => b"nvram-verify-mismatch",
        wifi_sdio::WifiSdioFirmwareStartStatus::NvramSizeWriteFailed => b"nvram-size-write-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::PullUpWriteFailed => b"pull-up-write-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::IoEnableWriteFailed => b"io-enable-write-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::InterruptEnableWriteFailed => {
            b"interrupt-enable-write-failed"
        }
        wifi_sdio::WifiSdioFirmwareStartStatus::ArmResetFailed => b"arm-reset-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::ArmStateReadFailed => b"arm-state-read-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::HtReadFailed => b"ht-read-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::HtTimeout => b"ht-timeout",
        wifi_sdio::WifiSdioFirmwareStartStatus::HostInterruptMaskWriteFailed => {
            b"host-interrupt-mask-write-failed"
        }
        wifi_sdio::WifiSdioFirmwareStartStatus::FunctionInterruptMaskWriteFailed => {
            b"function-interrupt-mask-write-failed"
        }
        wifi_sdio::WifiSdioFirmwareStartStatus::WatermarkWriteFailed => b"watermark-write-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::F2ReadyReadFailed => b"f2-ready-read-failed",
        wifi_sdio::WifiSdioFirmwareStartStatus::F2ReadyTimeout => b"f2-ready-timeout",
    }
}

fn wifi_sdio_f2_frame_status(status: wifi_sdio::WifiSdioF2FrameStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioF2FrameStatus::NotRun => b"not-run",
        wifi_sdio::WifiSdioF2FrameStatus::Ready => b"ready",
        wifi_sdio::WifiSdioF2FrameStatus::HeaderReadFailed => b"header-read-failed",
        wifi_sdio::WifiSdioF2FrameStatus::InvalidHeader => b"invalid-header",
        wifi_sdio::WifiSdioF2FrameStatus::FrameTooShort => b"frame-too-short",
        wifi_sdio::WifiSdioF2FrameStatus::FrameTooLarge => b"frame-too-large",
        wifi_sdio::WifiSdioF2FrameStatus::UnsupportedLength => b"unsupported-length",
        wifi_sdio::WifiSdioF2FrameStatus::BodyReadFailed => b"body-read-failed",
    }
}

fn wifi_sdio_f2_control_status(status: wifi_sdio::WifiSdioF2ControlStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioF2ControlStatus::NotRun => b"not-run",
        wifi_sdio::WifiSdioF2ControlStatus::Ready => b"ready",
        wifi_sdio::WifiSdioF2ControlStatus::NoTxCredit => b"no-tx-credit",
        wifi_sdio::WifiSdioF2ControlStatus::PacketTooLarge => b"packet-too-large",
        wifi_sdio::WifiSdioF2ControlStatus::DataSetupBusy => b"data-setup-busy",
        wifi_sdio::WifiSdioF2ControlStatus::Cmd53Failed => b"cmd53-failed",
        wifi_sdio::WifiSdioF2ControlStatus::TransferTimeout => b"transfer-timeout",
        wifi_sdio::WifiSdioF2ControlStatus::DataLineBusy => b"data-line-busy",
    }
}

fn wifi_sdio_wlc_up_status(status: wifi_sdio::WifiSdioWlcUpStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioWlcUpStatus::Ready => b"ready",
        wifi_sdio::WifiSdioWlcUpStatus::HtRequestFailed => b"ht-request-failed",
        wifi_sdio::WifiSdioWlcUpStatus::SendFailed => b"send-failed",
        wifi_sdio::WifiSdioWlcUpStatus::StartupFrameFailed => b"startup-frame-failed",
        wifi_sdio::WifiSdioWlcUpStatus::UnexpectedStartupFrame => b"unexpected-startup-frame",
        wifi_sdio::WifiSdioWlcUpStatus::ResponseFrameFailed => b"response-frame-failed",
        wifi_sdio::WifiSdioWlcUpStatus::UnexpectedResponseFrame => b"unexpected-response-frame",
        wifi_sdio::WifiSdioWlcUpStatus::CdcStatusError => b"cdc-status-error",
    }
}

fn wifi_sdio_get_version_status(status: wifi_sdio::WifiSdioGetVersionStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioGetVersionStatus::Ready => b"ready",
        wifi_sdio::WifiSdioGetVersionStatus::HtRequestFailed => b"ht-request-failed",
        wifi_sdio::WifiSdioGetVersionStatus::SendFailed => b"send-failed",
        wifi_sdio::WifiSdioGetVersionStatus::ResponseFrameFailed => b"response-frame-failed",
        wifi_sdio::WifiSdioGetVersionStatus::UnexpectedResponseFrame => {
            b"unexpected-response-frame"
        }
        wifi_sdio::WifiSdioGetVersionStatus::CdcStatusError => b"cdc-status-error",
    }
}

fn wifi_sdio_get_mpc_status(status: wifi_sdio::WifiSdioGetMpcStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioGetMpcStatus::Ready => b"ready",
        wifi_sdio::WifiSdioGetMpcStatus::HtRequestFailed => b"ht-request-failed",
        wifi_sdio::WifiSdioGetMpcStatus::SendFailed => b"send-failed",
        wifi_sdio::WifiSdioGetMpcStatus::ResponseFrameFailed => b"response-frame-failed",
        wifi_sdio::WifiSdioGetMpcStatus::UnexpectedResponseFrame => b"unexpected-response-frame",
        wifi_sdio::WifiSdioGetMpcStatus::CdcStatusError => b"cdc-status-error",
    }
}

fn wifi_sdio_interrupt_ack_status(status: wifi_sdio::WifiSdioInterruptAckStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioInterruptAckStatus::Ready => b"ready",
        wifi_sdio::WifiSdioInterruptAckStatus::IntStatusReadFailed => b"int-status-read-failed",
        wifi_sdio::WifiSdioInterruptAckStatus::MailboxReadFailed => b"mailbox-read-failed",
        wifi_sdio::WifiSdioInterruptAckStatus::MailboxAckWriteFailed => b"mailbox-ack-write-failed",
        wifi_sdio::WifiSdioInterruptAckStatus::InterruptClearWriteFailed => {
            b"interrupt-clear-write-failed"
        }
        wifi_sdio::WifiSdioInterruptAckStatus::FinalIntStatusReadFailed => {
            b"final-int-status-read-failed"
        }
    }
}

fn wifi_sdio_interrupt_state_status(
    status: wifi_sdio::WifiSdioInterruptStateStatus,
) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioInterruptStateStatus::Ready => b"ready",
        wifi_sdio::WifiSdioInterruptStateStatus::IoEnableReadFailed => b"io-enable-read-failed",
        wifi_sdio::WifiSdioInterruptStateStatus::IoReadyReadFailed => b"io-ready-read-failed",
        wifi_sdio::WifiSdioInterruptStateStatus::InterruptEnableReadFailed => {
            b"interrupt-enable-read-failed"
        }
        wifi_sdio::WifiSdioInterruptStateStatus::InterruptPendingReadFailed => {
            b"interrupt-pending-read-failed"
        }
        wifi_sdio::WifiSdioInterruptStateStatus::BusControlReadFailed => b"bus-control-read-failed",
    }
}

fn wifi_sdio_keep_awake_status(status: wifi_sdio::WifiSdioKeepAwakeStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioKeepAwakeStatus::Ready => b"ready",
        wifi_sdio::WifiSdioKeepAwakeStatus::Timeout => b"timeout",
    }
}

fn wifi_sdio_ht_request_status(status: wifi_sdio::WifiSdioHtRequestStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioHtRequestStatus::Ready => b"ready",
        wifi_sdio::WifiSdioHtRequestStatus::WriteFailed => b"write-failed",
        wifi_sdio::WifiSdioHtRequestStatus::ReadFailed => b"read-failed",
        wifi_sdio::WifiSdioHtRequestStatus::Timeout => b"timeout",
    }
}

fn wifi_sdio_core_state_status(status: wifi_sdio::WifiSdioCoreStateStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioCoreStateStatus::Ready => b"ready",
        wifi_sdio::WifiSdioCoreStateStatus::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioCoreStateStatus::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioCoreStateStatus::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioCoreStateStatus::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioCoreStateStatus::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioCoreStateStatus::WindowHighWriteFailed => b"window-high-write-failed",
        wifi_sdio::WifiSdioCoreStateStatus::WindowMidWriteFailed => b"window-mid-write-failed",
        wifi_sdio::WifiSdioCoreStateStatus::WindowLowWriteFailed => b"window-low-write-failed",
        wifi_sdio::WifiSdioCoreStateStatus::IoctrlReadFailed => b"ioctrl-read-failed",
        wifi_sdio::WifiSdioCoreStateStatus::ResetctrlReadFailed => b"resetctrl-read-failed",
        wifi_sdio::WifiSdioCoreStateStatus::ResetstatusReadFailed => b"resetstatus-read-failed",
    }
}

fn wifi_sdio_core_reset_status(status: wifi_sdio::WifiSdioCoreResetStatus) -> &'static [u8] {
    match status {
        wifi_sdio::WifiSdioCoreResetStatus::Ready => b"ready",
        wifi_sdio::WifiSdioCoreResetStatus::SetupFailed => b"setup-failed",
        wifi_sdio::WifiSdioCoreResetStatus::AlpWriteFailed => b"alp-write-failed",
        wifi_sdio::WifiSdioCoreResetStatus::AlpReadFailed => b"alp-read-failed",
        wifi_sdio::WifiSdioCoreResetStatus::AlpTimeout => b"alp-timeout",
        wifi_sdio::WifiSdioCoreResetStatus::AlpClearFailed => b"alp-clear-failed",
        wifi_sdio::WifiSdioCoreResetStatus::WindowHighWriteFailed => b"window-high-write-failed",
        wifi_sdio::WifiSdioCoreResetStatus::WindowMidWriteFailed => b"window-mid-write-failed",
        wifi_sdio::WifiSdioCoreResetStatus::WindowLowWriteFailed => b"window-low-write-failed",
        wifi_sdio::WifiSdioCoreResetStatus::BeforeIoctrlReadFailed => b"before-ioctrl-read-failed",
        wifi_sdio::WifiSdioCoreResetStatus::BeforeResetctrlReadFailed => {
            b"before-resetctrl-read-failed"
        }
        wifi_sdio::WifiSdioCoreResetStatus::BeforeResetstatusReadFailed => {
            b"before-resetstatus-read-failed"
        }
        wifi_sdio::WifiSdioCoreResetStatus::DisableResetctrlReadFailed => {
            b"disable-resetctrl-read-failed"
        }
        wifi_sdio::WifiSdioCoreResetStatus::DisableIoctrlWriteFailed => {
            b"disable-ioctrl-write-failed"
        }
        wifi_sdio::WifiSdioCoreResetStatus::DisableIoctrlReadFailed => {
            b"disable-ioctrl-read-failed"
        }
        wifi_sdio::WifiSdioCoreResetStatus::DisableResetctrlWriteFailed => {
            b"disable-resetctrl-write-failed"
        }
        wifi_sdio::WifiSdioCoreResetStatus::ResetIoctrlWriteFailed => b"reset-ioctrl-write-failed",
        wifi_sdio::WifiSdioCoreResetStatus::ResetIoctrlReadFailed => b"reset-ioctrl-read-failed",
        wifi_sdio::WifiSdioCoreResetStatus::ResetctrlWriteFailed => b"resetctrl-write-failed",
        wifi_sdio::WifiSdioCoreResetStatus::FinalIoctrlWriteFailed => b"final-ioctrl-write-failed",
        wifi_sdio::WifiSdioCoreResetStatus::AfterIoctrlReadFailed => b"after-ioctrl-read-failed",
        wifi_sdio::WifiSdioCoreResetStatus::AfterResetctrlReadFailed => {
            b"after-resetctrl-read-failed"
        }
        wifi_sdio::WifiSdioCoreResetStatus::AfterResetstatusReadFailed => {
            b"after-resetstatus-read-failed"
        }
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

fn wifi_sdio_cmd53_read_report(
    report: wifi_sdio::WifiSdioCmd53ReadReport,
) -> lisp::WifiSdioCmd53ReadReport {
    lisp::WifiSdioCmd53ReadReport {
        status: wifi_sdio_cmd53_read_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        function: report.function,
        address: report.address,
        count: report.count,
        response: report.response,
        bytes: report.bytes,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_backplane_read_report(
    report: wifi_sdio::WifiSdioBackplaneReadReport,
) -> lisp::WifiSdioBackplaneReadReport {
    lisp::WifiSdioBackplaneReadReport {
        status: wifi_sdio_backplane_read_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        address: report.address,
        count: report.count,
        window_base: report.window_base,
        window_address: report.window_address,
        response: report.response,
        bytes: report.bytes,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_backplane_write8_report(
    report: wifi_sdio::WifiSdioBackplaneWrite8Report,
) -> lisp::WifiSdioBackplaneWrite8Report {
    lisp::WifiSdioBackplaneWrite8Report {
        status: wifi_sdio_backplane_write8_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        address: report.address,
        value: report.value,
        window_base: report.window_base,
        window_address: report.window_address,
        response: report.response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_backplane_write32_report(
    report: wifi_sdio::WifiSdioBackplaneWrite32Report,
) -> lisp::WifiSdioBackplaneWrite32Report {
    lisp::WifiSdioBackplaneWrite32Report {
        status: wifi_sdio_backplane_write32_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        address: report.address,
        value: report.value,
        window_base: report.window_base,
        window_address: report.window_address,
        response: report.response,
        readback: report.readback,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_socram_probe_report(
    report: wifi_sdio::WifiSdioSocramProbeReport,
) -> lisp::WifiSdioSocramProbeReport {
    lisp::WifiSdioSocramProbeReport {
        status: wifi_sdio_socram_probe_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        write_status: wifi_sdio_backplane_write32_status(report.write_status),
        address: report.address,
        pattern: report.pattern,
        original: report.original,
        readback: report.readback,
        restored: report.restored,
        last_response: report.last_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_socram_block_probe_report(
    report: wifi_sdio::WifiSdioSocramBlockProbeReport,
) -> lisp::WifiSdioSocramBlockProbeReport {
    lisp::WifiSdioSocramBlockProbeReport {
        status: wifi_sdio_socram_block_probe_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        read_status: wifi_sdio_backplane_read_status(report.read_status),
        write_status: wifi_sdio_backplane_write32_status(report.write_status),
        address: report.address,
        seed: report.seed,
        original_checksum: report.original_checksum,
        readback_checksum: report.readback_checksum,
        restored_checksum: report.restored_checksum,
        mismatch_index: report.mismatch_index,
        mismatch_expected: report.mismatch_expected,
        mismatch_actual: report.mismatch_actual,
        last_response: report.last_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_firmware_load_report(
    report: wifi_sdio::WifiSdioFirmwareLoadReport,
) -> lisp::WifiSdioFirmwareLoadReport {
    lisp::WifiSdioFirmwareLoadReport {
        status: wifi_sdio_firmware_load_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        read_status: wifi_sdio_backplane_read_status(report.read_status),
        write_status: wifi_sdio_backplane_write32_status(report.write_status),
        firmware_bytes: report.firmware_bytes,
        processed_bytes: report.processed_bytes,
        chunk_count: report.chunk_count,
        firmware_checksum: report.firmware_checksum,
        verify_checksum: report.verify_checksum,
        mismatch_offset: report.mismatch_offset,
        mismatch_expected: report.mismatch_expected,
        mismatch_actual: report.mismatch_actual,
        last_response: report.last_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_firmware_start_report(
    report: wifi_sdio::WifiSdioFirmwareStartReport,
) -> lisp::WifiSdioFirmwareStartReport {
    lisp::WifiSdioFirmwareStartReport {
        status: wifi_sdio_firmware_start_status(report.status),
        firmware_status: wifi_sdio_firmware_load_status(report.firmware_status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        read_status: wifi_sdio_backplane_read_status(report.read_status),
        write_status: wifi_sdio_backplane_write32_status(report.write_status),
        firmware_bytes: report.firmware_bytes,
        nvram_bytes: report.nvram_bytes,
        nvram_rounded_bytes: report.nvram_rounded_bytes,
        nvram_address: report.nvram_address,
        nvram_size_word: report.nvram_size_word,
        firmware_checksum: report.firmware_checksum,
        nvram_checksum: report.nvram_checksum,
        nvram_verify_checksum: report.nvram_verify_checksum,
        mismatch_offset: report.mismatch_offset,
        mismatch_expected: report.mismatch_expected,
        mismatch_actual: report.mismatch_actual,
        arm_before: wifi_sdio_core_snapshot_report(report.arm_before),
        arm_after: wifi_sdio_core_snapshot_report(report.arm_after),
        ht_clock_csr: report.ht_clock_csr,
        ht_attempts: report.ht_attempts,
        io_enable: report.io_enable,
        io_ready: report.io_ready,
        f2_attempts: report.f2_attempts,
        last_response: report.last_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_f2_header_report(
    report: wifi_sdio::WifiSdioF2HeaderReport,
) -> lisp::WifiSdioF2HeaderReport {
    lisp::WifiSdioF2HeaderReport {
        status: wifi_sdio_cmd53_read_status(report.status),
        response: report.response,
        bytes: report.bytes,
        length: report.length,
        checksum: report.checksum,
        valid: report.valid,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_f2_frame_report(
    report: wifi_sdio::WifiSdioF2FrameReport,
) -> lisp::WifiSdioF2FrameReport {
    lisp::WifiSdioF2FrameReport {
        status: wifi_sdio_f2_frame_status(report.status),
        header_status: wifi_sdio_cmd53_read_status(report.header_status),
        body_status: wifi_sdio_cmd53_read_status(report.body_status),
        header_response: report.header_response,
        body_response: report.body_response,
        bytes: report.bytes,
        byte_count: report.byte_count,
        length: report.length,
        checksum: report.checksum,
        valid: report.valid,
        sequence: report.sequence,
        channel_and_flags: report.channel_and_flags,
        channel: report.channel,
        flags: report.flags,
        next_length: report.next_length,
        header_length: report.header_length,
        wireless_flow_control: report.wireless_flow_control,
        bus_data_credit: report.bus_data_credit,
        reserved0: report.reserved0,
        reserved1: report.reserved1,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_f2_control_report(
    report: wifi_sdio::WifiSdioF2ControlReport,
) -> lisp::WifiSdioF2ControlReport {
    lisp::WifiSdioF2ControlReport {
        status: wifi_sdio_f2_control_status(report.status),
        initial_tx_credit: report.initial_tx_credit,
        packet_length: report.packet_length,
        write_response: report.write_response,
        host_normal_int: report.host_normal_int,
        host_error_int: report.host_error_int,
        write_last_error: report.write_last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_wlc_up_report(report: wifi_sdio::WifiSdioWlcUpReport) -> lisp::WifiSdioWlcUpReport {
    lisp::WifiSdioWlcUpReport {
        status: wifi_sdio_wlc_up_status(report.status),
        ht_status: wifi_sdio_ht_request_status(report.ht_status),
        ht_attempts: report.ht_attempts,
        ht_write_response: report.ht_write_response,
        ht_read_value: report.ht_read_value,
        ht_read_response: report.ht_read_response,
        ht_available: report.ht_available,
        send_status: wifi_sdio_f2_control_status(report.send_status),
        send_packet_length: report.send_packet_length,
        send_write_response: report.send_write_response,
        startup_status: wifi_sdio_f2_frame_status(report.startup_status),
        startup_length: report.startup_length,
        startup_channel: report.startup_channel,
        startup_bus_data_credit: report.startup_bus_data_credit,
        response_status: wifi_sdio_f2_frame_status(report.response_status),
        response_length: report.response_length,
        response_sequence: report.response_sequence,
        response_channel: report.response_channel,
        response_bus_data_credit: report.response_bus_data_credit,
        cdc_command: report.cdc_command,
        cdc_length: report.cdc_length,
        cdc_flags: report.cdc_flags,
        cdc_id: report.cdc_id,
        cdc_status: report.cdc_status,
        ht_last_error: report.ht_last_error.map(wifi_sdio_command_error_report),
        send_last_error: report.send_last_error.map(wifi_sdio_command_error_report),
        startup_last_error: report
            .startup_last_error
            .map(wifi_sdio_command_error_report),
        response_last_error: report
            .response_last_error
            .map(wifi_sdio_command_error_report),
        host_normal_int: report.host_normal_int,
        host_error_int: report.host_error_int,
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_get_version_report(
    report: wifi_sdio::WifiSdioGetVersionReport,
) -> lisp::WifiSdioGetVersionReport {
    lisp::WifiSdioGetVersionReport {
        status: wifi_sdio_get_version_status(report.status),
        ht_status: wifi_sdio_ht_request_status(report.ht_status),
        ht_attempts: report.ht_attempts,
        ht_write_response: report.ht_write_response,
        ht_read_value: report.ht_read_value,
        ht_read_response: report.ht_read_response,
        ht_available: report.ht_available,
        send_status: wifi_sdio_f2_control_status(report.send_status),
        send_packet_length: report.send_packet_length,
        send_write_response: report.send_write_response,
        response_status: wifi_sdio_f2_frame_status(report.response_status),
        response_length: report.response_length,
        response_sequence: report.response_sequence,
        response_channel: report.response_channel,
        response_bus_data_credit: report.response_bus_data_credit,
        cdc_command: report.cdc_command,
        cdc_length: report.cdc_length,
        cdc_flags: report.cdc_flags,
        cdc_id: report.cdc_id,
        cdc_status: report.cdc_status,
        version: report.version,
        ht_last_error: report.ht_last_error.map(wifi_sdio_command_error_report),
        send_last_error: report.send_last_error.map(wifi_sdio_command_error_report),
        response_last_error: report
            .response_last_error
            .map(wifi_sdio_command_error_report),
        host_normal_int: report.host_normal_int,
        host_error_int: report.host_error_int,
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_get_mpc_report(report: wifi_sdio::WifiSdioGetMpcReport) -> lisp::WifiSdioGetMpcReport {
    lisp::WifiSdioGetMpcReport {
        status: wifi_sdio_get_mpc_status(report.status),
        ht_status: wifi_sdio_ht_request_status(report.ht_status),
        ht_attempts: report.ht_attempts,
        ht_write_response: report.ht_write_response,
        ht_read_value: report.ht_read_value,
        ht_read_response: report.ht_read_response,
        ht_available: report.ht_available,
        send_status: wifi_sdio_f2_control_status(report.send_status),
        send_packet_length: report.send_packet_length,
        send_write_response: report.send_write_response,
        response_status: wifi_sdio_f2_frame_status(report.response_status),
        response_length: report.response_length,
        response_sequence: report.response_sequence,
        response_channel: report.response_channel,
        response_bus_data_credit: report.response_bus_data_credit,
        cdc_command: report.cdc_command,
        cdc_length: report.cdc_length,
        cdc_flags: report.cdc_flags,
        cdc_id: report.cdc_id,
        cdc_status: report.cdc_status,
        value: report.value,
        ht_last_error: report.ht_last_error.map(wifi_sdio_command_error_report),
        send_last_error: report.send_last_error.map(wifi_sdio_command_error_report),
        response_last_error: report
            .response_last_error
            .map(wifi_sdio_command_error_report),
        host_normal_int: report.host_normal_int,
        host_error_int: report.host_error_int,
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_interrupt_ack_report(
    report: wifi_sdio::WifiSdioInterruptAckReport,
) -> lisp::WifiSdioInterruptAckReport {
    lisp::WifiSdioInterruptAckReport {
        status: wifi_sdio_interrupt_ack_status(report.status),
        int_status_before: report.int_status_before,
        mailbox_data: report.mailbox_data,
        mailbox_ack_value: report.mailbox_ack_value,
        clear_value: report.clear_value,
        int_status_after: report.int_status_after,
        host_normal_int_before: report.host_normal_int_before,
        host_error_int_before: report.host_error_int_before,
        host_normal_int_after: report.host_normal_int_after,
        host_error_int_after: report.host_error_int_after,
        int_status_response: report.int_status_response,
        mailbox_response: report.mailbox_response,
        mailbox_ack_response: report.mailbox_ack_response,
        mailbox_ack_readback: report.mailbox_ack_readback,
        clear_response: report.clear_response,
        clear_readback: report.clear_readback,
        final_response: report.final_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_interrupt_state_report(
    report: wifi_sdio::WifiSdioInterruptStateReport,
) -> lisp::WifiSdioInterruptStateReport {
    lisp::WifiSdioInterruptStateReport {
        status: wifi_sdio_interrupt_state_status(report.status),
        io_enable: report.io_enable,
        io_ready: report.io_ready,
        interrupt_enable: report.interrupt_enable,
        interrupt_pending: report.interrupt_pending,
        bus_control: report.bus_control,
        master_enabled: report.master_enabled,
        function1_enabled: report.function1_enabled,
        function2_enabled: report.function2_enabled,
        function1_ready: report.function1_ready,
        function2_ready: report.function2_ready,
        function1_pending: report.function1_pending,
        function2_pending: report.function2_pending,
        host_card_interrupt: report.host_card_interrupt,
        io_enable_response: report.io_enable_response,
        io_ready_response: report.io_ready_response,
        interrupt_enable_response: report.interrupt_enable_response,
        interrupt_pending_response: report.interrupt_pending_response,
        bus_control_response: report.bus_control_response,
        host_normal_int: report.host_normal_int,
        host_error_int: report.host_error_int,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_keep_awake_report(
    report: wifi_sdio::WifiSdioKeepAwakeReport,
) -> lisp::WifiSdioKeepAwakeReport {
    lisp::WifiSdioKeepAwakeReport {
        status: wifi_sdio_keep_awake_status(report.status),
        attempts: report.attempts,
        write_value: report.write_value,
        first_write_response: report.first_write_response,
        second_write_response: report.second_write_response,
        retry_write_response: report.retry_write_response,
        read_value: report.read_value,
        read_response: report.read_response,
        keep_wl_kso: report.keep_wl_kso,
        wl_devon: report.wl_devon,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_ht_request_report(
    report: wifi_sdio::WifiSdioHtRequestReport,
) -> lisp::WifiSdioHtRequestReport {
    lisp::WifiSdioHtRequestReport {
        status: wifi_sdio_ht_request_status(report.status),
        attempts: report.attempts,
        write_value: report.write_value,
        write_response: report.write_response,
        read_value: report.read_value,
        read_response: report.read_response,
        ht_available: report.ht_available,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_host_reset_report(
    report: wifi_sdio::WifiSdioHostResetReport,
) -> lisp::WifiSdioHostResetReport {
    lisp::WifiSdioHostResetReport {
        command_reset: report.command_reset,
        data_reset: report.data_reset,
        before: wifi_sdio_host_report(report.before),
        after: wifi_sdio_host_report(report.after),
    }
}

fn wifi_sdio_abort_read_report(
    report: wifi_sdio::WifiSdioAbortReadReport,
) -> lisp::WifiSdioAbortReadReport {
    lisp::WifiSdioAbortReadReport {
        io_abort_response: report.io_abort_response,
        frame_control_response: report.frame_control_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_core_state_report(
    report: wifi_sdio::WifiSdioCoreStateReport,
) -> lisp::WifiSdioCoreStateReport {
    lisp::WifiSdioCoreStateReport {
        status: wifi_sdio_core_state_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        base: report.base,
        ioctrl: report.ioctrl,
        resetctrl: report.resetctrl,
        resetstatus: report.resetstatus,
        clock_enabled: report.clock_enabled,
        force_gated: report.force_gated,
        in_reset: report.in_reset,
        reset_busy: report.reset_busy,
        core_up: report.core_up,
        last_response: report.last_response,
        last_error: report.last_error.map(wifi_sdio_command_error_report),
        host: wifi_sdio_host_report(report.host),
    }
}

fn wifi_sdio_core_snapshot_report(
    snapshot: wifi_sdio::WifiSdioCoreSnapshot,
) -> lisp::WifiSdioCoreSnapshotReport {
    lisp::WifiSdioCoreSnapshotReport {
        ioctrl: snapshot.ioctrl,
        resetctrl: snapshot.resetctrl,
        resetstatus: snapshot.resetstatus,
        clock_enabled: snapshot.clock_enabled,
        force_gated: snapshot.force_gated,
        in_reset: snapshot.in_reset,
        reset_busy: snapshot.reset_busy,
        core_up: snapshot.core_up,
    }
}

fn wifi_sdio_core_reset_report(
    report: wifi_sdio::WifiSdioCoreResetReport,
) -> lisp::WifiSdioCoreResetReport {
    lisp::WifiSdioCoreResetReport {
        status: wifi_sdio_core_reset_status(report.status),
        setup_status: wifi_sdio_backplane_status(report.setup_status),
        base: report.base,
        before: wifi_sdio_core_snapshot_report(report.before),
        after: wifi_sdio_core_snapshot_report(report.after),
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
        block_size: snapshot.block_size,
        block_count: snapshot.block_count,
        sdmasa: snapshot.sdmasa,
        adma_sa_low: snapshot.adma_sa_low,
        adma_id_low: snapshot.adma_id_low,
        adma_err_stat: snapshot.adma_err_stat,
        bgap_ctrl: snapshot.bgap_ctrl,
        host_ctrl1: snapshot.host_ctrl1,
        host_ctrl2: snapshot.host_ctrl2,
        capabilities1: snapshot.capabilities1,
        capabilities2: snapshot.capabilities2,
        mbiu_ctrl: snapshot.mbiu_ctrl,
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
