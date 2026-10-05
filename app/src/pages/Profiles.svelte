<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "../components/Icon.svelte";
  import ProfileAvatar from "../components/ProfileAvatar.svelte";
  import ProfileEditor from "../components/ProfileEditor.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { profileColor, profileGlyph } from "../lib/appIcons";
  import { api } from "../lib/api";
  import { app, DPI_COLORS } from "../lib/state.svelte";
  import type { AutoSwitch, Profile, ProfileMeta } from "../lib/types";

  let profiles = $state<Profile[]>([]);
  let newName = $state("");
  let importOpen = $state(false);
  let importName = $state("");
  let importJson = $state("");
  let auto = $state<AutoSwitch>({ enabled: false, defaultProfile: null });
  let editing = $state<Profile | null>(null);
  const linkedCount = $derived(profiles.filter((p) => p.meta.app).length);
  const ledLabel = { off: "Kapalı", neon: "Neon", static: "Sabit", breathing: "Nefes" } as const;

  onMount(refresh);

  async function refresh() {
    try {
      [profiles, auto] = await Promise.all([api.listProfiles(), api.getAutoSwitch()]);
    } catch (e) {
      app.toast(String(e), "err");
    }
  }

  async function setAuto(next: AutoSwitch, ok: string) {
    try {
      auto = await api.setAutoSwitch(next);
      app.toast(ok);
    } catch (e) {
      app.toast(String(e), "err");
    }
  }

  async function saveMeta(name: string, meta: ProfileMeta) {
    if (await run(() => api.setProfileMeta(name, meta), `“${name}” güncellendi`)) editing = null;
  }

  async function run(fn: () => Promise<Profile[]>, ok: string) {
    try {
      profiles = await fn();
      app.toast(ok);
      return true;
    } catch (e) {
      app.toast(String(e), "err");
      return false;
    }
  }

  async function save() {
    const n = newName.trim();
    if (!n) return;
    if (profiles.some((p) => p.name === n) && !(await app.confirm("Mevcut ayarlar bu profilin yerine kaydedilecek.", { title: `“${n}” üzerine yazılsın mı?`, confirmLabel: "Üzerine yaz" }))) return;
    if (await run(() => api.saveProfile(n), `“${n}” kaydedildi`)) newName = "";
  }

  async function removeProfile(name: string) {
    if (!(await app.confirm("Bu işlem geri alınamaz.", { title: `“${name}” silinsin mi?`, confirmLabel: "Sil", danger: true }))) return;
    if (await run(() => api.deleteProfile(name), `“${name}” silindi`)) auto = await api.getAutoSwitch();
  }

  async function exportProfile(name: string) {
    try {
      await navigator.clipboard.writeText(await api.exportProfile(name));
      app.toast(`“${name}” JSON olarak panoya kopyalandı`);
    } catch (e) {
      app.toast(String(e), "err");
    }
  }

  async function doImport() {
    if (await run(() => api.importProfile(importName.trim(), importJson), `“${importName.trim()}” içe aktarıldı`)) {
      importOpen = false;
      importName = importJson = "";
    }
  }

  const fmtDate = (s: number) => new Date(s * 1000).toLocaleString("tr-TR", { dateStyle: "medium", timeStyle: "short" });
</script>

<div class="stack rise">
  <div class="card save">
    <div>
      <h3>Mevcut ayarları profil olarak kaydet</h3>
      <span class="muted">DPI, polling, LED, tuş atamaları ve makrolar birlikte kaydedilir.</span>
    </div>
    <form class="save-form" onsubmit={(e) => (e.preventDefault(), save())}>
      <input type="text" placeholder="Profil adı (ör. FPS, Tasarım)" bind:value={newName} maxlength="40" />
      <button class="btn primary" disabled={!newName.trim()}><Icon name="save" size={16} />Kaydet</button>
    </form>
    <button class="btn ghost" onclick={() => (importOpen = !importOpen)}><Icon name="import" size={16} />İçe aktar</button>
  </div>

  <div class="card auto">
    <span class="auto-ico" class:on={auto.enabled}><Icon name="bolt" size={20} /></span>
    <div class="auto-main">
      <Toggle
        checked={auto.enabled}
        label="Program öne gelince profili otomatik uygula"
        hint={linkedCount
          ? `${linkedCount} profil bir programa bağlı. Başka bir programa geçince varsayılan profile dönülür.`
          : "Bir profili düzenleyip ona oyun veya program bağlayın."}
        onchange={(on) => setAuto({ ...auto, enabled: on }, `Otomatik geçiş ${on ? "açık" : "kapalı"}`)}
      />
    </div>
    <label class="field default">
      Varsayılan profil
      <select
        value={auto.defaultProfile ?? ""}
        onchange={(e) => {
          const v = e.currentTarget.value || null;
          setAuto({ ...auto, defaultProfile: v }, v ? `Varsayılan profil “${v}”` : "Varsayılana dönüş kapalı");
        }}
      >
        <option value="">Dönme (son ayarlar kalsın)</option>
        {#each profiles as p (p.name)}<option value={p.name}>{p.name}</option>{/each}
      </select>
    </label>
  </div>

  {#if importOpen}
    <div class="card import">
      <label class="field">Profil adı<input type="text" bind:value={importName} maxlength="40" /></label>
      <label class="field">JSON<textarea rows="7" bind:value={importJson} placeholder="Dışa aktarılan profil JSON'unu yapıştırın"></textarea></label>
      <div class="row-end">
        <button class="btn ghost" onclick={() => (importOpen = false)}>Vazgeç</button>
        <button class="btn primary" disabled={!importName.trim() || !importJson.trim()} onclick={doImport}>İçe aktar</button>
      </div>
    </div>
  {/if}

  {#if profiles.length === 0}
    <div class="card empty">
      <Icon name="profiles" size={32} stroke={1.4} />
      <p>Henüz profil yok. Ayarlarınızı farklı oyunlar veya işler için profil olarak saklayabilirsiniz.</p>
    </div>
  {:else}
    <div class="profiles">
      {#each profiles as p (p.name)}
        {@const ps = p.settings}
        {@const g = profileGlyph(p)}
        {@const color = profileColor(p)}
        <div class="card profile" style="--pc:{color}">
          <div class="top">
            <ProfileAvatar {color} glyph={"glyph" in g ? g.glyph : null} app={"app" in g ? g.app : null} size={44} />
            <div class="title">
              <h3>{p.name}</h3>
              <span class="muted">{fmtDate(p.modified)}</span>
            </div>
            {#if auto.enabled && app.autoProfile === p.name}<span class="live">Otomatik</span>{/if}
            {#if auto.defaultProfile === p.name}<span class="tag">Varsayılan</span>{/if}
          </div>
          {#if p.meta.app}
            <div class="linked" title={p.meta.app.path}><Icon name="link" size={13} />{p.meta.app.name} öne gelince</div>
          {/if}
          <div class="facts">
            <span><i style="background:{DPI_COLORS[ps.dpiStage - 1]}"></i>{ps.dpiValues[ps.dpiStage - 1]} DPI</span>
            <span>{ps.pollingHz} Hz</span>
            <span>{ledLabel[ps.ledMode]}</span>
            <span>{ps.macros.length} makro</span>
          </div>
          <div class="dpis">
            {#each ps.dpiValues as v, i (i)}<span class:on={ps.dpiStage === i + 1} style="--c:{DPI_COLORS[i]}">{v}</span>{/each}
          </div>
          <div class="actions">
            <button class="btn primary sm" onclick={() => app.act(() => api.applyProfile(p.name), `“${p.name}” uygulandı`)}>
              <Icon name="play" size={14} />Uygula
            </button>
            <button class="icon-btn" title="Renk, simge ve program" onclick={() => (editing = p)}><Icon name="edit" size={16} /></button>
            <button class="icon-btn" title="JSON olarak kopyala" onclick={() => exportProfile(p.name)}><Icon name="copy" size={16} /></button>
            <button
              class="icon-btn danger"
              title="Sil"
              onclick={() => removeProfile(p.name)}
            >
              <Icon name="trash" size={16} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if editing}
  {@const p = editing}
  <ProfileEditor profile={p} onclose={() => (editing = null)} onsave={(meta) => saveMeta(p.name, meta)} />
{/if}

<style>
  .auto {
    display: flex;
    align-items: center;
    gap: 18px;
    flex-wrap: wrap;
  }
  .auto-ico {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 12px;
    background: var(--hover);
    color: var(--text-3);
    transition: background 0.2s, color 0.2s;
  }
  .auto-ico.on {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .auto-main {
    flex: 1;
    min-width: 280px;
  }
  .default {
    min-width: 220px;
  }
  .title {
    flex: 1;
    min-width: 0;
  }
  .title h3 {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .live,
  .tag {
    padding: 3px 9px;
    border-radius: 999px;
    font-size: 10.5px;
    font-weight: 700;
    white-space: nowrap;
  }
  .live {
    background: color-mix(in srgb, var(--pc) 22%, transparent);
    color: var(--text);
  }
  .tag {
    border: 1px solid var(--line);
    color: var(--text-3);
  }
  .linked {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    align-self: flex-start;
    max-width: 100%;
    padding: 4px 10px;
    border-radius: 8px;
    background: color-mix(in srgb, var(--pc) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--pc) 30%, transparent);
    font-size: 12px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .profile {
    border-top: 2px solid color-mix(in srgb, var(--pc) 70%, transparent);
  }
  .save {
    display: flex;
    align-items: center;
    gap: 20px;
    flex-wrap: wrap;
  }
  .save > div {
    flex: 1;
    min-width: 220px;
  }
  .save-form {
    display: flex;
    gap: 8px;
    flex: 1;
    min-width: 300px;
  }
  .import {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .row-end {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 46px;
    color: var(--muted);
    text-align: center;
  }
  .profiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(270px, 1fr));
    gap: 16px;
  }
  .profile {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .facts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .facts span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 9px;
    border-radius: 999px;
    background: var(--bg-sunk);
    border: 1px solid var(--line);
    font-size: 12px;
  }
  .facts i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .dpis {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 4px;
  }
  .dpis span {
    padding: 4px 0;
    border-radius: 6px;
    border-bottom: 2px solid color-mix(in srgb, var(--c) 40%, transparent);
    background: var(--bg-sunk);
    font-family: var(--mono);
    font-size: 11px;
    text-align: center;
    color: var(--muted);
  }
  .dpis span.on {
    color: var(--text);
    border-bottom-color: var(--c);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: auto;
  }
  .actions .btn {
    margin-right: auto;
  }
</style>
