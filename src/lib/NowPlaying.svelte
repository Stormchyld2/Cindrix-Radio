<script lang="ts">
  import { player, actions } from "./player.svelte";
  import { store } from "./settings.svelte";
  import { ui, uiActions } from "./ui.svelte";

  const cur = $derived(player.s.current);
  const dur = $derived(cur?.duration_secs ?? 0);
  const seekable = $derived(!!cur?.path && dur > 0);
  const back = $derived(store.s?.prefs.skip_back ?? 15);
  const fwd = $derived(store.s?.prefs.skip_fwd ?? 30);
  let dragging = $state<number | null>(null);
  let now = $state(Date.now());
  $effect(() => { const id = setInterval(() => (now = Date.now()), 1000); return () => clearInterval(id); });
  const left = $derived(ui.sleepEnd ? Math.max(0, Math.ceil((ui.sleepEnd - now) / 60_000)) : 0);
  const mmss = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  const jump = (d: number) => actions.seek(Math.max(0, Math.min(dur ? dur - 1 : 1e9, player.s.position_secs + d)));
  const title = $derived(player.s.now_playing || cur?.title || "Nothing playing");
  const sub = $derived(player.s.now_playing ? cur?.title : cur?.subtitle);
</script>

{#if ui.np}
  <div class="np" class:car={ui.car} role="dialog" aria-modal="true" aria-label="Now playing">
    <div class="top">
      <button class="btn" onclick={() => { ui.np = false; ui.car = false; }}>Close</button>
      <button class="btn" class:primary={ui.car} onclick={() => (ui.car = !ui.car)}>Car mode</button>
    </div>

    <div class="body">
      {#if !ui.car}<img class="art" src="/logo.png" alt="" />{/if}
      <h2>{title}</h2>
      <p class="sub">{sub ?? ""}</p>
      {#if player.s.error}<p class="err">{player.s.error}</p>{/if}

      {#if seekable && !ui.car}
        <div class="seek">
          <span>{mmss(dragging ?? player.s.position_secs)}</span>
          <input type="range" min="0" max={dur} step="1" value={dragging ?? player.s.position_secs} aria-label="Seek"
            oninput={(e) => (dragging = +e.currentTarget.value)}
            onchange={async (e) => { await actions.seek(+e.currentTarget.value); dragging = null; }} />
          <span>{mmss(dur)}</span>
        </div>
      {/if}

      <div class="ctl">
        {#if seekable}<button class="side" aria-label="Back {back} seconds" onclick={() => jump(-back)}>−{back}</button>{/if}
        <button class="main" aria-label={player.s.playing ? "Pause" : "Play"} onclick={actions.toggle}>{player.s.playing ? "❚❚" : "▶"}</button>
        <button class="side" aria-label="Next" onclick={actions.next}>Next</button>
        {#if seekable}<button class="side" aria-label="Forward {fwd} seconds" onclick={() => jump(fwd)}>+{fwd}</button>{/if}
      </div>

      {#if !ui.car}
        <label class="vol">Volume <input type="range" min="0" max="1" step="0.01" value={player.s.volume} oninput={(e) => actions.volume(+e.currentTarget.value)} /></label>
        <div class="sleep">
          <label>Sleep timer
            <select onchange={(e) => uiActions.setSleep(+e.currentTarget.value || null)}>
              <option value="0">Off</option>
              {#each [10, 15, 30, 45, 60, 90] as m}<option value={m}>{m} minutes</option>{/each}
            </select>
          </label>
          {#if ui.sleepEnd}<span class="note">Stops in {left} min</span>{/if}
        </div>
        {#if player.s.queue.length}
          <h3>Up next</h3>
          <ol>{#each player.s.queue.slice(0, 8) as q}<li><b>{q.title}</b> <span>{q.subtitle}</span></li>{/each}</ol>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  .np{position:fixed;inset:0;z-index:35;background:var(--bg);display:flex;flex-direction:column;padding:16px;overflow:auto}
  .top{display:flex;justify-content:space-between}
  .body{flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;text-align:center;max-width:560px;width:100%;margin:0 auto}
  .art{width:min(220px,50vw);mix-blend-mode:screen;opacity:.9}
  h2{font-size:26px;overflow-wrap:anywhere}
  .sub{color:var(--muted);margin:0}.err{color:var(--red);font-size:13px}
  .seek{display:flex;align-items:center;gap:10px;width:100%;font-size:12px;color:var(--muted)}.seek input{flex:1}
  .ctl{display:flex;align-items:center;gap:14px;margin:8px 0}
  .main{width:76px;height:76px;border-radius:38px;border:0;background:var(--blue);color:#04121C;font-size:26px}
  .side{min-width:60px;height:48px;padding:0 12px;border-radius:24px;border:1px solid #2A3747;background:transparent;color:var(--silver);font-size:15px}
  .vol,.sleep{display:flex;align-items:center;gap:10px;font-size:13px}
  select{height:34px;border-radius:10px;border:1px solid #2A3747;background:#0D1620;color:var(--silver);padding:0 10px}
  .note{color:var(--lblue)}
  h3{font-size:16px;margin-top:10px}ol{text-align:left;width:100%;padding-left:20px;font-size:13px}li span{color:var(--muted)}
  .car h2{font-size:clamp(34px,8vw,64px)}
  .car .sub{font-size:clamp(18px,4vw,28px)}
  .car .main{width:min(40vw,200px);height:min(40vw,200px);border-radius:100px;font-size:64px}
  .car .side{min-width:min(26vw,140px);height:min(26vw,140px);font-size:clamp(20px,5vw,34px);border-radius:70px}
  .car .ctl{gap:20px;flex-wrap:wrap;justify-content:center}
</style>
