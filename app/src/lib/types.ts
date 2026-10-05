export type LedMode = "off" | "neon" | "static" | "breathing";
export type ButtonId = "left" | "right" | "middle" | "back" | "forward";

export interface MacroStep {
  key: number;
  delay: number;
}

export interface Macro {
  id: number;
  name: string;
  repeat: number;
  mode: number;
  steps: MacroStep[];
}

export interface ButtonSetting {
  button: ButtonId;
  code: number;
  macroId: number | null;
}

export interface Settings {
  dpiStage: number;
  dpiValues: number[];
  /** Kademe başına DPI gösterge rengi (#rrggbb). */
  dpiColors: string[];
  angleSnap: boolean;
  ripple: boolean;
  pollingHz: number;
  ledMode: LedMode;
  ledColor: string;
  ledBrightness: number;
  ledSpeed: number;
  responseMs: number;
  buttons: ButtonSetting[];
  macros: Macro[];
}

export interface Battery {
  level: number | null;
  charging: boolean;
}

export interface DeviceView {
  path: string;
  pid: number;
  transport: string;
}

export interface DeviceStatus {
  device: DeviceView | null;
  battery: Battery | null;
}

export interface Status extends DeviceStatus {
  settings: Settings;
  configDir: string;
}

export interface FunctionDef {
  code: number;
  id: string;
  label: string;
  group: string;
}

export interface Catalog {
  functions: FunctionDef[];
  pollingRates: number[];
  responseSteps: number[];
}

export interface AppLink {
  path: string;
  name: string;
}

export interface ProfileMeta {
  /** #rrggbb */
  color: string | null;
  /** Simge setinden bir ad; "app" = bağlı programın kendi simgesi. */
  icon: string | null;
  app: AppLink | null;
}

export interface Profile {
  name: string;
  modified: number;
  settings: Settings;
  meta: ProfileMeta;
}

export interface AutoSwitch {
  enabled: boolean;
  defaultProfile: string | null;
}

export interface AppEntry {
  name: string;
  path: string;
  /** data: URL */
  icon: string | null;
}

export interface AutoApplied {
  profile: string;
  app: string | null;
  settings: Settings | null;
  error: string | null;
}

export interface Frame {
  report: number;
  name: string;
  bytes: number[];
  /** Checksum baytlarının indeksleri (vurgulamak için). */
  checksum: number[];
}

export interface LogEntry {
  /** ms, Unix zamanı */
  at: number;
  report: number;
  name: string;
  bytes: number[];
  ok: boolean;
  error: string | null;
}

export interface SystemUsage {
  /** % */
  cpu: number;
  /** bayt */
  memUsed: number;
  memTotal: number;
}
