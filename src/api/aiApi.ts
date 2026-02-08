import { Channel, invoke } from "@tauri-apps/api/core";
import { StreamLlmResponseEvent } from "../types/backend/events/streamLlmResponseEvent";
import Chat from "../types/backend/entity/chat";

export function streamAiResponse(
	prompt: string,
	chatId: string | null,
	onEvent: Channel<StreamLlmResponseEvent>,
): Promise<void> {
	return invoke("stream_ai_response", {
		prompt,
		onEvent,
		chatId,
	});
}

export function stopAiGeneration(): Promise<void> {
	return invoke("stop_ai_generation");
}

export function getAllAiChats(): Promise<Chat[]> {
	return invoke("get_all_ai_chats");
}
