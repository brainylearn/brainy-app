import { RootState } from "../store";

export const selectIsInitialStateLoaded = (state: RootState) =>
	state.app.isInitialStateLoaded;
