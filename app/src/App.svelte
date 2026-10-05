<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import DeviceRail from "./components/DeviceRail.svelte";
  import Toasts from "./components/Toasts.svelte";
  import TopBar from "./components/TopBar.svelte";
  import { fromDevice } from "./lib/ledColor";
  import { ledFrame, type LedLook } from "./lib/ledClock";
  import { app } from "./lib/state.svelte";
  import Device from "./pages/Device.svelte";
  import Devices from "./pages/Devices.svelte";
  import Profiles from "./pages/Profiles.svelte";
  import Settings from "./pages/Settings.svelte";

  let error = $state<string | null>(null);

  onMount(() => {
    app.init().catch((e) => (error = String(e)));
  });

  // Farenin tekerlek ışığı: 3D modelle aynı saatten (ledClock) her karede --led (renk) ve
  // --led-i (yoğunluk 0–1) güncellenir; modelin arkasındaki parıltı nefes/neon ile eş zamanlı.
  const look = $derived.by<LedLook>(() => {
    const s = app.settings;
    if (!s) return { ledMode: "off", ledColor: "#ff3347", brightness: 8, speed: 3 };
    const color = fromDevice(s.dpiColors[s.dpiStage - 1] ?? s.ledColor);
    return { ledMode: s.ledMode, ledColor: color, brightness: s.ledBrightness, speed: s.ledSpeed };
  });
  onMount(() => {
    let raf = 0;
    const root = document.documentElement;
    const tick = () => {
      const f = ledFrame(look);
      root.style.setProperty("--led", f.color);
      root.style.setProperty("--led-i", f.intensity.toFixed(3));
      raf = requestAnimationFrame(tick);
    };
    tick();
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="app">
  <TopBar />
  <DeviceRail />
  <main class="content">
    {#if error}
      <div class="card error">Başlatılamadı: {error}</div>
    {:else if !app.settings || !app.catalog}
      <div class="loading"><span class="spinner big"></span></div>
    {:else}
      {#key app.page}
        <div class="page" in:fade={{ duration: 160 }}>
          {#if app.page === "devices"}<Devices />
          {:else if app.page === "device"}<Device />
          {:else if app.page === "profiles"}<Profiles />
          {:else}<Settings />{/if}
        </div>
      {/key}
    {/if}
  </main>
</div>
<Toasts />
<ConfirmDialog />

<style>
  .app {
    display: grid;
    grid-template-columns: 250px minmax(0, 1fr);
    grid-template-rows: 78px minmax(0, 1fr);
    height: 100vh;
    background: var(--bg);
  }
  .app > :global(.topbar) {
    grid-column: 1 / -1;
  }
  .content {
    min-width: 0;
    overflow-y: auto;
    padding: 30px 32px 48px;
  }
  .page {
    max-width: 1440px;
    margin: 0 auto;
  }
  .loading {
    display: grid;
    place-items: center;
    height: 60%;
  }
  .error {
    color: var(--danger);
  }
  @media (max-width: 1100px) {
    .app {
      grid-template-columns: 208px minmax(0, 1fr);
    }
  }
</style>
