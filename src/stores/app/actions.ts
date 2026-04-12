import { NavigateFunction } from "react-router";
import { getReviewTreeFolderForRoot } from "../fileSystem/fileSystemActions";
import { initialLoadAndApplySettings } from "../settings/settingsActions";
import { AppDispatch } from "../store";
import { sync } from "../sync/syncActions";
import { loadInitialUserState } from "../user/userActions";
import { UserInformationDto } from "../../types/backend/dto/userInformationDto";
import { setUserInformation } from "../user/userReducer";

export function initialLoadApplicationState() {
	return async function (dispatch: AppDispatch): Promise<void> {
		const settings = await dispatch(initialLoadAndApplySettings());
		await dispatch(getReviewTreeFolderForRoot());
		await dispatch(loadInitialUserState());
		// Sync on app close is added as an event in the settings actions.
		if (settings?.autoSync) await dispatch(sync());
	};
}

export function reloadApplicationState(
	navigate: NavigateFunction,
	userInformationDto?: UserInformationDto,
) {
	return async function (dispatch: AppDispatch): Promise<void> {
		const settings = await dispatch(initialLoadAndApplySettings());
		await dispatch(getReviewTreeFolderForRoot());

		if (userInformationDto) {
			dispatch(setUserInformation(userInformationDto));
		} else {
			await dispatch(loadInitialUserState());
		}

		// Sync on app close is added as an event in the settings actions.
		if (settings?.autoSync) await dispatch(sync());

		await navigate("/");
	};
}
