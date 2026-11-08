<script lang="ts">
  import { store, settingsActions } from "../settings.svelte";
  const p = $derived(store.s?.prefs);
</script>
{#if p}
<div class="card">
  <h3>Playback</h3>
  <p class="note">The jump buttons appear in the player for podcasts, audiobooks and downloaded songs.</p>
  <label class="row">Jump back
    <select value={p.skip_back} onchange={(e) => settingsActions.setPrefs({ skip_back: +e.currentTarget.value })}>
      {#each [5, 10, 15, 30, 60] as n}<option value={n}>{n} seconds</option>{/each}
    </select>
  </label>
  <label class="row">Jump forward
    <select value={p.skip_fwd} onchange={(e) => settingsActions.setPrefs({ skip_fwd: +e.currentTarget.value })}>
      {#each [10, 15, 30, 60, 120] as n}<option value={n}>{n} seconds</option>{/each}
    </select>
  </label>
  <p class="note">Podcasts and audiobooks remember where you stopped and pick up there. Use the sleep timer in the full player to stop playback after a set time.</p>
</div>
{/if}
<style>
  .row{display:flex;gap:12px;align-items:center;font-size:14px;margin:14px 0}
  select{height:36px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 10px}
  .note{font-size:12px;color:var(--muted);line-height:1.6}
</style>
