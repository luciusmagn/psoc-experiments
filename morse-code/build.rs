//! This build script copies the `memory-xxxx.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.
//!
//! The build script also sets the linker flags to tell it which link script to use.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

static MEMORY_X: &[u8] = include_bytes!("memory.x");
static MEMORY_X_BOOTLOADER: &[u8] =
    include_bytes!("memory-bootloader.x");

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // the feature flag is "use-bootloader"
    let memory_x_contents =
        if env::var("CARGO_FEATURE_USE_BOOTLOADER").is_ok() {
            &MEMORY_X_BOOTLOADER
        } else {
            &MEMORY_X
        };
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(memory_x_contents)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory-raw.x");
    println!("cargo:rerun-if-changed=memory-bootloader.x");

    // Specify linker arguments.

    // `--nmagic` is required if memory section addresses are not aligned to 0x10000,
    // for example the FLASH and RAM sections in your `memory.x`.
    // See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    println!("cargo:rustc-link-arg=--nmagic");

    // Set the linker script to the one provided by cortex-m-rt.
    println!("cargo:rustc-link-arg=-Tlink.x");
}
