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

// TODO: should be used for both editor and reviewer
// TODO: rename and refactor, and move to own folder
// TODO: responsivity
// TODO: unit test
export default function AiBot() {
	const [isOpen, setIsOpen] = useState(false);
	const [userPrompt, setUserPrompt] = useState("");

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		if (!userPrompt) return;

		// TODO:
		const onEvent = new Channel<StreamLlmResponseEvent>();
		onEvent.onmessage = console.log;
		void streamAiResponse("userPrompt", onEvent);
		setUserPrompt("");
	};

	const handleKeyDown = (e: React.KeyboardEvent) => {
		if (e.key === "Escape") setIsOpen(false);
	};

	return (
		<div className={styles.aiBotContainer} onKeyDown={handleKeyDown}>
			{isOpen && (
				<div className={styles.aiChatPanel}>
					<div className={styles.header}>
						<p>AI Assistant</p>
						<button onClick={() => setIsOpen(false)}>
							<Icon path={mdiClose} size={1} />
						</button>
					</div>

					<div className={styles.messages}>
						<div className={styles.bot}>Message form the bot</div>

						<div className={styles.human}>
							Message form the human
						</div>
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
					className={`primary ${styles.aiFloatingButton}`}
					onClick={() => setIsOpen(true)}>
					<Icon path={mdiRobotOutline} size={1.6} />
				</button>
			)}
		</div>
	);
}
