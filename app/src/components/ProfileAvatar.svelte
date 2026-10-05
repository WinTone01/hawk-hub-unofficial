<script lang="ts">
  // Profil rozeti: profil renginde kutu + seçilen simge ya da bağlı programın simgesi.
  import Icon from "./Icon.svelte";
  import { appIcon } from "../lib/appIcons";

  let {
    color,
    glyph = null,
    app = null,
    size = 40,
  }: { color: string; glyph?: string | null; app?: string | null; size?: number } = $props();

  let src = $state<string | null>(null);
  $effect(() => {
    const path = app;
    src = null;
    if (path) appIcon(path).then((s) => path === app && (src = s));
  });
</script>

<span class="avatar" class:img={!!src} style="--c:{color}; --s:{size}px">
  {#if app && src}
    <img {src} alt="" />
  {:else}
    <Icon name={app ? "window" : (glyph ?? "profiles")} size={Math.round(size * 0.5)} stroke={1.9} />
  {/if}
</span>

<style>
  .avatar {
    display: grid;
    place-items: center;
    flex: none;
    width: var(--s);
    height: var(--s);
    border-radius: calc(var(--s) * 0.3);
    background: linear-gradient(145deg, color-mix(in srgb, var(--c) 92%, #fff), color-mix(in srgb, var(--c) 70%, #000));
    color: #fff;
    box-shadow:
      0 0 22px -6px var(--c),
      inset 0 1px 0 rgba(255, 255, 255, 0.25);
  }
  .avatar.img {
    background: linear-gradient(145deg, color-mix(in srgb, var(--c) 30%, #1a1d26), color-mix(in srgb, var(--c) 12%, #0d0f14));
    border: 1px solid color-mix(in srgb, var(--c) 55%, transparent);
  }
  img {
    width: 66%;
    height: 66%;
    object-fit: contain;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.4));
  }
</style>
