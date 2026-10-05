<script lang="ts">
  // Farenin gövde rengi (yalnızca görsel; cihaz rengini bildirmez, seçim hatırlanır).
  import { mouseImage, VARIANTS } from "../lib/assets";
  import { app } from "../lib/state.svelte";

  let { compact = false }: { compact?: boolean } = $props();
</script>

{#if mouseImage("black")}
  <div class="variants" class:compact role="radiogroup" aria-label="Fare rengi">
    {#each VARIANTS as v (v.id)}
      <button
        role="radio"
        aria-checked={app.variant === v.id}
        class:on={app.variant === v.id}
        title="Fare rengi: {v.label}"
        onclick={() => app.setVariant(v.id)}
      >
        <span class="dot" style="background:{v.hex}"></span>
        {#if !compact}{v.label}{/if}
      </button>
    {/each}
  </div>
{/if}

<style>
  .variants {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  button {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 5px 11px 5px 7px;
    border-radius: 999px;
    border: 1px solid var(--line-hi);
    background: none;
    color: var(--muted);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }
  button:hover {
    color: var(--text);
  }
  button.on {
    color: var(--text);
    border-color: var(--accent);
  }
  .dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1px solid rgba(128, 128, 128, 0.45);
  }
  .compact {
    gap: 4px;
  }
  .compact button {
    padding: 3px;
  }
  .compact .dot {
    width: 13px;
    height: 13px;
  }
</style>
