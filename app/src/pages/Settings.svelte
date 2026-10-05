<script lang="ts">
  // Uygulama ayarları, cihaz işlemleri, hakkında ve geliştirici paneli.
  import Icon from "../components/Icon.svelte";
  import Logo from "../components/Logo.svelte";
  import Segmented from "../components/Segmented.svelte";
  import VariantPicker from "../components/VariantPicker.svelte";
  import { api } from "../lib/api";
  import { app } from "../lib/state.svelte";
  import Developer from "./Developer.svelte";

  let devOpen = $state(false);

  async function reset() {
    if (
      await app.confirm("DPI, polling, ışık ve tuş atamaları fabrika değerlerine döner; makro atamaları kaldırılır.", {
        title: "Fare sıfırlansın mı?",
        confirmLabel: "Sıfırla",
        danger: true,
      })
    )
      app.act(() => api.reset(), "Fabrika ayarları yüklendi");
  }
</script>

<div class="page-head">
  <h1>Ayarlar</h1>
  <p>Uygulama görünümü, cihaz işlemleri ve geliştirici araçları.</p>
</div>

<div class="cols rise">
  <section class="card">
    <h3>Görünüm</h3>
    <div class="eyebrow top">Tema</div>
    <Segmented
      options={[
        { value: "dark", label: "Koyu" },
        { value: "light", label: "Açık" },
      ]}
      value={app.theme}
      onchange={(t) => app.setTheme(t)}
    />
    <div class="eyebrow top">Farenin rengi</div>
    <p class="muted">3D model ve görseller bu renkte gösterilir. Fare kendi rengini bildirmediği için buradan seçilir.</p>
    <VariantPicker />
  </section>

  <section class="card">
    <h3>Cihaz</h3>
    <div class="actions">
      <button class="btn" disabled={!app.device || app.syncing} onclick={() => app.syncFromDevice()}>
        <Icon name="refresh" size={16} />Ayarları fareden oku
      </button>
      <button class="btn" disabled={!app.device} onclick={() => app.act(() => api.applyAll(), "Tüm ayarlar fareye yazıldı")}>
        <Icon name="upload" size={16} />Tüm ayarları yeniden yaz
      </button>
      <button class="btn danger" disabled={!app.device} onclick={reset}>
        <Icon name="reset" size={16} />Fabrika ayarlarına dön
      </button>
    </div>
    <p class="muted">Ayarlar farenin kendi hafızasında saklanır; uygulama kapalıyken de geçerlidir.</p>
  </section>

  <section class="card about">
    <div class="brand">
      <Logo size={48} />
      <div>
        <h3>Hawk Hub Unofficial</h3>
        <span class="muted">Sürüm {__APP_VERSION__} · Windows ve Linux</span>
      </div>
    </div>
    <p>
      Hawk HM220 için resmî olmayan, açık kaynak yapılandırma aracı. Hawk Chair ile bağlantılı değildir; logo ve ürün
      görselleri Hawk Chair'e aittir.
    </p>
    <div class="kv">
      <span>Ayar klasörü</span><code>{app.configDir}</code>
      {#if app.device}
        <span>Cihaz</span><code>{app.device.transport} · PID {app.device.pid.toString(16).padStart(4, "0")}</code>
      {/if}
    </div>
  </section>
</div>

<section class="card dev">
  <button class="dev-head" onclick={() => (devOpen = !devOpen)} aria-expanded={devOpen}>
    <Icon name="code" size={18} />
    <div>
      <h3>Geliştirici</h3>
      <span class="muted">Ham HID frame'leri ve gönderim günlüğü</span>
    </div>
    <span class="chev" class:open={devOpen}><Icon name="chevron" size={18} /></span>
  </button>
  {#if devOpen}
    <div class="dev-body"><Developer /></div>
  {/if}
</section>

<style>
  .page-head {
    margin-bottom: 28px;
  }
  h1 {
    font-size: 34px;
    font-weight: 800;
  }
  .page-head p {
    margin: 6px 0 0;
    color: var(--text-2);
  }
  .cols {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 22px;
    align-items: start;
  }
  .top {
    margin-top: 18px;
  }
  .actions {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 16px 0 14px;
  }
  .actions .btn {
    justify-content: flex-start;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .about p {
    color: var(--text-2);
    line-height: 1.6;
  }
  .kv {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 14px;
    align-items: center;
    font-size: 12.5px;
    color: var(--text-3);
  }
  .kv code {
    overflow-wrap: anywhere;
  }
  .dev {
    margin-top: 22px;
    padding: 0;
  }
  .dev-head {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 18px 24px;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .dev-head div {
    flex: 1;
  }
  .chev {
    transition: transform 0.2s;
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .dev-body {
    padding: 0 24px 24px;
  }
</style>
