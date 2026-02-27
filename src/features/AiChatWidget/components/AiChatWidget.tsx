import Icon from "@mdi/react";
import {
	mdiAttachment,
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
	getChatMessagesOrdered,
	renameAiChat,
	stopAiGeneration,
	streamAiResponse,
} from "../../../api/aiApi";
import Message, {
	MessageContentHumanAssistant,
} from "../../../types/backend/entity/message";
import Markdown from "react-markdown";
import errorToString from "../../../utils/errorToString";
import Alert from "../../../components/Alert/Alert";
import {
	AUTO_SCROLL_THRESHOLD,
	NEW_SESSION_CHAT_ID,
	TEMP_ASSISTANT_MESSAGE_ID,
} from "../config/constants";
import Chat from "../../../types/backend/entity/chat";
import ConfirmationDialog from "../../../components/ConfirmationDialog/ConfirmationDialog";
import useAppSelector from "../../../hooks/useAppSelector";
import { selectSettings } from "../../../stores/settings/settingsSelector";
import useGlobalKey from "../../../hooks/useGlobalKey";
import { useSearchParams } from "react-router";
import { FILE_ID_QUERY_PARAMETER } from "../../../config/constants";
import ToolCallDisplay from "./ToolCallDisplay";
import Header from "./Header";
import RenameDialog from "./RenameDialog";

export default function AiChatWidget() {
	const settings = useAppSelector(selectSettings);
	return settings?.enableAi ? <AiChatWidgetInner /> : null;
}

function AiChatWidgetInner() {
	const [isOpen, setIsOpen] = useState(false);
	const [showDeleteChatDialog, setShowDeleteChatDialog] = useState(false);
	const [showRenameDialog, setShowRenameDialog] = useState(false);
	const [userPrompt, setUserPrompt] = useState("");
	const [errorMessage, setErrorMessage] = useState("");
	const [isStreamingResponse, setIsStreamingResponse] = useState(false);
	const [messages, setMessages] = useState<Message[]>([]);
	const [chats, setChats] = useState<Chat[]>([]);
	const [selectedChatId, setSelectedChatId] =
		useState<string>(NEW_SESSION_CHAT_ID);
	// Used to have reference to the same selected chat id, useful for streaming.
	const selectedChatIdRef = useRef(selectedChatId);
	const [searchParams] = useSearchParams();
	const textAreaRef = useRef<HTMLTextAreaElement | null>(null);
	const messagesContainerRef = useRef<HTMLDivElement | null>(null);
	const selectedFileId = searchParams.get(FILE_ID_QUERY_PARAMETER);

	useEffect(() => {
		selectedChatIdRef.current = selectedChatId;
	}, [selectedChatId]);

	useGlobalKey(e => {
		if (e.ctrlKey && e.key.toLowerCase() === "j") {
			setIsOpen(isOpen => !isOpen);
		}
	});

	const handleChangeSelectedChatId = async (newChatId: string) => {
		if (newChatId !== selectedChatId) {
			setErrorMessage("");
			await stopAiGeneration();
			setIsStreamingResponse(false);
			setSelectedChatId(newChatId);
		}

		if (newChatId === NEW_SESSION_CHAT_ID) {
			setMessages([]);
		} else {
			setMessages(await getChatMessagesOrdered(newChatId));
		}
	};

	const sendMessage = async () => {
		if (!userPrompt || isStreamingResponse) return;

		setErrorMessage("");
		setIsStreamingResponse(true);
		setMessages(messages => [
			...messages,
			{
				chatId: selectedChatId ?? "tmp",
				id: "tmp",
				content: {
					type: "human",
					value: userPrompt,
				},
			},
			{
				chatId: selectedChatId ?? "tmp",
				id: TEMP_ASSISTANT_MESSAGE_ID,
				contentType: "assistant",
				content: {
					type: "assistant",
					value: "",
				},
			},
		]);

		const onEvent = new Channel<StreamLlmResponseEvent>();
		onEvent.onmessage = event => {
			if (event.event === "createdChat") {
				setChats(chats => {
					let newValue = chats;
					if (!newValue.some(chat => chat.id === event.data.id)) {
						newValue = [event.data, ...chats];
					}
					return newValue;
				});
				setSelectedChatId(event.data.id);
			} else if (event.event === "inProgress") {
				setMessages(messages => {
					const tempAssistantMessage = messages.find(
						m => m.id === TEMP_ASSISTANT_MESSAGE_ID,
					)!;

					return [
						...messages.filter(
							m => m.id !== TEMP_ASSISTANT_MESSAGE_ID,
						),
						{
							...tempAssistantMessage,
							content: {
								...tempAssistantMessage.content,
								value:
									(tempAssistantMessage.content
										.value as string) + event.data,
							} as MessageContentHumanAssistant,
						},
					];
				});
			} else if (event.event === "error") {
				setErrorMessage(event.data);
			} else if (event.event === "toolCalled") {
				// TODO: unit test
				setMessages(messages => {
					const tempAssistantMessage = messages.find(
						m => m.id === TEMP_ASSISTANT_MESSAGE_ID,
					)!;

					return [
						...messages.filter(
							m => m.id !== TEMP_ASSISTANT_MESSAGE_ID,
						),
						event.data,
						tempAssistantMessage,
					];
				});
			}
		};
		setUserPrompt("");

		try {
			await streamAiResponse(
				{
					prompt: userPrompt,
					chatId:
						selectedChatId === NEW_SESSION_CHAT_ID
							? null
							: selectedChatId,
					fileId: selectedFileId,
				},
				onEvent,
			);
		} catch (e) {
			setErrorMessage(errorToString(e));
			setIsStreamingResponse(false);
		} finally {
			setMessages(
				await getChatMessagesOrdered(selectedChatIdRef.current),
			);
		}
	};

	const handleToolCallUpdate = async () => {
		setMessages(await getChatMessagesOrdered(selectedChatId));
	};

	const handleSubmit = (e: React.SubmitEvent) => {
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
		void (async () => {
			setChats(await getAllAiChatsSortedByDateDesc());
		})();

		return () => {
			void stopAiGeneration();
		};
	}, []);

	useEffect(() => {
		if (!messagesContainerRef.current) return;
		messagesContainerRef.current.scrollTop =
			messagesContainerRef.current.scrollHeight;
	}, [selectedChatId]);

	const handleDelete = async () => {
		await deleteAiChat(selectedChatId);
		await handleChangeSelectedChatId(NEW_SESSION_CHAT_ID);
		setErrorMessage("");
		setShowDeleteChatDialog(false);
		setChats(await getAllAiChatsSortedByDateDesc());
	};

	const handleRenameSubmit = async (
		e: React.SubmitEvent,
		newTitle: string,
	) => {
		e.stopPropagation();
		e.preventDefault();
		setShowRenameDialog(false);

		try {
			await renameAiChat(selectedChatId, newTitle);
			setChats(await getAllAiChatsSortedByDateDesc());
		} catch (e) {
			setErrorMessage(errorToString(e));
		}
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

			{showRenameDialog && (
				<RenameDialog
					onHide={() => setShowRenameDialog(false)}
					onSubmit={(e, newTitle) =>
						void handleRenameSubmit(e, newTitle)
					}
					initialTitle={
						chats.find(c => c.id === selectedChatId)?.title ?? ""
					}
				/>
			)}

			<div className={styles.container}>
				{isOpen && (
					<div className={styles.chatPanel}>
						<Header
							selectedChatId={selectedChatId}
							chats={chats}
							onChangeSelectedChatId={value =>
								void handleChangeSelectedChatId(value)
							}
							onClose={() => setIsOpen(false)}
							onRenameClick={() => setShowRenameDialog(true)}
							onDeleteClick={() => setShowDeleteChatDialog(true)}
						/>

						<div
							className={styles.messages}
							ref={messagesContainerRef}
							data-testid="messages-container">
							{messages.map((message, i) => (
								<div
									key={i}
									className={`${styles.message} ${styles[message.content.type]}`}>
									{(message.content.type === "human" ||
										message.content.type ==
											"assistant") && (
										<Markdown>
											{message.content.value}
										</Markdown>
									)}
									{message.content.type === "toolCall" && (
										<ToolCallDisplay
											isStreamingResponse={
												isStreamingResponse
											}
											message={message}
											onUpdate={handleToolCallUpdate}
										/>
									)}
									{isStreamingResponse &&
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
								placeholder="Speak with AI"
								value={userPrompt}
								onChange={e => setUserPrompt(e.target.value)}
								onKeyDown={handleTextAreaKeyDown}
								rows={1}
								autoFocus
							/>
							<button
								className="transparent"
								title="Add attachment">
								<Icon path={mdiAttachment} size={1} />
							</button>
							{!isStreamingResponse && (
								<button className="transparent" title="Send">
									<Icon
										path={mdiSendVariantOutline}
										size={1}
									/>
								</button>
							)}

							{isStreamingResponse && (
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
						onClick={() => setIsOpen(true)}
						title="Open AI assistant (Ctrl + J)">
						<Icon path={mdiRobotOutline} size={1.6} />
					</button>
				)}
			</div>
		</>
	);
}
