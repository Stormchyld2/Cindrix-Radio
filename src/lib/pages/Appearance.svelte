<script lang="ts">
  import { store, settingsActions } from "../settings.svelte";
  const p = $derived(store.s?.prefs);
  const accents: [string, string][] = [["blue", "#1FA8FF"], ["green", "#2FD08A"], ["amber", "#F5B43C"], ["violet", "#A78BFA"]];
</script>
{#if p}
<div class="card">
  <h3>Appearance</h3>
  <p class="lab">Text size · {Math.round(p.text_scale * 100)}%</p>
  <input type="range" min="0.9" max="1.5" step="0.05" value={p.text_scale} aria-label="Text size" onchange={(e) => settingsActions.setPrefs({ text_scale: +e.currentTarget.value })} />
  <p class="lab">Accent colour</p>
  <div class="sw">
    {#each accents as [name, hex]}
      <button class="dot" class:on={p.accent === name} style="background:{hex}" aria-label={name} aria-pressed={p.accent === name} onclick={() => settingsActions.setPrefs({ accent: name })}></button>
    {/each}
  </div>
  <label class="row"><input type="checkbox" checked={p.high_contrast} onchange={(e) => settingsActions.setPrefs({ high_contrast: e.currentTarget.checked })} /> High contrast text and borders</label>
  <label class="row"><input type="checkbox" checked={p.reduce_motion} onchange={(e) => settingsActions.setPrefs({ reduce_motion: e.currentTarget.checked })} /> Reduce motion</label>
  <label class="row"><input type="checkbox" checked={p.keep_awake} onchange={(e) => settingsActions.setPrefs({ keep_awake: e.currentTarget.checked })} /> Keep the screen on while music is playing</label>
  <p class="note">Changes apply straight away.</p>
</div>
{/if}
<style>
  .lab{font-size:13px;color:var(--muted);margin:14px 0 6px}
  .sw{display:flex;gap:12px}
  .dot{width:36px;height:36px;border-radius:18px;border:2px solid transparent}.dot.on{border-color:#fff}
  .row{display:flex;gap:8px;align-items:center;font-size:14px;margin:14px 0 0}
  .note{font-size:12px;color:var(--muted);margin-top:14px}
  input[type=range]{width:min(360px,100%)}
</style>
