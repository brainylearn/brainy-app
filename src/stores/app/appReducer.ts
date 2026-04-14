import { createSlice } from "@reduxjs/toolkit";

export interface AppState {
	startedInitialStateLoading: boolean;
}

const initialState: AppState = {
	startedInitialStateLoading: false,
};

export const appSlice = createSlice({
	name: "settings",
	initialState,
	reducers: {
		markStartLoadingOfInitialState: state => {
			state.startedInitialStateLoading = true;
		},
	},
});

export default appSlice.reducer;

export const { markStartLoadingOfInitialState } = appSlice.actions;
