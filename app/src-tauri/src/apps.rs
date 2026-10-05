//! Programlar: öndeki pencerenin programı, çalışan programlar ve program simgeleri.
//! Windows'ta Win32 API, Linux'ta X11 (xprop) + /proc + .desktop/simge teması kullanılır.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    /// data: URL (PNG/SVG); bulunamazsa None.
    pub icon: Option<String>,
}

/// Karşılaştırma için yol: Windows'ta küçük harf.
pub fn normalize(path: &str) -> String {
    if cfg!(windows) {
        path.to_lowercase()
    } else {
        path.to_string()
    }
}

/// Çalışan programlar (yol başına bir kez, sistem süreçleri hariç), simgeleriyle.
pub fn running_apps() -> Vec<AppEntry> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing().with_exe(UpdateKind::Always));
    let mut seen = BTreeMap::new();
    for p in sys.processes().values() {
        let Some(exe) = p.exe() else { continue };
        let path = exe.to_string_lossy().into_owned();
        if is_system(&path) {
            continue;
        }
        seen.entry(normalize(&path)).or_insert(path);
    }
    let mut apps: Vec<AppEntry> = seen
        .into_values()
        .map(|path| AppEntry { name: display_name(Path::new(&path)), icon: app_icon(&path), path })
        .collect();
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

/// Bu programlardan biri çalışıyor mu (öndeki pencere bilinemezse yedek).
pub fn running_paths() -> Vec<String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing().with_exe(UpdateKind::Always));
    sys.processes().values().filter_map(|p| p.exe().map(|e| normalize(&e.to_string_lossy()))).collect()
}

pub fn display_name(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn is_system(path: &str) -> bool {
    let p = path.to_lowercase();
    if cfg!(windows) {
        p.starts_with("c:\\windows\\") || p.contains("\\windowsapps\\microsoft.") || p.ends_with("\\hawk-hub-unofficial.exe")
    } else {
        p.starts_with("/usr/lib/") || p.starts_with("/usr/libexec/") || p.starts_with("/lib/") || p.starts_with("/usr/sbin/")
    }
}

fn data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes))
}

// ── Windows ─────────────────────────────────────────────────────────────────

#[cfg(windows)]
mod imp {
    use super::*;
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, GetForegroundWindow, GetIconInfo, GetWindowThreadProcessId, PrivateExtractIconsW, HICON, ICONINFO,
    };

    pub fn foreground_exe() -> Option<PathBuf> {
        // SAFETY: Win32 çağrıları; tüm tutamaçlar kontrol edilip kapatılır.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return None;
            }
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return None;
            }
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len);
            CloseHandle(h);
            (ok != 0).then(|| PathBuf::from(String::from_utf16_lossy(&buf[..len as usize])))
        }
    }

    /// Programın kendi simgesi (64×64 PNG).
    pub fn app_icon(path: &str) -> Option<String> {
        let wide: Vec<u16> = std::ffi::OsStr::new(path).encode_wide().chain([0]).collect();
        // SAFETY: GDI nesneleri oluşturulduğu sırayla serbest bırakılır.
        unsafe {
            let mut icon: HICON = null_mut();
            let n = PrivateExtractIconsW(wide.as_ptr(), 0, 64, 64, &mut icon, null_mut(), 1, 0);
            if n == 0 || n == u32::MAX || icon.is_null() {
                return None;
            }
            let mut info: ICONINFO = std::mem::zeroed();
            if GetIconInfo(icon, &mut info) == 0 {
                DestroyIcon(icon);
                return None;
            }
            let mut bm: BITMAP = std::mem::zeroed();
            GetObjectW(info.hbmColor as _, size_of::<BITMAP>() as i32, &mut bm as *mut _ as *mut c_void);
            let (w, h) = (bm.bmWidth, bm.bmHeight);
            let mut px = vec![0u8; (w * h * 4).max(0) as usize];
            let mut bi: BITMAPINFO = std::mem::zeroed();
            bi.bmiHeader = BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..std::mem::zeroed()
            };
            let dc = CreateCompatibleDC(null_mut());
            let rows = GetDIBits(dc, info.hbmColor, 0, h as u32, px.as_mut_ptr() as *mut c_void, &mut bi, DIB_RGB_COLORS);
            DeleteDC(dc);
            DeleteObject(info.hbmColor as _);
            DeleteObject(info.hbmMask as _);
            DestroyIcon(icon);
            if rows == 0 || w <= 0 || h <= 0 {
                return None;
            }
            // BGRA → RGBA; eski simgelerde alfa kanalı boş olabilir.
            let has_alpha = px.chunks(4).any(|p| p[3] != 0);
            for p in px.chunks_mut(4) {
                p.swap(0, 2);
                if !has_alpha {
                    p[3] = 255;
                }
            }
            let mut out = Vec::new();
            {
                let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                let mut wr = enc.write_header().ok()?;
                wr.write_image_data(&px).ok()?;
            }
            Some(data_url("image/png", &out))
        }
    }
}

// ── Linux ───────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod imp {
    use super::*;
    use std::process::Command;

    /// X11 (XWayland dahil) üzerinde öndeki pencerenin programı. Saf Wayland'de None döner;
    /// o durumda otomatik geçiş "program çalışıyor mu" yedeğine düşer.
    pub fn foreground_exe() -> Option<PathBuf> {
        let out = Command::new("xprop").args(["-root", "_NET_ACTIVE_WINDOW"]).output().ok()?;
        let s = String::from_utf8_lossy(&out.stdout);
        let id = s.split_whitespace().last()?.trim_end_matches(',').to_string();
        if !id.starts_with("0x") || id == "0x0" {
            return None;
        }
        let out = Command::new("xprop").args(["-id", &id, "_NET_WM_PID"]).output().ok()?;
        let pid: u32 = String::from_utf8_lossy(&out.stdout).split_whitespace().last()?.parse().ok()?;
        std::fs::read_link(format!("/proc/{pid}/exe")).ok()
    }

    fn desktop_dirs() -> Vec<PathBuf> {
        let mut dirs = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        ];
        if let Some(home) = std::env::var_os("HOME") {
            let h = PathBuf::from(home);
            dirs.push(h.join(".local/share/applications"));
            dirs.push(h.join(".local/share/flatpak/exports/share/applications"));
        }
        dirs
    }

    /// .desktop dosyalarında programın simge adını bulur (Exec/TryExec dosya adı eşleşmesi).
    fn icon_name(exe: &Path) -> Option<String> {
        let base = exe.file_name()?.to_string_lossy().into_owned();
        for dir in desktop_dirs() {
            let Ok(entries) = std::fs::read_dir(dir) else { continue };
            for e in entries.flatten() {
                let Ok(text) = std::fs::read_to_string(e.path()) else { continue };
                let mut icon = None;
                let mut hit = false;
                for line in text.lines() {
                    if let Some(v) = line.strip_prefix("Icon=") {
                        icon.get_or_insert_with(|| v.trim().to_string());
                    }
                    let cmd = line.strip_prefix("Exec=").or_else(|| line.strip_prefix("TryExec="));
                    if let Some(c) = cmd {
                        let first = c.split_whitespace().next().unwrap_or("").trim_matches('"');
                        if Path::new(first).file_name().is_some_and(|f| f.to_string_lossy() == base) {
                            hit = true;
                        }
                    }
                }
                if hit {
                    return icon;
                }
            }
        }
        None
    }

    fn resolve_icon(name: &str) -> Option<PathBuf> {
        let p = Path::new(name);
        if p.is_absolute() && p.exists() {
            return Some(p.to_path_buf());
        }
        let mut roots = vec![PathBuf::from("/usr/share/icons/hicolor"), PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor")];
        if let Some(home) = std::env::var_os("HOME") {
            roots.insert(0, PathBuf::from(home).join(".local/share/icons/hicolor"));
        }
        for root in &roots {
            for size in ["256x256", "128x128", "96x96", "64x64", "48x48", "scalable"] {
                for ext in ["png", "svg"] {
                    let f = root.join(size).join("apps").join(format!("{name}.{ext}"));
                    if f.exists() {
                        return Some(f);
                    }
                }
            }
        }
        ["png", "svg"].iter().map(|e| PathBuf::from(format!("/usr/share/pixmaps/{name}.{e}"))).find(|f| f.exists())
    }

    pub fn app_icon(path: &str) -> Option<String> {
        let file = resolve_icon(&icon_name(Path::new(path))?)?;
        let bytes = std::fs::read(&file).ok()?;
        let mime = if file.extension().is_some_and(|e| e == "svg") { "image/svg+xml" } else { "image/png" };
        Some(data_url(mime, &bytes))
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod imp {
    use super::*;
    pub fn foreground_exe() -> Option<PathBuf> {
        None
    }
    pub fn app_icon(_path: &str) -> Option<String> {
        None
    }
}

pub use imp::{app_icon, foreground_exe};

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn windows_icon_and_apps() {
        let icon = app_icon(r"C:\Windows\explorer.exe").expect("explorer simgesi");
        assert!(icon.starts_with("data:image/png;base64,"));
        let apps = running_apps();
        assert!(!apps.is_empty());
        let with_icon = apps.iter().filter(|a| a.icon.is_some()).count();
        println!("{} program, {} simgeli; ön: {:?}", apps.len(), with_icon, foreground_exe());
    }
}
