# Hawk HM220 V2 HID protokolü

HM220 V2 (PAW3311, Beken yongası) Attack Shark X11 / Delux M600 Pro ile aynı firmware ailesini kullanır.
Kaynaklar:

- Hawk Hub 1.0.20-beta (`resources/app.asar`): DPI, polling, tuş atama ve makro frame'leri doğru. **LED frame'i ve LED checksum'ı yanlış**; fare `50 01 05` ile reddeder. Hawk Hub'da ışık ayarlarının çalışmamasının sebebi bu.
- [attack-shark-x11-driver](https://github.com/HarukaYamamoto0/attack-shark-x11-driver) (MIT): okuma akışı, komut onayları, LED yerleşimi, PAW3311 DPI tablosu.
- HM220 V2 üzerinde yapılan testler (2026-10-04). ✅ işaretli maddeler donanımda doğrulandı.

## Cihaz

| | |
|---|---|
| VID | `0x1D57` |
| PID | `0x2027` kablolu, `0xFA60` 2.4 GHz alıcı |
| Komut kanalı | USB interface 2, usage page `0x000B`, feature report (Windows'ta koleksiyon boyu kabloluda 262, alıcıda 134 bayt) |
| Olay kanalı | USB interface 2, usage page `0x000A`, input report: pil ve komut onayları |
| Bluetooth | Yalnızca pil (Windows PnP özelliğinden) |

Linux'ta interface 2'nin iki koleksiyonu tek bir `/dev/hidrawN` altında görünür.

Frame biçimi: `[report ID, paket uzunluğu, profil (01), …]`. Gönderilen uzunluk (report ID dahil):

| Report | Kablolu | Kablosuz |
|---|---|---|
| `0x04` DPI | 52 | 56 |
| `0x05` LED | 13 | 15 |
| `0x06` Polling | 9 | 9 |
| `0x08` Tuş atama | 59 | 59 |
| `0x09` Makro | 64 | 64 |

## Olaylar (input report, usage page 0x0A)

`03 <model> <tür> <p1> <p2>`. HM220'nin model baytı `0x5C`.

| Tür | Anlam |
|---|---|
| `40` | Pil: `p1` durum (1 deşarj, 2 dolu, 3 şarj oluyor), `p2` seviye %. ✅ |
| `50` | Komut onayı: `p1` 0 = kabul, 1 = red; `p2` report ID. ✅ |
| `10` | Farenin DPI tuşu: `p1` yeni aktif kademe (1–5). ✅ |
| `80` | Aktif profil değişti (X11 belgelerinden; HM220'de gözlenmedi) |

Kablosuzda fare 5 sn hareketsizlikten sonra uyur ve komutları kaçırır (onay gelmez). Onay gelmeyen komut tekrar gönderilir. ✅

## Ayar okuma ✅

Fare ayarları okunabiliyor. Hawk Hub "okumayı desteklemiyor" sanıyordu çünkü izin istemiyordu.

1. Feature `A0 <report> <uzunluk> 00 01 00 00 00` gönder (uzunluk: DPI `38`, LED `0f`, polling `09`, tuş atama `3b`).
2. ~250 ms bekle, feature `A0` oku (8 bayt): `a0 01 …` ise izin var.
3. Feature `<report>` oku (o uzunlukta). İzin tek bir okuma için geçerli.

## Report 0x04 — DPI / sensör ✅

Frame indeksleri (report ID = 0):

| İndeks | Anlam |
|---|---|
| 1 | `38` |
| 3 | Açı düzeltme (alt 4 bit) |
| 4 | Ripple control (alt 4 bit) |
| 5 | Etkin kademe maskesi (`1f` = 5 kademe) |
| 6 | ×2 bayrakları (bit = kademe), 10000 DPI üstü |
| 8–15 | Kademe 1–8 DPI X baytı (PAW3311 tablosu) |
| 16–23 | Kademe 1–8 DPI Y baytı (5000 üstü 100'lük adımlarda 1) |
| 24 | Aktif kademe (1–5) |
| 25–48 | Kademe 1–8 **tekerlek ışığı** rengi (R, G, B), mod 4/5/6 kullanır. Alttaki DPI göstergesi bunlardan etkilenmez; renkleri firmware'e gömülü (kırmızı, mavi, yeşil, mor, sarı) ✅ |
| 50–51 | Checksum: `sum(frame[3..=49])`, 16 bit big-endian |

DPI tablosu fabrika değerleriyle doğrulandı: 400→`09`, 800→`12`, 1600→`25`, 3200→`4b`, 4800→`70`. En fazla 12000 DPI.

## Report 0x05 — Işık + uyku + tepki süresi ✅ (renk hariç)

Fabrika: `05 0f 01 02 03 c8 00 e6 0f 3c 00 01 fe`

| İndeks | Anlam |
|---|---|
| 3 | Mod (alt 4 bit), aşağıdaki tabloya bakın |
| 4 | Alt 4 bit: efekt hızı; üst 4 bit: derin uyku süresinin yüksek yarısı |
| 5 | Alt 4 bit: parlaklık 1–8; üst 4 bit: derin uyku süresinin düşük yarısı |
| 6–8 | R, G, B. Fare saklıyor ama hiçbir modda görünür rengi etkilemiyor |
| 9 | Uyku süresi (dakika × 2) |
| 10 | Tuş tepki süresi / 2 ms |
| 11–12 | Checksum: `sum(frame[3..=10])`, 16 bit big-endian ✅ |

Modlar (HM220 V2'de gözle doğrulandı ✅):

| Değer | Davranış |
|---|---|
| 0 | Kapalı |
| 1 | Neon (gökkuşağı) |
| 2 / 3 | Sabit / nefes, firmware'e gömülü kırmızı (renk baytları yok sayılır) |
| 4 / 5 | Sabit, **aktif DPI kademesinin rengi** (report 0x04, indeks 25–48) |
| 6 | Nefes, **aktif DPI kademesinin rengi** |

Özel renk bu yüzden DPI kademe renkleri üzerinden verilir: uygulama "sabit"i 5, "nefes"i 6 olarak yazar;
tek renk seçilince beş kademenin rengi aynı yapılır. DPI tuşuyla kademe değişince ışık da o kademenin rengine geçer.
Hawk Hub bu modda da hatalı: mod 2/3 yazıyor (hep kırmızı) ve rengi LED raporuna koyuyor.

Kablo takılıyken tekerlek ışığı şarj göstergesi olarak sürekli kırmızıdır ve ayarları geçersiz kılar (kitapçık).

## Report 0x06 — Polling ✅

`06 09 01 v (255−v) 00 00 00 00`, `v = 1000 / Hz` (125/250/500/1000). Kabloluda yalnızca 1000 Hz.

## Report 0x08 — Tuş atama ✅ (okuma/yazma onayı)

`08 3b 01` + 18 slot × 3 bayt (`kod, p1, p2`) + `00 00` + checksum (`sum` slot baytları, 16 bit big-endian, son iki bayt).
Slot `n` → frame indeksi `3 + 3n`. Slot 0 sol, 1 sağ, 2 orta, 3 geri, 4 ileri, 5 DPI; fabrika kodları `2,3,4,5,6,13,0×10,9,10`.

Fonksiyon kodları: 1 kapalı, 2 sol, 3 sağ, 4 orta, 5 geri, 6 ileri, 7 çift tık, 8 ateş, 9–12 kaydırma (yukarı/aşağı/sol/sağ),
13 DPI döngü, 14 DPI+, 15 DPI−, 16 nişan, 17 kısayol, 18 makro (`18, mod, makroID`), 21–28 medya, 29–38 uygulama/tarayıcı.

## Report 0x09 — Makro ✅

Yerleşim attack-shark-x11-driver ile aynı. Hawk Hub'ın gecikme kodlaması yanlıştı (ms'yi 10 ms birimi sanıyor, her basışa
500 ms koyuyor, uzun beklemede 100 ms blok kullanıyor) ve tuşa bağlarken değiştirici baytına modu yazıyordu.

| Sayfa | Frame |
|---|---|
| 0 | `09 40 id 00 mod R G B tekrar ad[20 bayt UTF-8] eylem_sayısı eylemler[0..34]` |
| 1 | `09 40 id 01 eylemler[34..94]` |
| 2 | `09 40 id 02 eylemler[94..100] ck_hi ck_lo` |

- Checksum: `sayfa0[4..=63] + sayfa1[4..=63] + sayfa2[4..=8]` toplamı, 16 bit big-endian.
- Eylem: `[yön<<7 | bekleme/10ms, HID kodu]` (bekleme ≤ 1270 ms) ya da `[yön<<7 | kalan/10ms, kod, 200ms_blok, 03]`.
  Yön 0 bas, 1 bırak. Eylem sayısı: 2 baytlık eylem 1, 4 baytlık 2 sayılır. Fare tuşları `F1`–`F5`.
- Fare **yalnızca son sayfadan sonra** onay verir. İlk iki sayfayı onay bekleyip tekrar göndermek fareyi kilitliyordu.
- **2.4 GHz alıcı üzerinden** (Hawk Hub'ın yöntemi, doğrulandı ✅): önce onaylı zararsız bir komutla fare uyandırılır,
  sayfalar **1 sn arayla** gönderilir ve son sayfanın uzunluk baytı **`0x0C`** olur (taşıdığı 12 bayt). Fare sayfalar
  arasında uyursa makro `50 01 09` ile reddedilir. Sayfaları hızlı ya da tekrar tekrar göndermek fare–alıcı
  bağlantısını koparabiliyor (fareyi kapatıp alıcıyı yeniden takınca düzeliyor).
- Okuma: `A0 09 83 00 <id> 00 00 00` izni, sonra 131 bayt; boş yuva `ff` döner. Checksum `sum([3..=128])`, little-endian `[129..130]`.
- Tuşa bağlama (report 0x08): slot = `12 00 <id>` (orta bayt değiştirici tuşlar).
- Makro fiziksel tuş konumu (HID kodu) gönderir; Türkçe Q düzeninde `0x0C` "ı" yazar.

## LED renk düzeltmesi ✅

Tekerlek LED'inde yeşil kanal baskın: üreticinin "sarı"sı `ff 40 00`. Uygulama ekran rengini fareye gönderirken
yeşili ×0,25 ile ölçekler, fareden okurken ×4 ile geri çevirir (`app/src/lib/ledColor.ts`). Hazır renklerin hepsi
bu düzeltmeyle farede doğru göründü.
