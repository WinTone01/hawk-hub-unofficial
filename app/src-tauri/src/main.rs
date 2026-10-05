// Hawk Hub Unofficial — Tauri arka ucu. Tüm cihaz işleri hm220 kütüphanesinde.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod apps;

use std::path::Path;

use hm220::{
    config_dir, find_devices, parse_hex_color, AutoSwitch, Battery, Button, Config, Device, DeviceInfo, Event, Function,
    LedMode, Macro, ProfileInfo, ProfileMeta, Report, Session, Settings, FUNCTIONS, POLLING_RATES, RESPONSE_STEPS_MS,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

const LOG_LIMIT: usize = 200;

#[derive(Default)]
struct AppState {
    /// Hawk Hub'daki writeChain gibi: yazmalar sırayla yapılır.
    write_lock: Arc<Mutex<()>>,
    status: Arc<Mutex<DeviceStatus>>,
    log: Arc<Mutex<VecDeque<LogEntry>>>,
    /// İşlemci kullanımı iki ölçüm arasındaki farktan hesaplandığı için saklanır.
    sys: Mutex<Option<sysinfo::System>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DeviceView {
    path: String,
    pid: u16,
    transport: &'static str,
}

impl From<&DeviceInfo> for DeviceView {
    fn from(d: &DeviceInfo) -> Self {
        DeviceView { path: d.path.display().to_string(), pid: d.pid, transport: d.transport() }
    }
}

#[derive(Serialize, Clone, PartialEq, Default)]
struct DeviceStatus {
    device: Option<DeviceView>,
    battery: Option<Battery>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    #[serde(flatten)]
    device: DeviceStatus,
    settings: Settings,
    config_dir: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Catalog {
    functions: &'static [Function],
    polling_rates: [u16; 4],
    response_steps: [u8; 5],
}

#[derive(Serialize, Clone)]
struct Frame {
    report: u8,
    name: &'static str,
    bytes: Vec<u8>,
    checksum: Vec<usize>,
}

#[derive(Serialize, Clone)]
struct LogEntry {
    at: u64,
    report: u8,
    name: &'static str,
    bytes: Vec<u8>,
    ok: bool,
    error: Option<String>,
}

fn report_name(r: Report) -> &'static str {
    match r {
        Report::Dpi => "DPI / sensör",
        Report::Led => "LED + tepki",
        Report::Polling => "Polling",
        Report::Remap => "Tuş atama",
        Report::Macro => "Makro",
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

// ── Okuma komutları ─────────────────────────────────────────────────────────

#[tauri::command]
fn status(state: State<'_, AppState>) -> Status {
    Status {
        device: lock(&state.status).clone(),
        settings: Config::load().settings(),
        config_dir: config_dir().display().to_string(),
    }
}

#[tauri::command]
fn catalog() -> Catalog {
    Catalog { functions: FUNCTIONS, polling_rates: POLLING_RATES, response_steps: RESPONSE_STEPS_MS }
}

#[tauri::command]
fn frames() -> Vec<Frame> {
    let cfg = Config::load();
    [(Report::Dpi, vec![49, 50]), (Report::Led, vec![11]), (Report::Polling, vec![]), (Report::Remap, vec![56, 57])]
        .into_iter()
        .map(|(r, checksum)| Frame { report: r as u8, name: report_name(r), bytes: cfg.frame(r), checksum })
        .collect()
}

#[tauri::command]
fn report_log(state: State<'_, AppState>) -> Vec<LogEntry> {
    lock(&state.log).iter().cloned().collect()
}

// ── Yazma komutları ─────────────────────────────────────────────────────────

/// Cihazı açar, `f` ile ayarı değiştirip gönderir, başarılıysa kaydeder.
fn run_blocking<F>(write_lock: &Mutex<()>, log: &Mutex<VecDeque<LogEntry>>, f: F) -> hm220::Result<Settings>
where
    F: FnOnce(&mut Session) -> hm220::Result<()>,
{
    let _guard = lock(write_lock);
    let mut s = Session::open(None)?;
    let result = f(&mut s).and_then(|()| s.save());

    // Hata olduysa yalnızca son gönderim başarısız sayılır.
    let at = now_ms();
    let n = s.sent.len();
    let mut log = lock(log);
    for (i, (r, bytes)) in s.sent.drain(..).enumerate() {
        let failed = result.is_err() && i + 1 == n;
        log.push_front(LogEntry {
            at,
            report: r as u8,
            name: report_name(r),
            bytes,
            ok: !failed,
            error: failed.then(|| result.as_ref().unwrap_err().to_string()),
        });
    }
    log.truncate(LOG_LIMIT);
    drop(log);

    result.map(|()| s.cfg.settings())
}

/// `run_blocking`'i ayrı bir thread'de çalıştırır (makro yükleme saniyeler sürebilir).
async fn run<F>(state: &AppState, f: F) -> Result<Settings, String>
where
    F: FnOnce(&mut Session) -> hm220::Result<()> + Send + 'static,
{
    let write_lock = state.write_lock.clone();
    let log = state.log.clone();
    tauri::async_runtime::spawn_blocking(move || run_blocking(&write_lock, &log, f))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_dpi_stage(state: State<'_, AppState>, stage: u8) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_dpi_stage(stage)?;
        s.push(Report::Dpi)
    })
    .await
}

#[tauri::command]
async fn set_dpi_value(state: State<'_, AppState>, stage: u8, dpi: u32) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_dpi_value(stage, dpi)?;
        s.push(Report::Dpi)
    })
    .await
}

#[tauri::command]
async fn set_dpi_color(state: State<'_, AppState>, stage: u8, color: String) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_dpi_color(stage, parse_hex_color(&color)?)?;
        s.push(Report::Dpi)
    })
    .await
}

/// Farenin gerçek ayarlarını okur (fare ayarları cihazda saklar; önbellek yalnızca yedek).
#[tauri::command]
async fn sync_device(state: State<'_, AppState>) -> Result<Settings, String> {
    run(&state, |s| s.sync_from_device()).await
}

#[tauri::command]
async fn set_angle_snap(state: State<'_, AppState>, on: bool) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_angle_snap(on);
        s.push(Report::Dpi)
    })
    .await
}

#[tauri::command]
async fn set_ripple(state: State<'_, AppState>, on: bool) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_ripple(on);
        s.push(Report::Dpi)
    })
    .await
}

#[tauri::command]
async fn set_polling(state: State<'_, AppState>, hz: u16) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_polling(hz)?;
        s.push(Report::Polling)
    })
    .await
}

#[tauri::command]
async fn set_led(
    state: State<'_, AppState>,
    mode: Option<LedMode>,
    color: Option<String>,
    brightness: Option<u8>,
    speed: Option<u8>,
) -> Result<Settings, String> {
    run(&state, move |s| {
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
        s.push(Report::Led)
    })
    .await
}

#[tauri::command]
async fn set_response(state: State<'_, AppState>, ms: u8) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_response_ms(ms)?;
        s.push(Report::Led)
    })
    .await
}

#[tauri::command]
async fn set_button(state: State<'_, AppState>, button: Button, code: u8) -> Result<Settings, String> {
    run(&state, move |s| {
        s.cfg.set_button(button, code)?;
        s.push(Report::Remap)
    })
    .await
}

#[tauri::command]
async fn upload_macro(state: State<'_, AppState>, button: Button, definition: Macro) -> Result<Settings, String> {
    run(&state, move |s| s.upload_macro(button, definition)).await
}

#[tauri::command]
async fn delete_macro(state: State<'_, AppState>, id: u8) -> Result<Settings, String> {
    run(&state, move |s| s.delete_macro(id)).await
}

#[tauri::command]
async fn apply_all(state: State<'_, AppState>) -> Result<Settings, String> {
    run(&state, |s| s.apply_all()).await
}

#[tauri::command]
async fn reset(state: State<'_, AppState>) -> Result<Settings, String> {
    run(&state, |s| s.reset()).await
}

// ── Profiller ───────────────────────────────────────────────────────────────

#[tauri::command]
fn list_profiles() -> Vec<ProfileInfo> {
    hm220::list_profiles()
}

#[tauri::command]
fn save_profile(name: String) -> Result<Vec<ProfileInfo>, String> {
    hm220::save_profile(&name, &Config::load()).map_err(|e| e.to_string())?;
    Ok(hm220::list_profiles())
}

#[tauri::command]
fn delete_profile(name: String) -> Result<Vec<ProfileInfo>, String> {
    hm220::delete_profile(&name).map_err(|e| e.to_string())?;
    Ok(hm220::list_profiles())
}

#[tauri::command]
fn export_profile(name: String) -> Result<String, String> {
    hm220::load_profile(&name).map(|c| c.to_json()).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_profile(name: String, json: String) -> Result<Vec<ProfileInfo>, String> {
    let cfg = Config::from_json(json.as_bytes()).map_err(|e| e.to_string())?;
    hm220::save_profile(&name, &cfg).map_err(|e| e.to_string())?;
    Ok(hm220::list_profiles())
}

#[tauri::command]
async fn apply_profile(state: State<'_, AppState>, name: String) -> Result<Settings, String> {
    let cfg = hm220::load_profile(&name).map_err(|e| e.to_string())?;
    run(&state, move |s| s.apply_config(cfg)).await
}

#[tauri::command]
fn set_profile_meta(name: String, meta: ProfileMeta) -> Result<Vec<ProfileInfo>, String> {
    hm220::set_profile_meta(&name, meta).map_err(|e| e.to_string())?;
    Ok(hm220::list_profiles())
}

#[tauri::command]
fn get_auto_switch() -> AutoSwitch {
    hm220::auto_switch()
}

#[tauri::command]
fn set_auto_switch(auto: AutoSwitch) -> Result<AutoSwitch, String> {
    hm220::set_auto_switch(auto).map_err(|e| e.to_string())?;
    Ok(hm220::auto_switch())
}

#[tauri::command]
async fn running_apps() -> Result<Vec<apps::AppEntry>, String> {
    tauri::async_runtime::spawn_blocking(apps::running_apps).await.map_err(|e| e.to_string())
}

/// Dosya seçiciyle seçilen program: ad + simge.
#[tauri::command]
async fn describe_app(path: String) -> Result<apps::AppEntry, String> {
    tauri::async_runtime::spawn_blocking(move || apps::AppEntry {
        name: apps::display_name(Path::new(&path)),
        icon: apps::app_icon(&path),
        path,
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn app_icon(path: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || apps::app_icon(&path)).await.map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemUsage {
    cpu: f32,
    mem_used: u64,
    mem_total: u64,
}

#[tauri::command]
fn system_usage(state: State<'_, AppState>) -> SystemUsage {
    let mut guard = lock(&state.sys);
    let sys = guard.get_or_insert_with(sysinfo::System::new);
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    SystemUsage { cpu: sys.global_cpu_usage(), mem_used: sys.used_memory(), mem_total: sys.total_memory() }
}

// ── Otomatik profil geçişi ──────────────────────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AutoApplied {
    profile: String,
    /// Profili tetikleyen program; varsayılana dönüldüyse None.
    app: Option<String>,
    settings: Option<Settings>,
    error: Option<String>,
}

/// Bağlı program öne gelince profilini uygular; başka bir programa geçilince varsayılan profile döner.
/// Öndeki pencere bilinemiyorsa (saf Wayland) bağlı programın çalışıyor olması yeterli sayılır.
fn spawn_auto_switcher(app: AppHandle, write_lock: Arc<Mutex<()>>, log: Arc<Mutex<VecDeque<LogEntry>>>) {
    let own = std::env::current_exe().ok().map(|p| apps::normalize(&p.to_string_lossy()));
    thread::spawn(move || {
        // Otomatik uygulanan bağlı profil; elle yapılan ayarlara yalnızca bundan dönerken dokunulur.
        let mut active: Option<String> = None;
        loop {
            thread::sleep(Duration::from_millis(1000));
            let auto = hm220::auto_switch();
            if !auto.enabled {
                active = None;
                continue;
            }
            let links = hm220::linked_profiles();
            if links.is_empty() && active.is_none() {
                continue;
            }
            let matched = match apps::foreground_exe() {
                Some(fg) => {
                    let fg = apps::normalize(&fg.to_string_lossy());
                    // Hawk Hub öndeyken (ayar yapılırken) hiçbir şey değiştirme.
                    if own.as_deref() == Some(fg.as_str()) {
                        continue;
                    }
                    links.into_iter().find(|(_, l)| apps::normalize(&l.path) == fg)
                }
                None => {
                    let running = apps::running_paths();
                    links.into_iter().find(|(_, l)| running.contains(&apps::normalize(&l.path)))
                }
            };
            let (target, app_name) = match matched {
                Some((name, _)) if active.as_deref() == Some(name.as_str()) => continue,
                Some((name, link)) => {
                    active = Some(name.clone());
                    (name, Some(link.name))
                }
                None if active.is_some() => {
                    active = None;
                    match auto.default_profile {
                        Some(d) => (d, None),
                        None => continue,
                    }
                }
                None => continue,
            };
            let result =
                hm220::load_profile(&target).and_then(|cfg| run_blocking(&write_lock, &log, move |s| s.apply_config(cfg)));
            let (settings, error) = match result {
                Ok(s) => (Some(s), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let _ = app.emit("profile-auto", AutoApplied { profile: target, app: app_name, settings, error });
        }
    });
}

// ── Cihaz izleyici ──────────────────────────────────────────────────────────

/// Cihazı takar/çıkarır ve pil raporlarını dinler; değişince `device-status` olayı yayar.
fn spawn_device_watcher(app: AppHandle, status: Arc<Mutex<DeviceStatus>>, write_lock: Arc<Mutex<()>>) {
    let events = app.clone();
    // Farenin DPI tuşu: kayıtlı ayarlardaki aktif kademeyi güncelle ve arayüze bildir.
    let on_dpi_stage = move |stage: u8| {
        {
            let _guard = lock(&write_lock);
            let mut cfg = Config::load();
            if cfg.set_dpi_stage(stage).is_ok() {
                let _ = cfg.save();
            }
        }
        let _ = events.emit("dpi-stage", stage);
    };
    let profile_events = app.clone();
    let publish = move |next: DeviceStatus| {
        let mut cur = lock(&status);
        if *cur != next {
            *cur = next.clone();
            drop(cur);
            let _ = app.emit("device-status", next);
        }
    };
    thread::spawn(move || loop {
        let Some(info) = find_devices().into_iter().next() else {
            publish(DeviceStatus::default());
            thread::sleep(Duration::from_secs(2));
            continue;
        };
        let view = DeviceView::from(&info);
        let dev = match Device::open(info) {
            Ok(d) => d,
            Err(_) => {
                // Bulundu ama açılamadı (ör. izin yok): bağlı göster, pil bilinmiyor.
                publish(DeviceStatus { device: Some(view), battery: None });
                thread::sleep(Duration::from_secs(3));
                continue;
            }
        };
        let mut battery = None;
        publish(DeviceStatus { device: Some(view.clone()), battery });
        loop {
            match dev.read_event(Duration::from_secs(5)) {
                Ok(Some(Event::Battery(b))) => {
                    battery = Some(b);
                    publish(DeviceStatus { device: Some(view.clone()), battery });
                }
                Ok(Some(Event::DpiStage(stage))) => on_dpi_stage(stage),
                // Profil değişince tüm ayarlar değişmiş olabilir: arayüz fareden yeniden okur.
                Ok(Some(Event::Profile(_))) => {
                    let _ = profile_events.emit("profile-changed", ());
                }
                Ok(Some(Event::Ack { .. })) => {}
                Ok(None) if find_devices().iter().any(|d| d.path == dev.info.path) => {}
                _ => break,
            }
        }
        publish(DeviceStatus::default());
        thread::sleep(Duration::from_secs(1));
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            let state = app.state::<AppState>();
            spawn_device_watcher(app.handle().clone(), state.status.clone(), state.write_lock.clone());
            spawn_auto_switcher(app.handle().clone(), state.write_lock.clone(), state.log.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            status,
            catalog,
            frames,
            report_log,
            set_dpi_stage,
            set_dpi_value,
            set_dpi_color,
            sync_device,
            set_angle_snap,
            set_ripple,
            set_polling,
            set_led,
            set_response,
            set_button,
            upload_macro,
            delete_macro,
            apply_all,
            reset,
            list_profiles,
            save_profile,
            delete_profile,
            export_profile,
            import_profile,
            apply_profile,
            set_profile_meta,
            get_auto_switch,
            set_auto_switch,
            running_apps,
            describe_app,
            app_icon,
            system_usage
        ])
        .run(tauri::generate_context!())
        .expect("Hawk Hub başlatılamadı");
}
