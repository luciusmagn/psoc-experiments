# psoc6 bootloader

This is a promise that life will get easier this way.

The active setup packages the `lisp-psoc-pc` CM4 application into this
bootloader, which runs on the CM0+ core. When the board boots without GDB, CM0+
runs first while CM4 is held in reset, so CM0+ installs the CM4 vector table and
then releases CM4.

## Build

For normal work, run the repo-local build script from the repository root:

```sh
tools/build-lisp.scm
```

The manual flow is:

1. Build the [lisp-psoc-pc](../lisp-psoc-pc/) CM4 application first.

    ```sh
    pushd ../lisp-psoc-pc
    RUSTFLAGS=-Awarnings cargo build --release --features use-bootloader
    popd
    ```

2. Copy the file, rename it so it matches the bootloader requirements (`app.bin`)

    ```sh
    arm-none-eabi-objcopy -O binary ../lisp-psoc-pc/target/thumbv7em-none-eabihf/release/lisp-psoc-pc ./src/app.bin
    ```

3. Build the bootloader.

    ```sh
    RUSTFLAGS=-Awarnings cargo build --release
    ```
