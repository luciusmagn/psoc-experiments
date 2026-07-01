# psoc6-demo

A demo project for the CY8C624ABZI-S2D44A0 SoC, as fitted to the
*CY8CPROTO-062-4343W PSoC 6 Wi-Fi BT Prototyping kit*.

See [quickstart.org](quickstart.org) for the short board-control workflow and
[runtime-processes.org](runtime-processes.org) for the cooperative process
direction.

Schematic: <https://www.infineon.com/dgdl/Infineon-CY8CPROTO-062-4343W_Schematic-PCBDesignData-v01_00-EN.pdf?fileId=8ac78c8c7d0d8da4017d0f010c6d183a>

## Building

The active firmware is split across the two cores in the PSoC 6:

- `lisp-psoc-pc/` builds the CM4 serial-console Lisp firmware.
- `psoc6-cm0-bootloader/` builds the CM0+ bootloader that packages and
  releases the CM4 application.
- `psoc6-pac/` provides the generated peripheral access API.

Build and flash the active firmware with the Chez Scheme scripts under
`tools/`:

```sh
tools/setup-modustoolbox.scm --check
tools/build-lisp.scm
tools/flash-lisp.scm
```

`tools/build-flash-lisp.scm` runs the build and flash steps together. The
flashing script looks for Infineon OpenOCD in this order: `OPENOCD_ROOT`,
`MODUSTOOLBOX_OPENOCD_ROOT`, `MODUSTOOLBOX_ROOT`, `.local/ModusToolbox`,
and `/opt/ModusToolbox`.

To include the local CYW4343W firmware blob in the CM4 image and enable the
firmware-loading Wi-Fi forms to use real local Wi-Fi resources:

```sh
tools/prepare-wifi-resources.scm --check
tools/build-lisp.scm --wifi-firmware
tools/flash-lisp.scm
```

`tools/build-flash-lisp.scm --wifi-firmware` runs the same build and flash
flow in one step. `(wifi-load-firmware)` loads and verifies SOCSRAM without
starting the WLAN core. `(wifi-start-firmware)` also writes NVRAM, releases the
WLAN ARM core, waits for HT clock, and polls function 2 readiness.
`(wifi-f2-read-header)` then reads the first function-2 SDPCM hardware tag from
the already-started firmware session. `(wifi-f2-read-frame)` reads the full
initial 12-byte SDPCM startup frame and decodes its control fields.
`(wifi-ack-interrupts)` reports and clears device-side SDIO software interrupt
bits when they are present. `(wifi-interrupt-state)`, `(wifi-keep-awake)`,
`(wifi-host-reset-lines)`, and `(wifi-abort-read)` are stateful diagnostics for
CCCR interrupt state, WHD KSO wake, PSoC SDHC line reset, and WHD read-abort
behavior after the startup frame.

To include ignored local Wi-Fi credentials in the CM4 image for association
tests, prepare the credential blobs and build with the opt-in credentials
feature:

```sh
tools/prepare-wifi-credential-blobs.scm
tools/build-lisp.scm --wifi-firmware --wifi-credentials
tools/flash-lisp.scm
```

`(wifi-connect-local)` then runs the same prepare-and-join path as
`(wifi-connect-wpa2 ...)` without sending the SSID or passphrase over the serial
console. The credential blob script prints paths and byte lengths only.
`(wifi-link-status)` reports the current link through sanitized MAC/BSSID
presence fields, short fingerprints, RSSI, and CDC status values.
`(wifi-dhcp-discover)` sends one DHCP Discover over the SDPCM data channel and
reports sanitized packet metadata. `(wifi-dhcp-acquire)` sends Discover,
parses Offer, sends Request, parses Ack, and stores the lease fields in the
Wi-Fi state for later network primitives. `(wifi-lease-status)` returns the
stored lease fields without sending SDIO traffic. `(wifi-arp-router)` uses the
stored lease to resolve the router MAC and stores it internally while reporting
only sanitized status fields and a MAC hash. `(wifi-dns-query "example.com")`
sends a UDP DNS A-record query through the stored lease and router ARP state,
then reports sanitized response status and the first answer. These high-level
network forms are stepping stones toward a framed network REPL protocol that is
less dependent on the flaky serial RX path. `(wifi-tcp-syn "example.com" 80)`
resolves a host through the stored DNS server, sends a raw TCP SYN, parses a
SYN-ACK, and sends RST/ACK cleanup. `(wifi-tcp-syn-ip #xc0a80001 80)` skips DNS
and probes a numeric IPv4 address directly, which is useful while DNS behavior
is being debugged. `(wifi-tcp-listen-once 2323 80)` accepts one inbound TCP
handshake and closes it with RST/ACK cleanup. `(wifi-tcp-receive-once 2323 80)`
accepts one inbound TCP connection, captures one payload frame preview, and
then closes with RST/ACK cleanup. `(wifi-tcp-repl-once 2323 80)` accepts one
TCP connection, evaluates one received Lisp form, sends the pretty-printed
result over TCP, and closes. `(wifi-tcp-repl-service on 2323 1)` enables the
current Telnet Lisp REPL service. It keeps one TCP session open, parses Telnet
IAC commands, refuses unsupported options with standard WONT/DONT replies,
handles NVT CR/LF line endings, evaluates complete Lisp lines, and sends NVT
text responses with a prompt. `(http-get "http://example.com/")` resolves the
host, opens a raw TCP connection to port 80, sends an HTTP/1.0 GET with
`Connection: close`, captures a short response preview, and sends RST/ACK
cleanup. Plain HTTP only is implemented; HTTPS/TLS is not. Dotted numeric URLs
such as
`(http-get "http://192.168.0.1/")` skip DNS.
`(wifi-net-repl-once)` polls UDP port 4665 for one framed request. The current
request payload is `LPS3`, a big-endian 32-bit sequence number, a big-endian
32-bit FNV-1a checksum of the request bytes, and one Lisp expression. The
firmware still accepts legacy `LPS0` requests without request checksums for
compatibility with older host tooling. The current reply payload is `LPS2`, the
same sequence number, a big-endian 32-bit FNV-1a checksum of the reply text,
and the pretty-printed result or error text. The host client still accepts
legacy `LPS1` responses without a checksum while older images are being
replaced. After a verified response, the host client sends an optional `LPS4`
ACK with the same sequence number and the response checksum. The board records
ACK counts and the last ACK hash in `(wifi-net-repl-service status)` without
evaluating anything from the ACK frame.
Requests are capped at 96 bytes and replies at 512 bytes.
`(wifi-net-repl-service status)`, `(wifi-net-repl-service on)`,
`(wifi-net-repl-service on 1)`, and `(wifi-net-repl-service off)` control the
same framed UDP evaluator as a background service from the main firmware loop.
While polling, the service answers ARP requests for its DHCP lease so host-side
clients do not need a pinned neighbor entry.
`(wifi-tcp-repl-service status)`, `(wifi-tcp-repl-service on)`,
`(wifi-tcp-repl-service on 2323 1)`, and `(wifi-tcp-repl-service off)` control
the Telnet evaluator. The service is currently single-session. The UDP REPL and
Telnet service poll concurrently through a small HAL demux cache for framed UDP
REPL requests and active TCP REPL packets.
`(wifi-network-bootstrap)` runs local association, DHCP, lease status, and
router ARP resolution as one compact high-level form.

For unattended Wi-Fi association and link-status smoke testing on the flaky
UART RX path:

```sh
tools/build-lisp.scm --wifi-boot-smoke
tools/flash-lisp.scm
```

`--wifi-boot-smoke` implies the firmware and credential build features and runs
`(console-echo off)`, `(wifi-connect-local)`, and `(wifi-link-status)` at boot.
Do not use it for normal quiet images.

For unattended Wi-Fi association plus DHCP lease acquisition testing:

```sh
tools/build-lisp.scm --wifi-dhcp-boot-smoke
tools/flash-lisp.scm
```

`--wifi-dhcp-boot-smoke` implies `--wifi-boot-smoke` and additionally runs
`(wifi-dhcp-acquire)` at boot. Flash a non-smoke image afterward.

For unattended Wi-Fi association, DHCP, and router ARP smoke testing:

```sh
tools/build-lisp.scm --wifi-arp-boot-smoke
tools/flash-lisp.scm
```

`--wifi-arp-boot-smoke` implies `--wifi-dhcp-boot-smoke`. It runs a
UART-silent smoke path and records a RAM marker named
`WIFI_ARP_BOOT_SMOKE_MARKER` for SWD inspection. Flash a non-smoke image
afterward.

For unattended Wi-Fi association, DHCP, router ARP, and DNS smoke testing:

```sh
tools/build-lisp.scm --wifi-dns-boot-smoke
tools/flash-lisp.scm
```

`--wifi-dns-boot-smoke` implies `--wifi-arp-boot-smoke` and resolves
`example.com` through DNS at boot. It extends the same
`WIFI_ARP_BOOT_SMOKE_MARKER` words with DNS status and answer fields for SWD
inspection. Flash a non-smoke image afterward.

For unattended Wi-Fi association, DHCP, router ARP, DNS, and one framed UDP REPL
request smoke test:

```sh
tools/build-lisp.scm --wifi-net-repl-boot-smoke
tools/flash-lisp.scm
```

`--wifi-net-repl-boot-smoke` implies `--wifi-dns-boot-smoke` and then silently
runs `(wifi-net-repl-once 240)` at boot. Send an `LPS3` request frame to UDP
port 4665 while it waits; the board replies with `LPS2`. Flash a non-smoke
image afterward.

Use `tools/send-net-repl.scm --host BOARD_IP '(+ 40 2)'` to send one framed UDP
request from the host. The script writes a checked `LPS3` binary request under
ignored `.local/net-repl/`, uses `ncat` when available or `nc` as a fallback,
parses the `LPS2` response, verifies the reply checksum, sends an `LPS4` ACK,
and prints request/response/ACK metadata plus payload text. Use
`--legacy-request` to send an older `LPS0` request frame.
Use `--read-only` to send the current `LPS5` request frame, which has the same
checksum and payload layout as `LPS3` but asks current firmware to reject forms
outside its conservative read-only allowlist before evaluation.

`tools/send-net-repl.scm --color` wraps payload text in ANSI color. Plain output
is the default so logs and scripts do not receive escape codes. Use
`--payload-only` for a cleaner REPL-like display without transport metadata.
Use `--read-only` for status, directory, FAT info, Wi-Fi link/lease status, and
simple-path file-read operations:

```sh
tools/send-net-repl.scm --host BOARD_IP --payload-only --read-only --color \
  '(wifi-net-repl-service status)'
tools/send-net-repl.scm --host BOARD_IP --payload-only --read-only \
  '(wifi-tcp-repl-service status)'
```

This guard is for avoiding accidental writes from the host client and current
firmware; it is not an authentication or authorization boundary. Each client
invocation uses its own ignored
request/response files under `.local/net-repl/`; use explicit unique sequences
when comparing concurrent UDP requests.

For unattended Wi-Fi association, DHCP, router ARP, DNS, and background framed
UDP REPL service smoke testing:

```sh
tools/build-lisp.scm --wifi-net-repl-service-boot-smoke
tools/flash-lisp.scm
```

`--wifi-net-repl-service-boot-smoke` implies `--wifi-dns-boot-smoke`, enables
the background service at boot, and then enters the normal firmware loop. Use
`tools/send-net-repl.scm` to send requests while it is running. Flash a
non-smoke image afterward.

For normal quiet Wi-Fi startup from microSD, install a FAT `boot.lisp` with:

```scheme
(begin (wifi-network-bootstrap) (wifi-net-repl-service on))
```

The current installer path uses the temporary service-smoke image, then writes
the file over the UDP REPL:

```sh
tools/send-net-repl.scm --host BOARD_IP --wait 15 \
  '(save-file "boot.lisp" "(begin (wifi-network-bootstrap) (wifi-net-repl-service on))")'
tools/send-net-repl.scm --host BOARD_IP --wait 15 '(cat "boot.lisp")'
tools/build-flash-lisp.scm --wifi-firmware --wifi-credentials
```

Use `tools/build-flash-lisp.scm --wifi-firmware --wifi-credentials
--skip-boot-file` for recovery or Wi-Fi debugging when `boot.lisp` is blocking
startup. That image leaves the FAT card untouched and starts without loading
`boot.lisp`.

The longer `--wait` matters for FAT operations; small arithmetic forms usually
reply quickly, while `save-file`, `append-file`, `read-file`, `load`, and
`cat` can exceed the default receive window. Use `--wait 60` when validating
larger FAT files over the UDP REPL.

For microSD-backed Lisp files, the active `save-file`, `append-file`,
`save-defs`, `read-file`, `load`, `ls`, and `cat` forms use the FAT root
directory. `save-file` truncates or creates a file; `append-file` creates or
appends one short Lisp string chunk. `save-defs` writes reloadable global
definitions to one FAT source file, saving plain data and top-level lambdas
while reporting captured closures or other non-reloadable bindings as skipped.
`load` reads source files up to 512 bytes from FAT, which lets a larger source
file be assembled from several UDP requests, generated by `save-defs`, or edited
on a PC. `read-file` reports the full file length, but only includes inline
`content` when the file fits in the 96-byte Lisp string representation. `cat`
returns a Lisp string for those small files and errors with
`file too long for string` for larger files. The firmware accepts Lisp paths
such as `boot.lisp`; on disk these are stored as 8.3 short names such as
`BOOT.LSP` because the embedded FAT crate currently creates and opens short
names only. `ls` maps `.LSP` files back to `.lisp` for the Lisp console.

For unattended storage smoke testing:

```sh
tools/build-lisp.scm --storage-boot-smoke
tools/flash-lisp.scm
```

The storage smoke image skips automatic `boot.lisp` preload, then runs
`(save-file "boot.lisp" "(+ 40 2)")`, `(read-file "boot.lisp")`, `(ls)`, and
`(load "boot.lisp")` at boot. Do not use it for normal quiet images.

To destructively format the inserted microSD card as FAT32 before that storage
smoke test:

```sh
tools/build-lisp.scm --storage-format-boot-smoke
tools/flash-lisp.scm
```

`--storage-format-boot-smoke` implies `--storage-boot-smoke` and destroys card
contents at boot. Flash a non-formatting image immediately afterward.

To keep the large vendor tools out of git, install ModusToolbox into the
ignored `.local/ModusToolbox` directory:

```sh
tools/setup-modustoolbox.scm --archive /path/to/ModusToolbox-linux.tar.gz
```

If the download URL is directly accessible, `tools/setup-modustoolbox.scm
--help` shows the fetch mode.

For the serial console:

```sh
tools/serial-console.scm
tools/send-lisp.scm '(wifi-setup-backplane)'
```

`tools/serial-console.scm` mirrors bytes into
`.local/logs/serial-console.log` by default. Use
`tools/serial-console.scm --tail-log` to watch that file without opening the
UART device. The shared serial setup explicitly enables `CREAD`; keep the
`tools/send-lisp.scm` default 500 ms byte pacing for recovery smoke tests
because faster live UART input is still unreliable on the current hardware path.

For local Wi-Fi credentials, generate the ignored env file from a local IWD
PSK profile, or all usable non-enterprise PSK profiles:

```sh
tools/prepare-wifi-credentials.scm
tools/prepare-wifi-credentials.scm --all
tools/prepare-wifi-credentials.scm --check
tools/prepare-wifi-credentials.scm --check --all
tools/prepare-wifi-credential-blobs.scm
tools/prepare-wifi-credential-blobs.scm --check
```

The script writes `.local/wifi/selected.env` with mode `600`. With `--all`, it
also writes numbered env files under `.local/wifi/profiles/`. It reports only
paths, counts, and field lengths, never credential values or SSIDs. Use `--ssid`
or `--profile` to choose a specific non-enterprise profile. The credential blob
script converts `.local/wifi/selected.env` into ignored binary files under
`.local/wifi/credentials/` for `--wifi-credentials` builds.

Prepare local CYW4343W firmware, CLM, and NVRAM resources before building Wi-Fi
firmware-loader work:

```sh
tools/prepare-wifi-resources.scm
tools/prepare-wifi-resources.scm --check
```

The resource script writes `.local/wifi/resources/`, including a generated local
MAC address used in the NVRAM image. The files are ignored by git. The script
prints sizes, hashes, and paths, but not the generated MAC address.

### Prerequisites

1. Install correct Rust targets for both cores.

    ```sh
    rustup target add thumbv7em-none-eabihf
    rustup target add thumbv6m-none-eabi
    ```

2. Make sure you have GDB installed.
3. Make sure your `openocd` installation is the patched version from Infineon.
    Regular off-the-shelf won't work. Download the
    [Toolset from Infineon](https://softwaretools.infineon.com/tools/com.ifx.tb.tool.cypressprogrammer).
    We used Version 4.2.0.999 for this work.
4. Export the correct environment variable for `openocd`

    ```sh
    export OPENOCD_ROOT=/where/you/installed/infineon/openocd
    ```

    Please make sure this is available to all shell/terminal processes,
    so it is good to set this in your "rc".

### Bootloader application

Check out [psoc6-cm0-bootloader/README.md](./psoc6-cm0-bootloader/README.md)
for the manual bootloader packaging flow. Prefer `tools/build-lisp.scm` for
normal work because it performs the CM4 build, `app.bin` packaging, and CM0+
bootloader build together.

## Misc. notes

### Interrupts

The SVD file (and hence the PAC) describes all 187-odd interrupts. However, the
Cortex-M0+ core only has 8 external and 8 internal interrupts. You therefore
can't use the PAC in "rt" mode on the Cortex-M0+ core - the interrupt vector
table won't fit.

### Booting

Document AN215656 *PSoC™ 6 MCU dual-core system design* says:

> After CM0+ executes the system and security code, it executes the application
> code. In the application code, CM0+ may release the CM4 reset, causing CM4 to
> start executing its application code.

When the system comes out of reset normally, the Cortex-M0+ starts executing
from the on-board ROM and the Cortex-M4 is held in reset. The ROM bootloader
will at some point jump to the reset routine defined in the vector table at the
start of flash. However, it will not update the Cortex-M0+ VTOR register, so any
exceptions or interrupts will still be pointing at the ROM and will not work. In
practice this seems to cause a bounce off to random addresses and a double
fault.

The mechanism described in AN215656 *PSoC™ 6 MCU dual-core system design* is:

1. The ROM runs on Cortex-M0+.
2. The ROM jumps to your reset routine in flash.
3. The code running on the Cortex-M0+ should set up the Cortex-M4's VTOR, using
   a special Cypress system register (because the Cortex-M4's VTOR isn't
   available to the Cortex-M0+ as they are in the private core-local address
   range)..
4. The code on the Cortex-M0+ should use the same block to take the Cortex-M4 out of reset.

So I imagine your firmware would look like:

```text
0x1000_0000 +---------------------------+
            | Stack Ptr for Cortex-M0+  |
            | Reset Ptr for Cortex-M0+  |
            | Exceptions for Cortex-M0+ |
            ¦ ...                       ¦
            | Interrupts for Cortex-M0+ |
            ¦ ...                       ¦
            | Code/Data  for Cortex-M0+ |
            ¦ ...                       ¦
0x1001_0000 +---------------------------+
            | Stack Ptr for Cortex-M4   |
            | Reset Ptr for Cortex-M4   |
            | Exceptions for Cortex-M4  |
            ¦ ...                       ¦
            | Interrupts for Cortex-M4  |
            ¦ ...                       ¦
            | Code/Data  for Cortex-M4  |
            ¦ ...                       ¦
            +---------------------------+
```

Your Cortex-M0+ and Cortex-M4 binaries will need different `memory.x` files so
they each get their own piece of RAM and Flash, and you then need to join them
together. You could do that as a hex-merge, or by including the Cortex-M4
firmware as a static `[u8; nnnn]` within the Cortex-M0+ firmware, located within
an appropriate section so it is linked into memory in the Cortex-M0+ binary at
the same place the Cortex-M4 linker thought it was going to be.

## License

This template is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
