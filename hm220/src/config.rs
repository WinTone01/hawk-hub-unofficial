//! Son yazılan ayarlar. Fare ayar okumayı desteklemediği için Hawk Hub gibi
//! frame'leri biz saklarız (bkz. `config_dir`).

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::protocol::*;
use crate::{invalid, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub dpi: Vec<u8>,
    pub led: Vec<u8>,
    pub remap: Vec<u8>,
    pub polling: u16,
    #[serde(default)]
    pub macros: BTreeMap<u8, Macro>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dpi: DPI_TEMPLATE.to_vec(),
            led: LED_TEMPLATE.to_vec(),
            remap: remap_template(),
            polling: 1000,
            macros: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonSetting {
    pub button: Button,
    pub code: u8,
    pub macro_id: Option<u8>,
}

/// Arayüz/CLI için çözülmüş ayarlar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub dpi_stage: u8,
    pub dpi_values: [u32; 5],
    /// Kademe başına DPI gösterge rengi (#rrggbb).
    pub dpi_colors: Vec<String>,
    pub angle_snap: bool,
    pub ripple: bool,
    pub polling_hz: u16,
    pub led_mode: LedMode,
    pub led_color: String,
    pub led_brightness: u8,
    pub led_speed: u8,
    pub response_ms: u8,
    pub buttons: Vec<ButtonSetting>,
    pub macros: Vec<Macro>,
}

impl Config {
    pub fn path() -> PathBuf {
        config_dir().join("hm220.json")
    }

    /// Kayıtlı ayarları yükler; dosya yoksa veya bozuksa Hawk Hub varsayılanları.
    pub fn load() -> Self {
        std::fs::read(Self::path()).ok().and_then(|b| Self::from_json(&b).ok()).unwrap_or_default()
    }

    /// JSON'dan okur; frame uzunlukları beklenenden farklıysa reddeder.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let cfg: Config = serde_json::from_slice(bytes).or_else(|e| invalid(format!("Geçersiz ayar dosyası: {e}")))?;
        let def = Config::default();
        if cfg.dpi.len() != def.dpi.len() || cfg.led.len() != def.led.len() || cfg.remap.len() != def.remap.len() {
            return invalid("Ayar dosyasındaki frame uzunlukları hatalı");
        }
        Ok(cfg)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Config her zaman serileşir")
    }

    pub fn save(&self) -> Result<()> {
        write_file(&Self::path(), &self.to_json())
    }

    /// Fareden okunan bir frame'i (report ID dahil) ayarlara işler.
    pub fn apply_device_frame(&mut self, report: Report, frame: &[u8]) -> Result<()> {
        let p = frame.get(1..).unwrap_or(&[]);
        let take = |len: usize| -> Result<Vec<u8>> {
            p.get(..len).map(<[u8]>::to_vec).ok_or_else(|| crate::Error::Invalid("Fareden eksik veri okundu".into()))
        };
        match report {
            Report::Dpi => self.dpi = take(DPI_TEMPLATE.len())?,
            Report::Led => self.led = take(LED_TEMPLATE.len())?,
            Report::Remap => self.remap = take(self.remap.len())?,
            Report::Polling => {
                let v = *p.get(2).ok_or_else(|| crate::Error::Invalid("Fareden eksik veri okundu".into()))?;
                self.polling = 1000 / v.max(1) as u16;
            }
            Report::Macro => {}
        }
        Ok(())
    }

    /// Bir ayar report'unun payload'ı. Makrolar `macro_reports` ile gönderilir.
    pub fn frame(&self, report: Report) -> Vec<u8> {
        match report {
            Report::Dpi => self.dpi.clone(),
            Report::Led => self.led.clone(),
            Report::Remap => self.remap.clone(),
            Report::Polling => polling_frame(self.polling),
            Report::Macro => unreachable!("makrolar macro_reports ile gönderilir"),
        }
    }

    // ── DPI / sensör (0x04) ─────────────────────────────────────────────────

    pub fn set_dpi_stage(&mut self, stage: u8) -> Result<()> {
        if !(1..=5).contains(&stage) {
            return invalid("DPI kademesi 1-5 olmalı");
        }
        self.dpi[DPI_STAGE] = stage;
        fix_dpi_checksum(&mut self.dpi);
        Ok(())
    }

    pub fn set_dpi_value(&mut self, stage: u8, dpi: u32) -> Result<()> {
        if !(1..=5).contains(&stage) || !(DPI_MIN..=DPI_MAX).contains(&dpi) {
            return invalid(format!("Kademe 1-5, DPI {DPI_MIN}-{DPI_MAX} olmalı"));
        }
        set_stage_dpi(&mut self.dpi, stage as usize - 1, dpi);
        fix_dpi_checksum(&mut self.dpi);
        Ok(())
    }

    /// Kademenin rengi: sabit/nefes modunda tekerlek ışığı aktif kademenin rengiyle yanar.
    pub fn set_dpi_color(&mut self, stage: u8, [r, g, b]: [u8; 3]) -> Result<()> {
        if !(1..=5).contains(&stage) {
            return invalid("DPI kademesi 1-5 olmalı");
        }
        let i = DPI_COLOR_BASE + 3 * (stage as usize - 1);
        self.dpi[i..i + 3].copy_from_slice(&[r, g, b]);
        fix_dpi_checksum(&mut self.dpi);
        Ok(())
    }

    pub fn set_angle_snap(&mut self, on: bool) {
        self.dpi[ANGLE_SNAP] = on as u8;
        fix_dpi_checksum(&mut self.dpi);
    }

    pub fn set_ripple(&mut self, on: bool) {
        self.dpi[RIPPLE] = on as u8;
        fix_dpi_checksum(&mut self.dpi);
    }

    // ── Polling (0x06) ──────────────────────────────────────────────────────

    pub fn set_polling(&mut self, hz: u16) -> Result<()> {
        if !POLLING_RATES.contains(&hz) {
            return invalid("Polling 125, 250, 500 veya 1000 Hz olmalı");
        }
        self.polling = hz;
        Ok(())
    }

    // ── LED + tepki süresi (0x05) ───────────────────────────────────────────

    pub fn set_led_mode(&mut self, mode: LedMode) {
        self.led[LED_MODE] = (self.led[LED_MODE] & 0xF0) | mode as u8;
        fix_led_checksum(&mut self.led);
    }

    /// Tek renk: HM220 V2 sabit/nefes rengini DPI kademe renklerinden aldığı için beş kademenin
    /// rengi birden ayarlanır (LED raporundaki renk baytları da eşitlenir). DPI ve LED frame'leri
    /// birlikte gönderilmelidir.
    pub fn set_led_color(&mut self, rgb: [u8; 3]) {
        for stage in 1..=5 {
            self.set_dpi_color(stage, rgb).expect("1..=5 geçerli");
        }
        self.led[LED_COLOR..LED_COLOR + 3].copy_from_slice(&rgb);
        fix_led_checksum(&mut self.led);
    }

    /// Parlaklık seviyesi 1–8 (alt 4 bit; üst 4 bit derin uyku ayarı, korunur).
    pub fn set_led_brightness(&mut self, v: u8) -> Result<()> {
        if !(1..=LED_BRIGHTNESS_MAX).contains(&v) {
            return invalid("Parlaklık 1-8 olmalı");
        }
        self.led[LED_BRIGHTNESS] = (self.led[LED_BRIGHTNESS] & 0xF0) | v;
        fix_led_checksum(&mut self.led);
        Ok(())
    }

    pub fn set_led_speed(&mut self, v: u8) -> Result<()> {
        if v > LED_SPEED_MAX {
            return invalid("LED hızı 0-5 olmalı");
        }
        self.led[LED_SPEED] = (self.led[LED_SPEED] & 0xF0) | v;
        fix_led_checksum(&mut self.led);
        Ok(())
    }

    pub fn set_response_ms(&mut self, ms: u8) -> Result<()> {
        if !RESPONSE_STEPS_MS.contains(&ms) {
            return invalid("Tepki süresi 0, 2, 4, 8 veya 12 ms olmalı");
        }
        self.led[RESPONSE] = ms / 2;
        fix_led_checksum(&mut self.led);
        Ok(())
    }

    // ── Tuş atama (0x08) ────────────────────────────────────────────────────

    pub fn set_button(&mut self, button: Button, code: u8) -> Result<()> {
        if !FUNCTIONS.iter().any(|f| f.code == code) {
            return invalid(format!("Bilinmeyen fonksiyon kodu {code}"));
        }
        self.write_slot(button, [code, 0, 0]);
        Ok(())
    }

    pub fn bind_macro(&mut self, button: Button, m: &Macro) {
        // 2. bayt değiştirici tuşlar (Ctrl, Shift …); makroda 0. Hawk Hub buraya modu yazıyordu.
        self.write_slot(button, [MACRO_CODE, 0, m.id]);
        self.macros.insert(m.id, m.clone());
    }

    /// Makroyu siler ve ona bağlı tuşları varsayılana döndürür. Remap değiştiyse true.
    pub fn delete_macro(&mut self, id: u8) -> bool {
        let mut changed = false;
        for b in Button::ALL {
            let i = remap_index(b);
            if self.remap[i] == MACRO_CODE && self.remap[i + 2] == id {
                self.write_slot(b, [default_remap_code(b), 0, 0]);
                changed = true;
            }
        }
        self.macros.remove(&id);
        changed
    }

    fn write_slot(&mut self, button: Button, bytes: [u8; 3]) {
        let i = remap_index(button);
        self.remap[i..i + 3].copy_from_slice(&bytes);
        fix_remap_checksum(&mut self.remap);
    }

    pub fn settings(&self) -> Settings {
        let (d, l, r) = (&self.dpi, &self.led, &self.remap);
        let mut dpi_values = [0; 5];
        for (i, v) in dpi_values.iter_mut().enumerate() {
            *v = stage_dpi(d, i);
        }
        Settings {
            dpi_stage: d[DPI_STAGE],
            dpi_values,
            angle_snap: d[ANGLE_SNAP] != 0,
            ripple: d[RIPPLE] != 0,
            polling_hz: self.polling,
            led_mode: LedMode::from_u8(l[LED_MODE] & 15).unwrap_or(LedMode::Off),
            // Görünen renk aktif kademenin rengi.
            led_color: {
                let i = DPI_COLOR_BASE + 3 * (d[DPI_STAGE].clamp(1, 5) as usize - 1);
                format!("#{:02x}{:02x}{:02x}", d[i], d[i + 1], d[i + 2])
            },
            led_brightness: l[LED_BRIGHTNESS] & 0x0F,
            led_speed: l[LED_SPEED] & 0x0F,
            dpi_colors: (0..5)
                .map(|i| {
                    let c = &d[DPI_COLOR_BASE + 3 * i..DPI_COLOR_BASE + 3 * i + 3];
                    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
                })
                .collect(),
            response_ms: l[RESPONSE] * 2,
            buttons: Button::ALL
                .iter()
                .map(|&b| {
                    let i = remap_index(b);
                    ButtonSetting {
                        button: b,
                        code: r[i],
                        macro_id: (r[i] == MACRO_CODE).then_some(r[i + 2]),
                    }
                })
                .collect(),
            macros: self.macros.values().cloned().collect(),
        }
    }
}

/// Ayar dizini: $XDG_CONFIG_HOME/hawk-hub, ~/.config/hawk-hub veya Windows'ta %APPDATA%\hawk-hub-unofficial.
pub fn config_dir() -> PathBuf {
    let var = |k| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(x) = var("XDG_CONFIG_HOME") {
        return x.join("hawk-hub");
    }
    if cfg!(windows) {
        // Hawk Hub'ın kendi %APPDATA%\hawk-hub dizinine karışmamak için ayrı ad.
        if let Some(a) = var("APPDATA") {
            let dir = a.join("hawk-hub-unofficial");
            // Eski adla (hawk-hub-linux) kaydedilmiş profiller bir kez taşınır.
            let old = a.join("hawk-hub-linux");
            if !dir.exists() && old.is_dir() {
                let _ = std::fs::rename(&old, &dir);
            }
            return dir;
        }
    }
    var("HOME").map(|h| h.join(".config").join("hawk-hub")).unwrap_or_else(|| PathBuf::from("hawk-hub"))
}

fn write_file(path: &std::path::Path, contents: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
}

// ── Profiller ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub name: String,
    /// Unix zamanı, saniye.
    pub modified: u64,
    pub settings: Settings,
    pub meta: ProfileMeta,
}

/// Bir profile bağlı program: öne geldiğinde profil otomatik uygulanır.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLink {
    /// Çalıştırılabilir dosyanın tam yolu.
    pub path: String,
    /// Gösterilecek ad.
    pub name: String,
}

/// Profilin görünümü ve bağlı programı (profil dosyasından ayrı, `profiles.json` içinde).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProfileMeta {
    /// #rrggbb
    pub color: Option<String>,
    /// Uygulamanın simge setinden bir simge adı (bağlı programın simgesi yoksa kullanılır).
    pub icon: Option<String>,
    pub app: Option<AppLink>,
}

/// Otomatik profil geçişi ayarları.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoSwitch {
    pub enabled: bool,
    /// Bağlı programlardan hiçbiri öndeyken değilse dönülecek profil; None ise dönülmez.
    pub default_profile: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ProfileIndex {
    profiles: std::collections::BTreeMap<String, ProfileMeta>,
    auto: AutoSwitch,
}

fn index_path() -> PathBuf {
    config_dir().join("profiles.json")
}

fn load_index() -> ProfileIndex {
    std::fs::read(index_path()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn save_index(ix: &ProfileIndex) -> Result<()> {
    write_file(&index_path(), &serde_json::to_string_pretty(ix).expect("serileşir"))
}

pub fn set_profile_meta(name: &str, meta: ProfileMeta) -> Result<()> {
    profile_path(name)?;
    let mut ix = load_index();
    ix.profiles.insert(name.trim().to_string(), meta);
    save_index(&ix)
}

pub fn auto_switch() -> AutoSwitch {
    load_index().auto
}

pub fn set_auto_switch(auto: AutoSwitch) -> Result<()> {
    let mut ix = load_index();
    ix.auto = auto;
    save_index(&ix)
}

/// Programa bağlı profiller: (profil adı, bağlı program).
pub fn linked_profiles() -> Vec<(String, AppLink)> {
    load_index().profiles.into_iter().filter_map(|(n, m)| m.app.map(|a| (n, a))).collect()
}

fn profiles_dir() -> PathBuf {
    config_dir().join("profiles")
}

fn profile_path(name: &str) -> Result<PathBuf> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 40 || name.chars().any(|c| "/\\:*?\"<>|".contains(c) || c.is_control()) || name.starts_with('.') {
        return invalid("Profil adı 1-40 karakter olmalı ve / \\ : * ? \" < > | içermemeli");
    }
    Ok(profiles_dir().join(format!("{name}.json")))
}

pub fn list_profiles() -> Vec<ProfileInfo> {
    let index = load_index();
    let Ok(entries) = std::fs::read_dir(profiles_dir()) else { return Vec::new() };
    let mut out: Vec<ProfileInfo> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if path.extension()? != "json" {
                return None;
            }
            let cfg = Config::from_json(&std::fs::read(&path).ok()?).ok()?;
            let modified = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            let name = path.file_stem()?.to_string_lossy().into_owned();
            let meta = index.profiles.get(&name).cloned().unwrap_or_default();
            Some(ProfileInfo { name, modified, settings: cfg.settings(), meta })
        })
        .collect();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

pub fn save_profile(name: &str, cfg: &Config) -> Result<()> {
    write_file(&profile_path(name)?, &cfg.to_json())
}

pub fn load_profile(name: &str) -> Result<Config> {
    let path = profile_path(name)?;
    let bytes = std::fs::read(&path).or_else(|_| invalid(format!("Profil bulunamadı: {name}")))?;
    Config::from_json(&bytes)
}

pub fn delete_profile(name: &str) -> Result<()> {
    std::fs::remove_file(profile_path(name)?)?;
    let mut ix = load_index();
    ix.profiles.remove(name.trim());
    if ix.auto.default_profile.as_deref() == Some(name.trim()) {
        ix.auto.default_profile = None;
    }
    save_index(&ix)
}

/// "#rrggbb" veya "rrggbb".
pub fn parse_hex_color(s: &str) -> Result<[u8; 3]> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return invalid("Renk RRGGBB biçiminde olmalı");
    }
    let mut out = [0; 3];
    for (i, v) in out.iter_mut().enumerate() {
        *v = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).or_else(|_| invalid("Geçersiz renk"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn led_color_round_trip() {
        let mut c = Config::default();
        c.set_led_color([0x12, 0x34, 0x56]);
        assert_eq!(&c.led[5..8], &[0x12, 0x34, 0x56]);
        assert_eq!(c.settings().led_color, "#123456");
    }

    #[test]
    fn macro_bind_and_delete() {
        let mut c = Config::default();
        let m = Macro { id: 4, name: "x".into(), repeat: 1, mode: 0, steps: vec![MacroStep { key: 4, delay: 10 }] };
        c.bind_macro(Button::Back, &m);
        assert_eq!(c.settings().buttons[3].macro_id, Some(4));
        assert!(c.delete_macro(4));
        assert_eq!(c.remap, remap_template());
    }
}
