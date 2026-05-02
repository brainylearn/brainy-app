import { UserInformationDto } from "../../../api/backend/dto/userInformationDto";
import SettingsDto from "../../../api/settings/dto/settingsDto";
import { SecurityTabState } from "./securityTabState";

export interface SettingsState {
	localSettings: SettingsDto | null;
	userInformation: UserInformationDto | null;
	securityTabState: SecurityTabState;
}
