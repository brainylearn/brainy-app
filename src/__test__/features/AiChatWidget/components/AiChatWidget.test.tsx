import { screen } from "@testing-library/react";
import AiChatWidget from "../../../../features/AiChatWidget/components/AiChatWidget";
import Settings from "../../../../types/backend/model/settings";
import { renderWithProviders } from "../../../test-utils/renderWithProviders";

vi.mock(import("../../../../api/aiApi.ts"));
vi.mock(import("@tauri-apps/api/core"));

function renderComponent({ enableAi = true }) {
	return renderWithProviders(<AiChatWidget />, {
		preloadedState: {
			settings: {
				settings: {
					enableAi,
				} as Partial<Settings> as Settings,
			},
		},
	});
}

describe("AiChatWidget", () => {
	it("Should not show open chat button when AI is not enabled", () => {
		// Act

		renderComponent({ enableAi: false });

		// Assert

		expect(screen.queryByTitle("Open AI assistant")).toBeNull();
	});
});
