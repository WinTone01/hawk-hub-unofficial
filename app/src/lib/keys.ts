// KeyboardEvent.code ↔ HID klavye usage kodu (HM220 makroları 4–164 aralığını kabul eder).

const map: Record<string, number> = {};
for (let i = 0; i < 26; i++) map["Key" + String.fromCharCode(65 + i)] = 4 + i;
for (let i = 1; i <= 9; i++) map["Digit" + i] = 29 + i;
map.Digit0 = 39;
for (let i = 1; i <= 12; i++) map["F" + i] = 57 + i;
for (let i = 1; i <= 9; i++) map["Numpad" + i] = 88 + i;
Object.assign(map, {
  Enter: 40, Escape: 41, Backspace: 42, Tab: 43, Space: 44, Minus: 45, Equal: 46, BracketLeft: 47,
  BracketRight: 48, Backslash: 49, Semicolon: 51, Quote: 52, Backquote: 53, Comma: 54, Period: 55,
  Slash: 56, CapsLock: 57, PrintScreen: 70, ScrollLock: 71, Pause: 72, Insert: 73, Home: 74,
  PageUp: 75, Delete: 76, End: 77, PageDown: 78, ArrowRight: 79, ArrowLeft: 80, ArrowDown: 81,
  ArrowUp: 82, NumLock: 83, NumpadDivide: 84, NumpadMultiply: 85, NumpadSubtract: 86, NumpadAdd: 87,
  NumpadEnter: 88, Numpad0: 98, NumpadDecimal: 99, IntlBackslash: 100,
});

export const KEYMAP: Readonly<Record<string, number>> = map;

const pretty: Record<string, string> = {
  Enter: "Enter", Escape: "Esc", Backspace: "⌫", Tab: "Tab", Space: "Boşluk", Minus: "-", Equal: "=",
  BracketLeft: "[", BracketRight: "]", Backslash: "\\", Semicolon: ";", Quote: "'", Backquote: "`",
  Comma: ",", Period: ".", Slash: "/", CapsLock: "Caps", PrintScreen: "PrtSc", ScrollLock: "ScrLk",
  Pause: "Pause", Insert: "Ins", Home: "Home", PageUp: "PgUp", Delete: "Del", End: "End", PageDown: "PgDn",
  ArrowRight: "→", ArrowLeft: "←", ArrowDown: "↓", ArrowUp: "↑", NumLock: "NumLk", NumpadDivide: "Num /",
  NumpadMultiply: "Num *", NumpadSubtract: "Num -", NumpadAdd: "Num +", NumpadEnter: "Num Enter",
  NumpadDecimal: "Num .", IntlBackslash: "<",
};

const labels: Record<number, string> = {};
for (const [code, usage] of Object.entries(map)) {
  labels[usage] = pretty[code] ?? code.replace(/^Key|^Digit/, "").replace(/^Numpad(\d)$/, "Num $1");
}

export function keyLabel(usage: number): string {
  return labels[usage] ?? `0x${usage.toString(16).padStart(2, "0")}`;
}

/** Manuel ekleme menüsü için tüm tuşlar (usage sırasıyla). */
export const ALL_KEYS = Object.values(map)
  .sort((a, b) => a - b)
  .map((usage) => ({ usage, label: keyLabel(usage) }));
