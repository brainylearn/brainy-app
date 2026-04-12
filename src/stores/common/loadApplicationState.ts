import { NavigateFunction } from "react-router";
import { getReviewTreeFolderForRoot } from "../fileSystem/fileSystemActions";
import { initialLoadAndApplySettings } from "../settings/settingsActions";
import { AppDispatch } from "../store";
import { sync } from "../sync/syncActions";
import { loadInitialUserState } from "../user/userActions";
import { UserInformationDto } from "../../types/backend/dto/userInformationDto";
import { setUserInformation } from "../user/userReducer";

/** Loads the global state, if the navigate is given, it navigates to '/'.
 * If the user information is given it sets the global information, otherwise
 * it retrieve from the backend.
 */
// TODO: better?
export function loadApplicationState(
	navigate?: NavigateFunction,
	userInformation?: UserInformationDto,
) {
	return async function (dispatch: AppDispatch): Promise<void> {
		const settings = await dispatch(initialLoadAndApplySettings());
		await dispatch(getReviewTreeFolderForRoot());

		if (userInformation) {
			dispatch(setUserInformation(userInformation));
		} else {
			await dispatch(loadInitialUserState());
		}

		// Sync on app close is added as an event in the settings actions.
		if (settings?.autoSync) await dispatch(sync());

		if (navigate) await navigate("/");
	};
}
