<script lang="ts">
  // HM220'nin 3D görünümü: varsa Hawk'ın resmî modeli, yoksa prosedürel model; LED efekti canlı.
  // `onselect` verilirse tuşlara tıklanabilir. WebGL yoksa ürün fotoğrafına (MouseView) düşer.
  import { onMount } from "svelte";
  import { mouseModel, VARIANTS } from "../lib/assets";
  import { Mouse3D, webglAvailable, type MouseLook } from "../lib/mouse3d";
  import { app } from "../lib/state.svelte";
  import type { ButtonId, LedMode } from "../lib/types";
  import MouseView from "./MouseView.svelte";

  let {
    mode = "static",
    color = "#ff0000",
    brightness = 8,
    speed = 3,
    height = 420,
    autoRotate,
    distance = 2.6,
    view: viewMode = "orbit",
    selected = null,
    hovered = $bindable(null),
    onselect,
  }: {
    mode?: LedMode;
    color?: string;
    brightness?: number;
    speed?: number;
    height?: number;
    /** Verilmezse: yörünge görünümünde döner, üstten görünümde sabit. */
    autoRotate?: boolean;
    distance?: number;
    view?: "orbit" | "top";
    selected?: ButtonId | null;
    hovered?: ButtonId | null;
    onselect?: (b: ButtonId) => void;
  } = $props();

  let host = $state<HTMLDivElement>();
  let view = $state<Mouse3D | null>(null);
  const can3d = webglAvailable();
  const body = $derived(VARIANTS.find((v) => v.id === app.variant)?.hex ?? "#1d1d1f");
  const look = $derived<MouseLook>({ body, ledMode: mode, ledColor: color, brightness, speed });
  const model = $derived(mouseModel(app.variant));

  onMount(() => {
    if (!can3d || !host) return;
    const v = new Mouse3D(host, look, { autoRotate, distance, view: viewMode });
    view = v;
    return () => v.dispose();
  });

  $effect(() => {
    view?.setLook(look);
  });
  $effect(() => {
    view?.loadModel(model);
  });
  $effect(() => {
    view?.setPicking(onselect ?? null, onselect ? (b) => (hovered = b) : null);
  });
  $effect(() => {
    view?.setSelection(selected, hovered);
  });
</script>

{#if can3d}
  <div class="stage" style="height:{height}px" bind:this={host}>
    <button class="reset" title="Görünümü sıfırla" aria-label="Görünümü sıfırla" onclick={() => view?.resetView()}>⟲</button>
    <span class="hint">{onselect ? "Tuşa tıkla · sürükleyerek döndür" : "Sürükleyerek döndür"}</span>
  </div>
{:else}
  <MouseView {mode} {color} {brightness} {speed} {height} {selected} bind:hovered {onselect} />
{/if}

<style>
  .stage {
    position: relative;
    width: 100%;
    cursor: grab;
  }
  .stage:active {
    cursor: grabbing;
  }
  .stage :global(canvas) {
    position: absolute;
    inset: 0;
    width: 100% !important;
    height: 100% !important;
    outline: none;
  }
  .reset,
  .hint {
    position: absolute;
    z-index: 1;
    opacity: 0;
    transition: opacity 0.2s;
  }
  .stage:hover .reset,
  .stage:hover .hint {
    opacity: 1;
  }
  .reset {
    right: 10px;
    top: 10px;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: 1px solid var(--line-hi);
    background: color-mix(in srgb, var(--surface-2) 80%, transparent);
    color: var(--text-2);
    cursor: pointer;
    font-size: 15px;
  }
  .hint {
    left: 50%;
    bottom: 8px;
    transform: translateX(-50%);
    color: var(--text-3);
    font-size: 11px;
    letter-spacing: 0.4px;
    pointer-events: none;
    white-space: nowrap;
  }
</style>
