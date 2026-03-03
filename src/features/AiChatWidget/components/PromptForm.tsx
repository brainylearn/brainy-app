import { open } from "@tauri-apps/plugin-dialog";
import {
	mdiAttachment,
	mdiSendVariantOutline,
	mdiStopCircleOutline,
} from "@mdi/js";
import Icon from "@mdi/react";
import { useEffect, useRef } from "react";
import { uploadAttachment } from "../../../api/aiApi";

interface Props {
	isStreamingResponse: boolean;
	userPrompt: string;
	chatId: string | null;
	onTextAreaKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
	onUserPromptChange: (value: string) => void;
	onSubmit: (e: React.SubmitEvent) => void;
	onStopGeneration: () => Promise<void>;
}

export default function PromptForm({
	isStreamingResponse,
	userPrompt,
	chatId,
	onUserPromptChange,
	onSubmit,
	onStopGeneration,
	onTextAreaKeyDown,
}: Props) {
	const textAreaRef = useRef<HTMLTextAreaElement | null>(null);

	useEffect(() => {
		if (textAreaRef.current) {
			textAreaRef.current.style.height = "auto";
			textAreaRef.current.style.height =
				textAreaRef.current.scrollHeight +
				// Adding some extra space to not show scrollbars when there is
				// no need.
				2 +
				"px";
		}
	}, [userPrompt]);

	const handleAddAttachment = async () => {
		const attachmentPath = await open({
			directory: false,
		});

		if (!attachmentPath) return;
		await uploadAttachment(attachmentPath, chatId);
	};

	return (
		<form onSubmit={onSubmit}>
			<textarea
				ref={textAreaRef}
				placeholder="Speak with AI"
				value={userPrompt}
				onChange={e => onUserPromptChange(e.target.value)}
				onKeyDown={onTextAreaKeyDown}
				rows={1}
				autoFocus
			/>
			<button
				className="transparent"
				title="Add attachment"
				onClick={() => void handleAddAttachment()}
				disabled={isStreamingResponse}>
				<Icon path={mdiAttachment} size={1} />
			</button>
			{!isStreamingResponse && (
				<button className="transparent" title="Send">
					<Icon path={mdiSendVariantOutline} size={1} />
				</button>
			)}

			{isStreamingResponse && (
				<button
					className="transparent"
					title="Stop"
					onClick={() => void onStopGeneration()}>
					<Icon path={mdiStopCircleOutline} size={1} />
				</button>
			)}
		</form>
	);
}
