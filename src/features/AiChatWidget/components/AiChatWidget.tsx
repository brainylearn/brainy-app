import Icon from "@mdi/react";
import {
	mdiAttachment,
	mdiClose,
	mdiDeleteOutline,
	mdiRobotOutline,
	mdiSendVariantOutline,
	mdiStopCircleOutline,
} from "@mdi/js";
import styles from "./styles.module.css";
import { useEffect, useRef, useState } from "react";
import { Channel } from "@tauri-apps/api/core";
import { StreamLlmResponseEvent } from "../../../types/backend/events/streamLlmResponseEvent";
import {
	deleteAiChat,
	getAllAiChatsSortedByDateDesc,
	stopAiGeneration,
	streamAiResponse,
} from "../../../api/aiApi";
import Message from "../types/message";
import Markdown from "react-markdown";
import errorToString from "../../../utils/errorToString";
import Alert from "../../../components/Alert/Alert";
import { AUTO_SCROLL_THRESHOLD } from "../config/constants";
import Select from "../../../components/Select/Select";
import Chat from "../../../types/backend/entity/chat";
import ConfirmationDialog from "../../../components/ConfirmationDialog/ConfirmationDialog";

// TODO: unit test
export default function AiChatWidget() {
	const [isOpen, setIsOpen] = useState(false);
	const [showDeleteChatDialog, setShowDeleteChatDialog] = useState(false);
	const [userPrompt, setUserPrompt] = useState("");
	const [errorMessage, setErrorMessage] = useState("");
	const [isSendingRequest, setIsSendingRequest] = useState(false);
	const [messages, setMessages] = useState<Message[]>([]);
	const [chats, setChats] = useState<Chat[]>([]);
	const [selectedChatId, setSelectedChatId] = useState<string | null>(null);
	const textAreaRef = useRef<HTMLTextAreaElement | null>(null);
	const messagesContainerRef = useRef<HTMLDivElement | null>(null);

	const loadChats = async () => {
		setChats(await getAllAiChatsSortedByDateDesc());
		// TODO: get messages when opening or switching session
	};

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
				} else if (event.event === "createdChat") {
					// TODO: fix messages
					setChats(chats => {
						let newValue = chats;
						if (!newValue.some(chat => chat.id === event.data.id)) {
							newValue = [event.data, ...chats];
						}
						return newValue;
					});
					setSelectedChatId(event.data.id);
				} else if (event.event === "error") {
					setErrorMessage(event.data);
					setIsSendingRequest(false);
				}
				return messages;
			});
		};
		setUserPrompt("");

		try {
			await streamAiResponse(userPrompt, selectedChatId, onEvent);
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

	useEffect(() => {
		void (async () => {
			await loadChats();
		})();
	}, []);

	const handleDelete = async () => {
		await deleteAiChat(selectedChatId!);
		setSelectedChatId(null);
		setShowDeleteChatDialog(false);
		await loadChats();
	};

	return (
		<>
			{showDeleteChatDialog && (
				<ConfirmationDialog
					title="Delete chat"
					text="Are you sure you want to delete the selected chat"
					icon={mdiDeleteOutline}
					onCancel={() => setShowDeleteChatDialog(false)}
					onConfirm={() => void handleDelete()}
				/>
			)}
			<div className={styles.container}>
				{isOpen && (
					<div className={styles.chatPanel}>
						<div className={styles.header}>
							<Select
								onChange={setSelectedChatId}
								value={selectedChatId}
								options={[
									{
										value: null,
										label: "+ New chat",
									},
									...chats.map(chat => ({
										value: chat.id,
										label: chat.title,
									})),
								]}
							/>
							<div className="row">
								<button
									onClick={() =>
										setShowDeleteChatDialog(true)
									}
									className="transparent"
									title="Delete chat"
									disabled={!selectedChatId}>
									<Icon path={mdiDeleteOutline} size={1} />
								</button>
								<button
									onClick={() => setIsOpen(false)}
									className="transparent"
									title="Close chat">
									<Icon path={mdiClose} size={1} />
								</button>
							</div>
						</div>

						<div
							className={styles.messages}
							ref={messagesContainerRef}>
							{messages.map((message, i) => (
								<div
									key={i}
									className={`${styles.message} ${styles[message.from]}`}>
									<Markdown>{message.content}</Markdown>
									{isSendingRequest &&
										i === messages.length - 1 && (
											<div
												className={
													styles.spinner
												}></div>
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
							<button
								className="transparent"
								title="Add attachment">
								<Icon path={mdiAttachment} size={1} />
							</button>
							{!isSendingRequest && (
								<button className="transparent" title="Send">
									<Icon
										path={mdiSendVariantOutline}
										size={1}
									/>
								</button>
							)}

							{isSendingRequest && (
								<button
									className="transparent"
									title="Stop"
									onClick={() => void stopAiGeneration()}>
									<Icon
										path={mdiStopCircleOutline}
										size={1}
									/>
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
		</>
	);
}
