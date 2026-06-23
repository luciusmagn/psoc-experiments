# psoc6-demo

A demo project for the CY8C624ABZI-S2D44A0 SoC, as fitted to the
*CY8CPROTO-062-4343W PSoC 6 Wi-Fi BT Prototyping kit*.

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

For local Wi-Fi credentials, generate the ignored env file from a local IWD
PSK profile:

```sh
tools/prepare-wifi-credentials.scm
tools/prepare-wifi-credentials.scm --check
```

The script writes `.local/wifi/selected.env` with mode `600` and reports only
field lengths, never credential values. Use `--ssid` or `--profile` to choose a
specific non-enterprise profile.

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
