// Hawk Hub ürün görselleri (npm run assets ile kopyalanır). Yoksa bileşenler SVG çizime düşer.

const files = import.meta.glob<string>("../assets/hawk/*.png", { eager: true, query: "?url", import: "default" });
const file = (name: string): string | null => files[`../assets/hawk/${name}.png`] ?? null;

export type VariantId = "black" | "white" | "red" | "pink";

// `highlight`: tuş vurgusunun rengi — fare gövdesi üzerinde seçilebilir olsun diye renge göre.
export const VARIANTS: { id: VariantId; label: string; hex: string; highlight: string }[] = [
  { id: "black", label: "Siyah", hex: "#1d1d1f", highlight: "#ffffff" },
  { id: "white", label: "Beyaz", hex: "#ededed", highlight: "#e3262d" },
  { id: "red", label: "Kırmızı", hex: "#d32f2f", highlight: "#ffffff" },
  { id: "pink", label: "Pembe", hex: "#f48fb1", highlight: "#b5179e" },
];

export function mouseImage(variant: VariantId): string | null {
  return file(`hm220-${variant}`) ?? file("hm220-black");
}

export const cardImage = file("hm220-card");

/** Fotoğrafın piksel boyutu; üstüne bindirilen tuş bölgeleri bu koordinatlarda. */
export const PHOTO_W = 600;
export const PHOTO_H = 1115;

// Hawk'ın resmî 3D modelleri (npm run assets → scripts/fetch-hawk-models.mjs). Yoksa prosedürel model.
const models = import.meta.glob<string>("../assets/hawk/3d/*.glb", { eager: true, query: "?url", import: "default" });

export function mouseModel(variant: VariantId): string | null {
  return models[`../assets/hawk/3d/hm220-${variant}.glb`] ?? models["../assets/hawk/3d/hm220-black.glb"] ?? null;
}
