import { type } from "@tauri-apps/plugin-os";

export default function IsMobile() {
	const osType = type();
	return osType == "android" || osType == "ios";
}
