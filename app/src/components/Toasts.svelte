<script lang="ts">
  import { fly } from "svelte/transition";
  import { app } from "../lib/state.svelte";
  import Icon from "./Icon.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each app.toasts as t (t.id)}
    <div class="toast {t.kind}" transition:fly={{ y: 12, duration: 180 }}>
      <Icon name={t.kind === "err" ? "x" : t.kind === "info" ? "info" : "check"} size={16} />
      <span>{t.text}</span>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 22px;
    bottom: 22px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 50;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    max-width: 400px;
    padding: 12px 16px;
    border-radius: 13px;
    background: var(--surface-2);
    backdrop-filter: blur(16px);
    border: 1px solid var(--line-hi);
    box-shadow: var(--shadow);
    font-size: 13px;
  }
  .ok :global(svg) {
    color: var(--ok);
  }
  .info :global(svg) {
    color: var(--info);
  }
  .err {
    border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  }
  .err :global(svg) {
    color: var(--accent);
  }
</style>
