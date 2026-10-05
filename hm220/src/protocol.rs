//! Frame şablonları, checksum'lar ve frame üreticileri.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{invalid, Result};

pub const VID: u16 = 0x1D57;
pub const PID_WIRED: u16 = 0x2027;
pub const PID_RECEIVER: u16 = 0xFA60;
pub const CMD_INTERFACE: u8 = 2;
pub const FEATURE_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    Dpi = 4,
    Led = 5,
    Polling = 6,
    Remap = 8,
    Macro = 9,
}

// Varsayılan payload'lar (report ID hariç). HM220 V2'den okunan fabrika değerleri; Hawk Hub'ınkilerle
// DPI ve tuş atamada aynı. Hawk Hub'ın LED şablonu ve LED checksum'ı yanlıştı (fare reddediyordu).
pub const DPI_TEMPLATE: [u8; 51] = [
    56, 1, 0, 1, 31, 0, 0, 9, 18, 37, 75, 112, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255, 0, 0, 0,
    0, 255, 0, 255, 0, 255, 0, 255, 255, 64, 0, 0, 255, 255, 255, 64, 0, 255, 255, 255, 3, 13,
    147,
];
pub const LED_TEMPLATE: [u8; 12] = [0x0f, 0x01, 0x02, 0x03, 0xc8, 0x00, 0xe6, 0x0f, 0x3c, 0x00, 0x01, 0xfe];
const REMAP_DEFAULT_CODES: [u8; 18] = [2, 3, 4, 5, 6, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 10];

/// Gönderilen frame uzunluğu (report ID dahil). Beken firmware'i kablolu ve kablosuzda farklı
/// uzunluk bekler (attack-shark-x11-driver ile aynı); Windows zaten koleksiyon boyuna tamamlar.
pub fn frame_len(report: Report, wireless: bool) -> usize {
    match (report, wireless) {
        (Report::Dpi, false) => 52,
        (Report::Dpi, true) => 56,
        (Report::Led, false) => 13,
        (Report::Led, true) => 15,
        (Report::Polling, _) => 9,
        (Report::Remap, _) => 59,
        (Report::Macro, _) => 64,
    }
}

/// Okuma uzunlukları (`A0` izin isteğindeki uzunluk baytı ve GET uzunluğu, report ID dahil).
pub fn read_len(report: Report) -> u8 {
    match report {
        Report::Dpi => 0x38,
        Report::Led => 0x0f,
        Report::Polling => 0x09,
        Report::Remap => 0x3b,
        Report::Macro => 0x83,
    }
}

/// Olay kanalından gelen mesajlar: `03 <model> <tür> <p1> <p2>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// `40`: pil durumu.
    Battery(Battery),
    /// `10`: farenin DPI tuşuyla aktif kademe değişti (1–5). HM220'de doğrulandı.
    DpiStage(u8),
    /// `80`: aktif profil değişti (0 tabanlı). X11 belgelerinden; HM220'de gözlenmedi.
    Profile(u8),
    /// `50`: komut onayı.
    Ack { report: u8, ok: bool },
}

pub fn parse_event(d: &[u8], wired: bool) -> Option<Event> {
    if d.len() < 5 || d[0] != 3 {
        return None;
    }
    match d[2] {
        0x40 => parse_battery(d, wired).map(Event::Battery),
        0x10 if (1..=8).contains(&d[3]) => Some(Event::DpiStage(d[3])),
        0x80 => Some(Event::Profile(d[3])),
        0x50 => parse_ack(d).map(|(report, ok)| Event::Ack { report, ok }),
        _ => None,
    }
}

/// Komut onayı (olay kanalı): `03 <model> 50 <durum> <report>`, durum 0 = başarılı.
pub fn parse_ack(d: &[u8]) -> Option<(u8, bool)> {
    (d.len() >= 5 && d[0] == 3 && d[2] == 0x50).then(|| (d[4], d[3] == 0))
}

pub fn remap_template() -> Vec<u8> {
    let mut out = vec![59, 1, REMAP_DEFAULT_CODES[0]];
    for &c in &REMAP_DEFAULT_CODES[1..] {
        out.extend([0, 0, c]);
    }
    let ck = REMAP_DEFAULT_CODES.iter().fold(0u8, |a, &c| a.wrapping_add(c));
    out.extend([0, 0, 0, ck]);
    out
}

// Payload indeksleri (frame indeksi − 1).
pub(crate) const ANGLE_SNAP: usize = 2;
pub(crate) const RIPPLE: usize = 3;
/// ×2 bayrakları (bit = kademe), 10000 DPI üstü.
pub(crate) const DPI_DOUBLE: usize = 5;
pub(crate) const DPI_X_BASE: usize = 7;
pub(crate) const DPI_Y_BASE: usize = 15;
pub(crate) const DPI_STAGE: usize = 23;
/// Kademe başına DPI gösterge rengi (R, G, B), 8 kademe.
pub(crate) const DPI_COLOR_BASE: usize = 24;
/// Alt 4 bit: 0 kapalı, 1 neon, 2 sabit, 3 nefes (donanımda doğrulandı).
pub(crate) const LED_MODE: usize = 2;
/// Alt 4 bit hız; üst 4 bit derin uyku süresinin yüksek yarısı.
pub(crate) const LED_SPEED: usize = 3;
/// Alt 4 bit parlaklık (1–8); üst 4 bit derin uyku süresinin düşük yarısı.
pub(crate) const LED_BRIGHTNESS: usize = 4;
/// R, G, B. Fare saklıyor ama HM220 V2'de görünür rengi değiştirmediği gözlendi.
pub(crate) const LED_COLOR: usize = 5;
pub(crate) const RESPONSE: usize = 9;

pub const MACRO_CODE: u8 = 18;
pub const POLLING_RATES: [u16; 4] = [125, 250, 500, 1000];
pub const RESPONSE_STEPS_MS: [u8; 5] = [0, 2, 4, 8, 12];
pub const DPI_MIN: u32 = 100;
pub const DPI_MAX: u32 = 12_000;
pub const LED_SPEED_MAX: u8 = 5;
pub const LED_BRIGHTNESS_MAX: u8 = 8;

pub(crate) fn remap_index(button: Button) -> usize {
    2 + 3 * button as usize
}

pub(crate) fn default_remap_code(button: Button) -> u8 {
    REMAP_DEFAULT_CODES[button as usize]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Tekerlek ışığı modu. HM220 V2'de donanımda gözlenen değerler:
/// 0 kapalı, 1 neon, 2/3 sabit/nefes ama firmware'e gömülü kırmızı (renk baytları yok sayılır),
/// 4/5 sabit ve 6 nefes, rengi aktif DPI kademesinin renginden alır. Özel renk bu yüzden
/// DPI kademe renkleri üzerinden verilir; biz 5 ve 6'yı yazarız.
pub enum LedMode {
    Off = 0,
    Neon = 1,
    Static = 5,
    Breathing = 6,
}

impl LedMode {
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => LedMode::Off,
            1 => LedMode::Neon,
            2 | 4 | 5 => LedMode::Static,
            3 | 6 => LedMode::Breathing,
            _ => return None,
        })
    }
}

impl FromStr for LedMode {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        Ok(match s {
            "off" => LedMode::Off,
            "neon" => LedMode::Neon,
            "static" => LedMode::Static,
            "breathing" => LedMode::Breathing,
            _ => return Err("off, neon, static veya breathing".into()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Button {
    Left = 0,
    Right = 1,
    Middle = 2,
    Back = 3,
    Forward = 4,
}

impl Button {
    pub const ALL: [Button; 5] = [Button::Left, Button::Right, Button::Middle, Button::Back, Button::Forward];
}

impl FromStr for Button {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        Ok(match s {
            "left" => Button::Left,
            "right" => Button::Right,
            "middle" => Button::Middle,
            "back" => Button::Back,
            "forward" => Button::Forward,
            _ => return Err("left, right, middle, back veya forward".into()),
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Function {
    pub code: u8,
    pub id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
}

const fn func(code: u8, id: &'static str, label: &'static str, group: &'static str) -> Function {
    Function { code, id, label, group }
}

/// Tuşlara atanabilen fonksiyonlar (Hawk Hub'daki HM220_FUNCTIONS).
pub const FUNCTIONS: &[Function] = &[
    func(1, "disabled", "Tuş Kapalı", "Özel"),
    func(2, "left", "Sol Tık", "Temel"),
    func(3, "right", "Sağ Tık", "Temel"),
    func(4, "middle", "Orta Tık", "Temel"),
    func(5, "back", "Geri", "Temel"),
    func(6, "forward", "İleri", "Temel"),
    func(7, "double-click", "Çift Tık", "Temel"),
    func(8, "fire", "Ateş Tuşu", "Özel"),
    func(13, "dpi-cycle", "DPI Döngüsü", "DPI"),
    func(14, "dpi-up", "DPI +", "DPI"),
    func(15, "dpi-down", "DPI −", "DPI"),
    func(9, "scroll-up", "Yukarı Kaydır", "Kaydırma"),
    func(10, "scroll-down", "Aşağı Kaydır", "Kaydırma"),
    func(11, "scroll-left", "Sola Kaydır", "Kaydırma"),
    func(12, "scroll-right", "Sağa Kaydır", "Kaydırma"),
    func(21, "media-player", "Medya Oynatıcı", "Medya"),
    func(22, "prev-track", "Önceki Parça", "Medya"),
    func(23, "next-track", "Sonraki Parça", "Medya"),
    func(24, "play-pause", "Oynat/Duraklat", "Medya"),
    func(25, "stop", "Durdur", "Medya"),
    func(26, "mute", "Sessiz", "Medya"),
    func(27, "volume-up", "Ses +", "Medya"),
    func(28, "volume-down", "Ses −", "Medya"),
    func(29, "calculator", "Hesap Makinesi", "Uygulama"),
    func(30, "email", "E-posta", "Uygulama"),
    func(31, "browser-favorites", "Tarayıcı: Favoriler", "Uygulama"),
    func(32, "browser-forward", "Tarayıcı: İleri", "Uygulama"),
    func(33, "browser-back", "Tarayıcı: Geri", "Uygulama"),
    func(34, "browser-stop", "Tarayıcı: Dur", "Uygulama"),
    func(35, "my-computer", "Bilgisayarım", "Uygulama"),
    func(36, "browser-refresh", "Tarayıcı: Yenile", "Uygulama"),
    func(37, "browser-home", "Tarayıcı: Ana Sayfa", "Uygulama"),
    func(38, "browser-search", "Tarayıcı: Ara", "Uygulama"),
];

pub fn function_by_id(id: &str) -> Option<&'static Function> {
    FUNCTIONS.iter().find(|f| f.id == id)
}

// ── Checksum'lar ────────────────────────────────────────────────────────────

fn sum(bytes: &[u8]) -> u32 {
    bytes.iter().map(|&b| b as u32).sum()
}

/// Report 0x04: payload[2..=48] toplamı, 16 bit big-endian olarak [49], [50].
pub fn fix_dpi_checksum(f: &mut [u8]) {
    let s = sum(&f[2..49]) & 0xFFFF;
    f[49] = (s >> 8) as u8;
    f[50] = s as u8;
}

/// Report 0x05: payload[2..=9] toplamı, 16 bit big-endian olarak [10], [11].
/// (Hawk Hub burada 8 bitlik farklı bir formül kullanıyordu; fare o frame'leri `50 01 05` ile reddediyor.)
pub fn fix_led_checksum(f: &mut [u8]) {
    let s = sum(&f[2..10]) & 0xFFFF;
    f[10] = (s >> 8) as u8;
    f[11] = s as u8;
}

/// Report 0x08: payload[2..n-2] toplamı, 16 bit big-endian olarak son iki bayt.
pub fn fix_remap_checksum(f: &mut [u8]) {
    let n = f.len();
    let s = sum(&f[2..n - 2]) & 0xFFFF;
    f[n - 2] = (s >> 8) as u8;
    f[n - 1] = s as u8;
}

/// DPI payload'ındaki bir kademenin (0 tabanlı) değeri.
pub fn stage_dpi(dpi_payload: &[u8], i: usize) -> u32 {
    crate::dpi::decode(crate::dpi::DpiCode {
        x: dpi_payload[DPI_X_BASE + i],
        y: dpi_payload[DPI_Y_BASE + i],
        double: dpi_payload[DPI_DOUBLE] & (1 << i) != 0,
    })
}

/// DPI payload'ında bir kademeyi (0 tabanlı) ayarlar; checksum'ı çağıran düzeltir.
pub fn set_stage_dpi(dpi_payload: &mut [u8], i: usize, dpi: u32) {
    let c = crate::dpi::encode(dpi);
    dpi_payload[DPI_X_BASE + i] = c.x;
    dpi_payload[DPI_Y_BASE + i] = c.y;
    if c.double {
        dpi_payload[DPI_DOUBLE] |= 1 << i;
    } else {
        dpi_payload[DPI_DOUBLE] &= !(1 << i);
    }
}

pub fn polling_frame(hz: u16) -> Vec<u8> {
    let val = (1000.0 / hz as f64).round().max(1.0) as u8;
    vec![9, 1, val, 255 - val, 0, 0, 0, 0]
}

// ── Makrolar ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroStep {
    /// HID klavye usage kodu (4–164) veya 241–245.
    pub key: u8,
    /// ms, 0–10000.
    pub delay: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Macro {
    /// Cihazdaki yuva, 1–32.
    pub id: u8,
    #[serde(default)]
    pub name: String,
    pub repeat: u8,
    /// Tekrar modu 0–2 (Hawk Hub ile aynı).
    pub mode: u8,
    pub steps: Vec<MacroStep>,
}

/// Makro eylem tamponunun boyutu (bayt).
pub const MACRO_ACTIONS_LEN: usize = 100;
/// Bir tuşa basılı tutma süresi (ms).
const MACRO_HOLD_MS: u16 = 20;

/// Tek bir eylem: bit 7 yön (0 bas, 1 bırak), bit 0–6 eylemden sonraki bekleme (10 ms birim).
/// 1270 ms üstü 4 bayt: `[yön|kalan, tuş, 200ms_blok, 03]`. (attack-shark-x11-driver MacroAction)
fn encode_action(out: &mut Vec<u8>, release: bool, key: u8, delay_ms: u16) -> u8 {
    let dir = if release { 0x80 } else { 0 };
    let units = (delay_ms / 10).max(1);
    if units <= 0x7F {
        out.extend([dir | units as u8, key]);
        1
    } else {
        let blocks = (delay_ms / 200) as u8;
        let rem = ((delay_ms % 200) / 10) as u8;
        out.extend([dir | rem, key, blocks, 0x03]);
        2
    }
}

/// Report 0x09 payload'ları (report ID hariç, 3 sayfa × 63 bayt).
///
/// Yerleşim attack-shark-x11-driver ile aynı (aynı Beken firmware ailesi). Hawk Hub'ın kodlaması
/// gecikmeleri yanlış birimle yazıyordu ve HM220 V2'yi kilitliyordu.
/// Sayfa 0: `40 id 00 mod R G B tekrar ad[20] eylem_sayısı eylemler[0..34]`
/// Sayfa 1: `40 id 01 eylemler[34..94]`
/// Sayfa 2: `40 id 02 eylemler[94..100] ck_hi ck_lo`
pub fn macro_reports(m: &Macro) -> Result<Vec<Vec<u8>>> {
    if !(1..=32).contains(&m.id) || m.repeat == 0 || m.mode > 2 || !(1..=25).contains(&m.steps.len()) {
        return invalid("Makro yuvası (1-32), tekrar (1-255), mod (0-2) veya adım sayısı (1-25) geçersiz.");
    }
    let mut actions = Vec::new();
    let mut count = 0u8;
    for s in &m.steps {
        if !((4..=164).contains(&s.key) || (241..=245).contains(&s.key)) || s.delay > 10_000 {
            return invalid(format!("Geçersiz tuş {} veya bekleme {} ms (0-10000).", s.key, s.delay));
        }
        count += encode_action(&mut actions, false, s.key, MACRO_HOLD_MS);
        count += encode_action(&mut actions, true, s.key, s.delay);
    }
    if actions.len() > MACRO_ACTIONS_LEN {
        return invalid("Makro 100 baytlık sınırı aşıyor. Adım veya uzun bekleme sayısını azaltın.");
    }
    actions.resize(MACRO_ACTIONS_LEN, 0);

    let mut p0 = vec![0u8; 63];
    p0[..8].copy_from_slice(&[0x40, m.id, 0, m.mode, 0, 0, 0, m.repeat]);
    let name = m.name.as_bytes();
    let mut n = name.len().min(20);
    while !m.name.is_char_boundary(n) {
        n -= 1;
    }
    p0[8..8 + n].copy_from_slice(&name[..n]);
    p0[28] = count;
    p0[29..63].copy_from_slice(&actions[..34]);

    let mut p1 = vec![0u8; 63];
    p1[..3].copy_from_slice(&[0x40, m.id, 1]);
    p1[3..63].copy_from_slice(&actions[34..94]);

    let mut p2 = vec![0u8; 63];
    p2[..3].copy_from_slice(&[0x40, m.id, 2]);
    p2[3..9].copy_from_slice(&actions[94..100]);
    // Frame indeksleriyle: sayfa0[4..63] + sayfa1[4..63] + sayfa2[4..8], 16 bit big-endian.
    let ck = (sum(&p0[3..]) + sum(&p1[3..]) + sum(&p2[3..8])) & 0xFFFF;
    p2[9] = (ck >> 8) as u8;
    p2[10] = ck as u8;
    Ok(vec![p0, p1, p2])
}

/// Fareden okunan makro (131 bayt, report ID dahil) checksum'ı doğru mu: `sum([3..=128])`, little-endian [129..130].
pub fn macro_read_checksum_ok(frame: &[u8]) -> bool {
    frame.len() >= 131 && (sum(&frame[3..129]) & 0xFFFF) as u16 == u16::from_le_bytes([frame[129], frame[130]])
}

/// Tuş adını (a, 5, enter, f1, 0x2c …) HID usage koduna çevirir.
pub fn key_code(name: &str) -> Option<u8> {
    let n = name.trim().to_ascii_lowercase();
    let b = n.as_bytes();
    if b.len() == 1 && b[0].is_ascii_lowercase() {
        return Some(b[0] - b'a' + 4);
    }
    if b.len() == 1 && b[0].is_ascii_digit() {
        return Some(if b[0] == b'0' { 39 } else { b[0] - b'1' + 30 });
    }
    if let Some(f) = n.strip_prefix('f').and_then(|x| x.parse::<u8>().ok()) {
        if (1..=12).contains(&f) {
            return Some(57 + f);
        }
    }
    let named = match n.as_str() {
        "enter" => 40,
        "esc" | "escape" => 41,
        "backspace" => 42,
        "tab" => 43,
        "space" => 44,
        "right" => 79,
        "left" => 80,
        "down" => 81,
        "up" => 82,
        _ => 0,
    };
    if named != 0 {
        return Some(named);
    }
    match n.strip_prefix("0x") {
        Some(hex) => u8::from_str_radix(hex, 16).ok(),
        None => n.parse().ok(),
    }
}

// ── Pil ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Battery {
    /// %, bilinmiyorsa None.
    pub level: Option<u8>,
    pub charging: bool,
}

/// Input report `03 5C 40 durum seviye`.
pub fn parse_battery(d: &[u8], wired: bool) -> Option<Battery> {
    if d.len() < 5 || d[..3] != [3, 92, 64] || !(1..=3).contains(&d[3]) || d[4] > 100 {
        return None;
    }
    let charging = d[3] == 3;
    let level = match d[3] {
        2 => Some(100),
        3 => None,
        _ if wired && d[4] == 0 => None,
        _ => Some(d[4]),
    };
    Some(Battery { level, charging })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_have_valid_checksums() {
        let mut d = DPI_TEMPLATE.to_vec();
        fix_dpi_checksum(&mut d);
        assert_eq!(d, DPI_TEMPLATE);
        let mut l = LED_TEMPLATE.to_vec();
        fix_led_checksum(&mut l);
        assert_eq!(l, LED_TEMPLATE);
        let r0 = remap_template();
        assert_eq!(r0.len(), 58);
        let mut r = r0.clone();
        fix_remap_checksum(&mut r);
        assert_eq!(r, r0);
    }

    #[test]
    fn dpi_conversion_matches_defaults() {
        let dpis: Vec<u32> = (0..5).map(|i| stage_dpi(&DPI_TEMPLATE, i)).collect();
        assert_eq!(dpis, [400, 800, 1600, 3200, 4800]);
        let mut d = DPI_TEMPLATE.to_vec();
        set_stage_dpi(&mut d, 0, 12_000);
        assert_eq!(stage_dpi(&d, 0), 12_000);
        assert_eq!(d[DPI_DOUBLE] & 1, 1);
        set_stage_dpi(&mut d, 0, 400);
        assert_eq!(&d[..], &DPI_TEMPLATE[..]);
    }

    #[test]
    fn led_checksum_matches_device() {
        // HM220 V2'den okunan ve kabul edilen frame'ler.
        let mut l = [0x0f, 0x01, 0x02, 0x03, 0xc8, 0x00, 0x00, 0xff, 0x3c, 0x00, 0, 0];
        fix_led_checksum(&mut l);
        assert_eq!(&l[10..], &[0x02, 0x08]);
    }

    #[test]
    fn events() {
        // HM220'den kaydedilen gerçek mesajlar.
        assert_eq!(parse_event(&[3, 0x5c, 0x10, 3, 0], false), Some(Event::DpiStage(3)));
        assert_eq!(
            parse_event(&[3, 0x5c, 0x40, 1, 0x5e], false),
            Some(Event::Battery(Battery { level: Some(94), charging: false }))
        );
        assert_eq!(parse_event(&[3, 0x5c, 0x50, 0, 5], false), Some(Event::Ack { report: 5, ok: true }));
        assert_eq!(parse_event(&[3, 0x5c, 0xff, 0xff, 0], false), None);
    }

    #[test]
    fn ack() {
        assert_eq!(parse_ack(&[3, 0x5c, 0x50, 0, 5]), Some((5, true)));
        assert_eq!(parse_ack(&[3, 0x5c, 0x50, 1, 5]), Some((5, false)));
        assert_eq!(parse_ack(&[3, 0x5c, 0x40, 1, 95]), None);
    }

    #[test]
    fn polling() {
        assert_eq!(polling_frame(1000), [9, 1, 1, 254, 0, 0, 0, 0]);
        assert_eq!(polling_frame(125), [9, 1, 8, 247, 0, 0, 0, 0]);
    }

    #[test]
    fn macro_layout() {
        let m = Macro {
            id: 3,
            name: String::new(),
            repeat: 1,
            mode: 0,
            steps: vec![MacroStep { key: 11, delay: 50 }, MacroStep { key: 12, delay: 1500 }],
        };
        let r = macro_reports(&m).unwrap();
        assert_eq!(r.len(), 3);
        assert!(r.iter().all(|x| x.len() == 63));
        // Report ID dahil frame'ler, X11 belgesindeki indekslerle karşılaştırmak için.
        let f: Vec<Vec<u8>> = r.iter().map(|p| [vec![9u8], p.clone()].concat()).collect();
        assert_eq!(&f[0][..9], &[9, 0x40, 3, 0, 0, 0, 0, 0, 1]);
        // Eylem sayısı: bas + bırak + bas + (uzun) bırak = 1 + 1 + 1 + 2
        assert_eq!(f[0][29], 5);
        assert_eq!(&f[0][30..40], &[0x02, 11, 0x85, 11, 0x02, 12, 0x80 | 10, 12, 7, 3]);
        assert_eq!(&f[1][..4], &[9, 0x40, 3, 1]);
        assert_eq!(&f[2][..4], &[9, 0x40, 3, 2]);
        let ck = (sum(&f[0][4..64]) + sum(&f[1][4..64]) + sum(&f[2][4..9])) & 0xFFFF;
        assert_eq!(&f[2][10..12], &[(ck >> 8) as u8, ck as u8]);
    }

    #[test]
    fn battery() {
        assert_eq!(parse_battery(&[3, 92, 64, 1, 77], false), Some(Battery { level: Some(77), charging: false }));
        assert_eq!(parse_battery(&[3, 92, 64, 3, 0], false), Some(Battery { level: None, charging: true }));
        assert_eq!(parse_battery(&[3, 92, 64, 1, 0], true), Some(Battery { level: None, charging: false }));
        assert_eq!(parse_battery(&[1, 2, 3, 4, 5], false), None);
    }

    #[test]
    fn keys() {
        assert_eq!(key_code("a"), Some(4));
        assert_eq!(key_code("1"), Some(30));
        assert_eq!(key_code("0"), Some(39));
        assert_eq!(key_code("F12"), Some(69));
        assert_eq!(key_code("0x2c"), Some(44));
    }
}
