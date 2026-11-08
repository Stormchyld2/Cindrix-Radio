<script lang="ts">
  import { store, settingsActions } from "./settings.svelte";
  import type { EqSettings } from "./api";

  const labels = ["31", "62", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"];
  const presets: Record<string, number[]> = {
    Flat: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    "Bass Boost": [6, 5, 4, 2, 0, 0, 0, 0, 0, 0],
    Vocal: [-2, -2, -1, 1, 3, 3, 2, 1, 0, -1],
    "Warm Rock": [3, 2.5, 1.5, 0, -1, 0, 1, 2, 3, 2],
  };
  const eq = $derived(store.s?.eq);

  const push = (patch: Partial<EqSettings>) => eq && settingsActions.setEq({ ...eq, ...patch });
  const setBand = (i: number, v: number) => {
    if (!eq) return;
    const gains_db = [...eq.gains_db];
    gains_db[i] = v;
    push({ gains_db });
  };
</script>

{#if eq}
  <div class="card eq">
    <div class="top">
      <h3>Equalizer <span class="badge">Global</span></h3>
      <label class="bypass"><input type="checkbox" checked={eq.bypass} onchange={(e) => push({ bypass: e.currentTarget.checked })} /> Bypass</label>
    </div>
    <p class="note">One setting for everything: Music, Radio, Podcasts, Audiobooks. Karaoke uses its own faders.</p>
    <div class="presets">
      {#each Object.entries(presets) as [name, gains]}
        <button class="btn" onclick={() => push({ gains_db: gains })}>{name}</button>
      {/each}
    </div>
    <div class="bands">
      <div class="band">
        <span class="db">{eq.preamp_db > 0 ? "+" : ""}{eq.preamp_db}</span>
        <input type="range" min="-12" max="12" step="0.5" value={eq.preamp_db} orient="vertical"
          aria-label="Preamp" oninput={(e) => push({ preamp_db: +e.currentTarget.value })} />
        <span class="hz">Pre</span>
      </div>
      {#each labels as l, i}
        <div class="band">
          <span class="db">{eq.gains_db[i] > 0 ? "+" : ""}{eq.gains_db[i]}</span>
          <input type="range" min="-12" max="12" step="0.5" value={eq.gains_db[i]} orient="vertical"
            aria-label="{l} Hz" oninput={(e) => setBand(i, +e.currentTarget.value)} />
          <span class="hz">{l}</span>
        </div>
      {/each}
    </div>
    <div class="foot"><span class="note">±12 dB per band</span><button class="btn" onclick={() => push({ gains_db: presets.Flat, preamp_db: 0 })}>Reset to Flat</button></div>
  </div>
{/if}

<style>
  .eq{display:flex;flex-direction:column;gap:12px}
  .top{display:flex;justify-content:space-between;align-items:center}
  .badge{font-size:11px;color:var(--lblue);background:#1FA8FF18;padding:2px 8px;border-radius:8px;margin-left:8px}
  .bypass{font-size:13px;display:flex;gap:6px;align-items:center}
  .note{margin:0;font-size:12px;color:var(--muted)}
  .presets{display:flex;gap:8px;flex-wrap:wrap}
  .bands{display:flex;justify-content:space-between;gap:6px}
  .band{display:flex;flex-direction:column;align-items:center;gap:6px;width:44px}
  .band input{writing-mode:vertical-lr;direction:rtl;height:180px;accent-color:var(--blue)}
  .db{font-size:12px;color:#E3E8EE}.hz{font-size:12px;color:var(--muted)}
  .foot{display:flex;justify-content:space-between;align-items:center}
</style>
