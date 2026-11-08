<script lang="ts">
  import { store, settingsActions } from "../settings.svelte";
  const p = $derived(store.s?.prefs);
  const countries = [["", "Any country"], ["US", "United States"], ["CA", "Canada"], ["GB", "United Kingdom"], ["AU", "Australia"]];
  const sorts = [["votes", "Most voted"], ["clickcount", "Most played"], ["clicktrend", "Trending"], ["bitrate", "Best quality"], ["name", "A to Z"]];
</script>
{#if p}
<div class="card">
  <h3>Internet Radio</h3>
  <label class="row">Start searches in
    <select value={p.radio_country ?? ""} onchange={(e) => settingsActions.setPrefs({ radio_country: e.currentTarget.value || null })}>
      {#each countries as [v, l]}<option value={v}>{l}</option>{/each}
    </select>
  </label>
  <label class="row">Sort stations by
    <select value={p.radio_order ?? "votes"} onchange={(e) => settingsActions.setPrefs({ radio_order: e.currentTarget.value })}>
      {#each sorts as [v, l]}<option value={v}>{l}</option>{/each}
    </select>
  </label>
  <p class="note">These are the starting choices on the Radio tab; you can still change them there. Recordings are saved in Downloads/Cindrix/Radio.</p>
</div>
{/if}
<style>
  .row{display:flex;gap:12px;align-items:center;font-size:14px;margin:14px 0}
  select{height:36px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 10px}
  .note{font-size:12px;color:var(--muted);line-height:1.6}
</style>
