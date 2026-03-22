import { Webview, getCurrentWebview } from "@tauri-apps/api/webview";
import isMobile from "./isMobile";

export default function tryGetCurrentWebView(): Webview | null {
	if (isMobile()) {
		return null;
	}
	return getCurrentWebview();
}
