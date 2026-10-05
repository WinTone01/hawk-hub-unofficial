// Uygulama durumu: ayarlar, cihaz, bildirimler. Tüm yazmalar `act` üzerinden geçer.
import { api, onDeviceStatus, onDpiStage, onProfileAuto, onProfileChanged } from "./api";
import { VARIANTS, type VariantId } from "./assets";
import { setTheme, type Theme } from "./theme";
import type { Battery, Catalog, DeviceStatus, DeviceView, Settings } from "./types";

/** Üst sekmeler: Cihazlarım (kontrol merkezi veya seçili cihaz), Profiller, Ayarlar. */
export type Page = "devices" | "device" | "profiles" | "settings";
/** Cihaz sayfasının alt sekmeleri. */
export type DeviceTab = "general" | "buttons" | "macros";

export interface ConfirmRequest {
  title: string;
  message: string;
  confirmLabel: string;
  danger: boolean;
  resolve: (ok: boolean) => void;
}

export interface Toast {
  id: number;
  text: string;
  kind: "ok" | "err" | "info";
}

/** Farenin altındaki DPI göstergesinin renkleri: firmware'e gömülü (kitapçık), ayarlanamıyor. */
export const DPI_COLORS = ["#ff0000", "#0000ff", "#00ff00", "#8000ff", "#ffff00"];

export const BUTTON_LABELS: Record<string, string> = {
  left: "Sol Tık",
  right: "Sağ Tık",
  middle: "Orta Tık",
  back: "Geri (yan)",
  forward: "İleri (yan)",
};

export const MACRO_CODE = 18;

const VARIANT_KEY = "hawk.variant";

function loadVariant(): VariantId {
  try {
    const v = localStorage.getItem(VARIANT_KEY);
    if (VARIANTS.some((x) => x.id === v)) return v as VariantId;
  } catch {}
  return "black";
}

class AppState {
  page = $state<Page>("device");
  deviceTab = $state<DeviceTab>("general");
  theme = $state<Theme>(document.documentElement.dataset.theme === "light" ? "light" : "dark");
  /** Farenin rengi (yalnızca görsel; cihaz bunu bildirmez). */
  variant = $state<VariantId>(loadVariant());
  catalog = $state<Catalog | null>(null);
  settings = $state<Settings | null>(null);
  device = $state<DeviceView | null>(null);
  battery = $state<Battery | null>(null);
  configDir = $state("");
  busy = $state(0);
  /** Fareden ayar okunuyor. */
  syncing = $state(false);
  toasts = $state<Toast[]>([]);
  /** Geliştirici panelinin yenilenmesi için her yazmada artar. */
  writes = $state(0);
  /** Son otomatik uygulanan profil (kartta rozet olarak gösterilir). */
  autoProfile = $state<string | null>(null);
  private nextToast = 1;

  async init() {
    const [catalog, status] = await Promise.all([api.catalog(), api.status()]);
    this.catalog = catalog;
    this.settings = status.settings;
    this.configDir = status.configDir;
    this.applyDevice(status);
    await onDeviceStatus((s) => this.applyDevice(s));
    await onDpiStage((stage) => {
      if (!this.settings || this.settings.dpiStage === stage) return;
      this.settings = { ...this.settings, dpiStage: stage };
      this.toast(`DPI kademe ${stage} · ${this.settings.dpiValues[stage - 1]} DPI`, "info");
    });
    await onProfileChanged(() => void this.syncFromDevice());
    await onProfileAuto((e) => {
      if (e.settings) this.settings = e.settings;
      this.writes++;
      if (!e.error) this.autoProfile = e.profile;
      if (e.error) this.toast(`“${e.profile}” otomatik uygulanamadı: ${e.error}`, "err");
      else this.toast(e.app ? `${e.app} öne geldi · “${e.profile}” uygulandı` : `Varsayılan profil “${e.profile}” uygulandı`, "info");
    });
  }

  private applyDevice(s: DeviceStatus) {
    const was = this.device?.path;
    this.device = s.device;
    this.battery = s.battery;
    if (s.device && was !== s.device.path) {
      if (was !== undefined) this.toast(`HM220 bağlandı (${s.device.transport})`, "info");
      void this.syncFromDevice();
    }
    if (!s.device && was) this.toast("HM220 bağlantısı kesildi", "err");
  }

  /** Farenin gerçek ayarlarını okur (fare ayarları kendi hafızasında tutar). */
  async syncFromDevice() {
    this.syncing = true;
    try {
      this.settings = await api.syncDevice();
    } catch (e) {
      this.toast(`Ayarlar fareden okunamadı: ${e}`, "err");
    } finally {
      this.syncing = false;
      this.writes++;
    }
  }

  /** Uygulamanın kendi onay diyaloğu (tarayıcının confirm() penceresi yerine). */
  dialog = $state<ConfirmRequest | null>(null);
  confirm(message: string, opts: Partial<Omit<ConfirmRequest, "message" | "resolve">> = {}): Promise<boolean> {
    this.dialog?.resolve(false);
    return new Promise((resolve) => {
      this.dialog = {
        title: opts.title ?? "Emin misiniz?",
        message,
        confirmLabel: opts.confirmLabel ?? "Onayla",
        danger: opts.danger ?? false,
        resolve: (ok) => {
          this.dialog = null;
          resolve(ok);
        },
      };
    });
  }

  setTheme(t: Theme) {
    this.theme = t;
    setTheme(t);
  }

  /** Cihaz sayfasını belirli bir alt sekmede açar. */
  openDevice(tab: DeviceTab = this.deviceTab) {
    this.page = "device";
    this.deviceTab = tab;
  }

  setVariant(v: VariantId) {
    this.variant = v;
    try {
      localStorage.setItem(VARIANT_KEY, v);
    } catch {}
  }

  toast(text: string, kind: Toast["kind"] = "ok") {
    const id = this.nextToast++;
    this.toasts.push({ id, text, kind });
    setTimeout(() => (this.toasts = this.toasts.filter((t) => t.id !== id)), kind === "err" ? 6000 : 2600);
  }

  /** Ayar değiştiren bir komutu çalıştırır. Hata olursa arayüz kayıtlı ayarlara döner. */
  async act(fn: () => Promise<Settings>, okText?: string): Promise<boolean> {
    this.busy++;
    try {
      this.settings = await fn();
      if (okText) this.toast(okText);
      return true;
    } catch (e) {
      this.toast(String(e), "err");
      this.settings = { ...this.settings! };
      return false;
    } finally {
      this.busy--;
      this.writes++;
    }
  }

  functionLabel(code: number, macroId: number | null): string {
    if (macroId != null) {
      const m = this.settings?.macros.find((x) => x.id === macroId);
      return `Makro ${macroId}${m?.name ? " · " + m.name : ""}`;
    }
    return this.catalog?.functions.find((f) => f.code === code)?.label ?? `Özel (0x${code.toString(16)})`;
  }
}

export const app = new AppState();
