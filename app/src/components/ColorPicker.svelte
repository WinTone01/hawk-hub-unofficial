<script lang="ts">
  // HSV renk seçici: doygunluk/parlaklık karesi + ton şeridi + hex alanı.
  // Sürüklerken `oninput` (önizleme), bırakınca `oncommit` (cihaza yaz).
  let { value, oninput, oncommit }: { value: string; oninput?: (hex: string) => void; oncommit: (hex: string) => void } =
    $props();

  function hexToHsv(hex: string): [number, number, number] {
    const n = parseInt(hex.slice(1), 16);
    const r = ((n >> 16) & 255) / 255, g = ((n >> 8) & 255) / 255, b = (n & 255) / 255;
    const max = Math.max(r, g, b), d = max - Math.min(r, g, b);
    let h = 0;
    if (d) h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
    return [(h * 60 + 360) % 360, max ? d / max : 0, max];
  }

  function hsvToHex(h: number, s: number, v: number): string {
    const f = (n: number) => {
      const k = (n + h / 60) % 6;
      return Math.round(255 * (v - v * s * Math.max(0, Math.min(k, 4 - k, 1))));
    };
    return "#" + [f(5), f(3), f(1)].map((x) => x.toString(16).padStart(2, "0")).join("");
  }

  let hsv = $state<[number, number, number]>([0, 1, 1]);
  let dragging = false;
  let hexInput = $state("");

  // Dışarıdan değer değişince (sürüklemiyorsak) iç durumu güncelle.
  $effect(() => {
    if (!dragging) {
      hsv = hexToHsv(value);
      hexInput = value;
    }
  });

  const current = $derived(hsvToHex(...hsv));

  function emit() {
    hexInput = current;
    oninput?.(current);
  }

  function drag(node: HTMLElement, kind: "sv" | "hue") {
    const update = (e: PointerEvent) => {
      const r = node.getBoundingClientRect();
      const x = Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
      const y = Math.min(1, Math.max(0, (e.clientY - r.top) / r.height));
      hsv = kind === "sv" ? [hsv[0], x, 1 - y] : [x * 359.9, hsv[1], hsv[2]];
      emit();
    };
    const down = (e: PointerEvent) => {
      dragging = true;
      node.setPointerCapture(e.pointerId);
      update(e);
    };
    const move = (e: PointerEvent) => dragging && update(e);
    const up = () => {
      if (!dragging) return;
      dragging = false;
      oncommit(current);
    };
    node.addEventListener("pointerdown", down);
    node.addEventListener("pointermove", move);
    node.addEventListener("pointerup", up);
    node.addEventListener("pointercancel", up);
    return {
      destroy() {
        node.removeEventListener("pointerdown", down);
        node.removeEventListener("pointermove", move);
        node.removeEventListener("pointerup", up);
        node.removeEventListener("pointercancel", up);
      },
    };
  }

  function commitHex() {
    const v = hexInput.trim().replace(/^#?/, "#").toLowerCase();
    if (/^#[0-9a-f]{6}$/.test(v)) {
      if (v !== value) oncommit(v);
    } else hexInput = value;
  }
</script>

<div class="picker">
  <div class="sv" use:drag={"sv"} style="--hue: hsl({hsv[0]} 100% 50%)">
    <div class="knob" style="left:{hsv[1] * 100}%; top:{(1 - hsv[2]) * 100}%; background:{current}"></div>
  </div>
  <div class="hue" use:drag={"hue"}>
    <div class="knob" style="left:{(hsv[0] / 360) * 100}%; background: hsl({hsv[0]} 100% 50%)"></div>
  </div>
  <div class="hex-row">
    <span class="chip" style="background:{current}"></span>
    <input
      class="hex"
      bind:value={hexInput}
      maxlength="7"
      spellcheck="false"
      onkeydown={(e) => e.key === "Enter" && commitHex()}
      onblur={commitHex}
    />
  </div>
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .sv {
    position: relative;
    height: 150px;
    border-radius: 10px;
    background: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, var(--hue));
    cursor: crosshair;
    touch-action: none;
  }
  .hue {
    position: relative;
    height: 14px;
    border-radius: 7px;
    background: linear-gradient(90deg, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
    cursor: pointer;
    touch-action: none;
  }
  .knob {
    position: absolute;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 3px solid #fff;
    box-shadow: 0 0 0 1px #0008, 0 2px 8px #000a;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }
  .hue .knob {
    top: 50%;
  }
  .hex-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .chip {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: 1px solid var(--line-hi);
  }
  .hex {
    width: 110px;
    font-family: var(--mono);
    text-transform: uppercase;
  }
</style>
