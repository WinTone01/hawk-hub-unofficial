<script lang="ts">
  // HM220'nin üstten görünüşü: LED efekti canlı önizlenir, tuşlar isteğe bağlı tıklanabilir.
  import { mouseImage, PHOTO_H, PHOTO_W, VARIANTS } from "../lib/assets";
  import { app } from "../lib/state.svelte";
  import type { ButtonId, LedMode } from "../lib/types";

  let {
    mode = "static",
    color = "#ff2d2d",
    brightness = 168,
    speed = 2,
    dpiColor = "#ff2d2d",
    selected = null,
    hovered = $bindable(null),
    onselect,
    height = 380,
  }: {
    mode?: LedMode;
    color?: string;
    brightness?: number;
    speed?: number;
    dpiColor?: string;
    selected?: ButtonId | null;
    hovered?: ButtonId | null;
    onselect?: (b: ButtonId) => void;
    height?: number;
  } = $props();

  const glow = $derived(mode === "off" ? "transparent" : mode === "neon" ? "#ff2d2d" : color);
  // Parlaklık 1–8.
  const intensity = $derived(mode === "off" ? 0 : 0.35 + (0.65 * Math.max(0, Math.min(8, brightness) - 1)) / 7);
  // Bayt 0 en hızlı, 5 en yavaş.
  const dur = $derived(`${0.8 + speed * 0.55}s`);
  const interactive = $derived(!!onselect);

  const zones: { id: ButtonId; d: string }[] = [
    { id: "left", d: "M97 14 C 56 16 30 46 26 106 L 25 138 Q 60 146 97 141 Z" },
    { id: "right", d: "M103 14 C 144 16 170 46 174 106 L 175 138 Q 140 146 103 141 Z" },
    { id: "forward", d: "M14 128 q -4 0 -4 5 v 26 q 0 5 4 5 h 7 v -36 z" },
    { id: "back", d: "M14 170 q -4 0 -4 5 v 26 q 0 5 4 5 h 7 v -36 z" },
  ];

  function zoneClass(id: ButtonId) {
    return { zone: true, interactive, selected: selected === id, hovered: hovered === id };
  }

  // Ürün fotoğrafı varsa onu kullan (Hawk Hub görselleri; npm run assets).
  const photo = $derived(mouseImage(app.variant));

  // Fotoğraf koordinatlarında (600×1115) tuş bölgeleri, fotoğraftan ölçüldü: tuş ek yeri y=487,
  // orta çizgi x=301, tekerlek kanalı x=261–337 (y<288), yan tuşlar x<31. Dış kenarlar bilerek
  // gövdenin dışına taşar; çizim fotoğrafın alfa maskesiyle kırpıldığı için gövdeyi birebir izler.
  const photoZones: { id: ButtonId; d: string }[] = [
    { id: "left", d: "M0 0 H261 V288 H301 V487 H31 V300 H0 Z" },
    { id: "right", d: "M337 0 H600 V487 H301 V288 H337 Z" },
    { id: "middle", d: "M268 128 a 26 26 0 0 1 26 -26 h 14 a 26 26 0 0 1 26 26 v 136 a 26 26 0 0 1 -26 26 h -14 a 26 26 0 0 1 -26 -26 z" },
    { id: "forward", d: "M0 333 H31 V478 H0 Z" },
    { id: "back", d: "M0 488 H31 V652 H0 Z" },
  ];
  const maskId = `hm220-mask-${Math.random().toString(36).slice(2, 8)}`;
  const highlight = $derived(VARIANTS.find((v) => v.id === app.variant)?.highlight ?? "#ffffff");
</script>

{#if photo}
  <div
    class="photo {mode}"
    style="height:{height}px; width:{(height * PHOTO_W) / PHOTO_H}px; --glow:{glow}; --int:{intensity}; --dur:{dur}"
  >
    <div class="led under"></div>
    <img src={photo} alt="Hawk HM220" draggable="false" />
    {#if interactive || selected || hovered}
      <svg viewBox="0 0 {PHOTO_W} {PHOTO_H}" class="zones" style="--hl:{highlight}">
        <defs>
          <mask id={maskId} maskUnits="userSpaceOnUse" x="0" y="0" width={PHOTO_W} height={PHOTO_H} style="mask-type:alpha">
            <image href={photo} width={PHOTO_W} height={PHOTO_H} />
          </mask>
        </defs>
        <g mask="url(#{maskId})">
        {#each photoZones as z (z.id)}
          <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
          <path
            d={z.d}
            class={zoneClass(z.id)}
            role={interactive ? "button" : undefined}
            tabindex={interactive ? 0 : undefined}
            aria-label={z.id}
            onclick={() => onselect?.(z.id)}
            onkeydown={(e) => (e.key === "Enter" || e.key === " ") && onselect?.(z.id)}
            onmouseenter={() => (hovered = z.id)}
            onmouseleave={() => (hovered = null)}
          />
        {/each}
        </g>
      </svg>
    {/if}
  </div>
{:else}
<svg
  viewBox="0 0 200 330"
  style="height:{height}px; --glow:{glow}; --int:{intensity}; --dur:{dur}"
  class="mouse {mode}"
  role="img"
  aria-label="HM220"
>
  <defs>
    <linearGradient id="shell" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#2a2d35" />
      <stop offset="0.55" stop-color="#17191e" />
      <stop offset="1" stop-color="#0c0d10" />
    </linearGradient>
    <radialGradient id="sheen" cx="0.35" cy="0.25" r="0.7">
      <stop offset="0" stop-color="#ffffff" stop-opacity="0.09" />
      <stop offset="1" stop-color="#ffffff" stop-opacity="0" />
    </radialGradient>
    <filter id="blur" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="9" />
    </filter>
  </defs>

  <!-- alt ışık (underglow) -->
  <g class="led">
    <ellipse cx="100" cy="250" rx="84" ry="70" fill="var(--glow)" filter="url(#blur)" opacity="0.55" />
  </g>

  <!-- gövde -->
  <path
    d="M100 12 C 152 12 178 48 180 112 L 184 222 C 186 284 152 318 100 318 C 48 318 14 284 16 222 L 20 112 C 22 48 48 12 100 12 Z"
    fill="url(#shell)"
    stroke="#30343d"
    stroke-width="1.5"
  />
  <path
    d="M100 12 C 152 12 178 48 180 112 L 184 222 C 186 284 152 318 100 318 C 48 318 14 284 16 222 L 20 112 C 22 48 48 12 100 12 Z"
    fill="url(#sheen)"
  />

  <!-- tuşlar -->
  {#each zones as z (z.id)}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <path
      d={z.d}
      class={zoneClass(z.id)}
      role={interactive ? "button" : undefined}
      tabindex={interactive ? 0 : undefined}
      aria-label={z.id}
      onclick={() => onselect?.(z.id)}
      onkeydown={(e) => (e.key === "Enter" || e.key === " ") && onselect?.(z.id)}
      onmouseenter={() => (hovered = z.id)}
      onmouseleave={() => (hovered = null)}
    />
  {/each}
  <line x1="100" y1="14" x2="100" y2="141" stroke="#08090b" stroke-width="2.5" />

  <!-- tekerlek -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <g
    class={zoneClass("middle")}
    role={interactive ? "button" : undefined}
    tabindex={interactive ? 0 : undefined}
    aria-label="middle"
    onclick={() => onselect?.("middle")}
    onkeydown={(e) => (e.key === "Enter" || e.key === " ") && onselect?.("middle")}
    onmouseenter={() => (hovered = "middle")}
    onmouseleave={() => (hovered = null)}
  >
    <rect x="89" y="40" width="22" height="52" rx="11" fill="#0b0c0f" stroke="#3a3f4a" />
    <rect x="94" y="46" width="12" height="40" rx="6" class="wheel" />
  </g>

  <!-- DPI göstergesi -->
  <rect x="94" y="102" width="12" height="20" rx="4" fill="#0b0c0f" stroke="#3a3f4a" />
  <rect x="97" y="106" width="6" height="12" rx="2" fill={dpiColor} style="filter: drop-shadow(0 0 4px {dpiColor})" />

  <!-- logo ışığı -->
  <g class="led logo">
    <path d="M100 222 l -24 -16 l 8 22 l 16 10 l 16 -10 l 8 -22 z" fill="none" stroke="var(--glow)" stroke-width="3" stroke-linejoin="round" />
    <path d="M100 222 l 0 16" stroke="var(--glow)" stroke-width="3" stroke-linecap="round" />
  </g>
</svg>
{/if}

<style>
  svg {
    display: block;
    overflow: visible;
  }
  .led {
    opacity: var(--int);
    transition: opacity 0.4s;
  }
  .logo {
    filter: drop-shadow(0 0 6px var(--glow)) drop-shadow(0 0 14px var(--glow));
  }
  .breathing .led {
    animation: breathe var(--dur) ease-in-out infinite;
  }
  .neon .led {
    animation: neon calc(var(--dur) * 3) linear infinite;
  }
  @keyframes breathe {
    0%,
    100% {
      opacity: var(--int);
    }
    50% {
      opacity: 0.04;
    }
  }
  @keyframes neon {
    to {
      filter: hue-rotate(360deg) drop-shadow(0 0 6px var(--glow));
    }
  }

  .zone {
    fill: transparent;
    stroke: transparent;
    stroke-width: 1.5;
    transition: fill 0.15s, stroke 0.15s;
    outline: none;
  }
  :where(.mouse) path.zone[aria-label="forward"],
  :where(.mouse) path.zone[aria-label="back"] {
    fill: #1d2026;
    stroke: #3a3f4a;
  }
  .wheel {
    fill: #2b2f38;
  }
  .interactive {
    cursor: pointer;
  }
  .zone.interactive.hovered,
  .zone.interactive:focus-visible {
    fill: color-mix(in srgb, var(--accent) 18%, transparent);
    stroke: color-mix(in srgb, var(--accent) 60%, transparent);
  }
  .zone.interactive.selected {
    fill: color-mix(in srgb, var(--accent) 30%, transparent);
    stroke: var(--accent);
  }
  g.zone.interactive.selected .wheel,
  g.zone.interactive.hovered .wheel {
    fill: var(--accent);
  }

  /* ── Ürün fotoğrafı ── */
  .photo {
    position: relative;
    flex: none;
  }
  .photo img {
    position: relative;
    z-index: 1;
    display: block;
    width: 100%;
    height: 100%;
    filter: drop-shadow(0 22px 28px rgba(0, 0, 0, 0.65));
    pointer-events: none;
  }
  .under {
    position: absolute;
    inset: 14% -22% -6%;
    border-radius: 50%;
    background: radial-gradient(closest-side, var(--glow), transparent);
    filter: blur(26px);
  }
  .neon .under {
    animation: neon-under calc(var(--dur) * 3) linear infinite;
  }
  @keyframes neon-under {
    to {
      filter: blur(26px) hue-rotate(360deg);
    }
  }
  .zones {
    position: absolute;
    inset: 0;
    z-index: 2;
    width: 100%;
    height: 100%;
  }
  /* Maske dış kenarı kırptığı için çizgi yok; vurgu rengi fare rengine göre (--hl). */
  .zones .zone,
  .zones .zone.interactive.hovered,
  .zones .zone.interactive.selected {
    stroke: none;
  }
  .zones .zone.interactive.hovered,
  .zones .zone.interactive:focus-visible {
    fill: color-mix(in srgb, var(--hl) 20%, transparent);
  }
  .zones .zone.interactive.selected {
    fill: color-mix(in srgb, var(--hl) 36%, transparent);
  }
  .zones .zone.interactive.selected.hovered {
    fill: color-mix(in srgb, var(--hl) 44%, transparent);
  }
</style>
