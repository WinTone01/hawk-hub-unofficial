// HM220 V2 tekerlek LED'inin renk düzeltmesi.
//
// LED'de yeşil kanal kırmızı ve maviye göre çok baskın. Üreticinin fabrika renklerinden çıkarıldı:
// kitapçıkta "sarı" diye geçen 5. DPI kademesi farede `ff 40 00` olarak kayıtlı, yani ekrandaki
// sarı (#ffff00) farede yeşil ≈ %25 ile elde ediliyor. Kullanıcı gözlemi de bunu doğruluyor:
// biraz yeşil içeren kırmızı turuncu, turuncu ise sarı görünüyordu.
//
// Arayüz her zaman "ekranda nasıl görünecek" rengini gösterir; fareye giderken toDevice,
// fareden okunurken fromDevice uygulanır.

const G_SCALE = 0.25;

function parse(hex: string): [number, number, number] {
  const n = parseInt(hex.replace("#", ""), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function format([r, g, b]: number[]): string {
  return "#" + [r, g, b].map((v) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, "0")).join("");
}

/** Ekran rengi → farede aynı görünmesi için gönderilecek değer. */
export function toDevice(hex: string): string {
  const [r, g, b] = parse(hex);
  return format([r, g * G_SCALE, b]);
}

/** Farede kayıtlı değer → ekranda yaklaşık görünümü. */
export function fromDevice(hex: string): string {
  const [r, g, b] = parse(hex);
  return format([r, g / G_SCALE, b]);
}

/** Hazır renkler (ekran renkleri). Hawk Hub'ın LED paleti + turuncu, mor, pembe. */
export const LED_PRESETS: { label: string; hex: string }[] = [
  { label: "Kırmızı", hex: "#ff0000" },
  { label: "Turuncu", hex: "#ff6000" },
  { label: "Sarı", hex: "#ffff00" },
  { label: "Yeşil", hex: "#00ff00" },
  { label: "Camgöbeği", hex: "#00ffff" },
  { label: "Mavi", hex: "#0000ff" },
  { label: "Mor", hex: "#8000ff" },
  { label: "Pembe", hex: "#ff00ff" },
  { label: "Beyaz", hex: "#ffffff" },
];
