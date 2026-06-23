#[cfg(feature = "wifi-firmware-blob")]
static CYW4343W_FIRMWARE: &[u8] = include_bytes!("../../.local/wifi/resources/4343WA1.bin");

#[cfg(feature = "wifi-firmware-blob")]
static CYW4343W_NVRAM: &[u8] = include_bytes!("../../.local/wifi/resources/wifi_nvram.bin");

#[cfg(not(feature = "wifi-firmware-blob"))]
static CYW4343W_FIRMWARE: &[u8] = &[];

#[cfg(not(feature = "wifi-firmware-blob"))]
static CYW4343W_NVRAM: &[u8] = &[];

pub fn cyw4343w_firmware() -> &'static [u8] {
    CYW4343W_FIRMWARE
}

pub fn cyw4343w_nvram() -> &'static [u8] {
    CYW4343W_NVRAM
}
