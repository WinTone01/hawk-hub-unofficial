//! Hawk HM220 fare: protokol, ayar saklama ve Linux hidraw erişimi.
//!
//! Protokol Hawk Hub 1.0.20-beta'dan çıkarılmıştır (bkz. PROTOCOL.md).

mod config;
mod device;
mod dpi;
mod protocol;
mod session;

pub use config::{
    auto_switch, config_dir, delete_profile, linked_profiles, list_profiles, load_profile, parse_hex_color, save_profile,
    set_auto_switch, set_profile_meta, AppLink, AutoSwitch, ButtonSetting, Config, ProfileInfo, ProfileMeta, Settings,
};
pub use device::{find_devices, Device, DeviceInfo};
pub use protocol::*;
pub use session::Session;

use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    NotFound,
    PermissionDenied(PathBuf),
    Unsupported,
    Invalid(String),
    /// Fare komutu `50 01 <report>` ile reddetti.
    Rejected(u8),
    /// Fare komutu onaylamadı (kablosuzda uykuda olabilir).
    NoAck,
    Io(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound => write!(f, "HM220 bulunamadı (USB kablo veya 2.4 GHz alıcı takılı mı?)"),
            Error::PermissionDenied(p) => write!(
                f,
                "{} açılamadı: izin yok. 99-hawk-hub.rules udev kuralını kurun.",
                p.display()
            ),
            Error::Unsupported => write!(f, "Cihaz erişimi bu platformda desteklenmiyor"),
            Error::Invalid(msg) => write!(f, "{msg}"),
            Error::Rejected(r) => write!(f, "Fare komutu reddetti (report 0x{r:02x})"),
            Error::NoAck => write!(f, "Fare yanıt vermedi; uykuda olabilir. Fareyi oynatıp tekrar deneyin."),
            Error::Io(e) => write!(f, "G/Ç hatası: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub(crate) fn invalid<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Invalid(msg.into()))
}
