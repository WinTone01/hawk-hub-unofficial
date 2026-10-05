<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import { api } from "../lib/api";
  import { ALL_KEYS, KEYMAP, keyLabel } from "../lib/keys";
  import { app, BUTTON_LABELS } from "../lib/state.svelte";
  import type { ButtonId, Macro, MacroStep } from "../lib/types";

  const MAX_STEPS = 25;
  // Kablosuz makro yüklemesi sayfalar arasında 1 sn bekler (fare uyumamalı).
  const wireless = $derived(app.device?.transport.includes("2.4") ?? false);
  const s = $derived(app.settings!);

  function freeSlot(): number {
    for (let i = 1; i <= 32; i++) if (!app.settings!.macros.some((m) => m.id === i)) return i;
    return 1;
  }

  // Düzenleyici durumu
  let editingId = $state<number | null>(null);
  let name = $state("");
  let slot = $state(freeSlot());
  let repeat = $state(1);
  let target = $state<ButtonId>("back");
  let steps = $state<MacroStep[]>([]);
  let recording = $state(false);
  let lastAt = 0;
  let addKey = $state(4);

  const totalMs = $derived(steps.reduce((a, st) => a + st.delay, 0));
  // Cihaz sınırı: her adım 2 olay, >127 ms bekleme +1 olay; en fazla 50 olay.
  const events = $derived(steps.reduce((a, st) => a + (st.delay > 127 ? 3 : 2), 0));

  function newMacro() {
    editingId = null;
    name = "";
    slot = freeSlot();
    repeat = 1;
    steps = [];
    recording = false;
  }

  function edit(m: Macro) {
    editingId = m.id;
    name = m.name;
    slot = m.id;
    repeat = m.repeat;
    steps = m.steps.map((x) => ({ ...x }));
    target = s.buttons.find((b) => b.macroId === m.id)?.button ?? target;
    recording = false;
  }

  function toggleRecord() {
    recording = !recording;
    lastAt = 0;
  }

  function onKey(e: KeyboardEvent) {
    if (!recording || e.repeat) return;
    e.preventDefault();
    const key = KEYMAP[e.code];
    if (key == null) return app.toast(`${e.code} makroda desteklenmiyor (değiştirici tuşlar desteklenmez)`, "err");
    const now = performance.now();
    if (steps.length && lastAt) steps[steps.length - 1].delay = Math.min(10000, Math.round(now - lastAt));
    lastAt = now;
    steps.push({ key, delay: 50 });
    if (steps.length >= MAX_STEPS) {
      recording = false;
      app.toast("25 adım sınırına ulaşıldı", "info");
    }
  }

  function move(i: number, d: number) {
    const j = i + d;
    [steps[i], steps[j]] = [steps[j], steps[i]];
  }

  async function upload() {
    recording = false;
    if (!steps.length) return app.toast("Önce adım ekleyin", "err");
    if (events > 50) return app.toast("Makro 50 olay sınırını aşıyor; adım veya 127 ms üstü bekleme sayısını azaltın", "err");
    if (
      target === "left" &&
      !(await app.confirm("Sol tıka makro atamak fareyi kullanılmaz hale getirebilir.", { title: "Sol tıka makro atansın mı?", confirmLabel: "Ata", danger: true }))
    )
      return;
    const definition: Macro = { id: slot, name: name.trim(), repeat, mode: 0, steps: $state.snapshot(steps) };
    if (await app.act(() => api.uploadMacro(target, definition), `Makro ${slot} yüklendi → ${BUTTON_LABELS[target]}`)) {
      editingId = slot;
    }
  }

  async function remove(m: Macro) {
    if (!(await app.confirm(`Bağlı tuşlar varsayılan fonksiyonlarına döner.`, { title: `“${m.name || "Makro " + m.id}” silinsin mi?`, confirmLabel: "Sil", danger: true }))) return;
    if (await app.act(() => api.deleteMacro(m.id), `Makro ${m.id} silindi`)) if (editingId === m.id) newMacro();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="layout rise">
  <div class="card library">
    <div class="card-head">
      <h3>Kütüphane</h3>
      <button class="btn sm" onclick={newMacro}><Icon name="plus" size={14} />Yeni</button>
    </div>
    {#if s.macros.length === 0}
      <p class="muted">Fareye yüklenmiş makro yok.</p>
    {/if}
    {#each s.macros as m (m.id)}
      {@const bound = s.buttons.filter((b) => b.macroId === m.id)}
      <div class="item" class:active={editingId === m.id}>
        <button class="item-main" onclick={() => edit(m)}>
          <span class="slot">{m.id}</span>
          <span class="item-text">
            <b>{m.name || "Makro"}</b>
            <small>{m.steps.length} adım · {bound.length ? bound.map((b) => BUTTON_LABELS[b.button]).join(", ") : "bağlı değil"}</small>
          </span>
        </button>
        <button class="icon-btn danger" title="Sil" onclick={() => remove(m)}><Icon name="trash" size={16} /></button>
      </div>
    {/each}
  </div>

  <div class="stack">
    <div class="card">
      <div class="card-head">
        <h3>{editingId != null ? `Makro ${editingId} düzenleniyor` : "Yeni makro"}</h3>
        <span class="sub">{steps.length}/{MAX_STEPS} adım · {events}/50 olay · ≈{(totalMs / 1000).toFixed(2)} s</span>
      </div>
      <div class="form">
        <label class="field grow">Ad<input type="text" bind:value={name} placeholder="ör. Selamlama" maxlength="30" /></label>
        <label class="field">Yuva<input type="number" bind:value={slot} min="1" max="32" /></label>
        <label class="field">Tekrar<input type="number" bind:value={repeat} min="1" max="255" /></label>
        <label class="field">Atanacak tuş
          <select bind:value={target}>
            {#each Object.entries(BUTTON_LABELS) as [id, label] (id)}<option value={id}>{label}</option>{/each}
          </select>
        </label>
      </div>

      <div class="toolbar">
        <button class="btn" class:rec={recording} onclick={toggleRecord}>
          <Icon name={recording ? "stop" : "record"} size={16} />{recording ? "Kaydı durdur" : "Kaydet"}
        </button>
        <div class="manual">
          <select bind:value={addKey}>
            {#each ALL_KEYS as k (k.usage)}<option value={k.usage}>{k.label}</option>{/each}
          </select>
          <button class="btn" disabled={steps.length >= MAX_STEPS} onclick={() => steps.push({ key: addKey, delay: 50 })}>
            <Icon name="plus" size={16} />Ekle
          </button>
        </div>
        <button class="btn ghost" disabled={!steps.length} onclick={() => (steps = [])}>Temizle</button>
      </div>
      {#if recording}
        <div class="rec-hint"><span class="rec-dot"></span>Kayıt yapılıyor — tuşlara basın. Tuşlar arasındaki süreler bekleme olarak kaydedilir.</div>
      {/if}

      {#if steps.length}
        <div class="timeline">
          {#each steps as st, i (i)}
            <span class="keycap">{keyLabel(st.key)}</span>
            {#if i < steps.length - 1 || st.delay}
              <span class="gap" style="width:{Math.min(90, 10 + Math.sqrt(st.delay) * 2.4)}px" title="{st.delay} ms"><i>{st.delay}</i></span>
            {/if}
          {/each}
        </div>

        <div class="steps">
          {#each steps as st, i (i)}
            <div class="step">
              <span class="n">{i + 1}</span>
              <select bind:value={st.key}>
                {#each ALL_KEYS as k (k.usage)}<option value={k.usage}>{k.label}</option>{/each}
              </select>
              <label class="delay">
                <input type="number" min="0" max="10000" bind:value={st.delay} />
                <span>ms</span>
              </label>
              <div class="step-actions">
                <button class="icon-btn" title="Yukarı" disabled={i === 0} onclick={() => move(i, -1)}><Icon name="up" size={15} /></button>
                <button class="icon-btn" title="Aşağı" disabled={i === steps.length - 1} onclick={() => move(i, 1)}><Icon name="down" size={15} /></button>
                <button class="icon-btn danger" title="Sil" onclick={() => steps.splice(i, 1)}><Icon name="x" size={15} /></button>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="empty">
          <Icon name="keyboard" size={30} stroke={1.4} />
          <p>“Kaydet”e basıp klavyede tuşlara basın ya da listeden tuş ekleyin.</p>
        </div>
      {/if}

      {#if wireless}
        <div class="notice">
          <Icon name="wifi" size={16} />
          <span>Kablosuz yükleme yaklaşık 3 saniye sürer; bu sırada fareyi hafifçe oynatın ki uykuya geçmesin.</span>
        </div>
      {/if}
      <button class="btn primary upload" disabled={!steps.length || app.busy > 0} onclick={upload}>
        <Icon name="upload" size={16} />Fareye yükle ve {BUTTON_LABELS[target]} tuşuna ata
      </button>
      <p class="muted layout-note">
        Makrolar fiziksel tuş konumlarını kaydeder; yazılan karakter bilgisayarın klavye düzenine göre olur.
      </p>
    </div>
  </div>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 18px;
    align-items: start;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px;
    border-radius: 11px;
    border: 1px solid transparent;
  }
  .item.active {
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    background: linear-gradient(90deg, var(--accent-soft), transparent);
  }
  .item-main {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 6px;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .slot {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--warn) 12%, transparent);
    font-family: var(--mono);
    font-weight: 700;
    color: var(--warn);
  }
  .item-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .item-text small {
    color: var(--muted);
    font-size: 11.5px;
  }
  .form {
    display: grid;
    grid-template-columns: 2fr 0.7fr 0.7fr 1.3fr;
    gap: 12px;
  }
  .toolbar {
    display: flex;
    gap: 10px;
    margin: 18px 0 12px;
    flex-wrap: wrap;
  }
  .manual {
    display: flex;
    gap: 6px;
  }
  .manual select {
    width: 130px;
  }
  .rec {
    border-color: transparent;
    color: #fff;
    background: var(--danger);
    animation: blink 1.2s ease-in-out infinite;
  }
  @keyframes blink {
    50% {
      box-shadow: 0 0 22px color-mix(in srgb, var(--danger) 50%, transparent);
    }
  }
  .rec-hint {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-bottom: 12px;
    color: var(--muted);
    font-size: 12.5px;
  }
  .rec-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--accent);
    animation: blink 1s infinite;
  }
  .timeline {
    perspective: 700px;
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    row-gap: 10px;
    padding: 16px;
    margin-bottom: 14px;
    border-radius: 12px;
    background: var(--bg-sunk);
    border: 1px solid var(--line);
  }
  .gap {
    position: relative;
    height: 2px;
    background: repeating-linear-gradient(90deg, var(--line-hi) 0 4px, transparent 4px 7px);
    margin: 0 3px;
  }
  .gap i {
    position: absolute;
    top: -16px;
    left: 50%;
    transform: translateX(-50%);
    font-style: normal;
    font-size: 10px;
    font-family: var(--mono);
    color: var(--muted);
  }
  .steps {
    display: flex;
    flex-direction: column;
    gap: 5px;
    max-height: 320px;
    overflow-y: auto;
    padding-right: 4px;
  }
  .step {
    display: grid;
    grid-template-columns: 26px 1fr 130px auto;
    align-items: center;
    gap: 10px;
    padding: 5px 8px;
    border-radius: 9px;
    background: var(--bg-sunk);
  }
  .step select,
  .step input {
    padding: 5px 8px;
  }
  .n {
    color: var(--muted);
    font-family: var(--mono);
    font-size: 12px;
    text-align: center;
  }
  .delay {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    font-size: 12px;
  }
  .step-actions {
    display: flex;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 30px 20px;
    border: 1px dashed var(--line-hi);
    border-radius: 12px;
    color: var(--muted);
    text-align: center;
  }
  .empty p {
    margin: 0;
    font-size: 13px;
  }
  .upload {
    width: 100%;
    margin-top: 16px;
    padding: 12px;
  }
  .notice {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    margin-top: 16px;
    padding: 10px 12px;
    border-radius: 10px;
    font-size: 12.5px;
  }
  .notice {
    border: 1px solid var(--line-hi);
    background: var(--bg-sunk);
  }
  .notice :global(svg) {
    flex: none;
    margin-top: 1px;
    color: var(--muted);
  }
  .layout-note {
    margin: 10px 0 0;
    font-size: 12px;
  }
  @media (max-width: 1000px) {
    .layout {
      grid-template-columns: 1fr;
    }
    .form {
      grid-template-columns: 1fr 1fr;
    }
  }
</style>
