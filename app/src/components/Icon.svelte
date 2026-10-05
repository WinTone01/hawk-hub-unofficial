<script lang="ts" module>
  // 24×24 çizgi ikonlar (stroke = currentColor).
  const PATHS: Record<string, string> = {
    overview: "M3 13h8V3H3zM13 21h8V11h-8zM3 21h8v-6H3zM13 3v6h8V3z",
    gauge: "M12 14l4-4M3.5 15a9 9 0 1 1 17 0M12 14a1 1 0 1 0 0 .01",
    bulb: "M9 18h6M10 21h4M12 3a6 6 0 0 0-4 10.5c.6.6 1 1.3 1 2.5h6c0-1.2.4-1.9 1-2.5A6 6 0 0 0 12 3z",
    mouse: "M12 3a6 6 0 0 0-6 6v6a6 6 0 0 0 12 0V9a6 6 0 0 0-6-6zM12 7v4",
    macro: "M4 7h16M4 12h10M4 17h6M17 14l4 3-4 3z",
    profiles: "M4 6h16v4H4zM4 14h16v4H4zM8 8h.01M8 16h.01",
    code: "M8 7l-5 5 5 5M16 7l5 5-5 5M14 4l-4 16",
    plug: "M9 2v6M15 2v6M7 8h10v4a5 5 0 0 1-10 0zM12 17v5",
    wifi: "M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0M12 19.5h.01M2 9a15 15 0 0 1 20 0",
    check: "M5 12l5 5L20 7",
    x: "M6 6l12 12M18 6L6 18",
    plus: "M12 5v14M5 12h14",
    trash: "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3",
    record: "M12 12m-6 0a6 6 0 1 0 12 0a6 6 0 1 0-12 0",
    stop: "M7 7h10v10H7z",
    upload: "M12 16V4M7 9l5-5 5 5M4 20h16",
    refresh: "M20 11a8 8 0 0 0-14.9-3M4 4v4h4M4 13a8 8 0 0 0 14.9 3M20 20v-4h-4",
    reset: "M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5",
    search: "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM21 21l-5-5",
    copy: "M9 9h11v11H9zM5 15H4V4h11v1",
    save: "M5 3h11l3 3v15H5zM8 3v5h7M8 21v-7h8v7",
    play: "M7 4l13 8-13 8z",
    up: "M12 19V5M6 11l6-6 6 6",
    down: "M12 5v14M6 13l6 6 6-6",
    bolt: "M13 2L4 14h7l-1 8 9-12h-7z",
    target: "M12 12m-8 0a8 8 0 1 0 16 0a8 8 0 1 0-16 0M12 12m-3 0a3 3 0 1 0 6 0a3 3 0 1 0-6 0M12 2v3M12 19v3M2 12h3M19 12h3",
    timer: "M12 13l3-3M9 2h6M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16z",
    sun: "M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM12 1v2M12 21v2M4.2 4.2l1.4 1.4M18.4 18.4l1.4 1.4M1 12h2M21 12h2M4.2 19.8l1.4-1.4M18.4 5.6l1.4-1.4",
    keyboard: "M3 6h18v12H3zM7 10h.01M11 10h.01M15 10h.01M7 14h10",
    import: "M12 4v12M7 11l5 5 5-5M4 20h16",
    info: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 11v5M12 8h.01",
    edit: "M4 20h4L19 9l-4-4L4 16zM14 6l4 4",
    moon: "M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z",
    grid: "M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z",
    help: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.6.3-1 .9-1 1.6v.6M12 17h.01",
    tune: "M4 6h9M17 6h3M4 12h3M11 12h9M4 18h11M19 18h1M15 4v4M9 10v4M17 16v4",
    battery: "M9 3h6M7 5h10v16H7z",
    chevron: "M9 6l6 6-6 6",
    back: "M15 6l-6 6 6 6",
    cpu: "M7 7h10v10H7zM10 2v3M14 2v3M10 19v3M14 19v3M2 10h3M2 14h3M19 10h3M19 14h3",
    link: "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1",
    window: "M3 5h18v14H3zM3 9h18M6 7h.01M9 7h.01",
    folder: "M3 6h6l2 2h10v11H3z",
    // Profil simgeleri
    crosshair: "M12 12m-7 0a7 7 0 1 0 14 0a7 7 0 1 0-14 0M12 2v6M12 16v6M2 12h6M16 12h6",
    gamepad: "M6 9h12a4 4 0 0 1 4 4v1a3 3 0 0 1-5.5 1.7L15 14H9l-1.5 1.7A3 3 0 0 1 2 14v-1a4 4 0 0 1 4-4zM7 11v3M5.5 12.5h3M16 12h.01M18 13.5h.01",
    sword: "M14.5 17.5L3 6V3h3l11.5 11.5M13 19l6-6M16 16l4 4M19 21l2-2",
    shield: "M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z",
    trophy: "M8 4h8v5a4 4 0 0 1-8 0zM8 6H4a4 4 0 0 0 4 4M16 6h4a4 4 0 0 1-4 4M12 13v4M8 21h8",
    flame: "M12 22a7 7 0 0 0 7-7c0-4-3-6-4-10-2 2-3 4-3 6-1-1-2-2-2-4-2 2-5 5-5 8a7 7 0 0 0 7 7z",
    rocket: "M5 15c-1.5 1.3-2 5-2 5s3.7-.5 5-2M9 12a22 22 0 0 1 11-9c0 4-1 8-9 11zM9 12H5l2-4h5M12 15v4l4-2v-5",
    star: "M12 3l2.8 5.7 6.2.9-4.5 4.4 1 6.2L12 17.3 6.5 20.2l1-6.2L3 9.6l6.2-.9z",
    heart: "M12 20s-8-4.5-8-11a4.5 4.5 0 0 1 8-2.8A4.5 4.5 0 0 1 20 9c0 6.5-8 11-8 11z",
    brush: "M18.4 2.6a2 2 0 0 1 3 3L12 15l-3-3zM9 12c-2 0-4 1.5-4 4 0 1.5-1 3-3 3 4 2 9 1 10-3z",
    film: "M3 4h18v16H3zM7 4v16M17 4v16M3 9h4M3 15h4M17 9h4M17 15h4",
    music: "M9 18V5l12-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM21 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z",
    briefcase: "M3 7h18v13H3zM8 7V4h8v3M3 13h18",
    globe: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18",
  };
</script>

<script lang="ts">
  let { name, size = 18, stroke = 1.8 }: { name: string; size?: number; stroke?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={PATHS[name] ?? ""} />
</svg>
