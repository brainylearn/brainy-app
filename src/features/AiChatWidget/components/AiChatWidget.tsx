import Icon from "@mdi/react";
import {
	mdiAttachment,
	mdiClose,
	mdiRobotOutline,
	mdiSendVariantOutline,
} from "@mdi/js";
import styles from "./styles.module.css";
import { useState } from "react";
import { Channel } from "@tauri-apps/api/core";
import { StreamLlmResponseEvent } from "../../../types/backend/events/streamLlmResponseEvent";
import { streamAiResponse } from "../../../api/aiApi";
import Message from "../types/message";
import Markdown from "react-markdown";

// TODO: should be used for both editor and reviewer
// TODO: responsivity
// TODO: unit test
export default function AiChatWidget() {
	const [isOpen, setIsOpen] = useState(false);
	const [userPrompt, setUserPrompt] = useState("");
	const [isSendingRequest, setIsSendingRequest] = useState(false);
	const [messages, setMessages] = useState<Message[]>([]);

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		if (!userPrompt) return;

		setMessages([
			...messages,
			{
				from: "human",
				content: userPrompt,
			},
			{
				from: "bot",
				content: "",
			},
		]);

		const onEvent = new Channel<StreamLlmResponseEvent>();
		onEvent.onmessage = event => {
			setMessages(messages => {
				// TODO: handle start and error
				const lastMessage = messages[messages.length - 1];
				if (event.event === "inProgress") {
					return [
						...messages.slice(0, -1),
						{
							...lastMessage,
							content: lastMessage.content + event.data,
						},
					];
				}
				return messages;
			});
		};
		void streamAiResponse(userPrompt, onEvent);
		setUserPrompt("");
	};

	const handleKeyDown = (e: React.KeyboardEvent) => {
		if (e.key === "Escape") setIsOpen(false);
	};

	// TODO: make user input field expandable when writing
	// TODO: add stop generating button
	return (
		<div className={styles.container} onKeyDown={handleKeyDown}>
			{isOpen && (
				<div className={styles.chatPanel}>
					<div className={styles.header}>
						<p>AI Assistant</p>
						<button onClick={() => setIsOpen(false)}>
							<Icon path={mdiClose} size={1} />
						</button>
					</div>

					<div className={styles.messages}>
						{messages.map((message, i) => (
							<div key={i} className={styles[message.from]}>
								<Markdown>{message.content}</Markdown>
							</div>
						))}
					</div>

					<form onSubmit={handleSubmit}>
						<input
							type="text"
							placeholder="Ask any question, order to do anything"
							value={userPrompt}
							onChange={e => setUserPrompt(e.target.value)}
						/>
						<button className="transparent" title="Add attachment">
							<Icon path={mdiAttachment} size={1} />
						</button>
						<button className="transparent" title="Send">
							<Icon path={mdiSendVariantOutline} size={1} />
						</button>
					</form>
				</div>
			)}

			{!isOpen && (
				<button
					className={`primary ${styles.floatingButton}`}
					onClick={() => setIsOpen(true)}>
					<Icon path={mdiRobotOutline} size={1.6} />
				</button>
			)}
		</div>
	);
}
