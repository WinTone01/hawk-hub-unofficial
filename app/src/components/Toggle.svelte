<script lang="ts">
  let {
    checked,
    label,
    hint = "",
    onchange,
  }: { checked: boolean; label: string; hint?: string; onchange: (v: boolean) => void } = $props();
</script>

<label class="row">
  <span class="text">
    <b>{label}</b>
    {#if hint}<small>{hint}</small>{/if}
  </span>
  <input type="checkbox" {checked} onchange={(e) => onchange(e.currentTarget.checked)} />
  <span class="track"><span class="thumb"></span></span>
</label>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 14px 0;
    cursor: pointer;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  b {
    font-family: var(--display);
    font-weight: 600;
    font-size: 13.5px;
  }
  small {
    color: var(--text-3);
    font-size: 12px;
  }
  input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  .track {
    position: relative;
    flex: none;
    width: 46px;
    height: 26px;
    border-radius: 13px;
    background: var(--track);
    border: 1px solid var(--line-hi);
    transition: background 0.2s, border-color 0.2s;
  }
  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--text-3);
    transition: transform 0.25s cubic-bezier(0.3, 0.9, 0.3, 1), background 0.2s;
  }
  input:checked + .track {
    background: var(--accent);
    border-color: var(--accent);
  }
  input:checked + .track .thumb {
    transform: translateX(20px);
    background: var(--sunk);
  }
  input:focus-visible + .track {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
