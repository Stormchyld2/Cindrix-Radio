import type { Track } from "./api";

export interface Recent { track: Track; pos: number; at: number }
const KEY = "cindrix.recent";

function load(): Recent[] {
  try { const v = JSON.parse(localStorage.getItem(KEY) ?? "[]"); return Array.isArray(v) ? v.slice(0, 8) : []; } catch { return []; }
}
export const recent = $state<{ items: Recent[] }>({ items: load() });

let lastSave = 0;
/** Called from the player poll: remembers what you listened to for the Home screen. */
export function noteListening(t: Track | null, pos: number, playing: boolean) {
  if (!t || !playing) return;
  const id = t.id;
  const rest = recent.items.filter((r) => r.track.id !== id);
  recent.items = [{ track: t, pos, at: Date.now() }, ...rest].slice(0, 8);
  if (Date.now() - lastSave > 5000) {
    lastSave = Date.now();
    try { localStorage.setItem(KEY, JSON.stringify(recent.items)); } catch { /* storage unavailable */ }
  }
}
