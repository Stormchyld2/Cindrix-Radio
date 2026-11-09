<script lang="ts">
  let { name, size = 22 }: { name: string; size?: number } = $props();
  // simple line icons, drawn on a 24 grid (constants, not user input)
  const paths: Record<string, string> = {
    home: "M3 11l9-8 9 8v10h-6v-6H9v6H3z",
    music: "M9 18V5l11-2v13M9 18a3 3 0 11-6 0 3 3 0 016 0zM20 16a3 3 0 11-6 0 3 3 0 016 0z",
    radio: "M12 11a1.5 1.5 0 100 3 1.5 1.5 0 000-3zM8 8a5.5 5.5 0 000 8M16 8a5.5 5.5 0 010 8M5 5a9.5 9.5 0 000 14M19 5a9.5 9.5 0 010 14M12 14v6M9 20h6",
    podcast: "M12 3a3 3 0 013 3v5a3 3 0 01-6 0V6a3 3 0 013-3zM5 11a7 7 0 0014 0M12 18v3M8 21h8",
    book: "M2 5h7a3 3 0 013 3v13a2 2 0 00-2-2H2zM22 5h-7a3 3 0 00-3 3v13a2 2 0 012-2h8z",
    video: "M3 6h13v12H3zM16 10l5-3v10l-5-3",
    list: "M4 6h16M4 12h16M4 18h10",
    star: "M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1L3.2 9.5l6.1-.9z",
    rec: "M12 4a8 8 0 100 16 8 8 0 000-16zM12 9a3 3 0 100 6 3 3 0 000-6z",
    karaoke: "M15 3l6 6-5 5-6-6zM10 8L4 20l1 1 12-6M13 11l-5 5",
    sliders: "M4 7h9M17 7h3M4 17h3M11 17h9M15 4v6M9 14v6",
    share: "M12 4a2 2 0 100 4 2 2 0 000-4zM5 16a2 2 0 100 4 2 2 0 000-4zM19 16a2 2 0 100 4 2 2 0 000-4zM12 8l-6 8M12 8l6 8M7 18h10",
    car: "M5 16l1.6-5.5A2 2 0 018.5 9h7a2 2 0 011.9 1.5L19 16M3 16h18v3H3zM7 19v2M17 19v2M7 13h.01M17 13h.01",
    gear: "M12 8.5a3.5 3.5 0 100 7 3.5 3.5 0 000-7zM12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M5 19l2-2M17 7l2-2",
    help: "M12 21a9 9 0 100-18 9 9 0 000 18zM9.5 9.2a2.5 2.5 0 114 2c-.9.6-1.5 1.1-1.5 2.1M12 17h.01",
    wifi: "M2 9a15 15 0 0120 0M5 12.5a10 10 0 0114 0M8.5 16a5 5 0 017 0M12 19.5h.01",
    wifioff: "M2 9a15 15 0 013-2.4M8 5.2A15 15 0 0122 9M5 12.5a10 10 0 012.5-1.8M11 10.1a10 10 0 018 2.4M8.5 16a5 5 0 017 0M12 19.5h.01M3 3l18 18",
    db: "M4 6c0-1.7 3.6-3 8-3s8 1.3 8 3-3.6 3-8 3-8-1.3-8-3zM4 6v6c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6",
    download: "M12 4v11M7 11l5 5 5-5M5 20h14",
    device: "M3 5h14v10H3zM7 19h6M10 15v4M19 8h2v9h-4",
    vol: "M4 9v6h4l5 4V5L8 9zM16 9a4 4 0 010 6M18.5 6.5a8 8 0 010 11",
    queue: "M4 6h16M4 12h16M4 18h16",
    check: "M5 12l5 5 9-10",
  };
</script>
<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={paths[name] ?? ""} /></svg>
