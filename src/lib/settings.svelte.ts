import { api, type EqSettings, type Prefs, type Settings } from "./api";

/** Version of the legal documents shipped in this build. Bump when terms change materially. */
export const TERMS_VERSION = "1.0";

export const store = $state<{ s: Settings | null }>({ s: null });

export const settingsActions = {
  load: async () => { store.s = await api.settings(); },
  setLyrics: async (on: boolean) => { store.s = await api.setLyricsViewer(on); },
  setGridDown: async (on: boolean) => { store.s = await api.setGridDown(on); },
  accept: async () => { store.s = await api.acceptTerms(TERMS_VERSION); },
  setVocal: async (amount: number) => { store.s = await api.setVocalReduction(amount); },
  setPrefs: async (patch: Partial<Prefs>) => { if (store.s) store.s = await api.setPrefs({ ...store.s.prefs, ...patch }); },
  setPauseOnDisconnect: async (on: boolean) => { store.s = await api.setPauseOnDisconnect(on); },
  setDownloadRoot: async (path: string | null) => { store.s = await api.setDownloadRoot(path); },
  setEq: async (eq: EqSettings) => { store.s = await api.setEq(eq); },
};
