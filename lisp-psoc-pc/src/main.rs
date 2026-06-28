#![no_std]
#![no_main]

use core::fmt::{self, Write};

use cortex_m_rt::entry;
use hal::{board, console};
use panic_halt as _;
use psoc6_pac::Peripherals;

mod hal;
mod lisp;
mod lisp_fat;
mod lisp_store;
mod wifi_credentials;
mod wifi_resources;

const SYSCLK_HZ: u32 = 50_000_000;
const CONSOLE_POLL_DELAY_US: u32 = 50;
const WIFI_NET_REPL_SERVICE_INTERVAL_MS: u32 = 50;
#[cfg(feature = "wifi-boot-smoke")]
#[cfg(not(feature = "wifi-dhcp-boot-smoke"))]
const WIFI_BOOT_SMOKE_FORMS: [&[u8]; 3] = [
    b"(console-echo off)",
    b"(wifi-connect-local)",
    b"(wifi-link-status)",
];
#[cfg(feature = "wifi-dhcp-boot-smoke")]
#[cfg(not(feature = "wifi-arp-boot-smoke"))]
const WIFI_BOOT_SMOKE_FORMS: [&[u8]; 4] = [
    b"(console-echo off)",
    b"(wifi-connect-local)",
    b"(wifi-link-status)",
    b"(wifi-dhcp-acquire)",
];
#[cfg(feature = "wifi-arp-boot-smoke")]
const WIFI_ARP_BOOT_SMOKE_MAGIC: u32 = 0x4152_5030;
#[cfg(feature = "wifi-arp-boot-smoke")]
const WIFI_NETWORK_BOOT_SMOKE_MARKER_WORDS: usize = 32;
#[cfg(feature = "wifi-dns-boot-smoke")]
const WIFI_DNS_BOOT_SMOKE_NAME: &[u8] = b"example.com";
#[cfg(feature = "wifi-net-repl-boot-smoke")]
const WIFI_NET_REPL_BOOT_SMOKE_FORM: &[u8] = b"(wifi-net-repl-once 240)";
#[cfg(feature = "wifi-net-repl-boot-smoke")]
const WIFI_NET_REPL_BOOT_SMOKE_POLL_FRAMES: u32 = 240;
#[cfg(feature = "wifi-net-repl-service-boot-smoke")]
const WIFI_NET_REPL_SERVICE_BOOT_SMOKE_POLL_FRAMES: u8 = 1;
#[cfg(feature = "wifi-arp-boot-smoke")]
#[no_mangle]
pub static mut WIFI_ARP_BOOT_SMOKE_MARKER: [u32; WIFI_NETWORK_BOOT_SMOKE_MARKER_WORDS] =
    [0; WIFI_NETWORK_BOOT_SMOKE_MARKER_WORDS];
#[cfg(feature = "storage-boot-smoke")]
const STORAGE_BOOT_SMOKE_FORMS: [&[u8]; 4] = [
    b"(save-file \"boot.lisp\" \"(+ 40 2)\")",
    b"(read-file \"boot.lisp\")",
    b"(ls)",
    b"(load \"boot.lisp\")",
];
#[cfg(feature = "storage-format-boot-smoke")]
const STORAGE_FORMAT_BOOT_SMOKE_FORMS: [&[u8]; 1] = [b"(fat-format)"];

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = unsafe { Peripherals::steal() };

    extern "C" {
        static _svectors: u32;
    }
    unsafe {
        cp.SCB.vtor.write(&_svectors as *const u32 as u32);
    }

    board::State::configure_hardware(&p);
    console::Console::configure_hardware(&p);

    let mut delay = cortex_m::delay::Delay::new(cp.SYST, SYSCLK_HZ);
    let mut console = console::Console::new(&p.SCB5);
    let mut machine = lisp::Machine::new();
    let mut board_state = board::State::new();

    board_state.reboot_marker(&p, &mut delay);
    if let Err(error) = machine.bootstrap() {
        writeln!(console, "\nLisp bootstrap failed: {}", error.message()).ok();
    }

    writeln!(console, "\nPSoC6 lisp-psoc-pc").ok();
    writeln!(
        console,
        "UART: SCB5 P5.1 TX / P5.0 RX, {} 8N1",
        console::UART_BAUD
    )
    .ok();
    #[cfg(not(feature = "storage-boot-smoke"))]
    {
        let mut board = board_state.lisp_board(&p);
        load_boot_file(&mut machine, &mut board, &mut console).ok();
    }
    #[cfg(feature = "storage-boot-smoke")]
    writeln!(console, "boot.lisp preload: skipped for storage smoke").ok();
    #[cfg(feature = "storage-format-boot-smoke")]
    {
        let mut board = board_state.lisp_board(&p);
        run_storage_format_boot_smoke(&mut machine, &mut board, &mut console).ok();
    }
    #[cfg(feature = "wifi-boot-smoke")]
    {
        let mut board = board_state.lisp_board(&p);
        run_wifi_boot_smoke(&mut machine, &mut board, &mut console).ok();
    }
    #[cfg(feature = "storage-boot-smoke")]
    {
        let mut board = board_state.lisp_board(&p);
        run_storage_boot_smoke(&mut machine, &mut board, &mut console).ok();
    }
    writeln!(console, "Try: (help), (ls), (cat \"boot.lisp\"), (+ 1 2 3)").ok();
    console.prompt();

    let mut line = [0u8; 384];
    let mut line_len = 0usize;
    let mut tick_accumulated_us = 0u32;
    let mut net_repl_service_accumulated_ms = 0u32;

    loop {
        while let Some(byte) = console.read_byte() {
            match byte {
                b'\r' | b'\n' => {
                    console.write_bytes(b"\n");
                    let input = trim_ascii(&line[..line_len]);
                    if input.is_empty() {
                        line_len = 0;
                        console.prompt();
                    } else if input_needs_more(input) {
                        if line_len < line.len() {
                            line[line_len] = b'\n';
                            line_len += 1;
                            console.continuation_prompt();
                        } else {
                            line_len = 0;
                            writeln!(console, "error: input line too long").ok();
                            console.prompt();
                        }
                    } else {
                        let mut board = board_state.lisp_board(&p);
                        machine.eval_line(input, &mut board, &mut console).ok();
                        line_len = 0;
                        console.prompt();
                    }
                }
                0x03 => {
                    line_len = 0;
                    console.write_bytes(b"^C");
                    console.prompt();
                }
                0x08 | 0x7f => {
                    if line_len > 0 {
                        line_len -= 1;
                        if machine.console_echo_enabled() {
                            console.write_bytes(b"\x08 \x08");
                        }
                    }
                }
                b if b.is_ascii_graphic() || b == b' ' => {
                    if line_len < line.len() {
                        line[line_len] = b;
                        line_len += 1;
                        if machine.console_echo_enabled() {
                            console.write_byte(b);
                        }
                    } else {
                        console.write_byte(b'\x07');
                    }
                }
                _ => {}
            }
        }

        delay.delay_us(CONSOLE_POLL_DELAY_US);
        tick_accumulated_us += CONSOLE_POLL_DELAY_US;
        while tick_accumulated_us >= 1000 {
            tick_accumulated_us -= 1000;
            board_state.tick_ms(&p);
            net_repl_service_accumulated_ms = net_repl_service_accumulated_ms.saturating_add(1);
        }

        if machine.wifi_net_repl_service_enabled() {
            if net_repl_service_accumulated_ms >= WIFI_NET_REPL_SERVICE_INTERVAL_MS {
                net_repl_service_accumulated_ms = 0;
                let mut board = board_state.lisp_board(&p);
                machine.poll_wifi_net_repl_service(&mut board);
                #[cfg(feature = "wifi-net-repl-service-boot-smoke")]
                write_wifi_net_repl_service_marker(&machine);
            }
        } else {
            net_repl_service_accumulated_ms = 0;
        }
    }
}

#[cfg(feature = "wifi-boot-smoke")]
#[cfg(not(feature = "wifi-arp-boot-smoke"))]
fn run_wifi_boot_smoke<B: lisp::Board, W: Write>(
    machine: &mut lisp::Machine,
    board: &mut B,
    output: &mut W,
) -> fmt::Result {
    writeln!(output, "wifi boot smoke: start")?;
    for form in WIFI_BOOT_SMOKE_FORMS {
        write!(output, "boot> ")?;
        write_ascii(form, output)?;
        writeln!(output)?;
        machine.eval_line(form, board, output)?;
    }
    writeln!(output, "wifi boot smoke: done")
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn run_wifi_boot_smoke<B: lisp::Board, W: Write>(
    _machine: &mut lisp::Machine,
    board: &mut B,
    _output: &mut W,
) -> fmt::Result {
    clear_wifi_arp_boot_smoke_marker();
    write_wifi_arp_boot_smoke_marker(0, WIFI_ARP_BOOT_SMOKE_MAGIC);
    write_wifi_arp_boot_smoke_marker(1, 1);

    let ssid = match static_string_bytes(wifi_credentials::local_ssid()) {
        Some(value) => value,
        None => {
            write_wifi_arp_boot_smoke_marker(1, 0xe001);
            return Ok(());
        }
    };
    let passphrase = match static_string_bytes(wifi_credentials::local_passphrase()) {
        Some(value) => value,
        None => {
            write_wifi_arp_boot_smoke_marker(1, 0xe002);
            return Ok(());
        }
    };

    if !run_wifi_prepare_join_marker_smoke(board) {
        return Ok(());
    }
    write_wifi_arp_boot_smoke_marker(1, 2);
    write_wifi_arp_boot_smoke_marker(2, 1);

    let join = board.wifi_join_wpa2(ssid, passphrase);
    write_wifi_arp_boot_smoke_marker(1, 3);
    write_wifi_arp_boot_smoke_marker(3, status_word(join.status));
    write_wifi_arp_boot_smoke_marker(12, join.join_flags);
    if !status_ready(join.status) {
        return Ok(());
    }

    let dhcp = board.wifi_dhcp_acquire();
    write_wifi_arp_boot_smoke_marker(1, 4);
    write_wifi_arp_boot_smoke_marker(4, status_word(dhcp.status));
    write_wifi_arp_boot_smoke_marker(5, bool_word(dhcp.lease_valid));
    write_wifi_arp_boot_smoke_marker(13, dhcp.leased_ip_address);
    write_wifi_arp_boot_smoke_marker(14, dhcp.router);
    if !status_ready(dhcp.status) {
        return Ok(());
    }

    let lease = board.wifi_lease_status();
    write_wifi_arp_boot_smoke_marker(1, 5);
    write_wifi_arp_boot_smoke_marker(6, status_word(lease.status));
    write_wifi_arp_boot_smoke_marker(5, bool_word(lease.lease_valid));
    write_wifi_arp_boot_smoke_marker(13, lease.ip_address);
    write_wifi_arp_boot_smoke_marker(14, lease.router);
    if !status_ready(lease.status) {
        return Ok(());
    }

    let arp = board.wifi_arp_router();
    write_wifi_arp_boot_smoke_marker(1, 6);
    write_wifi_arp_boot_smoke_marker(7, status_word(arp.status));
    write_wifi_arp_boot_smoke_marker(8, status_word(arp.reply_poll_status));
    write_wifi_arp_boot_smoke_marker(9, status_word(arp.reply_parse_status));
    write_wifi_arp_boot_smoke_marker(10, bool_word(arp.router_mac_present));
    write_wifi_arp_boot_smoke_marker(11, bool_word(arp.router_mac_stored));
    write_wifi_arp_boot_smoke_marker(15, arp.router_mac_hash);
    if !status_ready(arp.status) {
        return Ok(());
    }
    write_wifi_arp_boot_smoke_marker(1, 7);

    #[cfg(feature = "wifi-dns-boot-smoke")]
    {
        let dns_name = match static_string_bytes(WIFI_DNS_BOOT_SMOKE_NAME) {
            Some(value) => value,
            None => {
                write_wifi_arp_boot_smoke_marker(1, 0xe003);
                return Ok(());
            }
        };
        let dns = board.wifi_dns_query(dns_name);
        write_wifi_arp_boot_smoke_marker(1, 8);
        write_wifi_arp_boot_smoke_marker(16, status_word(dns.status));
        write_wifi_arp_boot_smoke_marker(17, status_word(dns.response_poll_status));
        write_wifi_arp_boot_smoke_marker(18, status_word(dns.response_parse_status));
        write_wifi_arp_boot_smoke_marker(19, bool_word(dns.answer_valid));
        write_wifi_arp_boot_smoke_marker(20, dns.answer_ip_address);
        write_wifi_arp_boot_smoke_marker(21, dns.answer_ttl_seconds);
        write_wifi_arp_boot_smoke_marker(22, dns.response_answer_count as u32);
        if !status_ready(dns.status) {
            return Ok(());
        }
        write_wifi_arp_boot_smoke_marker(1, 9);
    }

    #[cfg(feature = "wifi-net-repl-boot-smoke")]
    {
        let mut output = SilentWriter;
        write_wifi_arp_boot_smoke_marker(1, 10);
        write_wifi_arp_boot_smoke_marker(23, WIFI_NET_REPL_BOOT_SMOKE_POLL_FRAMES);
        let result = _machine.eval_line(WIFI_NET_REPL_BOOT_SMOKE_FORM, board, &mut output);
        write_wifi_arp_boot_smoke_marker(24, bool_word(result.is_ok()));
        write_wifi_arp_boot_smoke_marker(1, 11);
    }

    #[cfg(feature = "wifi-net-repl-service-boot-smoke")]
    {
        write_wifi_arp_boot_smoke_marker(1, 12);
        _machine.enable_wifi_net_repl_service(WIFI_NET_REPL_SERVICE_BOOT_SMOKE_POLL_FRAMES);
        write_wifi_arp_boot_smoke_marker(25, WIFI_NET_REPL_SERVICE_BOOT_SMOKE_POLL_FRAMES as u32);
        write_wifi_arp_boot_smoke_marker(26, bool_word(_machine.wifi_net_repl_service_enabled()));
        write_wifi_net_repl_service_marker(_machine);
        write_wifi_arp_boot_smoke_marker(1, 13);
    }

    Ok(())
}

#[cfg(feature = "wifi-net-repl-boot-smoke")]
struct SilentWriter;

#[cfg(feature = "wifi-net-repl-boot-smoke")]
impl Write for SilentWriter {
    fn write_str(&mut self, _value: &str) -> fmt::Result {
        Ok(())
    }
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn static_string_bytes(bytes: &[u8]) -> Option<lisp::StringBytes> {
    if bytes.is_empty() || bytes.len() > lisp::MAX_STRING_BYTES {
        return None;
    }

    let mut value = lisp::StringBytes {
        len: bytes.len() as u8,
        bytes: [0; lisp::MAX_STRING_BYTES],
    };
    let mut index = 0usize;
    while index < bytes.len() {
        value.bytes[index] = bytes[index];
        index += 1;
    }
    Some(value)
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn run_wifi_prepare_join_marker_smoke<B: lisp::Board>(board: &mut B) -> bool {
    write_wifi_arp_boot_smoke_marker(27, 1);
    let sdio = board.wifi_sdio_init();
    write_wifi_arp_boot_smoke_marker(28, status_word(sdio.status));
    if !status_ready(sdio.status) {
        write_wifi_arp_boot_smoke_marker(1, 0xe101);
        return false;
    }

    write_wifi_arp_boot_smoke_marker(27, 2);
    let firmware = board.wifi_start_firmware();
    write_wifi_arp_boot_smoke_marker(28, status_word(firmware.status));
    if !status_ready(firmware.status) {
        write_wifi_arp_boot_smoke_marker(1, 0xe102);
        return false;
    }

    write_wifi_arp_boot_smoke_marker(27, 3);
    let wlc_up = board.wifi_wlc_up();
    write_wifi_arp_boot_smoke_marker(28, status_word(wlc_up.status));
    if !status_ready(wlc_up.status) {
        write_wifi_arp_boot_smoke_marker(1, 0xe103);
        return false;
    }

    write_wifi_arp_boot_smoke_marker(27, 4);
    let ack = board.wifi_ack_interrupts();
    write_wifi_arp_boot_smoke_marker(28, status_word(ack.status));
    if !status_ready(ack.status) {
        write_wifi_arp_boot_smoke_marker(1, 0xe104);
        return false;
    }

    write_wifi_arp_boot_smoke_marker(27, 5);
    let clm = board.wifi_load_clm();
    write_wifi_arp_boot_smoke_marker(28, status_word(clm.status));
    if !status_ready(clm.status) {
        write_wifi_arp_boot_smoke_marker(1, 0xe105);
        return false;
    }

    write_wifi_arp_boot_smoke_marker(27, 6);
    let tx_glomming = board.wifi_disable_tx_glomming();
    write_wifi_arp_boot_smoke_marker(28, status_word(tx_glomming.status));
    if !status_ready(tx_glomming.status) {
        write_wifi_arp_boot_smoke_marker(1, 0xe106);
        return false;
    }

    write_wifi_arp_boot_smoke_marker(27, 7);
    let country = board.wifi_set_country(*b"XX", -1);
    write_wifi_arp_boot_smoke_marker(28, status_word(country.status));
    if !status_ready(country.status) {
        write_wifi_arp_boot_smoke_marker(1, 0xe107);
        return false;
    }

    write_wifi_arp_boot_smoke_marker(27, 8);
    true
}

#[cfg(feature = "wifi-net-repl-service-boot-smoke")]
fn write_wifi_net_repl_service_marker(machine: &lisp::Machine) {
    write_wifi_arp_boot_smoke_marker(23, status_word(machine.wifi_net_repl_service_last_status()));
    write_wifi_arp_boot_smoke_marker(
        24,
        status_word(machine.wifi_net_repl_service_last_reply_status()),
    );
    write_wifi_arp_boot_smoke_marker(29, machine.wifi_net_repl_service_polls());
    write_wifi_arp_boot_smoke_marker(30, machine.wifi_net_repl_service_requests_handled());
    write_wifi_arp_boot_smoke_marker(31, machine.wifi_net_repl_service_last_sequence());
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn status_ready(status: &[u8]) -> bool {
    status == b"ready"
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn status_word(status: &[u8]) -> u32 {
    bool_word(status_ready(status))
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn bool_word(value: bool) -> u32 {
    if value {
        1
    } else {
        0
    }
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn clear_wifi_arp_boot_smoke_marker() {
    let mut index = 0usize;
    while index < WIFI_NETWORK_BOOT_SMOKE_MARKER_WORDS {
        write_wifi_arp_boot_smoke_marker(index, 0);
        index += 1;
    }
}

#[cfg(feature = "wifi-arp-boot-smoke")]
fn write_wifi_arp_boot_smoke_marker(index: usize, value: u32) {
    if index >= WIFI_NETWORK_BOOT_SMOKE_MARKER_WORDS {
        return;
    }

    unsafe {
        let marker = core::ptr::addr_of_mut!(WIFI_ARP_BOOT_SMOKE_MARKER).cast::<u32>();
        core::ptr::write_volatile(marker.add(index), value);
    }
}

#[cfg(feature = "storage-boot-smoke")]
fn run_storage_boot_smoke<B: lisp::Board, W: Write>(
    machine: &mut lisp::Machine,
    board: &mut B,
    output: &mut W,
) -> fmt::Result {
    writeln!(output, "storage boot smoke: start")?;
    for form in STORAGE_BOOT_SMOKE_FORMS {
        write!(output, "boot> ")?;
        write_ascii(form, output)?;
        writeln!(output)?;
        machine.eval_line(form, board, output)?;
    }
    writeln!(output, "storage boot smoke: done")
}

#[cfg(feature = "storage-format-boot-smoke")]
fn run_storage_format_boot_smoke<B: lisp::Board, W: Write>(
    machine: &mut lisp::Machine,
    board: &mut B,
    output: &mut W,
) -> fmt::Result {
    writeln!(output, "storage format boot smoke: start")?;
    for form in STORAGE_FORMAT_BOOT_SMOKE_FORMS {
        write!(output, "boot> ")?;
        write_ascii(form, output)?;
        writeln!(output)?;
        machine.eval_line(form, board, output)?;
    }
    writeln!(output, "storage format boot smoke: done")
}

fn load_boot_file<B: lisp::Board, W: Write>(
    machine: &mut lisp::Machine,
    board: &mut B,
    output: &mut W,
) -> fmt::Result {
    let path = string_bytes(b"boot.lisp");
    match machine.load_file(path, board, output) {
        Ok(lisp::LoadFileOutcome::Loaded(value)) => {
            write!(output, "boot.lisp => ")?;
            machine.write_value_to(value, output)?;
            writeln!(output)
        }
        Ok(lisp::LoadFileOutcome::NotReady(report)) => {
            if report.status != b"not-found" {
                write!(output, "boot.lisp: ")?;
                write_ascii(report.status, output)?;
                writeln!(output)?;
            }
            Ok(())
        }
        Err(error) => writeln!(output, "boot.lisp error: {}", error.message()),
    }
}

fn string_bytes(value: &[u8]) -> lisp::StringBytes {
    let mut bytes = [0u8; lisp::MAX_STRING_BYTES];
    let mut index = 0usize;
    while index < value.len() && index < lisp::MAX_STRING_BYTES {
        bytes[index] = value[index];
        index += 1;
    }
    lisp::StringBytes {
        len: index as u8,
        bytes,
    }
}

fn write_ascii<W: Write>(value: &[u8], output: &mut W) -> fmt::Result {
    for &byte in value {
        output.write_char(byte as char)?;
    }
    Ok(())
}

fn input_needs_more(input: &[u8]) -> bool {
    let mut depth = 0u16;
    let mut in_comment = false;

    for &byte in input {
        if in_comment {
            if byte == b'\n' {
                in_comment = false;
            }
            continue;
        }

        match byte {
            b';' => in_comment = true,
            b'(' => depth = depth.saturating_add(1),
            b')' => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    depth != 0
}

fn trim_ascii(mut input: &[u8]) -> &[u8] {
    while matches!(input.first(), Some(b' ' | b'\t')) {
        input = &input[1..];
    }
    while matches!(input.last(), Some(b' ' | b'\t')) {
        input = &input[..input.len() - 1];
    }
    input
}
