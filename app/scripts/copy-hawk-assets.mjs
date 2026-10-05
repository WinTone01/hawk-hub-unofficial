// Hawk Hub'dan çıkarılan görselleri, logoyu ve Sora fontunu (re/app, bkz. re/extract-asar.ps1) arayüze kopyalar.
// Görseller ve logo Hawk Chair'e aittir; depoya eklenmez (.gitignore). Yoksa arayüz yedeklere düşer.
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, "..", "..", "re", "app", "out", "renderer", "assets");
const dst = join(here, "..", "src", "assets", "hawk");

if (!existsSync(src)) {
  console.log(`Hawk Hub dosyaları bulunamadı (${src}); yedek görseller kullanılacak.`);
  process.exit(0);
}
mkdirSync(join(dst, "fonts"), { recursive: true });
const files = readdirSync(src);

// Ürün görselleri: hm220-black-DyoM08y8.png → hm220-black.png
let n = 0;
for (const f of files) {
  const m = /^(hm220-(?:black|white|red|pink|card))-[\w-]+\.png$/.exec(f);
  if (!m) continue;
  copyFileSync(join(src, f), join(dst, `${m[1]}.png`));
  n++;
}

// Logo: uygulama paketinde data: URL olarak gömülü SVG.
for (const f of files.filter((f) => /^index-.*\.js$/.test(f))) {
  const m = /const logo = "data:image\/svg\+xml,([^"]+)"/.exec(readFileSync(join(src, f), "utf8"));
  if (m) {
    writeFileSync(join(dst, "logo.svg"), decodeURIComponent(m[1]));
    n++;
    break;
  }
}

// Sora (SIL OFL): başlık fontu.
const faces = [];
for (const f of files) {
  const m = /^sora-(latin(?:-ext)?)-(\d+)-normal-[\w-]+\.woff2$/.exec(f);
  if (!m) continue;
  const name = `sora-${m[1]}-${m[2]}.woff2`;
  copyFileSync(join(src, f), join(dst, "fonts", name));
  const range = m[1] === "latin" ? "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD" : "U+0100-02AF,U+0304,U+0308,U+0329,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF";
  faces.push(`@font-face{font-family:"Sora";font-style:normal;font-display:swap;font-weight:${m[2]};src:url("./fonts/${name}") format("woff2");unicode-range:${range}}`);
}
if (faces.length) writeFileSync(join(dst, "fonts.css"), faces.join("\n") + "\n");

console.log(`${n} görsel, ${faces.length} font kopyalandı → ${dst}`);
