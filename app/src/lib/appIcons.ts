// Program simgeleri (data: URL) yol başına bir kez istenir.
import { api } from "./api";
import type { AppEntry, Profile } from "./types";

const cache = new Map<string, Promise<string | null>>();

export function appIcon(path: string): Promise<string | null> {
  let p = cache.get(path);
  if (!p) {
    p = api.appIcon(path).catch(() => null);
    cache.set(path, p);
  }
  return p;
}

/** Listeden gelen simgeleri önbelleğe koyar (tekrar istenmesin). */
export function remember(apps: AppEntry[]) {
  for (const a of apps) if (!cache.has(a.path)) cache.set(a.path, Promise.resolve(a.icon));
}

/** Profilin gösterilecek simgesi: açıkça seçilen glif, yoksa bağlı programın simgesi, yoksa varsayılan. */
export function profileGlyph(p: Profile): { app: string } | { glyph: string } {
  const icon = p.meta.icon;
  if (p.meta.app && (icon === "app" || icon == null)) return { app: p.meta.app.path };
  return { glyph: icon && icon !== "app" ? icon : "profiles" };
}

/** Profil renkleri (kendi rengi seçilmemişse profilin LED rengi kullanılır). */
export const PROFILE_COLORS = [
  "#ff4d5e", "#ff7a45", "#ffb020", "#f5e050", "#5ee38a", "#2fd4c0",
  "#3fa9ff", "#5a6cff", "#9b5cff", "#e05cff", "#ff5ca8", "#8a93a6",
];

export const PROFILE_GLYPHS = [
  "crosshair", "gamepad", "sword", "shield", "trophy", "flame", "rocket", "target",
  "star", "heart", "bolt", "brush", "film", "music", "code", "briefcase", "globe", "keyboard",
];

export function profileColor(p: Profile): string {
  if (p.meta.color) return p.meta.color;
  return p.settings.ledMode === "off" ? "#5b6170" : p.settings.ledColor;
}
