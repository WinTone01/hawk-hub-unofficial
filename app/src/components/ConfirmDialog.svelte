<script lang="ts">
  // app.confirm() ile açılan onay diyaloğu. Esc iptal, Enter onay.
  import { fade, scale } from "svelte/transition";
  import { app } from "../lib/state.svelte";
  import Icon from "./Icon.svelte";

  let confirmBtn = $state<HTMLButtonElement>();
  $effect(() => {
    if (app.dialog) queueMicrotask(() => confirmBtn?.focus());
  });

  function onKey(e: KeyboardEvent) {
    if (!app.dialog) return;
    if (e.key === "Escape") app.dialog.resolve(false);
  }
</script>

<svelte:window onkeydown={onKey} />

{#if app.dialog}
  {@const d = app.dialog}
  <div class="backdrop" transition:fade={{ duration: 140 }} onclick={() => d.resolve(false)} role="presentation"></div>
  <div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="dlg-title" transition:scale={{ start: 0.96, duration: 160 }}>
    <span class="ico" class:danger={d.danger}><Icon name={d.danger ? "trash" : "info"} size={20} /></span>
    <h3 id="dlg-title">{d.title}</h3>
    <p>{d.message}</p>
    <div class="actions">
      <button class="btn ghost" onclick={() => d.resolve(false)}>Vazgeç</button>
      <button class="btn {d.danger ? 'danger-solid' : 'primary'}" bind:this={confirmBtn} onclick={() => d.resolve(true)}>
        {d.confirmLabel}
      </button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
    background: var(--backdrop);
    backdrop-filter: blur(6px);
  }
  .dialog {
    position: fixed;
    left: 50%;
    top: 50%;
    z-index: 81;
    width: min(420px, calc(100vw - 40px));
    translate: -50% -50%;
    padding: 24px;
    border-radius: var(--r-lg);
    border: 1px solid var(--line-hi);
    background: var(--surface-2);
    box-shadow: var(--shadow);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    margin-bottom: 14px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--info) 14%, transparent);
    color: var(--info);
  }
  .ico.danger {
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    color: var(--danger);
  }
  h3 {
    font-size: 17px;
    margin-bottom: 6px;
  }
  p {
    margin: 0 0 22px;
    color: var(--text-2);
    line-height: 1.55;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
  .danger-solid {
    border-color: transparent;
    background: var(--danger);
    color: #fff;
  }
</style>
