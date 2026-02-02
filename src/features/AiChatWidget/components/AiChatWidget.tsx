import Icon from "@mdi/react";
import {
	mdiAttachment,
	mdiClose,
	mdiRobotOutline,
	mdiSendVariantOutline,
	mdiStopCircleOutline,
} from "@mdi/js";
import styles from "./styles.module.css";
import { useEffect, useRef, useState } from "react";
import { Channel } from "@tauri-apps/api/core";
import { StreamLlmResponseEvent } from "../../../types/backend/events/streamLlmResponseEvent";
import { stopAiGeneration, streamAiResponse } from "../../../api/aiApi";
import Message from "../types/message";
import Markdown from "react-markdown";
import errorToString from "../../../utils/errorToString";
import Alert from "../../../components/Alert/Alert";
import { AUTO_SCROLL_THRESHOLD } from "../config/constants";

// TODO: unit test
export default function AiChatWidget() {
	const [isOpen, setIsOpen] = useState(false);
	const [userPrompt, setUserPrompt] = useState("");
	const [errorMessage, setErrorMessage] = useState("");
	const [isSendingRequest, setIsSendingRequest] = useState(false);
	const [messages, setMessages] = useState<Message[]>([]);
	const textAreaRef = useRef<HTMLTextAreaElement | null>(null);
	const messagesContainerRef = useRef<HTMLDivElement | null>(null);

	const sendMessage = async () => {
		if (!userPrompt || isSendingRequest) return;

		setErrorMessage("");
		setIsSendingRequest(true);
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
				if (event.event === "inProgress") {
					const lastMessage = messages[messages.length - 1];
					return [
						...messages.slice(0, -1),
						{
							...lastMessage,
							content: lastMessage.content + event.data,
						},
					];
				} else if (event.event === "finished") {
					setIsSendingRequest(false);
				} else if (event.event === "error") {
					setErrorMessage(event.data);
					setIsSendingRequest(false);
				}
				return messages;
			});
		};
		setUserPrompt("");

		try {
			await streamAiResponse(userPrompt, onEvent);
		} catch (e) {
			setErrorMessage(errorToString(e));
			setIsSendingRequest(false);
		}
	};

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		void sendMessage();
	};

	const handleTextAreaKeyDown = (
		e: React.KeyboardEvent<HTMLTextAreaElement>,
	) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			void sendMessage();
		} else if (e.key === "Escape") {
			setIsOpen(false);
		}
	};

	useEffect(() => {
		if (textAreaRef.current) {
			textAreaRef.current.style.height = "auto";
			textAreaRef.current.style.height =
				textAreaRef.current.scrollHeight + "px";
		}
	}, [userPrompt]);

	useEffect(() => {
		if (!messagesContainerRef.current) return;

		const container = messagesContainerRef.current;

		const position = container.scrollTop + container.clientHeight;
		if (container.scrollHeight - position < AUTO_SCROLL_THRESHOLD) {
			container.scrollTop = container.scrollHeight;
		}
	}, [messages]);

	useEffect(() => {
		return () => {
			void stopAiGeneration();
		};
	}, []);

	return (
		<div className={styles.container}>
			{isOpen && (
				<div className={styles.chatPanel}>
					<div className={styles.header}>
						<p>AI Assistant</p>
						<button onClick={() => setIsOpen(false)}>
							<Icon path={mdiClose} size={1} />
						</button>
					</div>

					<div className={styles.messages} ref={messagesContainerRef}>
						{messages.map((message, i) => (
							<div
								key={i}
								className={`${styles.message} ${styles[message.from]}`}>
								<Markdown>{message.content}</Markdown>
								{isSendingRequest &&
									i === messages.length - 1 && (
										<div className={styles.spinner}></div>
									)}
							</div>
						))}

						{errorMessage && (
							<Alert
								type="error"
								onClose={() => setErrorMessage("")}>
								{errorMessage}
							</Alert>
						)}
					</div>

					<form onSubmit={handleSubmit}>
						<textarea
							ref={textAreaRef}
							placeholder="Ask a question"
							value={userPrompt}
							onChange={e => setUserPrompt(e.target.value)}
							onKeyDown={handleTextAreaKeyDown}
							rows={1}
						/>
						<button className="transparent" title="Add attachment">
							<Icon path={mdiAttachment} size={1} />
						</button>
						{!isSendingRequest && (
							<button className="transparent" title="Send">
								<Icon path={mdiSendVariantOutline} size={1} />
							</button>
						)}

						{isSendingRequest && (
							<button
								className="transparent"
								title="Stop"
								onClick={() => void stopAiGeneration()}>
								<Icon path={mdiStopCircleOutline} size={1} />
							</button>
						)}
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
