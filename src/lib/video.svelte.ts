import { player, actions } from "./player.svelte";

export interface VideoOpen { title: string; file?: string; embed?: string }

export const video = $state<{ open: VideoOpen | null }>({ open: null });

export const videoActions = {
  // Audio and video never play at once: starting a video pauses the audio player.
  open: async (v: VideoOpen) => {
    if (player.s.playing) { try { await actions.toggle(); } catch { /* nothing playing */ } }
    video.open = v;
  },
  close: () => { video.open = null; },
};
