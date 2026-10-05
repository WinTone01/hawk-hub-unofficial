<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import { api } from "../lib/api";
  import { app } from "../lib/state.svelte";
  import type { Frame, LogEntry } from "../lib/types";

  let frames = $state<Frame[]>([]);
  let log = $state<LogEntry[]>([]);
  let open = $state<number | null>(null);

  const hex = (b: number) => b.toString(16).padStart(2, "0");

  // Her yazmadan sonra yenile.
  $effect(() => {
    app.writes;
    load();
  });

  async function load() {
    try {
      [frames, log] = await Promise.all([api.frames(), api.reportLog()]);
    } catch (e) {
      app.toast(String(e), "err");
    }
  }

  function copy(f: Frame) {
    navigator.clipboard.writeText(f.bytes.map(hex).join(" ")).then(() => app.toast("Kopyalandı"));
  }

  const time = (ms: number) =>
    new Date(ms).toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit", second: "2-digit" }) +
    "." + String(ms % 1000).padStart(3, "0");
</script>

<div class="stack rise">
  <div class="grid info">
    <div class="card kv">
      <span class="muted">Cihaz yolu</span>
      <code class="path">{app.device?.path ?? "—"}</code>
    </div>
    <div class="card kv">
      <span class="muted">VID:PID</span>
      <code>{app.device ? `1d57:${app.device.pid.toString(16).padStart(4, "0")}` : "—"}</code>
    </div>
    <div class="card kv">
      <span class="muted">Ayar dizini</span>
      <code class="path">{app.configDir}</code>
    </div>
  </div>

  <div class="card">
    <div class="card-head">
      <div>
        <h3>Güncel frame'ler</h3>
        <span class="sub">Feature report payload'ları (report ID hariç, gönderimde 64 bayta doldurulur). Kırmızı baytlar checksum.</span>
      </div>
      <button class="btn sm" onclick={load}><Icon name="refresh" size={14} />Yenile</button>
    </div>
    {#each frames as f (f.report)}
      <div class="frame">
        <div class="frame-head">
          <span class="rid">0x{hex(f.report)}</span>
          <b>{f.name}</b>
          <span class="muted">{f.bytes.length} bayt</span>
          <button class="icon-btn" title="Hex kopyala" onclick={() => copy(f)}><Icon name="copy" size={15} /></button>
        </div>
        <div class="hex">
          {#each f.bytes as b, i (i)}
            <span class:ck={f.checksum.includes(i)} title="[{i}] = {b}">{hex(b)}</span>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <div class="card">
    <div class="card-head">
      <div><h3>Gönderim günlüğü</h3><span class="sub">Bu oturumda fareye gönderilen son 200 report (yeniden eskiye)</span></div>
    </div>
    {#if log.length === 0}
      <p class="muted">Henüz bir şey gönderilmedi. Bir ayar değiştirin.</p>
    {:else}
      <div class="log">
        {#each log as e, i (e.at + "-" + i)}
          <button class="entry" class:err={!e.ok} onclick={() => (open = open === i ? null : i)}>
            <span class="t">{time(e.at)}</span>
            <span class="rid">0x{hex(e.report)}</span>
            <span class="name">{e.name}</span>
            <span class="preview">{e.bytes.slice(0, 16).map(hex).join(" ")}{e.bytes.length > 16 ? " …" : ""}</span>
            <span class="status">{e.ok ? "✓" : "✕"}</span>
          </button>
          {#if open === i}
            <div class="detail">
              {#if e.error}<div class="error">{e.error}</div>{/if}
              <div class="hex">{#each e.bytes as b, j (j)}<span>{hex(b)}</span>{/each}</div>
            </div>
          {/if}
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .info {
    grid-template-columns: 1.6fr 0.6fr 1.2fr;
  }
  .kv {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px 18px;
    min-width: 0;
  }
  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    user-select: text;
  }
  .frame {
    padding: 12px 0;
    border-top: 1px solid var(--line);
  }
  .frame-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
  }
  .frame-head .muted {
    margin-right: auto;
  }
  .rid {
    padding: 2px 7px;
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--accent);
    font-family: var(--mono);
    font-size: 12px;
    font-weight: 700;
  }
  .hex {
    display: grid;
    grid-template-columns: repeat(auto-fill, 26px);
    gap: 3px;
    font-family: var(--mono);
    font-size: 12px;
    user-select: text;
  }
  .hex span {
    text-align: center;
    padding: 2px 0;
    border-radius: 4px;
    background: var(--bg-sunk);
    color: var(--text-2);
  }
  .hex span.ck {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .log {
    display: flex;
    flex-direction: column;
    max-height: 420px;
    overflow-y: auto;
  }
  .entry {
    display: grid;
    grid-template-columns: 105px 50px 90px 1fr 20px;
    align-items: center;
    gap: 10px;
    padding: 7px 8px;
    border: 0;
    border-bottom: 1px solid var(--line);
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    text-align: left;
    cursor: pointer;
  }
  .entry:hover {
    background: var(--panel-hi);
  }
  .t,
  .preview {
    font-family: var(--mono);
    color: var(--muted);
  }
  .preview {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .status {
    color: var(--ok);
  }
  .entry.err .status {
    color: var(--accent);
  }
  .detail {
    padding: 10px 8px 14px;
    background: var(--bg-sunk);
  }
  .error {
    color: var(--accent);
    margin-bottom: 8px;
    font-size: 12.5px;
  }
</style>
