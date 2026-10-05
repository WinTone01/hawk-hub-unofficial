//! Hawk HM220 komut satÄ±rÄ± aracÄ±.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use hm220::*;

#[derive(Parser)]
#[command(name = "hawk-hm220", about = "Hawk HM220 Linux yapÄ±landÄ±rma aracÄ±")]
struct Cli {
    /// hidraw yolu (varsayÄ±lan: otomatik)
    #[arg(long, global = true)]
    device: Option<PathBuf>,
    /// Cihaza yazma, gÃ¶nderilecek frame'leri gÃ¶ster
    #[arg(long, global = true)]
    dry_run: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// BaÄŸlÄ± HM220'leri listele
    List,
    /// KayÄ±tlÄ± (son yazÄ±lan) ayarlarÄ± gÃ¶ster
    Show,
    /// Farenin gerÃ§ek ayarlarÄ±nÄ± okuyup kaydet ve gÃ¶ster
    Sync,
    /// Pil durumunu oku
    Battery,
    /// KayÄ±tlÄ± tÃ¼m ayarlarÄ± cihaza yeniden yaz
    Apply,
    /// Hawk Hub varsayÄ±lanlarÄ±na dÃ¶ndÃ¼r
    Reset,
    /// DPI kademesi / deÄŸerleri
    Dpi {
        /// Aktif kademe (1-5)
        #[arg(long)]
        stage: Option<u8>,
        /// KADEME:DPI, Ã¶r. 2:1200 (tekrarlanabilir)
        #[arg(long = "set", value_parser = parse_stage_dpi)]
        set: Vec<(u8, u32)>,
    },
    /// Raporlama hÄ±zÄ± (125, 250, 500, 1000)
    Polling { hz: u16 },
    /// AydÄ±nlatma
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
        /// 0-5 (0 en hÄ±zlÄ±)
        #[arg(long)]
        speed: Option<u8>,
    },
    /// TuÅŸ tepki sÃ¼resi (0, 2, 4, 8, 12 ms)
    Response { ms: u8 },
    /// AÃ§Ä± dÃ¼zeltme
    AngleSnap { state: OnOff },
    /// Ripple control
    Ripple { state: OnOff },
    /// TuÅŸ atama: left|right|middle|back|forward FONKSÄ°YON
    Button {
        button: hm220::Button,
        /// Fonksiyon kimliÄŸi (bkz. `hawk-hm220 functions`)
        function: String,
    },
    /// Atanabilir fonksiyonlarÄ± listele
    Functions,
    /// Makro yÃ¼kle ve tuÅŸa ata
    Macro {
        button: hm220::Button,
        /// Makro yuvasÄ± 1-32
        #[arg(long, default_value_t = 1)]
        id: u8,
        /// TUÅ:MS listesi, Ã¶r. 'h:50,i:50' veya '0x0b:50,enter:30'
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
    /// Makroyu sil, baÄŸlÄ± tuÅŸlarÄ± varsayÄ±lana dÃ¶ndÃ¼r
    DeleteMacro { id: u8 },
    /// [RE] Olay kanalÄ±nÄ± dinle ve gelen her mesajÄ± yaz (pil, onay, DPI tuÅŸu â€¦)
    Listen {
        /// Saniye
        #[arg(default_value_t = 15)]
        secs: u64,
    },
    /// [RE] Komut arayÃ¼zÃ¼nÃ¼n HID rapor tanÄ±mlayÄ±cÄ±sÄ±nÄ± hex yaz
    Descriptor,
    /// [RE] Feature report oku: report ID (Ã¶r. 4 veya 0x04)
    Get {
        #[arg(value_parser = parse_byte)]
        id: u8,
        /// Okunacak uzunluk (report ID dahil)
        #[arg(long, default_value_t = 64)]
        len: usize,
    },
    /// [RE] Ham feature report yaz: hex baytlar, ilki report ID (Ã¶r. 05 0f 01 00 ...)
    Raw {
        #[arg(value_parser = parse_byte, num_args = 1..)]
        bytes: Vec<u8>,
        /// SÄ±fÄ±rlarla bu uzunluÄŸa doldur (report ID dahil)
        #[arg(long)]
        len: Option<usize>,
        /// GÃ¶nderdikten sonra olay kanalÄ±nÄ± bu kadar ms dinle (komut onayÄ± 03 xx 50 durum id)
        #[arg(long, default_value_t = 1500)]
        listen: u64,
    },
}

fn parse_byte(s: &str) -> std::result::Result<u8, String> {
    let s = s.trim_start_matches("0x");
    u8::from_str_radix(s, 16).map_err(|_| format!("geÃ§ersiz hex bayt: {s}"))
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
    let (a, b) = s.split_once(':').ok_or("KADEME:DPI biÃ§iminde olmalÄ±")?;
    Ok((a.parse().map_err(|_| "geÃ§ersiz kademe")?, b.parse().map_err(|_| "geÃ§ersiz DPI")?))
}

#[derive(Clone)]
struct Steps(Vec<MacroStep>);

fn parse_steps(s: &str) -> std::result::Result<Steps, String> {
    s.split(',')
        .map(|part| {
            let (key, delay) = part.split_once(':').unwrap_or((part, "50"));
            Ok(MacroStep {
                key: key_code(key).ok_or_else(|| format!("bilinmeyen tuÅŸ: {key}"))?,
                delay: delay.trim().parse().map_err(|_| format!("geÃ§ersiz bekleme: {delay}"))?,
            })
        })
        .collect::<std::result::Result<Vec<_>, String>>()
        .map(Steps)
}

fn describe(s: &Settings) {
    let dpis: Vec<String> = s.dpi_values.iter().enumerate().map(|(i, d)| format!("{}={d}", i + 1)).collect();
    let on = |b: bool| if b { "aÃ§Ä±k" } else { "kapalÄ±" };
    println!("DPI kademesi : {}", s.dpi_stage);
    println!("DPI deÄŸerleri: {}", dpis.join(", "));
    println!("AÃ§Ä± dÃ¼zeltme : {}   Ripple: {}", on(s.angle_snap), on(s.ripple));
    println!("Polling      : {} Hz", s.polling_hz);
    println!(
        "LED          : {:?}, renk {}, parlaklÄ±k {}, hÄ±z {}",
        s.led_mode, s.led_color, s.led_brightness, s.led_speed
    );
    println!("Tepki sÃ¼resi : {} ms", s.response_ms);
    for b in &s.buttons {
        let label = match b.macro_id {
            Some(id) => format!("makro {id}"),
            None => FUNCTIONS.iter().find(|f| f.code == b.code).map_or(format!("{:#04x}", b.code), |f| f.id.to_string()),
        };
        println!("TuÅŸ {:<8}: {label}", format!("{:?}", b.button).to_lowercase());
    }
}

fn run(cli: Cli) -> hm220::Result<()> {
    match cli.cmd {
        Cmd::List => {
            let devs = find_devices();
            if devs.is_empty() {
                println!("HM220 bulunamadÄ±.");
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
                println!("{} bayt gÃ¶nderildi", buf.len());
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
            None => println!("Pil raporu gelmedi (fare birkaÃ§ saniyede bir yollar; tekrar deneyin)."),
            Some(Battery { charging: true, .. }) => println!("Pil: ÅŸarj oluyor"),
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
        Cmd::List | Cmd::Show | Cmd::Functions | Cmd::Battery | Cmd::Listen { .. } | Cmd::Descriptor | Cmd::Get { .. } | Cmd::Raw { .. } => {
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
