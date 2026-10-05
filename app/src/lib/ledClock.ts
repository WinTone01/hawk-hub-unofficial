// Tekerlek ışığı efektinin ortak saati: 3D model ve uygulamanın ortam ışığı aynı fonksiyonu,
// aynı zaman kaynağıyla (performance.now) kullanır; böylece nefes ve neon birebir eş zamanlı akar.
import type { LedMode } from "./types";

export interface LedLook {
  ledMode: LedMode;
  ledColor: string; // ekranda görünen renk (#rrggbb)
  brightness: number; // 1–8
  speed: number; // 0 (hızlı) – 5 (yavaş)
}

/** Efekt periyodu (sn). Farenin hız baytı 0 en hızlı. */
export const ledPeriod = (speed: number) => 0.9 + Math.min(5, Math.max(0, speed)) * 0.55;

/** `t` anındaki renk ve yoğunluk (0–1). */
export function ledFrame(look: LedLook, t = performance.now() / 1000): { color: string; intensity: number } {
  const level = 0.35 + (0.65 * (Math.min(8, Math.max(1, look.brightness)) - 1)) / 7;
  const period = ledPeriod(look.speed);
  switch (look.ledMode) {
    case "off":
      return { color: look.ledColor, intensity: 0 };
    case "breathing":
      return { color: look.ledColor, intensity: level * (0.08 + 0.92 * (0.5 - 0.5 * Math.cos((t / period) * Math.PI * 2))) };
    case "neon": {
      const hue = Math.round((((t / (period * 3)) % 1) + 1) % 1 * 360);
      return { color: `hsl(${hue}, 100%, 50%)`, intensity: level };
    }
    default:
      return { color: look.ledColor, intensity: level };
  }
}
