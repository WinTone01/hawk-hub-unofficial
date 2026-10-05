//! HID erişimi: Linux'ta hidraw (libc ioctl), Windows'ta hidapi. Diğer platformlarda cihaz bulunmaz.
//!
//! Platform katmanı yalnızca cihaz bulma, feature report yazma ve input report okumayı sağlar;
//! frame kurma, tekrar deneme ve bekleme süreleri ortaktır.

use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::protocol::{
    frame_len, parse_ack, parse_battery, parse_event, read_len, Battery, Event, Report, FEATURE_LEN, PID_RECEIVER,
    PID_WIRED,
};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceInfo {
    /// Linux: /dev/hidrawN, Windows: komut koleksiyonunun HID yolu.
    pub path: PathBuf,
    pub pid: u16,
}

impl DeviceInfo {
    pub fn wireless(&self) -> bool {
        self.pid == PID_RECEIVER
    }

    pub fn transport(&self) -> &'static str {
        if self.wireless() { "2.4 GHz alıcı" } else { "kablolu" }
    }
}

pub struct Device {
    pub info: DeviceInfo,
    inner: imp::Handle,
}

/// Bağlı HM220'lerin komut arayüzleri (USB interface 2).
pub fn find_devices() -> Vec<DeviceInfo> {
    imp::find_devices()
}

impl Device {
    pub fn open(info: DeviceInfo) -> Result<Self> {
        let inner = imp::Handle::open(&info)?;
        Ok(Device { info, inner })
    }

    /// İlk bulunan HM220'nin komut arayüzünü açar.
    pub fn open_first() -> Result<Self> {
        let info = find_devices().into_iter().next().ok_or(Error::NotFound)?;
        Self::open(info)
    }

    /// Belirli bir yolu açar (bulunan cihazlar arasında yoksa kablolu varsayılır).
    pub fn open_path(path: PathBuf) -> Result<Self> {
        let info = find_devices()
            .into_iter()
            .find(|d| d.path == path)
            .unwrap_or(DeviceInfo { path, pid: PID_WIRED });
        Self::open(info)
    }

    /// Feature report gönderir: frame[0] = report ID, ardından payload, 64 bayta doldurulur.
    /// Hawk Hub'daki mouseSend ile aynı tekrar deneme ve bekleme süreleri.
    pub fn send(&self, report: Report, payload: &[u8]) -> Result<()> {
        let wireless = self.info.wireless();
        let mut frame = vec![0u8; frame_len(report, wireless).max(payload.len() + 1).min(FEATURE_LEN)];
        frame[0] = report as u8;
        let n = payload.len().min(frame.len() - 1);
        frame[1..1 + n].copy_from_slice(&payload[..n]);

        // Fare her komutu olay kanalından `03 xx 50 <durum> <report>` ile onaylar. Kablosuzda fare
        // 5 sn hareketsizlikten sonra uyur ve komutu kaçırır; onay gelmezse tekrar gönderilir.
        // Makro sayfaları asla tekrar gönderilmez: tekrar, fare–alıcı bağlantısını koparabiliyor.
        let attempts = if report == Report::Macro { 1 } else if wireless { 10 } else { 3 };
        let mut result = Err(Error::NoAck);
        for _ in 0..attempts {
            if let Err(e) = self.inner.write_feature(&frame) {
                result = Err(e);
                sleep(Duration::from_millis(30));
                continue;
            }
            if !self.inner.has_events() {
                result = Ok(()); // onay kanalı açılamadı: onaysız devam
                break;
            }
            match self.wait_ack(report as u8, Duration::from_millis(700))? {
                Some(true) => {
                    result = Ok(());
                    break;
                }
                Some(false) => return Err(Error::Rejected(report as u8)),
                None => result = Err(Error::NoAck),
            }
        }
        result?;
        if report == Report::Macro {
            sleep(Duration::from_millis(if wireless { 300 } else { 100 }));
        }
        Ok(())
    }

    /// `report` için onay bekler: Some(true) kabul, Some(false) red, None süre doldu.
    fn wait_ack(&self, report: u8, timeout: Duration) -> Result<Option<bool>> {
        let end = Instant::now() + timeout;
        let mut buf = [0u8; 64];
        while let Some(left) = end.checked_duration_since(Instant::now()) {
            let Some(n) = self.inner.read_report(&mut buf, left)? else { break };
            if let Some((r, ok)) = parse_ack(&buf[..n]) {
                if r == report {
                    return Ok(Some(ok));
                }
            }
        }
        Ok(None)
    }

    /// Farenin o anki ayarını okur (report ID dahil frame). Önce `A0` ile okuma izni istenir;
    /// izin tek bir okuma için geçerlidir (attack-shark-x11-driver ile aynı akış).
    pub fn read_settings(&self, report: Report) -> Result<Vec<u8>> {
        let len = read_len(report);
        let attempts = if self.info.wireless() { 6 } else { 2 };
        for _ in 0..attempts {
            self.inner.write_feature(&[0xA0, report as u8, len, 0x00, 0x01, 0x00, 0x00, 0x00])?;
            sleep(Duration::from_millis(250));
            let mut perm = [0u8; 8];
            perm[0] = 0xA0;
            if self.inner.read_feature(&mut perm).is_ok() && perm[1] == 0x01 {
                let mut buf = vec![0u8; len as usize];
                buf[0] = report as u8;
                let n = self.inner.read_feature(&mut buf)?;
                buf.truncate(n.min(len as usize));
                return Ok(buf);
            }
        }
        Err(Error::NoAck)
    }

    /// Fare pil durumunu kendiliğinden yollar; `timeout` içinde gelmezse Ok(None).
    /// Olay kanalından bir sonraki tanınan olayı bekler; `timeout` içinde gelmezse Ok(None).
    pub fn read_event(&self, timeout: Duration) -> Result<Option<Event>> {
        let end = Instant::now() + timeout;
        let mut buf = [0u8; 64];
        while let Some(left) = end.checked_duration_since(Instant::now()) {
            let Some(n) = self.inner.read_report(&mut buf, left)? else { return Ok(None) };
            if let Some(e) = parse_event(&buf[..n], !self.info.wireless()) {
                return Ok(Some(e));
            }
        }
        Ok(None)
    }

    pub fn read_battery(&self, timeout: Duration) -> Result<Option<Battery>> {
        let end = Instant::now() + timeout;
        let mut buf = [0u8; 64];
        loop {
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Ok(None);
            }
            let Some(n) = self.inner.read_report(&mut buf, left)? else { return Ok(None) };
            if let Some(b) = parse_battery(&buf[..n], !self.info.wireless()) {
                return Ok(Some(b));
            }
        }
    }

    /// Olay kanalından tek bir input report okur (pil, komut onayı …); süre dolarsa None.
    pub fn read_input(&self, timeout: Duration) -> Result<Option<Vec<u8>>> {
        let mut buf = [0u8; 64];
        Ok(self.inner.read_report(&mut buf, timeout)?.map(|n| buf[..n].to_vec()))
    }

    /// Ham feature report (report ID dahil), uzunluk olduğu gibi. Tersine mühendislik içindir.
    pub fn send_raw(&self, frame: &[u8]) -> Result<()> {
        self.inner.write_feature(frame)
    }

    /// Feature report okur (GET_FEATURE). `buf[0]` report ID olmalı; okunan bayt sayısını döner.
    pub fn get_feature(&self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read_feature(buf)
    }

    /// Komut arayüzünün HID rapor tanımlayıcısı.
    pub fn report_descriptor(&self) -> Result<Vec<u8>> {
        self.inner.report_descriptor(&self.info)
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use std::fs::{File, OpenOptions};
    use std::io::{ErrorKind, Read};
    use std::os::fd::AsRawFd;
    use std::path::Path;

    use super::*;
    use crate::protocol::{CMD_INTERFACE, VID};

    /// HIDIOCSFEATURE(len) = _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, len)
    const fn hidiocsfeature(len: usize) -> u64 {
        (3 << 30) | ((len as u64) << 16) | ((b'H' as u64) << 8) | 0x06
    }

    pub fn find_devices() -> Vec<DeviceInfo> {
        let Ok(entries) = std::fs::read_dir("/sys/class/hidraw") else { return Vec::new() };
        let mut found: Vec<DeviceInfo> = entries
            .flatten()
            .filter_map(|e| {
                let node = e.path();
                let pid = hid_pid(&node)?;
                // .../X-Y:1.2/0003:1D57:FA60.000N  →  üst dizin USB arayüzü
                let intf = std::fs::canonicalize(node.join("device")).ok()?.parent()?.to_path_buf();
                let num = std::fs::read_to_string(intf.join("bInterfaceNumber")).ok()?;
                (u8::from_str_radix(num.trim(), 16).ok()? == CMD_INTERFACE)
                    .then(|| DeviceInfo { path: Path::new("/dev").join(e.file_name()), pid })
            })
            .collect();
        found.sort_by(|a, b| a.path.cmp(&b.path));
        found
    }

    /// uevent'teki `HID_ID=0003:00001D57:0000FA60` satırından PID.
    fn hid_pid(node: &Path) -> Option<u16> {
        let uevent = std::fs::read_to_string(node.join("device/uevent")).ok()?;
        let id = uevent.lines().find_map(|l| l.strip_prefix("HID_ID="))?;
        let mut parts = id.split(':').skip(1);
        let vid = u32::from_str_radix(parts.next()?, 16).ok()?;
        let pid = u32::from_str_radix(parts.next()?, 16).ok()? as u16;
        (vid == VID as u32 && [PID_WIRED, PID_RECEIVER].contains(&pid)).then_some(pid)
    }

    /// Linux'ta interface 2'nin iki koleksiyonu (komut + pil) tek hidraw düğümündedir.
    pub struct Handle(File);

    impl Handle {
        /// Komut onayları aynı hidraw düğümünden okunur.
        pub fn has_events(&self) -> bool {
            true
        }

        pub fn open(info: &DeviceInfo) -> Result<Self> {
            OpenOptions::new().read(true).write(true).open(&info.path).map(Handle).map_err(|e| match e.kind() {
                ErrorKind::PermissionDenied => Error::PermissionDenied(info.path.clone()),
                ErrorKind::NotFound => Error::NotFound,
                _ => Error::Io(e),
            })
        }

        pub fn write_feature(&self, frame: &[u8]) -> Result<()> {
            let mut buf = frame.to_vec();
            // SAFETY: ioctl uzunluğu buf uzunluğuna eşit.
            let rc = unsafe { libc::ioctl(self.0.as_raw_fd(), hidiocsfeature(buf.len()) as _, buf.as_mut_ptr()) };
            if rc < 0 {
                return Err(Error::Io(std::io::Error::last_os_error()));
            }
            Ok(())
        }

        pub fn read_report(&self, buf: &mut [u8], timeout: Duration) -> Result<Option<usize>> {
            let mut pfd = libc::pollfd { fd: self.0.as_raw_fd(), events: libc::POLLIN, revents: 0 };
            // SAFETY: tek bir geçerli pollfd.
            let rc = unsafe { libc::poll(&mut pfd, 1, timeout.as_millis().min(i32::MAX as u128) as i32) };
            if rc < 0 {
                return Err(Error::Io(std::io::Error::last_os_error()));
            }
            if rc == 0 {
                return Ok(None);
            }
            if pfd.revents & (libc::POLLERR | libc::POLLHUP) != 0 {
                return Err(Error::NotFound);
            }
            Ok(Some((&self.0).read(buf)?))
        }

        pub fn read_feature(&self, buf: &mut [u8]) -> Result<usize> {
            // HIDIOCGFEATURE(len) = _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x07, len)
            let req = (3u64 << 30) | ((buf.len() as u64) << 16) | ((b'H' as u64) << 8) | 0x07;
            // SAFETY: ioctl uzunluğu buf uzunluğuna eşit.
            let rc = unsafe { libc::ioctl(self.0.as_raw_fd(), req as _, buf.as_mut_ptr()) };
            if rc < 0 {
                return Err(Error::Io(std::io::Error::last_os_error()));
            }
            Ok(rc as usize)
        }

        pub fn report_descriptor(&self, info: &DeviceInfo) -> Result<Vec<u8>> {
            let name = info.path.file_name().ok_or(Error::NotFound)?;
            Ok(std::fs::read(Path::new("/sys/class/hidraw").join(name).join("device/report_descriptor"))?)
        }
    }
}

#[cfg(windows)]
mod imp {
    use std::ffi::CString;

    use hidapi::{HidApi, HidDevice};

    use super::*;
    use crate::protocol::{CMD_INTERFACE, VID};

    /// Windows'ta interface 2 iki ayrı HID koleksiyonu olarak görünür.
    const CMD_USAGE_PAGE: u16 = 0x0B;
    const BATTERY_USAGE_PAGE: u16 = 0x0A;

    fn api() -> Result<HidApi> {
        HidApi::new().map_err(|e| Error::Invalid(format!("HID başlatılamadı: {e}")))
    }

    fn collections(api: &HidApi, usage_page: u16) -> impl Iterator<Item = (PathBuf, u16)> + '_ {
        api.device_list()
            .filter(move |d| {
                d.vendor_id() == VID
                    && [PID_WIRED, PID_RECEIVER].contains(&d.product_id())
                    && d.interface_number() == CMD_INTERFACE as i32
                    && d.usage_page() == usage_page
            })
            .map(|d| (PathBuf::from(d.path().to_string_lossy().into_owned()), d.product_id()))
    }

    pub fn find_devices() -> Vec<DeviceInfo> {
        let Ok(api) = api() else { return Vec::new() };
        let mut found: Vec<DeviceInfo> =
            collections(&api, CMD_USAGE_PAGE).map(|(path, pid)| DeviceInfo { path, pid }).collect();
        found.sort_by(|a, b| a.path.cmp(&b.path));
        found
    }

    fn hid_err(e: hidapi::HidError) -> Error {
        Error::Io(std::io::Error::other(e.to_string()))
    }

    pub struct Handle {
        cmd: HidDevice,
        battery: Option<HidDevice>,
    }

    impl Handle {
        /// Komut onayları olay koleksiyonundan (usage page 0x0A) okunur.
        pub fn has_events(&self) -> bool {
            self.battery.is_some()
        }

        pub fn open(info: &DeviceInfo) -> Result<Self> {
            let api = api()?;
            let open = |p: &PathBuf| {
                let c = CString::new(p.to_string_lossy().into_owned()).map_err(|_| Error::NotFound)?;
                api.open_path(&c).map_err(|_| Error::NotFound)
            };
            let cmd = open(&info.path)?;
            let battery = collections(&api, BATTERY_USAGE_PAGE)
                .find(|(_, pid)| *pid == info.pid)
                .and_then(|(p, _)| open(&p).ok());
            Ok(Handle { cmd, battery })
        }

        pub fn write_feature(&self, frame: &[u8]) -> Result<()> {
            self.cmd.send_feature_report(frame).map_err(hid_err)
        }

        pub fn read_report(&self, buf: &mut [u8], timeout: Duration) -> Result<Option<usize>> {
            let Some(dev) = &self.battery else {
                sleep(timeout);
                return Ok(None);
            };
            match dev.read_timeout(buf, timeout.as_millis().min(i32::MAX as u128) as i32) {
                Ok(0) => Ok(None),
                Ok(n) => Ok(Some(n)),
                Err(_) => Err(Error::NotFound),
            }
        }

        pub fn read_feature(&self, buf: &mut [u8]) -> Result<usize> {
            self.cmd.get_feature_report(buf).map_err(hid_err)
        }

        pub fn report_descriptor(&self, _info: &DeviceInfo) -> Result<Vec<u8>> {
            let mut buf = vec![0u8; 4096];
            let n = self.cmd.get_report_descriptor(&mut buf).map_err(hid_err)?;
            buf.truncate(n);
            Ok(buf)
        }
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
    use super::*;

    pub fn find_devices() -> Vec<DeviceInfo> {
        Vec::new()
    }

    pub struct Handle;

    impl Handle {
        pub fn has_events(&self) -> bool {
            false
        }

        pub fn open(_info: &DeviceInfo) -> Result<Self> {
            Err(Error::Unsupported)
        }
        pub fn write_feature(&self, _frame: &[u8]) -> Result<()> {
            Err(Error::Unsupported)
        }
        pub fn read_report(&self, _buf: &mut [u8], _timeout: Duration) -> Result<Option<usize>> {
            Err(Error::Unsupported)
        }
        pub fn read_feature(&self, _buf: &mut [u8]) -> Result<usize> {
            Err(Error::Unsupported)
        }
        pub fn report_descriptor(&self, _info: &DeviceInfo) -> Result<Vec<u8>> {
            Err(Error::Unsupported)
        }
    }
}
