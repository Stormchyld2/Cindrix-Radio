/** Small shared screen state: the full player, car mode and the sleep timer. */
export const ui = $state<{ np: boolean; car: boolean; sleepEnd: number | null; settingsTab: string | null }>({ np: false, car: false, sleepEnd: null, settingsTab: null });

export const uiActions = {
  setSleep: (minutes: number | null) => { ui.sleepEnd = minutes ? Date.now() + minutes * 60_000 : null; },
};
