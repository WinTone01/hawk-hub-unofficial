<script lang="ts" generics="T extends string | number">
  interface Option {
    value: T;
    label: string;
    hint?: string;
  }
  let { options, value, onchange }: { options: Option[]; value: T; onchange: (v: T) => void } = $props();
  const index = $derived(Math.max(0, options.findIndex((o) => o.value === value)));
</script>

<div class="seg" role="radiogroup" style="--n:{options.length}; --i:{index}">
  <span class="thumb" aria-hidden="true"></span>
  {#each options as o (o.value)}
    <button
      role="radio"
      aria-checked={o.value === value}
      class:active={o.value === value}
      onclick={() => o.value !== value && onchange(o.value)}
    >
      <span>{o.label}</span>
      {#if o.hint}<small>{o.hint}</small>{/if}
    </button>
  {/each}
</div>

<style>
  .seg {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--n), 1fr);
    padding: 4px;
    border-radius: var(--r-lg);
    background: var(--sunk);
    border: 1px solid var(--line);
  }
  /* Kayan seçim göstergesi */
  .thumb {
    position: absolute;
    top: 4px;
    bottom: 4px;
    left: 4px;
    width: calc((100% - 8px) / var(--n));
    transform: translateX(calc(100% * var(--i)));
    border-radius: var(--r);
    background: var(--accent);
    transition: transform 0.25s cubic-bezier(0.3, 0.9, 0.3, 1);
  }
  button {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 8px 6px;
    border: 0;
    background: none;
    color: var(--text-2);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
    transition: color 0.15s;
  }
  button:hover {
    color: var(--text);
  }
  button.active {
    color: var(--on-accent);
  }
  small {
    font-size: 10.5px;
    font-weight: 500;
    opacity: 0.7;
    font-family: var(--mono);
  }
</style>
