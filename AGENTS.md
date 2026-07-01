# Repository Guidelines

## Purpose

This repository contains Rust experiments for the CY8CPROTO-062-4343W
PSoC 6 Wi-Fi BT Prototyping Kit. The current active line of work is a
quiet serial-console firmware that will grow into a tiny Lisp machine
with microSD storage and Wi-Fi support.

Keep `README.md` for stable project setup, `quickstart.org` for the short
operator workflow, `board-status-brief.org` for a short human-readable status
overview, `runtime-processes.org` for the cooperative-process design direction,
and `bringup.org` for dated, experiment-level notes, measured hardware facts,
toolchain decisions, and commands that are known to work on the attached board.

## Project Structure

- `psoc6-pac/`: generated peripheral access crate and device metadata.
- `psoc6-cm0-bootloader/`: CM0+ bootloader that prepares and releases
  the CM4 application. Keep it focused on boot, reset, clocks needed for
  boot, and handoff.
- `lisp-psoc-pc/`: current CM4 serial console firmware. This is the
  active place for the tiny Lisp machine work unless a better split is
  deliberately introduced.
- `probe-rs-targets/`: local probe-rs target metadata and compatibility
  notes.
- `backups/`: flash and firmware backups captured before risky changes.
- `quickstart.org`: short build, flash, console, network REPL, and recovery
  workflow.
- `board-status-brief.org`: short readable overview of current board
  capabilities, core usage, Lisp shape, and nearest planned features.
- `runtime-processes.org`: current Lisp process/coroutine status and the
  cooperative scheduler design direction.
- `bringup.org`: running lab notebook. Update it when a command,
  hardware fact, or implementation decision becomes useful to preserve.

## Programming Policy

No shortcuts, oversimplifications, or cheap hacks. When implementing
features or fixing bugs:

- If asked to provide something that already exists, point it out instead
  of creating an alias or duplicate path.
- Never leave TODOs, FIXMEs, or placeholder implementations.
- Never simplify requirements without explicit approval.
- Never use "good enough for now" as the justification for shipped code.
- Implement the actual complexity when the hardware or runtime requires
  it.
- Prefer established, readable solutions over clever ones.
- Keep code reasonably generic and stratified: do not bury low-level
  register programming inside command parsing, Lisp evaluation, or
  console UX code.
- Prefer smaller functions with descriptive names, even when a helper is
  only used once.
- Do not abbreviate identifiers unless the abbreviation is the hardware
  name from the datasheet, PAC, schematic, or board support package.
- Do not introduce em dashes.

Keep boot files as boot files. Startup and handoff files should contain
startup, mounting, configuration, interrupt/vector setup, and core
handoff logic, not growing command interpreters or unrelated application
behavior.

Split by coherent workflow instead of dumping helpers into one utility
blob. Good splits for the active firmware are console I/O, command
parsing, Lisp reader/evaluator/printer, board peripherals, storage, Wi-Fi,
and diagnostics.

Preserve existing public entry points and console commands where
practical. Move one coherent concern at a time so behavior stays stable
and commits stay reviewable.

## Hardware Safety

Treat hardware writes as externally visible side effects.

- Keep the firmware quiet during unattended work. Do not enable heartbeat
  blinking, run LED-on tests, or flash a deliberately blinking image while
  the user has asked for the LED to stay off.
- LED4 on this board is active-low on `P13.7`. Driving `P13.7` high turns
  it off; driving it low turns it on.
- Confirm pin mappings against the schematic, BSP, or live register
  inspection before programming unfamiliar peripherals.
- Prefer diagnostic commands that report register state without changing
  board state.
- Before risky flash changes, keep or create a backup under `backups/`
  and document the restore path in `bringup.org`.
- Do not use destructive git commands to recover from hardware bring-up
  mistakes.

## Secrets and Local State

- Never commit Wi-Fi credentials, `/var/lib/iwd` contents, tokens, serial
  numbers that are not already public, or generated secret headers.
- Do not print secrets to logs, terminal transcripts, docs, or commit
  messages.
- Any generated credential material must live in an ignored local path.
- Skip enterprise profiles such as eduroam unless the user explicitly asks
  to support them.
- Use `tools/prepare-wifi-credentials.scm` for local IWD PSK extraction.
  It writes `.local/wifi/selected.env`. With `--all`, it also writes
  numbered env files under `.local/wifi/profiles/` for every non-enterprise
  PSK profile with a stored secret. Its output must stay limited to
  counts/paths/field lengths rather than SSIDs or secret values.
- Use `tools/prepare-wifi-credential-blobs.scm` to convert
  `.local/wifi/selected.env` into ignored byte blobs under
  `.local/wifi/credentials/` for `--wifi-credentials` builds. Its output must
  stay limited to counts/paths/field lengths rather than SSIDs or secret
  values.
- Use `tools/prepare-wifi-resources.scm` for local CYW4343W firmware, CLM,
  and NVRAM extraction. It writes `.local/wifi/resources/`, including a
  generated local MAC address, and its output must stay limited to
  counts/paths/sizes/hashes rather than generated address values.
- Treat other local checkouts, such as `/root/zmk` or `/root/project/hiisi`,
  as read-only references unless the user explicitly asks to edit them.

## Rust Style

- Format Rust code with `cargo fmt` in the crate being changed.
- Do not leave unrelated rustfmt spillover unstaged and unaccounted for.
  Include deliberate formatting changes in the commit that needs them, or
  avoid touching unrelated crates.
- Use `snake_case` for variables and functions, `CamelCase` for types, and
  descriptive names for hardware helpers.
- Keep `unsafe` and raw register manipulation narrowly scoped and named
  after the hardware behavior being established.
- Add succinct comments only where the register sequence or hardware
  assumption is not self-explanatory.

## Build, Test, and Flash

After any code, docs, or config change, run the smallest validation that
proves the changed surface is sane before committing. For docs-only
changes, `git diff --check` is enough unless the docs include commands
that were changed and should be tested.

For the active console firmware, use the repo-local Chez Scheme tooling by
default:

```sh
tools/setup-modustoolbox.scm --check
tools/prepare-wifi-resources.scm --check
tools/build-lisp.scm
tools/flash-lisp.scm
```

Use `tools/build-flash-lisp.scm` when a change should be built and flashed
in one step. Use `tools/serial-console.scm` for an interactive console;
it mirrors bytes into `.local/logs/serial-console.log` by default. Use
`tools/serial-console.scm --tail-log` when the user should watch console output
without opening the UART device. Use `tools/send-lisp.scm '(form ...)'` for
one-off Lisp forms. The serial setup must enable `CREAD`, and the current
known-good UART smoke path uses the default 500 ms byte pacing plus the default
500 ms post-CR hold. Faster pacing has corrupted live input, and even 1 second
byte pacing can still corrupt RX on the current KitProg path. Treat UART as a
recovery/status path and prefer UDP or Telnet once Wi-Fi is up. These scripts
encode the current known-good build, pack, flash, and serial-console commands
and should be kept up to date when that flow changes.

Use `tools/build-lisp.scm --uart-pin-probe` or
`tools/build-flash-lisp.scm --uart-pin-probe` only for USB-UART physical-path
diagnostics. It bit-bangs a short `P5.1` GPIO probe at 9600 baud and does not
start the REPL. Flash a normal or recovery image immediately afterward.

Use `tools/build-lisp.scm --wifi-firmware` or
`tools/build-flash-lisp.scm --wifi-firmware` only when the local CYW4343W
firmware blob should be embedded in the CM4 image for `(wifi-load-firmware)`.
The default build keeps the blob out of the image and returns `blob-missing`
from that form.

Use `tools/build-lisp.scm --wifi-firmware --wifi-credentials` or
`tools/build-flash-lisp.scm --wifi-firmware --wifi-credentials` only when the
ignored local SSID/passphrase blobs should be embedded in the CM4 image for
`(wifi-connect-local)`. Never commit the generated blobs or credential-bearing
firmware artifacts.

Use `tools/build-lisp.scm --wifi-boot-smoke` or
`tools/build-flash-lisp.scm --wifi-boot-smoke` only for unattended Wi-Fi
association smoke tests while UART RX is unreliable. It implies the Wi-Fi
firmware and local credential features, then runs `(console-echo off)`,
`(wifi-connect-local)`, and `(wifi-link-status)` at boot.

Use `tools/build-lisp.scm --wifi-dhcp-boot-smoke` or
`tools/build-flash-lisp.scm --wifi-dhcp-boot-smoke` only for unattended Wi-Fi
association plus DHCP lease acquisition smoke tests while UART RX is
unreliable. It implies `--wifi-boot-smoke`, then also runs
`(wifi-dhcp-acquire)` at boot. Flash a non-smoke image immediately afterward.

Use `tools/build-lisp.scm --wifi-arp-boot-smoke` or
`tools/build-flash-lisp.scm --wifi-arp-boot-smoke` only for unattended Wi-Fi
association, DHCP lease acquisition, and router ARP smoke tests while UART is
unreliable. It implies `--wifi-dhcp-boot-smoke`, runs a UART-silent smoke path,
and records `WIFI_ARP_BOOT_SMOKE_MARKER` in RAM for SWD inspection. Flash a
non-smoke image immediately afterward.

Use `tools/build-lisp.scm --wifi-dns-boot-smoke` or
`tools/build-flash-lisp.scm --wifi-dns-boot-smoke` only for unattended Wi-Fi
association, DHCP lease acquisition, router ARP, and DNS smoke tests while UART
is unreliable. It implies `--wifi-arp-boot-smoke`, resolves `example.com`, and
extends `WIFI_ARP_BOOT_SMOKE_MARKER` with DNS status and answer fields for SWD
inspection. Flash a non-smoke image immediately afterward.

Use `tools/build-lisp.scm --wifi-net-repl-boot-smoke` or
`tools/build-flash-lisp.scm --wifi-net-repl-boot-smoke` only for unattended
Wi-Fi association, DHCP lease acquisition, router ARP, DNS, and one framed UDP
REPL request smoke test while UART is unreliable. It implies
`--wifi-dns-boot-smoke`, runs `(wifi-net-repl-once 240)` silently at boot, and
extends `WIFI_ARP_BOOT_SMOKE_MARKER` with the network-REPL wait status. Flash a
non-smoke image immediately afterward.

Use `tools/build-lisp.scm --wifi-net-repl-service-boot-smoke` or
`tools/build-flash-lisp.scm --wifi-net-repl-service-boot-smoke` only for
unattended Wi-Fi association, DHCP lease acquisition, router ARP, DNS, and
background framed UDP REPL service smoke tests while UART is unreliable. It
implies `--wifi-dns-boot-smoke`, enables the background service at boot, and
then enters the normal firmware loop. Flash a non-smoke image immediately
afterward.

Use `tools/send-net-repl.scm --host BOARD_IP '(form ...)'` for host-side framed
UDP REPL requests. It writes ignored binary request/response files under
`.local/net-repl/`, uses `ncat` when available or `nc` as a fallback, and
prints response metadata plus payload text.
The script sends `LPS3`, sequence, request checksum, and payload by default.
With `--read-only`, the script sends `LPS5`, the same sequence/checksum/payload
layout, and current firmware rejects requests outside its conservative
read-only allowlist before evaluation.
Use `--legacy-request` only when talking to an older flashed image that still
expects `LPS0` requests. Current firmware replies with `LPS2`, sequence,
response checksum, and payload; the script verifies the checksum, sends an
optional `LPS4` ACK with the response checksum, and still accepts legacy `LPS1`
replies while older flashed images are being replaced. The board records ACK
counts in `(wifi-net-repl-service status)` and does not evaluate ACK frames.
Use a longer receive window for FAT-backed forms like `save-file`,
`append-file`, `save-defs`, `read-file`, `load`, and `cat`; use `--wait 60`
when validating larger files over the UDP REPL. `save-defs` writes reloadable
global data and top-level lambdas to one FAT source file, and reports captured
closures or other non-reloadable bindings as skipped. `load` can read source
files up to 512 bytes from FAT. `read-file` reports the full length but only
includes inline content for files that fit in a 96-byte Lisp string; `cat`
returns an error for larger files.
Use `(http-get "http://example.com/")` as the current high-level Wi-Fi smoke
test on the TP-Link network. It validates DNS, raw TCP open, HTTP/1.0 GET,
response preview parsing, and RST/ACK cleanup. Use
`(http-get "http://192.168.0.1/")` to test HTTP while skipping DNS, and
`(wifi-tcp-syn-ip #xc0a80001 80)` as the lower-level raw TCP smoke test.
Use `(wifi-tcp-listen-once 2323 80)` as the current raw inbound TCP smoke test.
It is a one-shot diagnostic for accepting a SYN, replying SYN-ACK, observing the
host ACK or RST, and sending RST/ACK cleanup; it is not Telnet yet. Start it
through the UDP REPL in the background, then connect with
`nc -vz -w 5 BOARD_IP 2323`. Keep its Lisp report compact because the current
UDP REPL client falls back to GNU `nc` when `ncat` is unavailable, and that path
has only been reliable with single captured replies below about 1 KiB. With
`ncat`, `tools/send-net-repl.scm` has validated full 1412-byte `LPS2` datagrams.
Use `(wifi-tcp-receive-once 2323 80)` as the current raw inbound TCP payload
smoke test. Start it through the UDP REPL in the background, then run
`printf 'hello from tcp\n' | ncat -w 5 -q 1 BOARD_IP 2323 || true`. The host
currently reports a reset because the diagnostic closes with RST/ACK after
capturing the payload; success is the board report showing `payload.status` as
`ready` and the expected preview string.
Use `(wifi-tcp-repl-once 2323 80)` as the current one-shot TCP Lisp REPL smoke
test. Start it through the UDP REPL in the background, then run
`printf '(+ 40 2)\n' | ncat -w 8 -q 3 BOARD_IP 2323`. Success is `ncat`
printing `=> 42` and the board report showing `request.status`, `eval.status`,
and `reply.status` as `ready`. This is a one-shot diagnostic and not a
persistent Telnet service yet.
Use `(wifi-tcp-repl-service on 2323 1)` as the current Telnet Lisp REPL service
smoke test. Enable it through the UDP REPL, connect with `telnet BOARD_IP 2323`
or `ncat`, then send `(wifi-tcp-repl-service off)` over Telnet before returning
to UDP checks.
Success is a Telnet prompt, standard IAC option refusal for unsupported options,
NVT CR/LF line handling, pretty-printed Lisp results, `requests-handled`
increasing, and the last eval status as `ready`. For scripted stock-telnet
tests, keep stdin open long enough for connection setup, for example
`(sleep 1; printf '(+ 3 4)\r\n(wifi-tcp-repl-service off)\r\n'; sleep 3) | telnet BOARD_IP 2323`.
The service is still single-session. The UDP REPL service and Telnet service now
poll concurrently through a small HAL demux cache that preserves framed UDP REPL
requests and active TCP REPL packets when the other service reads them first.
Use `(wifi-demux-status)` to inspect pending counts and cache/drop counters.
After Telnet turns itself off, validate UDP again with a small form. Do not
auto-start TCP from `boot.lisp` until the packet dispatcher is promoted from a
small service cache to a deliberate board-wide network dispatch layer.
Use `tools/send-net-repl.scm --color` only when ANSI payload coloring is wanted;
plain output is the default. Use `--read-only` as a conservative accidental-send
guard for status, directory, FAT info, Wi-Fi link/lease status, demux status,
and simple-path file-read forms. It is a host-client and firmware guard against
mistakes, not an authentication or authorization boundary. Use `--payload-only`
for clean REPL-like output when transport metadata is not needed. Each invocation uses
per-process ignored files under `.local/net-repl/`; use explicit unique
sequences when comparing concurrent UDP requests.

To make a normal quiet Wi-Fi image start the UDP REPL service from microSD,
install this `boot.lisp` through the temporary service-smoke image:

```scheme
(begin (wifi-network-bootstrap) (wifi-net-repl-service on))
```

Then rebuild and flash the quiet image with
`tools/build-flash-lisp.scm --wifi-firmware --wifi-credentials`. Keep the
temporary smoke image on the board only as long as needed for installation and
verification.

Use `tools/build-lisp.scm --skip-boot-file` or
`tools/build-flash-lisp.scm --skip-boot-file` for recovery and Wi-Fi debugging
when FAT `boot.lisp` contains a blocking startup form. The flag leaves the SD
card untouched and skips automatic `boot.lisp` loading; combine it with
`--wifi-firmware --wifi-credentials` when the recovery image still needs local
Wi-Fi resources embedded.

Use `tools/build-lisp.scm --storage-boot-smoke` or
`tools/build-flash-lisp.scm --storage-boot-smoke` only for unattended FAT
storage smoke tests while UART RX is unreliable. It skips automatic
`boot.lisp` preload for the smoke image, then runs `save-file`, `read-file`,
`ls`, and `load` forms at boot.

Use `tools/build-lisp.scm --storage-format-boot-smoke` or
`tools/build-flash-lisp.scm --storage-format-boot-smoke` only when the
inserted microSD card contents may be destroyed. It implies
`--storage-boot-smoke`, formats the card as FAT32 at boot, and then runs the
FAT storage smoke forms. Flash a non-formatting image immediately afterward.

The scripts keep vendor downloads and generated local state under `.local/`
and discover Infineon OpenOCD from `OPENOCD_ROOT`,
`MODUSTOOLBOX_OPENOCD_ROOT`, `MODUSTOOLBOX_ROOT`,
`.local/ModusToolbox`, or `/opt/ModusToolbox`. Use
`tools/setup-modustoolbox.scm --archive /path/to/ModusToolbox-linux.tar.gz`
to install a local ModusToolbox tarball into the ignored repo-local tools
directory.
The current KitProg3 firmware loader is installed under ignored
`.local/fw-loader/` when needed. Use its `fw-loader` binary for KitProg3
device-list, firmware update, mode, and UART flow-control checks, and document
any device firmware or bridge-setting change in `bringup.org`. The current
validated bridge preference is `kp3-hid` when `kp3-bulk` corrupts UART; check
with `.local/fw-loader/fw-loader/bin/fw-loader --device-list` and switch with
`.local/fw-loader/fw-loader/bin/fw-loader --mode kp3-hid` when needed.

The underlying manual flow for the active console firmware is:

```sh
cd lisp-psoc-pc
RUSTFLAGS=-Awarnings cargo build --release --features use-bootloader
arm-none-eabi-objcopy -O binary \
  target/thumbv7em-none-eabihf/release/lisp-psoc-pc \
  ../psoc6-cm0-bootloader/src/app.bin

cd ../psoc6-cm0-bootloader
RUSTFLAGS=-Awarnings cargo build --release

cd ..
/opt/ModusToolbox/tools_3.4/openocd/bin/openocd \
  -s /opt/ModusToolbox/tools_3.4/openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/psoc6_2m.cfg \
  -c "program psoc6-cm0-bootloader/target/thumbv6m-none-eabi/release/psoc6-cm0-bootloader verify reset exit"
```

OpenOCD's `target/psoc6_2m.cfg` warning is known. Keep using the working
target file until a replacement command is tested and documented.

## Commit Policy

- Commit messages use primitive style, title line only, under 72
  characters.
- No `Co-Authored-By` lines.
- Prefer rebase over merge; avoid merge commits.
- Commit after each completed logical change.
- Push after each commit when a remote is configured and the network/key
  setup allows it.
- Keep commits tiny, granular, and single-purpose.
- Do not bundle unrelated regressions, unrelated experiments, or broad
  refactors into one commit just because the files are nearby.
- Prefer one commit per regression fix, one commit per feature slice, and
  one commit per logical file-splitting group.
- If a task is broad, land it as a sequence of small vertical commits
  instead of one final polish commit.
- Do not include unrelated dirty worktree changes in a commit.
- Add short, dated entries to `bringup.org` for notable hardware,
  toolchain, or runtime behavior decisions.

## Worktree Discipline

The worktree may contain user changes. Never revert, overwrite, or stage
changes you did not make unless the user explicitly asks for that exact
operation. If unrelated dirty files exist, leave them alone. If user
changes affect the active task, work with them and preserve their intent.

Use non-interactive git commands where practical. Avoid destructive
commands such as `git reset --hard` or `git checkout --` unless the user
explicitly asks for them.
