import Icon from "@mdi/react";
import {
	mdiAttachment,
	mdiClose,
	mdiRobotOutline,
	mdiSendVariantOutline,
} from "@mdi/js";
import styles from "./styles.module.css";
import { useState } from "react";

// TODO: should be used for both editor and reviewer
// TODO: rename and refactor
// TODO: responsivity
// TODO: unit test
export default function AiBot() {
	const [isOpen, setIsOpen] = useState(false);

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		// TODO:
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
						{/*TODO: better on hover and click style*/}
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
