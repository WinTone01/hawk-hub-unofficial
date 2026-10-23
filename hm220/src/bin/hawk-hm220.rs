//! Hawk HM220 komut satırı aracı.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use hm220::*;

#[derive(Parser)]
#[command(name = "hawk-hm220", about = "Hawk HM220 Linux yapılandırma aracı")]
struct Cli {
    /// hidraw yolu (varsayılan: otomatik)
    #[arg(long, global = true)]
    device: Option<PathBuf>,
    /// Cihaza yazma, gönderilecek frame'leri göster
    #[arg(long, global = true)]
    dry_run: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Bağlı HM220'leri listele
    List,
    /// Kayıtlı (son yazılan) ayarları göster
    Show,
    /// Farenin gerçek ayarlarını okuyup kaydet ve göster
    Sync,
    /// Pil durumunu oku
    Battery,
    /// Bluetooth LE bağlantısını ve pilini göster (BlueZ / Windows cihaz özellikleri)
    Bluetooth,
    /// Kayıtlı tüm ayarları cihaza yeniden yaz
    Apply,
    /// Hawk Hub varsayılanlarına döndür
    Reset,
    /// DPI kademesi / değerleri
    Dpi {
        /// Aktif kademe (1-5)
        #[arg(long)]
        stage: Option<u8>,
        /// KADEME:DPI, ör. 2:1200 (tekrarlanabilir)
        #[arg(long = "set", value_parser = parse_stage_dpi)]
        set: Vec<(u8, u32)>,
    },
    /// Raporlama hızı (125, 250, 500, 1000)
    Polling { hz: u16 },
    /// Aydınlatma
    Led {
        /// off, neon, static, breathing
        #[arg(long)]
        mode: Option<LedMode>,
        /// RRGGBB
        #[arg(long)]
        color: Option<String>,
        /// 1-8
        #[arg(long)]
        brightness: Option<u8>,
        /// 0-5 (0 en hızlı)
        #[arg(long)]
        speed: Option<u8>,
    },
    /// Tuş tepki süresi (0, 2, 4, 8, 12 ms)
    Response { ms: u8 },
    /// Açı düzeltme
    AngleSnap { state: OnOff },
    /// Ripple control
    Ripple { state: OnOff },
    /// Tuş atama: left|right|middle|back|forward FONKSİYON
    Button {
        button: hm220::Button,
        /// Fonksiyon kimliği (bkz. `hawk-hm220 functions`)
        function: String,
    },
    /// Atanabilir fonksiyonları listele
    Functions,
    /// Makro yükle ve tuşa ata
    Macro {
        button: hm220::Button,
        /// Makro yuvası 1-32
        #[arg(long, default_value_t = 1)]
        id: u8,
        /// TUŞ:MS listesi, ör. 'h:50,i:50' veya '0x0b:50,enter:30'
        #[arg(long, value_parser = parse_steps)]
        steps: Steps,
        #[arg(long, default_value_t = 1)]
        repeat: u8,
        /// Tekrar modu 0-2
        #[arg(long, default_value_t = 0)]
        mode: u8,
        #[arg(long, default_value = "")]
        name: String,
    },
    /// Makroyu sil, bağlı tuşları varsayılana döndür
    DeleteMacro { id: u8 },
    /// [RE] Olay kanalını dinle ve gelen her mesajı yaz (pil, onay, DPI tuşu …)
    Listen {
        /// Saniye
        #[arg(default_value_t = 15)]
        secs: u64,
    },
    /// [RE] Komut arayüzünün HID rapor tanımlayıcısını hex yaz
    Descriptor,
    /// [RE] Feature report oku: report ID (ör. 4 veya 0x04)
    Get {
        #[arg(value_parser = parse_byte)]
        id: u8,
        /// Okunacak uzunluk (report ID dahil)
        #[arg(long, default_value_t = 64)]
        len: usize,
    },
    /// [RE] Ham feature report yaz: hex baytlar, ilki report ID (ör. 05 0f 01 00 ...)
    Raw {
        #[arg(value_parser = parse_byte, num_args = 1..)]
        bytes: Vec<u8>,
        /// Sıfırlarla bu uzunluğa doldur (report ID dahil)
        #[arg(long)]
        len: Option<usize>,
        /// Gönderdikten sonra olay kanalını bu kadar ms dinle (komut onayı 03 xx 50 durum id)
        #[arg(long, default_value_t = 1500)]
        listen: u64,
    },
}

fn parse_byte(s: &str) -> std::result::Result<u8, String> {
    let s = s.trim_start_matches("0x");
    u8::from_str_radix(s, 16).map_err(|_| format!("geçersiz hex bayt: {s}"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.chunks(16).map(|c| c.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join("\n")
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum OnOff {
    On,
    Off,
}

fn parse_stage_dpi(s: &str) -> std::result::Result<(u8, u32), String> {
    let (a, b) = s.split_once(':').ok_or("KADEME:DPI biçiminde olmalı")?;
    Ok((a.parse().map_err(|_| "geçersiz kademe")?, b.parse().map_err(|_| "geçersiz DPI")?))
}

#[derive(Clone)]
struct Steps(Vec<MacroStep>);

fn parse_steps(s: &str) -> std::result::Result<Steps, String> {
    s.split(',')
        .map(|part| {
            let (key, delay) = part.split_once(':').unwrap_or((part, "50"));
            Ok(MacroStep {
                key: key_code(key).ok_or_else(|| format!("bilinmeyen tuş: {key}"))?,
                delay: delay.trim().parse().map_err(|_| format!("geçersiz bekleme: {delay}"))?,
            })
        })
        .collect::<std::result::Result<Vec<_>, String>>()
        .map(Steps)
}

fn describe(s: &Settings) {
    let dpis: Vec<String> = s.dpi_values.iter().enumerate().map(|(i, d)| format!("{}={d}", i + 1)).collect();
    let on = |b: bool| if b { "açık" } else { "kapalı" };
    println!("DPI kademesi : {}", s.dpi_stage);
    println!("DPI değerleri: {}", dpis.join(", "));
    println!("Açı düzeltme : {}   Ripple: {}", on(s.angle_snap), on(s.ripple));
    println!("Polling      : {} Hz", s.polling_hz);
    println!(
        "LED          : {:?}, renk {}, parlaklık {}, hız {}",
        s.led_mode, s.led_color, s.led_brightness, s.led_speed
    );
    println!("Tepki süresi : {} ms", s.response_ms);
    for b in &s.buttons {
        let label = match b.macro_id {
            Some(id) => format!("makro {id}"),
            None => FUNCTIONS.iter().find(|f| f.code == b.code).map_or(format!("{:#04x}", b.code), |f| f.id.to_string()),
        };
        println!("Tuş {:<8}: {label}", format!("{:?}", b.button).to_lowercase());
    }
}

fn run(cli: Cli) -> hm220::Result<()> {
    match cli.cmd {
        Cmd::List => {
            let devs = find_devices();
            if devs.is_empty() {
                println!("HM220 bulunamadı.");
            }
            for d in devs {
                println!("{}  {:04x}:{:04x}  {}", d.path.display(), VID, d.pid, d.transport());
            }
            return Ok(());
        }
        Cmd::Show => {
            describe(&Config::load().settings());
            return Ok(());
        }
        Cmd::Functions => {
            for f in FUNCTIONS {
                println!("{:<18} {}", f.id, f.label);
            }
            return Ok(());
        }
        Cmd::Bluetooth => {
            match bluetooth_status() {
                None => println!("Bluetooth ile bağlı HM220 yok."),
                Some(BluetoothStatus { battery: Some(l) }) => println!("Bluetooth: bağlı, pil %{l}"),
                Some(_) => println!("Bluetooth: bağlı, pil bilinmiyor"),
            }
            return Ok(());
        }
        _ => {}
    }

    if let Cmd::Listen { secs } = cli.cmd {
        let dev = match cli.device.clone() {
            Some(p) => Device::open_path(p)?,
            None => Device::open_first()?,
        };
        let start = std::time::Instant::now();
        let end = start + Duration::from_secs(secs);
        while let Some(left) = end.checked_duration_since(std::time::Instant::now()) {
            if let Some(r) = dev.read_input(left)? {
                println!("{:6.2}s  {}", start.elapsed().as_secs_f32(), hex(&r));
            }
        }
        return Ok(());
    }

    if matches!(cli.cmd, Cmd::Descriptor | Cmd::Get { .. } | Cmd::Raw { .. }) {
        let dev = match cli.device.clone() {
            Some(p) => Device::open_path(p)?,
            None => Device::open_first()?,
        };
        match &cli.cmd {
            Cmd::Descriptor => println!("{}", hex(&dev.report_descriptor()?)),
            Cmd::Get { id, len } => {
                let mut buf = vec![0u8; *len];
                buf[0] = *id;
                let n = dev.get_feature(&mut buf)?;
                println!("{n} bayt:\n{}", hex(&buf[..n.min(buf.len())]));
            }
            Cmd::Raw { bytes, len, listen } => {
                let mut buf = bytes.clone();
                if let Some(l) = len {
                    buf.resize(*l, 0);
                }
                dev.send_raw(&buf)?;
                println!("{} bayt gönderildi", buf.len());
                let end = std::time::Instant::now() + Duration::from_millis(*listen);
                while let Some(left) = end.checked_duration_since(std::time::Instant::now()) {
                    match dev.read_input(left)? {
                        Some(r) => println!("  <- {}", hex(&r)),
                        None => break,
                    }
                }
            }
            _ => unreachable!(),
        }
        return Ok(());
    }

    if let Cmd::Battery = cli.cmd {
        let dev = match cli.device {
            Some(p) => Device::open_path(p)?,
            None => Device::open_first()?,
        };
        match dev.read_battery(Duration::from_secs(5))? {
            None => println!("Pil raporu gelmedi (fare birkaç saniyede bir yollar; tekrar deneyin)."),
            Some(Battery { charging: true, .. }) => println!("Pil: şarj oluyor"),
            Some(Battery { level: Some(l), .. }) => println!("Pil: %{l}"),
            Some(_) => println!("Pil: bilinmiyor"),
        }
        return Ok(());
    }

    let mut s = if cli.dry_run { Session::dry_run() } else { Session::open(cli.device)? };
    match cli.cmd {
        Cmd::Sync => {
            s.sync_from_device()?;
            s.save()?;
            describe(&s.cfg.settings());
            return Ok(());
        }
        Cmd::Apply => s.apply_all()?,
        Cmd::Reset => s.reset()?,
        Cmd::Dpi { stage, set } => {
            for (st, dpi) in set {
                s.cfg.set_dpi_value(st, dpi)?;
            }
            if let Some(st) = stage {
                s.cfg.set_dpi_stage(st)?;
            }
            s.push(Report::Dpi)?;
        }
        Cmd::Polling { hz } => {
            s.cfg.set_polling(hz)?;
            s.push(Report::Polling)?;
        }
        Cmd::Led { mode, color, brightness, speed } => {
            if let Some(m) = mode {
                s.cfg.set_led_mode(m);
            }
            let color_changed = color.is_some();
            if let Some(c) = color {
                s.cfg.set_led_color(parse_hex_color(&c)?);
            }
            if let Some(b) = brightness {
                s.cfg.set_led_brightness(b)?;
            }
            if let Some(v) = speed {
                s.cfg.set_led_speed(v)?;
            }
            // Renk DPI kademe renklerinde tutulur (bkz. Config::set_led_color).
            if color_changed {
                s.push(Report::Dpi)?;
            }
            s.push(Report::Led)?;
        }
        Cmd::Response { ms } => {
            s.cfg.set_response_ms(ms)?;
            s.push(Report::Led)?;
        }
        Cmd::AngleSnap { state } => {
            s.cfg.set_angle_snap(matches!(state, OnOff::On));
            s.push(Report::Dpi)?;
        }
        Cmd::Ripple { state } => {
            s.cfg.set_ripple(matches!(state, OnOff::On));
            s.push(Report::Dpi)?;
        }
        Cmd::Button { button, function } => {
            let f = function_by_id(&function)
                .ok_or_else(|| hm220::Error::Invalid(format!("bilinmeyen fonksiyon: {function}")))?;
            s.cfg.set_button(button, f.code)?;
            s.push(Report::Remap)?;
        }
        Cmd::Macro { button, id, steps, repeat, mode, name } => {
            s.upload_macro(button, Macro { id, name, repeat, mode, steps: steps.0 })?;
        }
        Cmd::DeleteMacro { id } => s.delete_macro(id)?,
        Cmd::List | Cmd::Show | Cmd::Functions | Cmd::Bluetooth | Cmd::Battery |Cmd::Listen { .. } | Cmd::Descriptor | Cmd::Get { .. } | Cmd::Raw { .. } => {
            unreachable!()
        }
    }
    s.save()?;
    println!("Tamam.");
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("hata: {e}");
            ExitCode::FAILURE
        }
    }
}
