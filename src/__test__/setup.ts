/* eslint-disable @typescript-eslint/no-empty-function */
import "@testing-library/jest-dom";

vi.mock("@tauri-apps/api/app", () => ({
	onBackButtonPress: vi.fn().mockResolvedValue({ unregister: vi.fn() }),
}));

vi.stubGlobal(
	"ResizeObserver",
	class {
		observe() {}
		unobserve() {}
		disconnect() {}
	},
);

vi.stubGlobal("alert", vi.fn());

// KaTeX checks for a doctype and warns if missing (quirks mode).
// Happy DOM doesn't set one by default, so add it here.
if (!document.doctype) {
	document.insertBefore(
		document.implementation.createDocumentType("html", "", ""),
		document.firstChild,
	);
}
