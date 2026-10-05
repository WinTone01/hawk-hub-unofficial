//! Ayar + cihaz birlikteliği: ayarı değiştir, ilgili frame'i gönder, kaydet.

use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use crate::protocol::{frame_len, macro_reports, Button, Macro, Report};
use crate::{Config, Device, DeviceInfo, Result};

pub struct Session {
    pub cfg: Config,
    /// Bu oturumda gönderilen (report, payload) çiftleri — geliştirici görünümü için.
    pub sent: Vec<(Report, Vec<u8>)>,
    /// None ise dry-run: frame'ler stdout'a yazılır, ayar kaydedilmez.
    dev: Option<Device>,
}

impl Session {
    pub fn open(path: Option<PathBuf>) -> Result<Self> {
        let dev = match path {
            Some(p) => Device::open_path(p)?,
            None => Device::open_first()?,
        };
        Ok(Session { cfg: Config::load(), sent: Vec::new(), dev: Some(dev) })
    }

    pub fn dry_run() -> Self {
        Session { cfg: Config::load(), sent: Vec::new(), dev: None }
    }

    pub fn device(&self) -> Option<&DeviceInfo> {
        self.dev.as_ref().map(|d| &d.info)
    }

    fn send(&mut self, report: Report, payload: &[u8]) -> Result<()> {
        self.sent.push((report, payload.to_vec()));
        match &self.dev {
            Some(d) => d.send(report, payload),
            None => {
                self.print_dry(report, payload);
                Ok(())
            }
        }
    }

    fn print_dry(&self, report: Report, payload: &[u8]) {
        let hex: Vec<String> = payload.iter().map(|b| format!("{b:02x}")).collect();
        println!("report {:#04x}: {}", report as u8, hex.join(" "));
    }

    /// Kayıtlı ayarlardan bir report'u gönderir (Dpi, Led, Polling, Remap).
    pub fn push(&mut self, report: Report) -> Result<()> {
        let frame = self.cfg.frame(report);
        self.send(report, &frame)
    }

    pub fn upload_macro(&mut self, button: Button, m: Macro) -> Result<()> {
        self.write_macro(&m)?;
        self.cfg.bind_macro(button, &m);
        self.push(Report::Remap)
    }

    /// Makronun 3 sayfasını yazar. HM220 V2'de doğrulandı:
    /// - fare yalnızca son sayfadan sonra onay (`50 00 09`) gönderir; ilk iki sayfa sessizdir,
    ///   o yüzden onlar onay beklenmeden ve tekrar edilmeden gönderilir (tekrarlar fareyi kilitliyordu);
    /// - 2.4 GHz alıcı üzerinden ayrıca uyandırma, 1 sn sayfa aralığı ve son sayfada `0x0C` gerekir (aşağıda).
    fn write_macro(&mut self, m: &Macro) -> Result<()> {
        let wireless = self.device().is_some_and(DeviceInfo::wireless);
        let mut pages = macro_reports(m)?;
        let gap = if wireless {
            // 2.4 GHz alıcı üzerinden (Hawk Hub'ın yöntemi, HM220 V2'de doğrulandı): fare sayfalar arasında
            // uyursa sayfa kaybolur ve makro reddedilir, bu yüzden önce onaylı zararsız bir komutla
            // (mevcut raporlama hızını aynen yeniden yaz) uyandırılır. Sayfalar 1 sn arayla gider ve
            // son sayfanın uzunluk baytı 0x0C'dir (gerçekten taşıdığı 12 bayt).
            self.push(Report::Polling)?;
            pages[2][0] = 0x0C;
            1000
        } else {
            60
        };
        let (last, first) = pages.split_last().expect("3 sayfa");
        for p in first {
            self.sent.push((Report::Macro, p.clone()));
            match &self.dev {
                Some(d) => {
                    let mut frame = vec![Report::Macro as u8];
                    frame.extend_from_slice(p);
                    frame.resize(frame_len(Report::Macro, false), 0);
                    d.send_raw(&frame)?;
                    sleep(Duration::from_millis(gap));
                }
                None => self.print_dry(Report::Macro, p),
            }
        }
        self.send(Report::Macro, last)
    }

    pub fn delete_macro(&mut self, id: u8) -> Result<()> {
        if self.cfg.delete_macro(id) {
            self.push(Report::Remap)?;
        }
        Ok(())
    }

    /// Farenin gerçek ayarlarını okuyup kayıtlı ayarların yerine koyar (makrolar okunmaz, korunur).
    pub fn sync_from_device(&mut self) -> Result<()> {
        let Some(dev) = &self.dev else { return Ok(()) };
        for r in [Report::Dpi, Report::Led, Report::Polling, Report::Remap] {
            let frame = dev.read_settings(r)?;
            self.cfg.apply_device_frame(r, &frame)?;
        }
        Ok(())
    }

    /// Kayıtlı tüm ayarları cihaza yeniden yazar.
    pub fn apply_all(&mut self) -> Result<()> {
        self.push(Report::Dpi)?;
        self.push(Report::Led)?;
        self.push(Report::Polling)?;
        let macros: Vec<Macro> = self.cfg.macros.values().cloned().collect();
        for m in &macros {
            self.write_macro(m)?;
        }
        self.push(Report::Remap)
    }

    /// Hawk Hub varsayılanlarına döner ve hepsini yazar.
    pub fn reset(&mut self) -> Result<()> {
        self.apply_config(Config::default())
    }

    /// Başka bir ayar setini (ör. profil) etkinleştirir ve hepsini yazar.
    pub fn apply_config(&mut self, cfg: Config) -> Result<()> {
        self.cfg = cfg;
        self.apply_all()
    }

    /// Ayarları kaydeder (dry-run'da hiçbir şey yapmaz).
    pub fn save(&self) -> Result<()> {
        if self.dev.is_some() {
            self.cfg.save()?;
        }
        Ok(())
    }
}
