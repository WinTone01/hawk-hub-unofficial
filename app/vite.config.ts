import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

import pkg from "./package.json" with { type: "json" };

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  // Three.js tek parça ~700 KB; masaüstü uygulamasında yerel diskten yüklenir.
  build: { target: "es2022", outDir: "dist", emptyOutDir: true, chunkSizeWarningLimit: 1200 },
});
