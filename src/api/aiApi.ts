import { Channel, invoke } from "@tauri-apps/api/core";
import { StreamLlmResponseEvent } from "../types/backend/events/streamLlmResponseEvent";

export function streamAiResponse(
	prompt: string,
	onEvent: Channel<StreamLlmResponseEvent>,
): Promise<void> {
	return invoke("stream_ai_response", {
		prompt,
		onEvent,
	});
}

export function stopAiGeneration(): Promise<void> {
	return invoke("stop_ai_generation");
}
