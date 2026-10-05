<script lang="ts">
  // Sürüklerken yerel değer gösterir, bırakınca (change) commit eder; cihaza her piksel için yazmayız.
  let {
    value,
    min,
    max,
    step = 1,
    format = (v: number) => String(v),
    ends = null,
    color = "var(--accent)",
    showValue = true,
    oncommit,
    oninput,
  }: {
    value: number;
    min: number;
    max: number;
    step?: number;
    format?: (v: number) => string;
    ends?: [string, string] | null;
    color?: string;
    showValue?: boolean;
    oncommit: (v: number) => void;
    oninput?: (v: number) => void;
  } = $props();

  let draft = $state<number | null>(null);
  const shown = $derived(draft ?? value);
  const pct = $derived(((shown - min) / (max - min)) * 100);
</script>

<div class="slider">
  <div class="track" style="--pct:{pct}%; --c:{color}">
    <input
      type="range"
      {min}
      {max}
      {step}
      value={shown}
      oninput={(e) => {
        draft = +e.currentTarget.value;
        oninput?.(draft);
      }}
      onchange={(e) => {
        const v = +e.currentTarget.value;
        draft = null;
        if (v !== value) oncommit(v);
      }}
    />
  </div>
  {#if showValue}<output>{format(shown)}</output>{/if}
</div>
{#if ends}
  <div class="ends" class:full={!showValue}><span>{ends[0]}</span><span>{ends[1]}</span></div>
{/if}

<style>
  .slider {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .track {
    flex: 1;
    position: relative;
    height: 22px;
    display: flex;
    align-items: center;
  }
  .track::before {
    content: "";
    position: absolute;
    inset: 9px 0;
    border-radius: 3px;
    background: var(--track);
  }
  .track::after {
    content: "";
    position: absolute;
    left: 0;
    top: 9px;
    bottom: 9px;
    width: var(--pct);
    border-radius: 3px;
    background: var(--c);
    pointer-events: none;
  }
  input {
    position: relative;
    z-index: 1;
    width: 100%;
    margin: 0;
    appearance: none;
    background: none;
    cursor: pointer;
  }
  input::-webkit-slider-thumb {
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #e3e8eb;
    border: 3px solid var(--sunk);
    box-shadow: 0 0 0 1px var(--line-hi), 0 2px 6px rgba(0, 0, 0, 0.3);
  }
  input::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #e3e8eb;
    border: 3px solid var(--sunk);
  }
  output {
    min-width: 64px;
    text-align: right;
    font-family: var(--mono);
    font-size: 13px;
    font-weight: 700;
    color: var(--text);
  }
  .ends.full {
    padding-right: 0;
  }
  .ends {
    display: flex;
    justify-content: space-between;
    margin-top: 2px;
    padding-right: 78px;
    font-family: var(--display);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-3);
  }
</style>
