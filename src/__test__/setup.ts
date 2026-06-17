/* eslint-disable @typescript-eslint/no-empty-function */
import "@testing-library/jest-dom";

// Node.js 22+ defines localStorage as a non-configurable global (unavailable
// without --localstorage-file), which prevents happy-dom from overriding it.
// vi.stubGlobal bypasses the non-configurable restriction.
const localStorageMock = (() => {
	let store: Record<string, string> = {};
	return {
		getItem: (key: string) => store[key] ?? null,
		setItem: (key: string, value: string) => {
			store[key] = value;
		},
		removeItem: (key: string) => {
			delete store[key];
		},
		clear: () => {
			store = {};
		},
		get length() {
			return Object.keys(store).length;
		},
		key: (index: number) => Object.keys(store)[index] ?? null,
	};
})();
vi.stubGlobal("localStorage", localStorageMock);

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
