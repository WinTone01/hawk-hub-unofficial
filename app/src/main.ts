import { mount } from "svelte";
import "@fontsource-variable/inter";
import "@fontsource/jetbrains-mono/500.css";
import "@fontsource/jetbrains-mono/700.css";
// Sora başlık fontu (npm run assets ile Hawk Hub'dan kopyalanır; yoksa Inter'e düşer).
import.meta.glob("./assets/hawk/fonts.css", { eager: true });
import "./app.css";
import { initTheme } from "./lib/theme";
import App from "./App.svelte";

initTheme();

export default mount(App, { target: document.getElementById("app")! });
