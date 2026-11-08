<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { video, videoActions } from "./video.svelte";

  const v = $derived(video.open);
  const src = $derived(v?.file ? convertFileSrc(v.file) : null);
  let error = $state<string | null>(null);
  let el = $state<HTMLVideoElement | null>(null);
  let lastSave = 0;

  // remember where the viewer stopped, per file (a per-device convenience, not shared)
  const key = (f: string) => `cindrix.video.pos.${f}`;
  const read = (f: string): number => { try { return Number(localStorage.getItem(key(f))) || 0; } catch { return 0; } };
  const write = (f: string, t: number | null) => { try { t == null ? localStorage.removeItem(key(f)) : localStorage.setItem(key(f), String(Math.floor(t))); } catch { /* storage unavailable */ } };

  $effect(() => { v; error = null; });
  function loaded() { if (el && v?.file) { const t = read(v.file); if (t > 5 && t < el.duration - 10) el.currentTime = t; } }
  function tick() {
    if (!el || !v?.file) return;
    const now = Date.now();
    if (now - lastSave > 5000) { lastSave = now; write(v.file, el.currentTime); }
  }
  const ended = () => { if (v?.file) write(v.file, null); };
  const failed = () => { error = "This file's format can't be opened by the built-in player. mp4, m4v and webm work; mkv and avi usually don't."; };
  const keydown = (e: KeyboardEvent) => { if (e.key === "Escape" && v) videoActions.close(); };
</script>

<svelte:window onkeydown={keydown} />

{#if v}
  <div class="wrap" role="dialog" aria-modal="true" aria-label="Video player">
    <div class="bar"><b>{v.title}</b><button class="btn" onclick={videoActions.close}>Close ✕</button></div>
    {#if src}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video bind:this={el} {src} controls autoplay onloadedmetadata={loaded} ontimeupdate={tick} onended={ended} onerror={failed}></video>
    {:else if v.embed}
      <iframe title={v.title} src={v.embed} allow="autoplay; fullscreen; encrypted-media; picture-in-picture" allowfullscreen referrerpolicy="strict-origin-when-cross-origin"></iframe>
      <p class="hint">Playing online. Use the download button on the card to keep it for offline.</p>
    {/if}
    {#if error}<p class="err">{error}</p>{/if}
  </div>
{/if}

<style>
  .wrap{position:fixed;inset:0;background:#000000F2;z-index:30;display:flex;flex-direction:column;padding:14px 18px;gap:10px}
  .bar{display:flex;justify-content:space-between;align-items:center;gap:12px}
  .bar b{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  video,iframe{flex:1;min-height:0;width:100%;border:0;background:#000;border-radius:10px}
  .hint{font-size:12px;color:var(--muted);margin:0}
  .err{color:var(--red);font-size:13px;margin:0}
</style>
