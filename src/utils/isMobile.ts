import { type } from "@tauri-apps/plugin-os";

export default function isMobile(): boolean {
	const osType = type();
	return osType == "android" || osType == "ios";
}
