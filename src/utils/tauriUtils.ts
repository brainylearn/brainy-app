import { Webview, getCurrentWebview } from "@tauri-apps/api/webview";
import IsMobile from "./isMobile";

export default function tryGetCurrentWebView(): Webview | null {
	if (IsMobile()) {
		return null;
	}
	return getCurrentWebview();
}
