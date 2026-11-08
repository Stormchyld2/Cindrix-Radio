<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Diagnostics } from "../api";
  let d = $state<Diagnostics | null>(null);
  onMount(async () => { d = await api.diagnostics(); });
</script>
<div class="card">
  <img src="/logo.png" alt="Cindrix Radio" height="48" />
  <h3>Cindrix Radio {d?.version ?? ""}</h3>
  <p class="note">By Cindrix Development · cindrixops@proton.me</p>
  <p class="note">Music, radio, podcasts, audiobooks, videos and karaoke in one player that works with no internet. Your files stay where they are.</p>
  <p class="note">Built with Tauri, Svelte, Rust, rodio and Symphonia. Radio listings come from the Radio Browser community directory. See the Legal tab for the terms and privacy notice.</p>
</div>
<style>
  img{mix-blend-mode:screen;margin-bottom:8px}
  .note{font-size:13px;color:var(--muted);line-height:1.6;margin:8px 0 0}
</style>
