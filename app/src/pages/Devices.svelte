<script lang="ts">
  // Kontrol merkezi: bağlı cihazlar ve sistem kullanımı.
  import { onMount } from "svelte";
  import Icon from "../components/Icon.svelte";
  import MouseSketch from "../components/MouseSketch.svelte";
  import { api } from "../lib/api";
  import { mouseImage } from "../lib/assets";
  import { app } from "../lib/state.svelte";
  import type { SystemUsage } from "../lib/types";

  const img = $derived(mouseImage(app.variant));
  let usage = $state<SystemUsage | null>(null);

  onMount(() => {
    const poll = () => api.systemUsage().then((u) => (usage = u)).catch(() => {});
    poll();
    const t = setInterval(poll, 2000);
    return () => clearInterval(t);
  });

  const gb = (b: number) => (b / 1024 ** 3).toFixed(1);
</script>

<div class="page-head">
  <h1>Kontrol Merkezi</h1>
  <p>Bağlı yüksek performanslı çevre birimlerinizi yönetin.</p>
</div>

<div class="grid-devices rise">
  <div class="card device">
    <div class="shot">
      {#if app.battery?.level != null}
        <span class="batt"><Icon name="battery" size={13} />{app.battery.level}%</span>
      {/if}
      {#if img}<img src={img} alt="Hawk HM220" draggable="false" />{:else}<MouseSketch />{/if}
    </div>
    <h2>Hawk HM220</h2>
    <span class="state" class:on={!!app.device}><i></i>{app.device ? "Bağlı" : "Bağlı değil"}</span>
    <div class="foot">
      <span class="mono">Mouse · {app.device?.transport ?? "—"}</span>
      <button class="btn sm ghost" onclick={() => app.openDevice("general")}><Icon name="tune" size={16} />Ayarlar</button>
    </div>
  </div>

  <div class="card usage">
    <div class="eyebrow mono-eyebrow">Sistem kullanımı</div>
    <div class="nums">
      <div><span>CPU</span><b>{usage ? `${Math.round(usage.cpu)}%` : "—"}</b></div>
      <div><span>RAM</span><b>{usage ? `${gb(usage.memUsed)}/${Math.round(usage.memTotal / 1024 ** 3)}GB` : "—"}</b></div>
    </div>
  </div>
</div>

<style>
  .page-head {
    margin-bottom: 28px;
  }
  h1 {
    font-size: 34px;
    font-weight: 800;
  }
  .page-head p {
    margin: 6px 0 0;
    color: var(--text-2);
  }
  .grid-devices {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 360px));
    gap: 18px;
    align-items: start;
  }
  .device {
    padding: 16px 16px 14px;
  }
  .shot {
    position: relative;
    display: grid;
    place-items: center;
    height: 260px;
    margin-bottom: 18px;
    border: 1px solid var(--line-soft);
    border-radius: var(--r);
    background: var(--stage);
    overflow: hidden;
  }
  /* Hawk Hub'daki gibi: fare üstten, kartın altından taşar. */
  .shot img {
    position: absolute;
    left: 50%;
    top: 10%;
    height: 118%;
    translate: -50% 0;
  }
  .shot :global(svg) {
    height: 60%;
  }
  .batt {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 8px;
    border-radius: 6px;
    background: var(--surface-2);
    font-family: var(--mono);
    font-size: 12px;
    font-weight: 700;
  }
  h2 {
    font-size: 21px;
  }
  .state {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    margin: 6px 0 10px;
    color: var(--text-2);
    font-size: 12.5px;
  }
  .state i {
    width: 7px;
    height: 7px;
    border-radius: 2px;
    background: var(--text-3);
  }
  .state.on i {
    background: var(--ok);
  }
  .foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 10px;
    border-top: 1px solid var(--line);
  }
  .foot .btn {
    border-color: var(--line);
  }
  .mono {
    font-family: var(--mono);
    font-size: 13px;
    color: var(--text-2);
  }
  .usage {
    padding: 18px;
  }
  .mono-eyebrow {
    font-family: var(--mono);
    letter-spacing: 0.12em;
  }
  .nums {
    display: flex;
    gap: 26px;
  }
  .nums div {
    display: flex;
    flex-direction: column;
  }
  .nums span {
    color: var(--text-3);
    font-size: 10.5px;
  }
  .nums b {
    font-family: var(--mono);
    font-size: 20px;
  }
</style>
