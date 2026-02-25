import styles from "./styles.module.css";
import { mdiClose, mdiCheckOutline } from "@mdi/js";
import Icon from "@mdi/react";
import Markdown from "react-markdown";
import {
	ToolCall,
	ToolCallStatus,
} from "../../../types/backend/entity/message";

interface Props {
	toolCall: ToolCall;
}

// TODO: unit test
// TODO : should not be able to accept card while creating answer
export default function ToolCallDisplay({ toolCall }: Props) {
	return (
		<div className={styles.toolCall}>
			<p className={styles.toolCallHeader}>{toolCall.displayName}</p>
			<Markdown>{toolCall.displayDescriptionMarkdown}</Markdown>
			<div className={styles.footer}>
				{toolCall.status === ToolCallStatus.Pending && (
					<>
						<button
							className={`transparent ${styles.reject}`}
							type="button">
							<Icon path={mdiClose} size={1} />
							<p>Reject</p>
						</button>
						<button
							className={`transparent ${styles.accept}`}
							type="button">
							<Icon path={mdiCheckOutline} size={1} />
							<p>Accept</p>
						</button>
					</>
				)}
			</div>
		</div>
	);
}
