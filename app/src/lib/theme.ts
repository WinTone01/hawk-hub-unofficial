// Koyu / açık tema: <html data-theme>. Seçim tarayıcı deposunda saklanır.
export type Theme = "dark" | "light";
const KEY = "hawk.theme";

export function initTheme(): Theme {
  let t: Theme = "dark";
  try {
    if (localStorage.getItem(KEY) === "light") t = "light";
  } catch {}
  document.documentElement.dataset.theme = t;
  return t;
}

export function setTheme(t: Theme) {
  document.documentElement.dataset.theme = t;
  try {
    localStorage.setItem(KEY, t);
  } catch {}
}
