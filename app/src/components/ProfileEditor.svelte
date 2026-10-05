<script lang="ts">
  // Profilin rengi, simgesi ve bağlı programı.
  import { untrack } from "svelte";
  import { fade, scale, slide } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import ProfileAvatar from "./ProfileAvatar.svelte";
  import { api, pickProgram } from "../lib/api";
  import { PROFILE_COLORS, PROFILE_GLYPHS, profileColor, profileGlyph, remember } from "../lib/appIcons";
  import { app } from "../lib/state.svelte";
  import type { AppEntry, Profile, ProfileMeta } from "../lib/types";

  let { profile, onclose, onsave }: { profile: Profile; onclose: () => void; onsave: (meta: ProfileMeta) => void } =
    $props();

  // Taslak açılıştaki değerden başlar; Kaydet'e basılana kadar profile yazılmaz.
  let draft = $state<ProfileMeta>(untrack(() => structuredClone($state.snapshot(profile.meta))));
  const preview = $derived({ ...profile, meta: draft });
  const shown = $derived(profileGlyph(preview));

  let apps = $state<AppEntry[] | null>(null);
  let listOpen = $state(false);
  let loading = $state(false);
  let query = $state("");
  const filtered = $derived(
    (apps ?? []).filter((a) => !query || (a.name + a.path).toLowerCase().includes(query.toLowerCase())),
  );

  async function openList() {
    listOpen = !listOpen;
    if (!listOpen || apps) return;
    loading = true;
    try {
      apps = await api.runningApps();
      remember(apps);
    } catch (e) {
      app.toast(String(e), "err");
    } finally {
      loading = false;
    }
  }

  function link(a: { name: string; path: string }) {
    draft.app = { name: a.name, path: a.path };
    if (draft.icon == null) draft.icon = "app";
    listOpen = false;
  }

  async function fromFile() {
    try {
      const path = await pickProgram();
      if (!path) return;
      const entry = await api.describeApp(path);
      remember([entry]);
      link(entry);
    } catch (e) {
      app.toast(String(e), "err");
    }
  }

  function unlink() {
    draft.app = null;
    if (draft.icon === "app") draft.icon = null;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="backdrop" transition:fade={{ duration: 140 }} onclick={onclose} role="presentation"></div>
<div class="editor" role="dialog" aria-modal="true" aria-label="Profili düzenle" transition:scale={{ start: 0.96, duration: 160 }}>
  <header>
    <ProfileAvatar color={profileColor(preview)} glyph={"glyph" in shown ? shown.glyph : null} app={"app" in shown ? shown.app : null} size={52} />
    <div>
      <h3>{profile.name}</h3>
      <span class="muted">Görünüm ve otomatik geçiş</span>
    </div>
    <button class="icon-btn close" title="Kapat" onclick={onclose}><Icon name="x" size={18} /></button>
  </header>

  <div class="body">
    <section>
      <h4>Renk</h4>
      <div class="swatches">
        <button class="sw auto" class:on={draft.color == null} title="Profilin LED rengi" onclick={() => (draft.color = null)}>
          <span style="--c:{profile.settings.ledColor}"></span>LED
        </button>
        {#each PROFILE_COLORS as c (c)}
          <button class="sw" class:on={draft.color === c} style="--c:{c}" title={c} onclick={() => (draft.color = c)} aria-label={c}></button>
        {/each}
        <label class="sw custom" title="Özel renk" style="--c:{draft.color ?? '#888'}">
          <input type="color" value={draft.color ?? "#3fa9ff"} oninput={(e) => (draft.color = e.currentTarget.value)} />
          <Icon name="plus" size={14} />
        </label>
      </div>
    </section>

    <section>
      <h4>Simge</h4>
      <div class="glyphs">
        {#if draft.app}
          <button class="glyph app" class:on={draft.icon === "app" || draft.icon == null} title="Programın simgesi" onclick={() => (draft.icon = "app")}>
            <ProfileAvatar color={profileColor(preview)} app={draft.app.path} size={28} />
          </button>
        {/if}
        {#each PROFILE_GLYPHS as g (g)}
          <button class="glyph" class:on={draft.icon === g} title={g} onclick={() => (draft.icon = g)}>
            <Icon name={g} size={20} />
          </button>
        {/each}
      </div>
    </section>

    <section>
      <h4>Bağlı program</h4>
      <p class="hint">Bu program öne geldiğinde profil otomatik uygulanır (Profiller sayfasında otomatik geçiş açık olmalı).</p>
      {#if draft.app}
        <div class="linked">
          <ProfileAvatar color={profileColor(preview)} app={draft.app.path} size={36} />
          <div class="meta">
            <b>{draft.app.name}</b>
            <span class="path" title={draft.app.path}>{draft.app.path}</span>
          </div>
          <button class="icon-btn danger" title="Bağlantıyı kaldır" onclick={unlink}><Icon name="x" size={16} /></button>
        </div>
      {/if}
      <div class="pick">
        <button class="btn sm" class:on={listOpen} onclick={openList}><Icon name="window" size={14} />Çalışan programlardan seç</button>
        <button class="btn sm ghost" onclick={fromFile}><Icon name="folder" size={14} />Dosyadan seç…</button>
      </div>
      {#if listOpen}
        <div class="list" transition:slide={{ duration: 160 }}>
          <div class="search">
            <Icon name="search" size={14} />
            <input type="text" placeholder="Program ara" bind:value={query} />
          </div>
          <div class="items">
            {#if loading}
              <span class="muted pad">Programlar listeleniyor…</span>
            {:else if filtered.length === 0}
              <span class="muted pad">Program bulunamadı.</span>
            {:else}
              {#each filtered as a (a.path)}
                <button class="item" class:on={draft.app?.path === a.path} onclick={() => link(a)} title={a.path}>
                  {#if a.icon}<img src={a.icon} alt="" />{:else}<span class="noimg"><Icon name="window" size={14} /></span>{/if}
                  <span class="name">{a.name}</span>
                  <span class="path">{a.path}</span>
                </button>
              {/each}
            {/if}
          </div>
        </div>
      {/if}
    </section>
  </div>

  <footer>
    <button class="btn ghost" onclick={onclose}>Vazgeç</button>
    <button class="btn primary" onclick={() => onsave($state.snapshot(draft))}><Icon name="check" size={16} />Kaydet</button>
  </footer>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 70;
    background: var(--backdrop);
    backdrop-filter: blur(6px);
  }
  .editor {
    position: fixed;
    left: 50%;
    top: 50%;
    z-index: 71;
    display: flex;
    flex-direction: column;
    width: min(560px, calc(100vw - 40px));
    max-height: calc(100vh - 60px);
    translate: -50% -50%;
    border-radius: var(--r-lg);
    border: 1px solid var(--line-hi);
    background: var(--surface-2);
    box-shadow: var(--shadow);
  }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 20px 22px 16px;
    border-bottom: 1px solid var(--line);
  }
  header div {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  header h3 {
    font-size: 17px;
  }
  .body {
    overflow-y: auto;
    padding: 6px 22px 10px;
  }
  section {
    padding: 14px 0;
  }
  section + section {
    border-top: 1px solid var(--line);
  }
  h4 {
    margin: 0 0 10px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-3);
  }
  .hint {
    margin: -4px 0 12px;
    color: var(--text-3);
    font-size: 12.5px;
  }

  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .sw {
    position: relative;
    width: 30px;
    height: 30px;
    padding: 0;
    border-radius: 9px;
    border: 2px solid transparent;
    background: var(--c);
    cursor: pointer;
    outline-offset: 2px;
    transition: transform 0.12s;
  }
  .sw:hover {
    transform: translateY(-1px);
  }
  .sw.on {
    border-color: var(--text);
    box-shadow: 0 0 14px -2px var(--c, #fff);
  }
  .sw.auto {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: auto;
    padding: 0 9px 0 6px;
    background: var(--sunk);
    border-color: var(--line);
    color: var(--text-2);
    font: inherit;
    font-size: 11px;
    font-weight: 700;
  }
  .sw.auto.on {
    border-color: var(--text);
    color: var(--text);
  }
  .sw.auto span {
    width: 14px;
    height: 14px;
    border-radius: 4px;
    background: var(--c);
  }
  .sw.custom {
    display: grid;
    place-items: center;
    background: conic-gradient(#ff4d5e, #ffb020, #5ee38a, #3fa9ff, #9b5cff, #ff5ca8, #ff4d5e);
    color: #fff;
  }
  .sw.custom input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }

  .glyphs {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(42px, 1fr));
    gap: 6px;
  }
  .glyph {
    display: grid;
    place-items: center;
    height: 42px;
    border-radius: 10px;
    border: 1px solid var(--line);
    background: var(--sunk);
    color: var(--text-2);
    cursor: pointer;
    transition: border-color 0.12s, color 0.12s, background 0.12s;
  }
  .glyph:hover {
    color: var(--text);
    border-color: var(--line-hi);
  }
  .glyph.on {
    color: var(--text);
    border-color: color-mix(in srgb, var(--accent) 70%, transparent);
    background: var(--accent-soft);
  }

  .linked {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    margin-bottom: 10px;
    border-radius: var(--r);
    border: 1px solid var(--line);
    background: var(--sunk);
  }
  .linked .meta {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .path {
    overflow: hidden;
    color: var(--text-3);
    font-size: 11.5px;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .pick {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .btn.on {
    border-color: color-mix(in srgb, var(--accent) 60%, transparent);
  }
  .list {
    margin-top: 10px;
    border-radius: var(--r);
    border: 1px solid var(--line);
    background: var(--sunk);
    overflow: hidden;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    border-bottom: 1px solid var(--line);
    color: var(--text-3);
  }
  .search input {
    flex: 1;
    padding: 10px 0;
    border: 0;
    background: none;
    box-shadow: none;
  }
  .items {
    display: flex;
    flex-direction: column;
    max-height: 240px;
    overflow-y: auto;
    padding: 4px;
  }
  .pad {
    padding: 14px;
  }
  .item {
    display: grid;
    grid-template-columns: 24px auto 1fr;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border: 0;
    border-radius: 8px;
    background: none;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .item:hover {
    background: var(--hover);
  }
  .item.on {
    background: var(--accent-soft);
  }
  .item img,
  .noimg {
    width: 24px;
    height: 24px;
    object-fit: contain;
  }
  .noimg {
    display: grid;
    place-items: center;
    border-radius: 6px;
    background: var(--hover);
    color: var(--text-3);
  }
  .item .name {
    font-weight: 600;
    white-space: nowrap;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 14px 22px 18px;
    border-top: 1px solid var(--line);
  }
</style>
