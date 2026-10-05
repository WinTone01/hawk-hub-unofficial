<script lang="ts">
  // Seçili cihazın sayfası: başlık + alt sekmeler (Genel, Tuşlar, Makro).
  import { fade } from "svelte/transition";
  import Icon from "../components/Icon.svelte";
  import { app, type DeviceTab } from "../lib/state.svelte";
  import Buttons from "./Buttons.svelte";
  import DeviceGeneral from "./DeviceGeneral.svelte";
  import Macros from "./Macros.svelte";

  const tabs: { id: DeviceTab; label: string }[] = [
    { id: "general", label: "Genel" },
    { id: "buttons", label: "Tuşlar" },
    { id: "macros", label: "Makro" },
  ];
  const subtitle: Record<DeviceTab, string> = {
    general: "DPI, polling, aydınlatma ve sensör ayarlarını yapılandırın.",
    buttons: "Fare tuşlarına fonksiyon veya makro atayın. Modelde tuşa tıklayarak seçin.",
    macros: "Tuş dizileri kaydedin ve farenin hafızasına yükleyin.",
  };
  const battery = $derived(
    !app.battery ? null : app.battery.charging ? "Şarj oluyor" : app.battery.level == null ? null : `%${app.battery.level}`,
  );
</script>

<div class="head">
  <div>
    <h1>Hawk Gaming HM220</h1>
    <p>{subtitle[app.deviceTab]}</p>
    {#if battery}<span class="batt"><Icon name="battery" size={14} />{battery}</span>{/if}
  </div>
  <div class="tabs" role="tablist">
    {#each tabs as t (t.id)}
      <button role="tab" aria-selected={app.deviceTab === t.id} onclick={() => (app.deviceTab = t.id)}>{t.label}</button>
    {/each}
  </div>
</div>

{#if !app.device}
  <div class="banner" transition:fade={{ duration: 150 }}>
    <Icon name="plug" />
    <div>
      <b>HM220 bulunamadı.</b> Fareyi kabloyla ya da 2.4 GHz alıcısıyla takın. Linux'ta
      <code>99-hawk-hub.rules</code> udev kuralı gerekir. Ayarlar görünür ama fareye yazılamaz.
    </div>
  </div>
{/if}

{#key app.deviceTab}
  <div in:fade={{ duration: 160 }}>
    {#if app.deviceTab === "general"}<DeviceGeneral />
    {:else if app.deviceTab === "buttons"}<Buttons />
    {:else}<Macros />{/if}
  </div>
{/key}

<style>
  .head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 20px;
    margin-bottom: 24px;
  }
  h1 {
    font-size: 34px;
    font-weight: 800;
    line-height: 1.15;
  }
  .head p {
    margin: 6px 0 4px;
    color: var(--text-2);
  }
  .batt {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 12.5px;
  }
  .tabs {
    display: flex;
    gap: 2px;
    padding: 4px;
    border: 1px solid var(--line);
    border-radius: var(--r);
    background: var(--surface-2);
  }
  .tabs button {
    padding: 7px 14px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--text-2);
    font: inherit;
    font-family: var(--display);
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
  }
  .tabs button:hover {
    color: var(--text);
  }
  .tabs button[aria-selected="true"] {
    background: var(--accent);
    color: var(--on-accent);
  }
  .banner {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    margin-bottom: 18px;
    padding: 12px 16px;
    border-radius: var(--r);
    border: 1px solid color-mix(in srgb, var(--warn) 40%, transparent);
    background: color-mix(in srgb, var(--warn) 8%, transparent);
    font-size: 13px;
  }
  .banner :global(svg) {
    flex: none;
    color: var(--warn);
    margin-top: 1px;
  }
</style>
