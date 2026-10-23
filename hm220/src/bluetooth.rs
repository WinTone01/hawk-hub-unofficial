//! Bluetooth LE ile bağlı HM220'nin durumu: Linux'ta BlueZ (D-Bus), Windows'ta cfgmgr32.
//!
//! BLE'de yapılandırma komutu yok; Hawk Hub da bu durumda yalnızca bağlantıyı ve pili gösterir.
//! Fare kendini "HM220" adıyla tanıtır (Hawk Hub 1.0.21 aynı adı arıyor).

use serde::Serialize;

const NAME: &str = "HM220";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BluetoothStatus {
    /// %, bilinmiyorsa None.
    pub battery: Option<u8>,
}

/// Bluetooth LE ile bağlı bir HM220 varsa durumunu döner.
pub fn bluetooth_status() -> Option<BluetoothStatus> {
    imp::status()
}

#[cfg(target_os = "linux")]
mod imp {
    use std::time::Duration;

    use dbus::arg::prop_cast;
    use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;
    use dbus::blocking::Connection;

    use super::*;

    /// BlueZ'nin nesne ağacında `Device1` (ad, bağlantı) ve aynı yolda `Battery1` (GATT pil servisi).
    pub fn status() -> Option<BluetoothStatus> {
        let conn = Connection::new_system().ok()?;
        let proxy = conn.with_proxy("org.bluez", "/", Duration::from_millis(1500));
        let objects = proxy.get_managed_objects().ok()?;
        objects.values().find_map(|ifaces| {
            let dev = ifaces.get("org.bluez.Device1")?;
            let named = [prop_cast::<String>(dev, "Name"), prop_cast::<String>(dev, "Alias")]
                .into_iter()
                .flatten()
                .any(|n| n == NAME);
            if !named || prop_cast::<bool>(dev, "Connected") != Some(&true) {
                return None;
            }
            let battery = ifaces
                .get("org.bluez.Battery1")
                .and_then(|b| prop_cast::<u8>(b, "Percentage"))
                .copied()
                .filter(|&p| p <= 100);
            Some(BluetoothStatus { battery })
        })
    }
}

#[cfg(windows)]
mod imp {
    use windows_sys::core::GUID;
    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
        CM_Get_DevNode_PropertyW, CM_Get_Device_ID_ListW, CM_Get_Device_ID_List_SizeW, CM_Locate_DevNodeW,
        CM_GETIDLIST_FILTER_ENUMERATOR, CM_GETIDLIST_FILTER_PRESENT, CM_LOCATE_DEVNODE_NORMAL, CR_SUCCESS,
    };
    use windows_sys::Win32::Devices::Properties::{DEVPROP_TYPE_BOOLEAN, DEVPROP_TYPE_BYTE, DEVPROP_TYPE_STRING};
    use windows_sys::Win32::Foundation::DEVPROPKEY;

    use super::*;

    // Hawk Hub 1.0.21'in kullandığı anahtarlar.
    /// DEVPKEY_Device_FriendlyName
    const KEY_NAME: DEVPROPKEY =
        DEVPROPKEY { fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0), pid: 14 };
    /// Cihaz bağlı (yalnızca eşleşmiş değil).
    const KEY_CONNECTED: DEVPROPKEY =
        DEVPROPKEY { fmtid: GUID::from_u128(0x83da6326_97a6_4088_9453_a1923f573b29), pid: 15 };
    /// Pil yüzdesi (bayt).
    const KEY_BATTERY: DEVPROPKEY =
        DEVPROPKEY { fmtid: GUID::from_u128(0x104ea319_6ee2_4701_bd47_8ddbf425bbe5), pid: 2 };

    enum Prop {
        Bool(bool),
        Byte(u8),
        Str(String),
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    /// Takılı `enumerator` cihazlarının kimlikleri (ör. `BTHLE\DEV_…`).
    fn list_ids(enumerator: &str) -> Vec<Vec<u16>> {
        let filter = wide(enumerator);
        let flags = CM_GETIDLIST_FILTER_ENUMERATOR | CM_GETIDLIST_FILTER_PRESENT;
        let mut len = 0u32;
        // SAFETY: filter NUL ile biter; len geçerli bir çıktı.
        if unsafe { CM_Get_Device_ID_List_SizeW(&mut len, filter.as_ptr(), flags) } != CR_SUCCESS || len == 0 {
            return Vec::new();
        }
        let mut buf = vec![0u16; len as usize];
        // SAFETY: buf `len` karakterlik.
        if unsafe { CM_Get_Device_ID_ListW(filter.as_ptr(), buf.as_mut_ptr(), len, flags) } != CR_SUCCESS {
            return Vec::new();
        }
        // Liste NUL ile ayrılmış, çift NUL ile biter.
        buf.split(|&c| c == 0).filter(|s| !s.is_empty()).map(|s| s.iter().copied().chain(Some(0)).collect()).collect()
    }

    fn prop(id: &[u16], key: &DEVPROPKEY) -> Option<Prop> {
        let mut dn = 0u32;
        // SAFETY: id NUL ile biter.
        if unsafe { CM_Locate_DevNodeW(&mut dn, id.as_ptr(), CM_LOCATE_DEVNODE_NORMAL) } != CR_SUCCESS {
            return None;
        }
        let (mut ty, mut size) = (0u32, 0u32);
        // Önce boyut sorulur (CR_BUFFER_SMALL döner).
        // SAFETY: boş tampon ve sıfır boyutla çağrı geçerli.
        unsafe { CM_Get_DevNode_PropertyW(dn, key, &mut ty, std::ptr::null_mut(), &mut size, 0) };
        if size == 0 {
            return None;
        }
        let mut out = vec![0u8; size as usize];
        // SAFETY: out `size` bayt.
        if unsafe { CM_Get_DevNode_PropertyW(dn, key, &mut ty, out.as_mut_ptr(), &mut size, 0) } != CR_SUCCESS {
            return None;
        }
        out.truncate(size as usize);
        match ty {
            DEVPROP_TYPE_BOOLEAN => Some(Prop::Bool(out[0] != 0)),
            DEVPROP_TYPE_BYTE => Some(Prop::Byte(out[0])),
            DEVPROP_TYPE_STRING => {
                let w: Vec<u16> = out.as_chunks::<2>().0.iter().map(|&c| u16::from_le_bytes(c)).collect();
                Some(Prop::Str(String::from_utf16_lossy(&w).trim_end_matches('\0').to_owned()))
            }
            _ => None,
        }
    }

    /// `BTHLE\DEV_…` düğümü: adı HM220 ve bağlıysa pil yüzdesiyle birlikte.
    pub fn status() -> Option<BluetoothStatus> {
        let mut found = None;
        for id in list_ids("BTHLE") {
            let is_dev = String::from_utf16_lossy(&id).to_ascii_uppercase().starts_with("BTHLE\\DEV_");
            if !is_dev || !matches!(prop(&id, &KEY_NAME), Some(Prop::Str(n)) if n == NAME) {
                continue;
            }
            if !matches!(prop(&id, &KEY_CONNECTED), Some(Prop::Bool(true))) {
                continue;
            }
            let battery = match prop(&id, &KEY_BATTERY) {
                Some(Prop::Byte(b)) if b <= 100 => Some(b),
                _ => None,
            };
            // Aynı fare için birden çok düğüm olabilir: pili bilinen tercih edilir.
            if found.is_none() || battery.is_some() {
                found = Some(BluetoothStatus { battery });
            }
        }
        found
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
    use super::*;

    pub fn status() -> Option<BluetoothStatus> {
        None
    }
}
