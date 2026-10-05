<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import Mouse3D from "../components/Mouse3D.svelte";
  import { fromDevice } from "../lib/ledColor";
  import VariantPicker from "../components/VariantPicker.svelte";
  import { api } from "../lib/api";
  import { app, BUTTON_LABELS, MACRO_CODE } from "../lib/state.svelte";
  import type { ButtonId, FunctionDef, Macro } from "../lib/types";

  const s = $derived(app.settings!);
  let selected = $state<ButtonId>("back");
  let hovered = $state<ButtonId | null>(null);
  let query = $state("");

  const current = $derived(s.buttons.find((b) => b.button === selected)!);
  const DEFAULTS: Record<ButtonId, number> = { left: 2, right: 3, middle: 4, back: 5, forward: 6 };

  const groups = $derived.by(() => {
    const q = query.trim().toLocaleLowerCase("tr");
    const out = new Map<string, FunctionDef[]>();
    for (const f of app.catalog!.functions) {
      if (q && !f.label.toLocaleLowerCase("tr").includes(q) && !f.id.includes(q)) continue;
      out.set(f.group, [...(out.get(f.group) ?? []), f]);
    }
    return [...out];
  });

  async function assign(f: FunctionDef) {
    if (current.macroId == null && current.code === f.code) return;
    if (
      selected === "left" &&
      f.code !== 2 &&
      !(await app.confirm("Sol tıkı değiştirmek fareyi kullanılmaz hale getirebilir. Sol tık olarak başka bir tuş kalmazsa geri almak için klavye gerekir.", { title: "Sol tık değiştirilsin mi?", confirmLabel: "Değiştir", danger: true }))
    )
      return;
    app.act(() => api.setButton(selected, f.code), `${BUTTON_LABELS[selected]} → ${f.label}`);
  }

  async function bindMacro(m: Macro) {
    if (
      selected === "left" &&
      !(await app.confirm("Sol tıka makro atamak fareyi kullanılmaz hale getirebilir.", { title: "Sol tıka makro atansın mı?", confirmLabel: "Ata", danger: true }))
    )
      return;
    app.act(() => api.uploadMacro(selected, m), `${BUTTON_LABELS[selected]} → Makro ${m.id}`);
  }
</script>

<div class="layout rise">
  <div class="card mouse-card">
    <Mouse3D
      mode={s.ledMode}
      color={fromDevice(s.ledColor)}
      brightness={s.ledBrightness}
      speed={s.ledSpeed}
      {selected}
      bind:hovered
      onselect={(b) => (selected = b)}
      height={440}
      autoRotate={false}
      distance={2.4}
    />
    <VariantPicker />
    <div class="legend">
      {#each s.buttons as b (b.button)}
        <button
          class="legend-row"
          class:sel={selected === b.button}
          class:hov={hovered === b.button}
          onclick={() => (selected = b.button)}
          onmouseenter={() => (hovered = b.button)}
          onmouseleave={() => (hovered = null)}
        >
          <span class="muted">{BUTTON_LABELS[b.button]}</span>
          <span class:macro={b.code === MACRO_CODE} class:changed={b.macroId == null && b.code !== DEFAULTS[b.button]}>
            {app.functionLabel(b.code, b.macroId)}
          </span>
        </button>
      {/each}
    </div>
  </div>

  <div class="card picker">
    <div class="card-head">
      <div>
        <div class="eyebrow">Seçili tuş</div>
        <h2>{BUTTON_LABELS[selected]}</h2>
        <span class="sub">Şu an: <b>{app.functionLabel(current.code, current.macroId)}</b></span>
      </div>
      {#if current.macroId != null || current.code !== DEFAULTS[selected]}
        <button class="btn sm" onclick={() => app.act(() => api.setButton(selected, DEFAULTS[selected]), `${BUTTON_LABELS[selected]} varsayılana döndü`)}>
          <Icon name="reset" size={14} />Varsayılan
        </button>
      {/if}
    </div>

    <div class="search">
      <Icon name="search" size={16} />
      <input placeholder="Fonksiyon ara…" bind:value={query} />
    </div>

    <div class="groups">
      {#each groups as [group, fns] (group)}
        <div>
          <div class="eyebrow">{group}</div>
          <div class="fns">
            {#each fns as f (f.code)}
              <button class="fn" class:on={current.macroId == null && current.code === f.code} onclick={() => assign(f)}>
                {f.label}
              </button>
            {/each}
          </div>
        </div>
      {/each}

      <div>
        <div class="eyebrow">Makrolar</div>
        {#if s.macros.length}
          <div class="fns">
            {#each s.macros as m (m.id)}
              <button class="fn macro-fn" class:on={current.macroId === m.id} onclick={() => bindMacro(m)}>
                <Icon name="macro" size={14} />{m.id}. {m.name || "Makro"}
              </button>
            {/each}
          </div>
        {:else}
          <p class="muted">Henüz makro yok. <button class="link" onclick={() => app.openDevice("macros")}>Makro oluşturun →</button></p>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 380px 1fr;
    gap: 18px;
    align-items: start;
  }
  .mouse-card {
    background:
      radial-gradient(360px 420px at 50% 40%, color-mix(in srgb, var(--led) calc(var(--led-i, 0) * 16%), transparent), transparent 70%),
      var(--surface);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
    padding-top: 28px;
  }
  .legend {
    width: 100%;
    display: flex;
    flex-direction: column;
  }
  .legend-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 9px 12px;
    border: 0;
    border-radius: 9px;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .legend-row.hov {
    background: var(--hover);
  }
  .legend-row.sel {
    background: linear-gradient(90deg, var(--accent-soft), transparent);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .macro {
    color: var(--warn);
  }
  .changed {
    color: var(--info);
  }
  h2 {
    font-size: 28px;
    font-weight: 750;
    letter-spacing: -0.02em;
    margin: 2px 0 4px;
  }
  .search {
    position: relative;
    margin-bottom: 18px;
  }
  .search :global(svg) {
    position: absolute;
    left: 11px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--muted);
  }
  .search input {
    padding-left: 36px;
  }
  .groups {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .fns {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
  }
  .fn {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border-radius: 9px;
    border: 1px solid var(--line);
    background: var(--sunk);
    color: var(--text);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
  }
  .fn:hover {
    border-color: var(--line-hi);
  }
  .fn.on {
    border-color: transparent;
    background: var(--accent);
    color: var(--on-accent);
  }
  .macro-fn :global(svg) {
    color: var(--warn);
  }
  .link {
    border: 0;
    background: none;
    color: var(--accent);
    font: inherit;
    cursor: pointer;
    padding: 0;
  }
  @media (max-width: 1000px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>
