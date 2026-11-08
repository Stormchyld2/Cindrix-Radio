import { api, type EqSettings, type Settings } from "./api";

/** Version of the legal documents shipped in this build. Bump when terms change materially. */
export const TERMS_VERSION = "1.0";

export const store = $state<{ s: Settings | null }>({ s: null });

export const settingsActions = {
  load: async () => { store.s = await api.settings(); },
  setLyrics: async (on: boolean) => { store.s = await api.setLyricsViewer(on); },
  accept: async () => { store.s = await api.acceptTerms(TERMS_VERSION); },
  setEq: async (eq: EqSettings) => { store.s = await api.setEq(eq); },
};
