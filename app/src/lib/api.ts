// Tauri komutları. Tauri dışında (tarayıcıda `npm run dev`) sahte bir arka uç kullanılır.
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppEntry,
  AutoApplied,
  AutoSwitch,
  ButtonId,
  Catalog,
  DeviceStatus,
  Frame,
  LogEntry,
  Macro,
  Profile,
  ProfileMeta,
  Settings,
  Status,
  SystemUsage,
} from "./types";

export const isTauri = "__TAURI_INTERNALS__" in window;

type Args = Record<string, unknown>;
const call = isTauri ? tauriInvoke : mockInvoke;

export const api = {
  catalog: () => call<Catalog>("catalog"),
  status: () => call<Status>("status"),
  setDpiStage: (stage: number) => call<Settings>("set_dpi_stage", { stage }),
  setDpiValue: (stage: number, dpi: number) => call<Settings>("set_dpi_value", { stage, dpi }),
  setDpiColor: (stage: number, color: string) => call<Settings>("set_dpi_color", { stage, color }),
  syncDevice: () => call<Settings>("sync_device"),
  setAngleSnap: (on: boolean) => call<Settings>("set_angle_snap", { on }),
  setRipple: (on: boolean) => call<Settings>("set_ripple", { on }),
  setPolling: (hz: number) => call<Settings>("set_polling", { hz }),
  setResponse: (ms: number) => call<Settings>("set_response", { ms }),
  setLed: (p: { mode?: string; color?: string; brightness?: number; speed?: number }) => call<Settings>("set_led", p),
  setButton: (button: ButtonId, code: number) => call<Settings>("set_button", { button, code }),
  uploadMacro: (button: ButtonId, definition: Macro) => call<Settings>("upload_macro", { button, definition }),
  deleteMacro: (id: number) => call<Settings>("delete_macro", { id }),
  applyAll: () => call<Settings>("apply_all"),
  reset: () => call<Settings>("reset"),
  listProfiles: () => call<Profile[]>("list_profiles"),
  saveProfile: (name: string) => call<Profile[]>("save_profile", { name }),
  applyProfile: (name: string) => call<Settings>("apply_profile", { name }),
  deleteProfile: (name: string) => call<Profile[]>("delete_profile", { name }),
  exportProfile: (name: string) => call<string>("export_profile", { name }),
  importProfile: (name: string, json: string) => call<Profile[]>("import_profile", { name, json }),
  setProfileMeta: (name: string, meta: ProfileMeta) => call<Profile[]>("set_profile_meta", { name, meta }),
  getAutoSwitch: () => call<AutoSwitch>("get_auto_switch"),
  setAutoSwitch: (auto: AutoSwitch) => call<AutoSwitch>("set_auto_switch", { auto }),
  runningApps: () => call<AppEntry[]>("running_apps"),
  describeApp: (path: string) => call<AppEntry>("describe_app", { path }),
  appIcon: (path: string) => call<string | null>("app_icon", { path }),
  systemUsage: () => call<SystemUsage>("system_usage"),
  frames: () => call<Frame[]>("frames"),
  reportLog: () => call<LogEntry[]>("report_log"),
};

/** Cihaz takıldığında/çıkarıldığında ve pil raporu geldiğinde çağrılır. */
export async function onDeviceStatus(cb: (s: DeviceStatus) => void): Promise<() => void> {
  if (isTauri) return listen<DeviceStatus>("device-status", (e) => cb(e.payload));
  const t = setInterval(() => cb(mock.deviceStatus()), 4000);
  return () => clearInterval(t);
}

// ── Sahte arka uç ────────────────────────────────────────────────────────────

const mock = (() => {
  const functions = [
    [1, "disabled", "Tuş Kapalı", "Özel"], [8, "fire", "Ateş Tuşu", "Özel"],
    [2, "left", "Sol Tık", "Temel"], [3, "right", "Sağ Tık", "Temel"], [4, "middle", "Orta Tık", "Temel"],
    [5, "back", "Geri", "Temel"], [6, "forward", "İleri", "Temel"], [7, "double-click", "Çift Tık", "Temel"],
    [13, "dpi-cycle", "DPI Döngüsü", "DPI"], [14, "dpi-up", "DPI +", "DPI"], [15, "dpi-down", "DPI −", "DPI"],
    [9, "scroll-up", "Yukarı Kaydır", "Kaydırma"], [10, "scroll-down", "Aşağı Kaydır", "Kaydırma"],
    [11, "scroll-left", "Sola Kaydır", "Kaydırma"], [12, "scroll-right", "Sağa Kaydır", "Kaydırma"],
    [21, "media-player", "Medya Oynatıcı", "Medya"], [22, "prev-track", "Önceki Parça", "Medya"],
    [23, "next-track", "Sonraki Parça", "Medya"], [24, "play-pause", "Oynat/Duraklat", "Medya"],
    [25, "stop", "Durdur", "Medya"], [26, "mute", "Sessiz", "Medya"], [27, "volume-up", "Ses +", "Medya"],
    [28, "volume-down", "Ses −", "Medya"], [29, "calculator", "Hesap Makinesi", "Uygulama"],
    [30, "email", "E-posta", "Uygulama"], [36, "browser-refresh", "Tarayıcı: Yenile", "Uygulama"],
    [37, "browser-home", "Tarayıcı: Ana Sayfa", "Uygulama"],
  ].map(([code, id, label, group]) => ({ code, id, label, group })) as Catalog["functions"];

  const defaults = (): Settings => ({
    dpiStage: 2, dpiValues: [400, 800, 1600, 3200, 4800], angleSnap: false, ripple: true, pollingHz: 1000,
    ledMode: "breathing", ledColor: "#00e60f", ledBrightness: 8, ledSpeed: 3, responseMs: 0,
    dpiColors: ["#ff0000", "#0000ff", "#00ff00", "#ff00ff", "#ffff40"],
    buttons: (["left", "right", "middle", "back", "forward"] as ButtonId[]).map((button, i) => ({ button, code: i + 2, macroId: null })),
    macros: [],
  });
  let s = defaults();
  const profiles = new Map<string, Settings>([["FPS", { ...defaults(), dpiStage: 1, ledMode: "static", ledColor: "#00e5ff" }]]);
  const log: LogEntry[] = [];
  let level = 76;
  const names: Record<number, string> = { 4: "DPI", 5: "LED", 6: "Polling", 8: "Tuş atama", 9: "Makro" };
  const write = (report: number, n: number) => {
    log.unshift({ at: Date.now(), report, name: names[report], bytes: Array.from({ length: n }, (_, i) => (i * 37 + report * 11) & 255), ok: true, error: null });
    log.length = Math.min(log.length, 200);
  };
  const metas = new Map<string, ProfileMeta>([["FPS", { color: "#ff4d5e", icon: "crosshair", app: null }]]);
  let auto: AutoSwitch = { enabled: false, defaultProfile: null };
  const apps: AppEntry[] = ["Valorant", "cs2", "Photoshop", "Discord", "firefox", "steam"].map((name) => ({
    name,
    path: `C:\\Games\\${name}\\${name}.exe`,
    icon: null,
  }));
  const profile = (name: string): Profile => ({
    name,
    modified: Date.now() / 1000,
    settings: structuredClone(profiles.get(name)!),
    meta: structuredClone(metas.get(name) ?? { color: null, icon: null, app: null }),
  });

  const handlers: Record<string, (a: any) => unknown> = {
    catalog: () => ({ functions, pollingRates: [125, 250, 500, 1000], responseSteps: [0, 2, 4, 8, 12] }),
    status: () => ({ ...mock.deviceStatus(), settings: s, configDir: "~/.config/hawk-hub" }),
    set_dpi_stage: (a) => { s.dpiStage = a.stage; write(4, 51); },
    set_dpi_value: (a) => { s.dpiValues[a.stage - 1] = a.dpi; write(4, 51); },
    set_dpi_color: (a) => { s.dpiColors[a.stage - 1] = a.color; write(4, 51); },
    sync_device: () => {},
    set_angle_snap: (a) => { s.angleSnap = a.on; write(4, 51); },
    set_ripple: (a) => { s.ripple = a.on; write(4, 51); },
    set_polling: (a) => { s.pollingHz = a.hz; write(6, 8); },
    set_response: (a) => { s.responseMs = a.ms; write(5, 12); },
    set_led: (a) => {
      if (a.mode != null) s.ledMode = a.mode;
      if (a.color != null) {
        s.ledColor = a.color;
        s.dpiColors = s.dpiColors.map(() => a.color);
      }
      if (a.brightness != null) s.ledBrightness = a.brightness;
      if (a.speed != null) s.ledSpeed = a.speed;
      write(5, 12);
    },
    set_button: (a) => { Object.assign(s.buttons.find((b) => b.button === a.button)!, { code: a.code, macroId: null }); write(8, 58); },
    upload_macro: (a) => {
      s.macros = s.macros.filter((m) => m.id !== a.definition.id).concat(a.definition).sort((x, y) => x.id - y.id);
      Object.assign(s.buttons.find((b) => b.button === a.button)!, { code: 18, macroId: a.definition.id });
      [0, 1, 2].forEach(() => write(9, 63));
      write(8, 58);
    },
    delete_macro: (a) => {
      s.macros = s.macros.filter((m) => m.id !== a.id);
      s.buttons.forEach((b, i) => { if (b.macroId === a.id) Object.assign(b, { code: i + 2, macroId: null }); });
      write(8, 58);
    },
    apply_all: () => [4, 5, 6, 8].forEach((r) => write(r, 12)),
    reset: () => { s = defaults(); [4, 5, 6, 8].forEach((r) => write(r, 12)); },
    list_profiles: () => [...profiles.keys()].map(profile),
    save_profile: (a) => { profiles.set(a.name, structuredClone(s)); return handlers.list_profiles(a); },
    apply_profile: (a) => { s = structuredClone(profiles.get(a.name)!); [4, 5, 6, 8].forEach((r) => write(r, 12)); },
    delete_profile: (a) => {
      profiles.delete(a.name);
      metas.delete(a.name);
      if (auto.defaultProfile === a.name) auto.defaultProfile = null;
      return handlers.list_profiles(a);
    },
    set_profile_meta: (a) => { metas.set(a.name, a.meta); return handlers.list_profiles(a); },
    get_auto_switch: () => auto,
    set_auto_switch: (a) => (auto = a.auto),
    running_apps: () => apps,
    describe_app: (a) => ({ name: a.path.split(/[\\/]/).pop()!.replace(/\.exe$/i, ""), path: a.path, icon: null }),
    app_icon: () => null,
    system_usage: () => ({ cpu: 8 + Math.random() * 14, memUsed: 16.6e9, memTotal: 31.9e9 }),
    export_profile: (a) => JSON.stringify(profiles.get(a.name), null, 2),
    import_profile: (a) => { profiles.set(a.name, JSON.parse(a.json)); return handlers.list_profiles(a); },
    frames: () => [
      { report: 4, name: "DPI / sensör", bytes: [56, 1, +s.angleSnap, +s.ripple, 31, 0, 0, 9, 18, 37, 75, 112, ...Array(11).fill(0), s.dpiStage, 255, 0, 0, 0, 0, 255, 0, 255, 0, 255, 0, 255, 255, 64, 0, 0, 255, 255, 255, 64, 0, 255, 255, 255, 3, 13, 147], checksum: [49, 50] },
      { report: 5, name: "LED + tepki", bytes: [15, 1, 2, s.ledSpeed, s.ledBrightness, 255, 0, 0, 60, s.responseMs / 2, 1, 236], checksum: [11] },
      { report: 6, name: "Polling", bytes: [9, 1, 1000 / s.pollingHz, 255 - 1000 / s.pollingHz, 0, 0, 0, 0], checksum: [] },
      { report: 8, name: "Tuş atama", bytes: [59, 1, 2, 0, 0, 3, 0, 0, 4, 0, 0, 5, 0, 0, 6, 0, 0, 13, ...Array(38).fill(1), 0, 62], checksum: [56, 57] },
    ],
    report_log: () => log,
  };

  return {
    settings: () => s,
    deviceStatus: (): DeviceStatus => {
      level = level > 5 ? level - (Math.random() < 0.2 ? 1 : 0) : 100;
      return { device: { path: "/dev/hidraw3", pid: 0xfa60, transport: "2.4 GHz alıcı" }, battery: { level, charging: false } };
    },
    handlers,
  };
})();

async function mockInvoke<T>(cmd: string, args: Args = {}): Promise<T> {
  const h = mock.handlers[cmd];
  if (!h) throw new Error(`sahte arka uçta komut yok: ${cmd}`);
  const readOnly = [
    "catalog", "status", "list_profiles", "export_profile", "frames", "report_log",
    "get_auto_switch", "app_icon", "system_usage", "describe_app", "set_profile_meta", "set_auto_switch",
  ].includes(cmd);
  if (!readOnly) await new Promise((r) => setTimeout(r, cmd === "upload_macro" ? 1200 : 180));
  const out = h(args);
  return structuredClone(out === undefined ? mock.settings() : out) as T;
}

/** Farenin DPI tuşuyla aktif kademe değişince çağrılır (1–5). */
export async function onDpiStage(cb: (stage: number) => void): Promise<() => void> {
  if (isTauri) return listen<number>("dpi-stage", (e) => cb(e.payload));
  return () => {};
}

/** Farede aktif profil değişince çağrılır (ayarların tamamı değişmiş olabilir). */
export async function onProfileChanged(cb: () => void): Promise<() => void> {
  if (isTauri) return listen("profile-changed", () => cb());
  return () => {};
}

/** Bağlı bir program öne geldiğinde (veya varsayılana dönülünce) profil otomatik uygulandı. */
export async function onProfileAuto(cb: (e: AutoApplied) => void): Promise<() => void> {
  if (isTauri) return listen<AutoApplied>("profile-auto", (e) => cb(e.payload));
  return () => {};
}

/** Programı dosya seçiciyle seçtirir; vazgeçilirse null. */
export async function pickProgram(): Promise<string | null> {
  if (!isTauri) return "C:\Games\Apex\r5apex.exe";
  const { open } = await import("@tauri-apps/plugin-dialog");
  const windows = navigator.userAgent.includes("Windows");
  const path = await open({
    title: "Program seç",
    multiple: false,
    directory: false,
    filters: windows ? [{ name: "Programlar", extensions: ["exe"] }] : undefined,
  });
  return typeof path === "string" ? path : null;
}
