// Hawk'ın sitesindeki resmî HM220 3D modellerini (GLB) indirir. Modeller Hawk Chair'e aittir;
// depoya eklenmez (.gitignore). Yoksa arayüz prosedürel 3D modele düşer.
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const dst = join(here, "..", "src", "assets", "hawk", "3d");
const base = "https://design.hawkchair.com/static/sayfalar/hawkWebsite/glbsOrj";
mkdirSync(dst, { recursive: true });

for (const v of ["White", "Black", "Red", "Pink"]) {
  const out = join(dst, `hm220-${v.toLowerCase()}.glb`);
  if (existsSync(out)) continue;
  try {
    const res = await fetch(`${base}/hm220${v}.glb`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    writeFileSync(out, Buffer.from(await res.arrayBuffer()));
    console.log(`indirildi: ${out}`);
  } catch (e) {
    console.log(`indirilemedi (hm220${v}.glb): ${e.message}; prosedürel model kullanılacak.`);
  }
}
