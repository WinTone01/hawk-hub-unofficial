<script lang="ts">
  // Genel: solda DPI + polling, ortada 3D model, sağda aydınlatma + tepki/sensör.
  import ColorPicker from "../components/ColorPicker.svelte";
  import Mouse3D from "../components/Mouse3D.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Slider from "../components/Slider.svelte";
  import Toggle from "../components/Toggle.svelte";
  import VariantPicker from "../components/VariantPicker.svelte";
  import { api } from "../lib/api";
  import { fromDevice, LED_PRESETS, toDevice } from "../lib/ledColor";
  import { app, DPI_COLORS } from "../lib/state.svelte";
  import type { LedMode } from "../lib/types";

  const s = $derived(app.settings!);
  const c = $derived(app.catalog!);
  const MAX_DPI = 12000;
  const SPEED_MAX = 5;

  // ── DPI ──
  function commitDpi(stage: number, input: HTMLInputElement) {
    const v = Math.min(MAX_DPI, Math.max(100, Math.round(+input.value / 50) * 50));
    if (!Number.isFinite(v)) return void (input.value = String(s.dpiValues[stage - 1]));
    input.value = String(v);
    if (v !== s.dpiValues[stage - 1]) app.act(() => api.setDpiValue(stage, v), `Kademe ${stage}: ${v} DPI`);
  }
  function activate(stage: number) {
    if (stage !== s.dpiStage) app.act(() => api.setDpiStage(stage), `Aktif kademe ${stage} · ${s.dpiValues[stage - 1]} DPI`);
  }

  // ── Aydınlatma ──
  const modes: { value: LedMode; label: string }[] = [
    { value: "off", label: "Kapalı" },
    { value: "neon", label: "Neon" },
    { value: "static", label: "Sabit" },
    { value: "breathing", label: "Nefes" },
  ];
  const shown = $derived(fromDevice(s.ledColor));
  let draftColor = $state<string | null>(null);
  let draftBrightness = $state<number | null>(null);
  let draftSpeed = $state<number | null>(null);
  let pickerOpen = $state(false);
  let pickerHost = $state<HTMLElement>();

  async function setColor(hex: string) {
    draftColor = null;
    const needsMode = s.ledMode === "off" || s.ledMode === "neon";
    const color = toDevice(hex);
    await app.act(() => api.setLed(needsMode ? { color, mode: "static" } : { color }), "Renk değişti");
  }
  function outside(e: PointerEvent) {
    if (pickerOpen && pickerHost && !pickerHost.contains(e.target as Node)) pickerOpen = false;
  }

  // ── Tepki ──
  const respIndex = $derived(Math.max(0, c.responseSteps.indexOf(s.responseMs)));
  let draftResp = $state<number | null>(null);
</script>

<svelte:window onpointerdown={outside} />

<div class="layout rise">
  <!-- Sol -->
  <div class="col">
    <section class="card">
      <h3>DPI Kademeleri</h3>
      <div class="stages">
        {#each s.dpiValues as v, i (i)}
          <div
            class="stage"
            class:active={s.dpiStage === i + 1}
            role="button"
            tabindex="0"
            title="Bu kademeyi etkinleştir"
            onclick={(e) => !(e.target instanceof HTMLInputElement) && activate(i + 1)}
            onkeydown={(e) => e.key === "Enter" && !(e.target instanceof HTMLInputElement) && activate(i + 1)}
          >
            <span class="dot" style="--c:{DPI_COLORS[i]}"></span>
            <input
              type="number"
              min="100"
              max={MAX_DPI}
              step="50"
              value={v}
              aria-label="Kademe {i + 1} DPI"
              onchange={(e) => commitDpi(i + 1, e.currentTarget)}
              onkeydown={(e) => e.key === "Enter" && e.currentTarget.blur()}
            />
            <span class="unit">DPI</span>
            {#if s.dpiStage === i + 1}<span class="act">AKTİF</span>{/if}
          </div>
        {/each}
      </div>
    </section>

    <section class="card">
      <div class="row-head">
        <h3>Polling Oranı</h3>
        <span class="val">{s.pollingHz} Hz</span>
      </div>
      <Segmented
        options={c.pollingRates.map((hz) => ({ value: hz, label: `${hz}` }))}
        value={s.pollingHz}
        onchange={(hz) => app.act(() => api.setPolling(hz), `Polling ${hz} Hz`)}
      />
    </section>
  </div>

  <!-- Orta: 3D model -->
  <div class="center">
    <div class="glow" aria-hidden="true"></div>
    <Mouse3D
      mode={s.ledMode}
      color={draftColor ?? shown}
      brightness={draftBrightness ?? s.ledBrightness}
      speed={draftSpeed ?? s.ledSpeed}
      height={540}
      distance={2.5}
      view="top"
    />
    <div class="variants"><VariantPicker compact /></div>
    <div class="readout">
      <div><b>{s.dpiValues[s.dpiStage - 1]}</b><span>DPI</span></div>
      <i></i>
      <div><b>{s.pollingHz}</b><span>HZ</span></div>
    </div>
  </div>

  <!-- Sağ -->
  <div class="col">
    <section class="card">
      <h3>Aydınlatma</h3>
      <div class="eyebrow gap">Mod</div>
      <Segmented
        options={modes}
        value={s.ledMode}
        onchange={(m) => app.act(() => api.setLed({ mode: m }), `Mod: ${modes.find((x) => x.value === m)?.label}`)}
      />
      <div class="light-grid">
        <div class="color" class:dim={s.ledMode === "off" || s.ledMode === "neon"} bind:this={pickerHost}>
          <div class="eyebrow">Renk</div>
          <button class="swatch-btn" onclick={() => (pickerOpen = !pickerOpen)} aria-expanded={pickerOpen}>
            <span class="swatch" style="--c:{draftColor ?? shown}"></span>
            <span class="hex">{(draftColor ?? shown).toUpperCase()}</span>
          </button>
          <div class="presets">
            {#each LED_PRESETS as p (p.hex)}
              <button
                class="pdot"
                class:on={toDevice(p.hex) === s.ledColor}
                style="--c:{p.hex}"
                title={p.label}
                aria-label={p.label}
                onclick={() => setColor(p.hex)}
              ></button>
            {/each}
          </div>
          {#if pickerOpen}
            <div class="popover">
              <ColorPicker value={shown} oninput={(x) => (draftColor = x)} oncommit={setColor} />
            </div>
          {/if}
        </div>
        <div class="sliders">
          <div class="eyebrow">Hız</div>
          <Slider
            value={SPEED_MAX - s.ledSpeed}
            min={0}
            max={SPEED_MAX}
            format={(v) => `${v + 1}/6`}
            ends={["Yavaş", "Hızlı"]}
            oninput={(v) => (draftSpeed = SPEED_MAX - v)}
            oncommit={(v) => ((draftSpeed = null), app.act(() => api.setLed({ speed: SPEED_MAX - v }), "Efekt hızı değişti"))}
          />
          <div class="eyebrow gap-top">Parlaklık</div>
          <Slider
            value={s.ledBrightness}
            min={1}
            max={8}
            format={(v) => `${v}/8`}
            ends={["Kısık", "Parlak"]}
            oninput={(v) => (draftBrightness = v)}
            oncommit={(v) => ((draftBrightness = null), app.act(() => api.setLed({ brightness: v }), "Parlaklık değişti"))}
          />
        </div>
      </div>
    </section>

    <section class="card">
      <div class="row-head">
        <h3>Tepki & Sensör</h3>
        <span class="val">{c.responseSteps[draftResp ?? respIndex]} ms</span>
      </div>
      <Slider
        value={respIndex}
        min={0}
        max={c.responseSteps.length - 1}
        format={(i) => `${c.responseSteps[i]} ms`}
        showValue={false}
        oninput={(i) => (draftResp = i)}
        oncommit={(i) => ((draftResp = null), app.act(() => api.setResponse(c.responseSteps[i]), `Tepki süresi ${c.responseSteps[i]} ms`))}
      />
      <Toggle
        checked={s.ripple}
        label="Dalga kontrolü"
        hint="İmleç titremesini azaltır"
        onchange={(on) => app.act(() => api.setRipple(on), `Dalga kontrolü ${on ? "açık" : "kapalı"}`)}
      />
      <div class="divider"></div>
      <Toggle
        checked={s.angleSnap}
        label="Açı düzeltme"
        hint="İmleci düz çizgilere hizalar"
        onchange={(on) => app.act(() => api.setAngleSnap(on), `Açı düzeltme ${on ? "açık" : "kapalı"}`)}
      />
    </section>
  </div>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: minmax(280px, 1fr) minmax(260px, 1.05fr) minmax(300px, 1fr);
    gap: 22px;
    align-items: start;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 22px;
  }
  .card h3 {
    margin-bottom: 16px;
  }
  .row-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 14px;
  }
  .row-head h3 {
    margin: 0;
  }
  .val {
    font-family: var(--mono);
    font-size: 15px;
    font-weight: 700;
  }

  /* DPI satırları */
  .stages {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .stage {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px 8px 12px;
    border: 1px solid var(--line);
    border-radius: var(--r);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .stage:hover {
    background: var(--hover);
  }
  .stage.active {
    border-color: var(--text-3);
    background: var(--hover);
  }
  .dot {
    flex: none;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: var(--c);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.25);
  }
  .stage input {
    flex: 1;
    min-width: 0;
    padding: 7px 10px;
    font-family: var(--mono);
    font-weight: 700;
    appearance: textfield;
  }
  .stage input::-webkit-inner-spin-button,
  .stage input::-webkit-outer-spin-button {
    appearance: none;
    margin: 0;
  }
  .unit,
  .act {
    font-family: var(--display);
    font-size: 10.5px;
    letter-spacing: 0.08em;
    color: var(--text-3);
  }
  .act {
    font-weight: 800;
    color: var(--text);
  }

  /* Orta sahne */
  .center {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
  }
  .glow {
    position: absolute;
    left: 50%;
    top: 46%;
    width: 70%;
    aspect-ratio: 1;
    translate: -50% -50%;
    border-radius: 50%;
    background: var(--led);
    filter: blur(80px);
    opacity: calc(var(--led-i, 0) * 0.22);
    pointer-events: none;
  }
  .variants {
    padding: 8px 12px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--surface);
  }
  .variants :global(.dot) {
    width: 26px !important;
    height: 26px !important;
  }
  .readout {
    display: flex;
    align-items: center;
    gap: 18px;
    padding: 8px 22px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--surface);
  }
  .readout div {
    display: flex;
    flex-direction: column;
    align-items: center;
    line-height: 1.1;
  }
  .readout b {
    font-family: var(--mono);
    font-size: 22px;
  }
  .readout span {
    color: var(--text-3);
    font-family: var(--display);
    font-size: 9px;
    letter-spacing: 0.1em;
  }
  .readout i {
    width: 1px;
    height: 30px;
    background: var(--line-hi);
  }

  /* Aydınlatma */
  .gap {
    margin-top: 2px;
  }
  .gap-top {
    margin-top: 14px;
  }
  .light-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 22px;
    margin-top: 22px;
  }
  .color {
    position: relative;
  }
  .color.dim .swatch-btn,
  .color.dim .presets {
    opacity: 0.5;
  }
  .swatch-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    cursor: pointer;
  }
  .swatch {
    width: 42px;
    height: 34px;
    border-radius: 6px;
    border: 3px solid var(--surface-3);
    outline: 1px solid var(--line-hi);
    background: var(--c);
  }
  .hex {
    font-family: var(--mono);
    font-size: 12.5px;
    font-weight: 700;
  }
  .presets {
    display: grid;
    grid-template-columns: repeat(5, 18px);
    gap: 7px;
    margin-top: 14px;
  }
  .pdot {
    width: 18px;
    height: 18px;
    padding: 0;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--c);
    box-shadow: 0 0 0 1px var(--line-hi);
    cursor: pointer;
  }
  .pdot.on {
    border-color: var(--surface);
    box-shadow: 0 0 0 2px var(--text);
  }
  .popover {
    position: absolute;
    z-index: 20;
    top: 62px;
    left: 0;
    width: 250px;
    padding: 14px;
    border: 1px solid var(--line-hi);
    border-radius: var(--r-lg);
    background: var(--surface-2);
    box-shadow: var(--shadow);
  }
  .sliders {
    min-width: 0;
  }

  @media (max-width: 1180px) {
    .layout {
      grid-template-columns: 1fr 1fr;
    }
    .center {
      grid-column: 1 / -1;
      order: -1;
    }
  }
</style>
