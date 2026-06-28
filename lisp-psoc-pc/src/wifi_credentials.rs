#[cfg(feature = "wifi-local-credentials")]
static LOCAL_WIFI_SSID: &[u8] = include_bytes!("../../.local/wifi/credentials/ssid.bin");

#[cfg(feature = "wifi-local-credentials")]
static LOCAL_WIFI_PASSPHRASE: &[u8] =
    include_bytes!("../../.local/wifi/credentials/passphrase.bin");

#[cfg(not(feature = "wifi-local-credentials"))]
static LOCAL_WIFI_SSID: &[u8] = &[];

#[cfg(not(feature = "wifi-local-credentials"))]
static LOCAL_WIFI_PASSPHRASE: &[u8] = &[];

pub fn local_ssid() -> &'static [u8] {
    LOCAL_WIFI_SSID
}

pub fn local_passphrase() -> &'static [u8] {
    LOCAL_WIFI_PASSPHRASE
}
