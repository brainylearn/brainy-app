import styles from "./styles.module.css";
import { mdiClose, mdiCheckOutline } from "@mdi/js";
import Icon from "@mdi/react";
import Markdown from "react-markdown";
import Message, {
	ToolCall,
	ToolCallStatus,
} from "../../../types/backend/entity/message";
import { useTransition } from "react";
import { rejectToolCall } from "../../../api/aiApi";

interface Props {
	isStreamingResponse: boolean;
	message: Message;
	onUpdate: () => Promise<void>;
}

// TODO: unit test
export default function ToolCallDisplay({
	isStreamingResponse,
	message,
	onUpdate,
}: Props) {
	const [isSendingRequest, startRequest] = useTransition();
	const toolCall = message.content.value as ToolCall;

	const handleRejectToolCall = () => {
		startRequest(async () => {
			await rejectToolCall(message.id);
			await onUpdate();
		});
	};

	return (
		<div className={styles.toolCall}>
			<p className={styles.toolCallHeader}>{toolCall.displayName}</p>
			<Markdown>{toolCall.displayDescriptionMarkdown}</Markdown>
			<div className={styles.footer}>
				{toolCall.status === ToolCallStatus.Rejected && (
					<div className={styles.reject}>
						<Icon path={mdiClose} size={1} />
						<p>Rejected</p>
					</div>
				)}

				{toolCall.status === ToolCallStatus.Pending && (
					<>
						<button
							className={`transparent ${styles.reject}`}
							type="button"
							disabled={isStreamingResponse || isSendingRequest}
							title={
								isStreamingResponse
									? "Please wait until generation is finished"
									: "Reject"
							}
							onClick={handleRejectToolCall}>
							<Icon path={mdiClose} size={1} />
							<p>Reject</p>
						</button>
						<button
							className={`transparent ${styles.accept}`}
							type="button"
							disabled={isStreamingResponse || isSendingRequest}
							title={
								isStreamingResponse
									? "Please wait until generation is finished"
									: "Accept"
							}>
							<Icon path={mdiCheckOutline} size={1} />
							<p>Accept</p>
						</button>
					</>
				)}
			</div>
		</div>
	);
}
