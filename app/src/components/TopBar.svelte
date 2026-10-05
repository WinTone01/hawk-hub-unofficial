<script lang="ts">
  // Üst çubuk: logo, ana sekmeler, durum ve tema seçimi.
  import Icon from "./Icon.svelte";
  import Logo from "./Logo.svelte";
  import { isTauri } from "../lib/api";
  import { app, type Page } from "../lib/state.svelte";

  const tabs: { label: string; active: Page[]; go: () => void }[] = [
    { label: "Cihazlarım", active: ["devices", "device"], go: () => (app.page = app.device ? "device" : "devices") },
    { label: "Profiller", active: ["profiles"], go: () => (app.page = "profiles") },
    { label: "Ayarlar", active: ["settings"], go: () => (app.page = "settings") },
  ];
</script>

<header class="topbar">
  <div class="brand">
    <Logo size={42} />
    <span class="word">HAWK HUB</span>
    <span class="tag">UNOFFICIAL</span>
  </div>

  <nav class="hub-nav">
    {#each tabs as t (t.label)}
      <button class:active={t.active.includes(app.page)} onclick={t.go}>{t.label}</button>
    {/each}
  </nav>

  <div class="right">
    {#if !isTauri}<span class="chip">önizleme · sahte cihaz</span>{/if}
    {#if app.syncing}
      <span class="chip"><span class="spinner"></span>Fareden okunuyor</span>
    {:else if app.busy}
      <span class="chip"><span class="spinner"></span>Yazılıyor</span>
    {/if}
    <div class="theme" role="radiogroup" aria-label="Tema">
      <button role="radio" aria-checked={app.theme === "dark"} title="Koyu tema" onclick={() => app.setTheme("dark")}>
        <Icon name="moon" size={18} />
      </button>
      <button role="radio" aria-checked={app.theme === "light"} title="Açık tema" onclick={() => app.setTheme("light")}>
        <Icon name="sun" size={18} />
      </button>
    </div>
    <span class="link-state" class:on={!!app.device} title={app.device ? `HM220 · ${app.device.transport}` : "Cihaz bağlı değil"}>
      <Icon name={app.device?.transport.includes("2.4") ? "wifi" : "plug"} size={18} />
    </span>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: stretch;
    gap: 30px;
    height: 78px;
    padding: 0 26px;
    border-bottom: 1px solid var(--line);
    background: var(--bg-2);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .word {
    font-family: var(--display);
    font-size: 21px;
    font-weight: 800;
    letter-spacing: -0.01em;
  }
  .tag {
    align-self: center;
    padding: 2px 6px;
    border: 1px solid var(--line-hi);
    border-radius: 4px;
    color: var(--text-3);
    font-family: var(--display);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.14em;
  }
  .hub-nav {
    display: flex;
    gap: 26px;
    margin-left: 22px;
  }
  .hub-nav button {
    position: relative;
    padding: 0 5px;
    border: 0;
    background: none;
    color: var(--text-2);
    font: inherit;
    font-family: var(--display);
    font-size: 15px;
    cursor: pointer;
    transition: color 0.15s;
  }
  .hub-nav button:hover {
    color: var(--text);
  }
  .hub-nav button.active {
    color: var(--text);
    font-weight: 700;
  }
  .hub-nav button.active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: 12px;
    height: 3px;
    border-radius: 3px;
    background: var(--accent);
  }
  .right {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: auto;
  }
  .theme {
    display: flex;
    gap: 2px;
    padding: 3px;
    border: 1px solid var(--line);
    border-radius: 22px;
  }
  .theme button {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: none;
    color: var(--text-2);
    cursor: pointer;
  }
  .theme button[aria-checked="true"] {
    background: var(--accent);
    color: var(--on-accent);
  }
  .link-state {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    color: var(--text-3);
  }
  .link-state.on {
    color: var(--text);
  }
</style>
