import { RootState } from "../store";

export const selectSettings = (state: RootState) => state.settings.settings;
export const selectIsSettingsLoaded = (state: RootState) =>
	state.settings.isSettingsLoaded;
