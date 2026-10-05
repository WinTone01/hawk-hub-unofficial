<script lang="ts" module>
  // Hawk Hub logosu (npm run assets ile resmî uygulamadan çıkarılır; depoda yoktur). Yoksa basit bir rozet.
  const found = import.meta.glob<string>("../assets/hawk/logo.svg", { eager: true, query: "?url", import: "default" });
  const logo = Object.values(found)[0] ?? null;
</script>

<script lang="ts">
  let { size = 40 }: { size?: number } = $props();
</script>

{#if logo}
  <img src={logo} alt="" width={size} height={size} draggable="false" />
{:else}
  <span class="fallback" style="--s:{size}px">H</span>
{/if}

<style>
  img {
    display: block;
    flex: none;
  }
  .fallback {
    display: grid;
    place-items: center;
    width: var(--s);
    height: var(--s);
    border-radius: 50%;
    background: #000;
    color: #fff;
    font-family: var(--display);
    font-weight: 800;
    font-size: calc(var(--s) * 0.45);
  }
</style>
