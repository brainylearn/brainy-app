import { UserInformationDto } from "../../../types/backend/dto/userInformationDto";
import Settings from "../../../types/backend/model/settings";
import { SecurityTabState } from "./securityTabState";

export interface SettingsState {
	localSettings: Settings;
	userInformation: UserInformationDto | null;
	securityTabState: SecurityTabState;
}
