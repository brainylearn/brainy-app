import { createSlice } from "@reduxjs/toolkit";

interface AppState {
	isInitialStateLoaded: boolean;
}

const initialState: AppState = {
	isInitialStateLoaded: false,
};

export const appSlice = createSlice({
	name: "settings",
	initialState,
	reducers: {
		markInitialStateAsLoaded: state => {
			state.isInitialStateLoaded = true;
		},
	},
});

export default appSlice.reducer;

export const { markInitialStateAsLoaded } = appSlice.actions;
