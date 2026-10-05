<script lang="ts">
  // Sol panel: bağlı cihazlar (ürün görseliyle) ve alt bağlantılar.
  import Icon from "./Icon.svelte";
  import MouseSketch from "./MouseSketch.svelte";
  import { mouseImage } from "../lib/assets";
  import { app } from "../lib/state.svelte";

  const img = $derived(mouseImage(app.variant));
  const battery = $derived(
    !app.battery ? null : app.battery.charging ? "Şarj oluyor" : app.battery.level == null ? null : `%${app.battery.level}`,
  );
  const version = __APP_VERSION__;
</script>

<aside class="rail">
  <button class="rail-device" class:active={app.page === "device"} onclick={() => app.openDevice()}>
    {#if img}<img src={img} alt="" draggable="false" />{:else}<span class="sketch"><MouseSketch /></span>{/if}
    <span class="info">
      <b>HM220</b>
      <span class="sub">{app.device?.transport ?? "Bağlı değil"}</span>
      {#if battery}<span class="sub batt"><Icon name="battery" size={13} />{battery}</span>{/if}
      <span class="state" class:on={!!app.device}><i></i>{app.device ? "Algılandı" : "Bekleniyor"}</span>
    </span>
  </button>

  <nav class="links">
    <button class:active={app.page === "devices"} onclick={() => (app.page = "devices")}>
      <Icon name="grid" size={20} />Tüm cihazlar
    </button>
    <button class:active={app.page === "settings"} onclick={() => (app.page = "settings")}>
      <Icon name="help" size={20} />Destek ve bilgi
    </button>
    <span class="ver">Hawk Hub Unofficial v{version}</span>
  </nav>
</aside>

<style>
  .rail {
    display: flex;
    flex-direction: column;
    gap: 20px;
    min-height: 0;
    padding: 24px 16px 16px;
    border-right: 1px solid var(--line);
    background: var(--bg-2);
    overflow-y: auto;
  }
  .rail-device {
    display: flex;
    align-items: center;
    gap: 13px;
    width: 100%;
    min-height: 106px;
    padding: 14px 10px;
    border: 1px solid transparent;
    border-radius: var(--r-lg);
    background: var(--rail);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }
  .rail-device:hover {
    background: var(--rail-hover);
  }
  .rail-device.active {
    background: var(--rail-active);
    border-color: var(--rail-active-line);
  }
  .rail-device img,
  .sketch :global(svg) {
    width: 82px;
    height: 68px;
    object-fit: contain;
    flex: none;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .info b {
    font-family: var(--display);
    font-size: 16px;
  }
  .sub {
    color: var(--text-3);
    font-size: 12px;
  }
  .batt {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .state {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
    font-size: 12px;
    color: var(--text-3);
  }
  .state i {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-3);
  }
  .state.on {
    color: var(--text);
  }
  .state.on i {
    background: var(--ok);
  }

  .links {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: auto;
    padding-top: 16px;
    border-top: 1px solid var(--line);
  }
  .links button {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 9px 10px;
    border: 0;
    border-radius: var(--r);
    background: none;
    color: var(--text-2);
    font: inherit;
    font-size: 13.5px;
    text-align: left;
    cursor: pointer;
  }
  .links button:hover,
  .links button.active {
    background: var(--hover);
    color: var(--text);
  }
  .ver {
    padding: 12px 10px 0;
    color: var(--text-3);
    font-size: 11px;
  }
</style>
